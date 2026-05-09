# Performance Review Fixes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Sửa các vấn đề hiệu năng đã phát hiện trong luồng thu âm, local pipeline, render transcript, lịch sử transcript, và TTS queue.

**Architecture:** Giữ thay đổi nhỏ và kiểm thử được. Sửa rò bộ nhớ Python trước, sau đó giảm render DOM, giảm I/O khi mở lịch sử, giới hạn hàng đợi TTS, rồi tách đường audio local khỏi vòng IPC JS nếu cần. Không đổi UX và không refactor lớn ngoài phần cần thiết.

**Tech Stack:** Tauri 2, Rust, JavaScript thuần, Python, MLX local pipeline, Node test runner, uv

---

## Scope Check

Các vấn đề nằm ở 5 khu vực độc lập:

- Local pipeline Python giữ audio thô quá lâu.
- Audio local đi qua JS rồi quay lại Rust.
- Transcript UI render nhiều lần trong một phản hồi.
- Lịch sử transcript đọc toàn bộ file khi chỉ cần metadata.
- TTS queue có thể phình khi đọc chậm hơn dịch.

Plan này chia thành các task độc lập. Có thể dừng sau mỗi task mà app vẫn chạy được.

## File Structure

- `scripts/local_pipeline.py`: sửa cơ chế compact buffer sau mỗi stride.
- `tests/test_local_pipeline_buffer.py`: test Python không load MLX model.
- `src/js/ui.js`: thêm render scheduler dùng `requestAnimationFrame`.
- `tests/transcript-ui-render-queue.test.mjs`: test JS cho việc gom render.
- `src-tauri/src/commands/transcript.rs`: đọc metadata bằng frontmatter reader giới hạn byte.
- `src/js/edge-tts.js`: thêm queue limit cho Edge TTS.
- `src/js/google-tts.js`: thêm queue limit cho Google TTS.
- `src/js/elevenlabs-tts.js`: thêm queue limit khi WebSocket chưa sẵn sàng.
- `tests/tts-queue-limit.test.mjs`: test JS cho queue limit.
- `src-tauri/src/commands/audio.rs`: thêm command Rust gửi audio capture trực tiếp vào local pipeline.
- `src-tauri/src/commands/local_pipeline.rs`: đổi state để Rust audio thread có thể ghi vào stdin pipeline.
- `src-tauri/src/lib.rs`: đăng ký command mới và khởi tạo state mới.
- `src/js/app.js`: local mode gọi command direct audio thay vì tạo JS audio channel.
- `package.json`: thêm script `test:js` nếu chưa có.

## Task 0: Baseline, GitNexus, Và Test Harness

**Files:**
- Modify: `package.json`
- Verify only: `.gitnexus/meta.json`, `.gitnexus/lbug`

- [ ] **Step 1: Kiểm tra worktree trước khi sửa**

Run:

```bash
git status --short
git diff -- src/js/app.js src/js/ui.js
```

Expected: thấy rõ thay đổi có sẵn. Nếu `src/js/app.js` hoặc `src/js/ui.js` đã có sửa đổi chưa commit, khi commit các task sau dùng `git add -p` để không gom nhầm hunk ngoài task.

- [ ] **Step 2: Cập nhật GitNexus nếu stale**

Run:

```bash
npx gitnexus status
```

Nếu output có `stale`, run:

```bash
npx gitnexus analyze
```

Expected: index hiện repo `my-translator`, không còn stale.

- [ ] **Step 3: Chạy impact analysis trước khi sửa symbol**

Run:

```bash
npx gitnexus impact --repo my-translator LocalPipeline --direction upstream
npx gitnexus impact --repo my-translator TranscriptUI --direction upstream
npx gitnexus impact --repo my-translator list_transcripts --direction upstream
npx gitnexus impact --repo my-translator AudioPlayer --direction upstream
npx gitnexus impact --repo my-translator start_capture --direction upstream
npx gitnexus impact --repo my-translator send_audio_to_pipeline --direction upstream
```

Expected: nếu có HIGH hoặc CRITICAL risk, dừng và báo user trước khi sửa.

- [ ] **Step 4: Thêm JS test script**

Modify `package.json`. Giữ nguyên script hiện có, thêm `test:js`:

```json
{
  "name": "my-translator",
  "private": true,
  "version": "0.5.3",
  "type": "module",
  "scripts": {
    "tauri": "tauri",
    "build": "TAURI_SIGNING_PRIVATE_KEY_PATH=~/.tauri/my-translator.key tauri build",
    "test:js": "node --test tests/*.test.mjs"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2"
  },
  "dependencies": {
    "@tauri-apps/plugin-process": "^2.3.1",
    "@tauri-apps/plugin-updater": "^2.10.0"
  }
}
```

- [ ] **Step 5: Cài test dependency cho DOM test**

Run:

```bash
npm install --save-dev jsdom
```

Expected: `package.json` có `jsdom` trong `devDependencies`, `package-lock.json` được cập nhật.

- [ ] **Step 6: Chạy baseline compile**

Run:

```bash
cd src-tauri && cargo check
```

Expected: `Finished dev profile`.

- [ ] **Step 7: Commit test harness**

Run:

```bash
git add package.json package-lock.json
git diff --cached --check
git commit -m "test: add javascript test harness"
```

Expected: commit chỉ chứa `package.json` và `package-lock.json`.

## Task 1: Sửa Rò Bộ Nhớ Trong Local Pipeline

**Files:**
- Create: `tests/test_local_pipeline_buffer.py`
- Modify: `scripts/local_pipeline.py:370-405`

- [ ] **Step 1: Viết failing test cho compact buffer**

Create `tests/test_local_pipeline_buffer.py`:

```python
import sys
import threading
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from scripts.local_pipeline import LocalPipeline


def make_pipeline_with_buffer(data: bytes):
    pipeline = LocalPipeline.__new__(LocalPipeline)
    pipeline.audio_buffer = bytearray(data)
    pipeline.lock = threading.Lock()
    return pipeline


def test_compact_audio_buffer_discards_processed_prefix():
    pipeline = make_pipeline_with_buffer(b"abcdefghij")

    new_pos = pipeline._compact_audio_buffer(4)

    assert new_pos == 0
    assert bytes(pipeline.audio_buffer) == b"efghij"


def test_compact_audio_buffer_keeps_buffer_when_nothing_processed():
    pipeline = make_pipeline_with_buffer(b"abcdefghij")

    new_pos = pipeline._compact_audio_buffer(0)

    assert new_pos == 0
    assert bytes(pipeline.audio_buffer) == b"abcdefghij"


def test_compact_audio_buffer_clears_when_processed_past_end():
    pipeline = make_pipeline_with_buffer(b"abc")

    new_pos = pipeline._compact_audio_buffer(10)

    assert new_pos == 0
    assert bytes(pipeline.audio_buffer) == b""
```

- [ ] **Step 2: Chạy test để thấy fail**

Run:

```bash
test -d ~/.venv/claude || uv venv ~/.venv/claude
uv pip install --python ~/.venv/claude/bin/python pytest numpy
~/.venv/claude/bin/python -m pytest tests/test_local_pipeline_buffer.py -q
```

Expected: FAIL với lỗi tương tự `AttributeError: 'LocalPipeline' object has no attribute '_compact_audio_buffer'`.

- [ ] **Step 3: Thêm helper `_compact_audio_buffer`**

Modify `scripts/local_pipeline.py`. Add method này vào class `LocalPipeline`, ngay trước `stdin_reader`:

```python
    def _compact_audio_buffer(self, processed_pos):
        """Drop processed audio bytes and reset processed_pos to buffer-relative zero."""
        if processed_pos <= 0:
            return 0

        with self.lock:
            if processed_pos >= len(self.audio_buffer):
                self.audio_buffer.clear()
            else:
                del self.audio_buffer[:processed_pos]

        return 0
```

- [ ] **Step 4: Dùng helper trong main loop**

In `scripts/local_pipeline.py`, replace đoạn trong `run()`:

```python
                self._process_chunk(chunk)
                processed_pos += self.stride_bytes
```

with:

```python
                self._process_chunk(chunk)
                processed_pos += self.stride_bytes
                processed_pos = self._compact_audio_buffer(processed_pos)
```

- [ ] **Step 5: Chạy test Python**

Run:

```bash
~/.venv/claude/bin/python -m pytest tests/test_local_pipeline_buffer.py -q
```

Expected: `3 passed`.

- [ ] **Step 6: Chạy local pipeline test mode nếu có file audio**

Run:

```bash
test -f /tmp/test_japanese.wav && ~/.venv/claude/bin/python scripts/local_pipeline.py --test --test-file /tmp/test_japanese.wav --transcript-only || true
```

Expected: nếu file tồn tại, pipeline chạy đến `done`. Nếu file không tồn tại, command vẫn exit thành công vì phần verify chính là unit test.

- [ ] **Step 7: Commit**

Run:

```bash
git add scripts/local_pipeline.py tests/test_local_pipeline_buffer.py
git diff --cached --check
git commit -m "perf(local): compact processed audio buffer"
```

Expected: commit chỉ chứa Python pipeline và test Python.

## Task 2: Gom Render Transcript Theo Khung Hình

**Files:**
- Create: `tests/transcript-ui-render-queue.test.mjs`
- Modify: `src/js/ui.js:13-626`

- [ ] **Step 1: Viết failing test cho render coalescing**

Create `tests/transcript-ui-render-queue.test.mjs`:

```javascript
import assert from 'node:assert/strict';
import test from 'node:test';
import { JSDOM } from 'jsdom';
import { TranscriptUI } from '../src/js/ui.js';

function setupDom() {
    const dom = new JSDOM('<!doctype html><body><div id="overlay-view"></div></body>');
    global.document = dom.window.document;
    global.window = dom.window;

    const callbacks = [];
    global.requestAnimationFrame = (callback) => {
        callbacks.push(callback);
        return callbacks.length;
    };

    const scrollHost = document.createElement('div');
    const container = document.createElement('div');
    scrollHost.appendChild(container);
    document.body.appendChild(scrollHost);

    return { container, callbacks };
}

test('TranscriptUI renders at most once per animation frame', () => {
    const { container, callbacks } = setupDom();
    const ui = new TranscriptUI(container);

    let renderCount = 0;
    const originalRenderNow = ui._renderNow.bind(ui);
    ui._renderNow = () => {
        renderCount += 1;
        originalRenderNow();
    };

    ui.addOriginal('こんにちは');
    ui.addTranslation('Xin chào');
    ui.setProvisional('テスト');

    assert.equal(renderCount, 0);
    assert.equal(callbacks.length, 1);

    callbacks.shift()();

    assert.equal(renderCount, 1);
    assert.match(container.innerHTML, /Xin chào/);
});

test('TranscriptUI cancels stale scheduled render after clear', () => {
    const { container, callbacks } = setupDom();
    const ui = new TranscriptUI(container);

    ui.addOriginal('古いテキスト');
    ui.clear();

    assert.equal(container.innerHTML, '');
    assert.equal(callbacks.length, 1);

    callbacks.shift()();

    assert.equal(container.innerHTML, '');
});
```

- [ ] **Step 2: Chạy test để thấy fail**

Run:

```bash
npm run test:js -- tests/transcript-ui-render-queue.test.mjs
```

Expected: FAIL vì `ui._renderNow` chưa tồn tại.

- [ ] **Step 3: Thêm render scheduler vào constructor**

In `src/js/ui.js`, inside `constructor(container)`, after scroll sync fields:

```javascript
        this._renderScheduled = false;
        this._renderToken = 0;
```

Constructor cuối phần state sẽ thành:

```javascript
        this._isSyncingScroll = false;
        this._scrollSyncCleanup = null;
        this._renderScheduled = false;
        this._renderToken = 0;
```

- [ ] **Step 4: Thêm `_scheduleRender`, `_cancelScheduledRender`, và `_renderNow`**

Replace method `_render()` trong `src/js/ui.js`:

```javascript
    _render() {
        this._ensureContent();
        this._trimSegments();

        if (this.showOriginal === 'dual') {
            this._renderDual();
        } else {
            this._renderSingle();
        }
    }
```

with:

```javascript
    _render() {
        this._scheduleRender();
    }

    _scheduleRender() {
        if (this._renderScheduled) return;
        this._renderScheduled = true;
        const token = this._renderToken;

        requestAnimationFrame(() => {
            this._renderScheduled = false;
            if (token !== this._renderToken) return;
            this._renderNow();
        });
    }

    _cancelScheduledRender() {
        this._renderToken++;
        this._renderScheduled = false;
    }

    _renderNow() {
        this._ensureContent();
        this._trimSegments();

        if (this.showOriginal === 'dual') {
            this._renderDual();
        } else {
            this._renderSingle();
        }
    }
```

- [ ] **Step 5: Huỷ scheduled render khi clear hoặc placeholder**

In `showPlaceholder()`, add first line:

```javascript
        this._cancelScheduledRender();
```

Method start becomes:

```javascript
    showPlaceholder() {
        this._cancelScheduledRender();
        this.container.innerHTML = `
```

In `clear()`, add first line:

```javascript
        this._cancelScheduledRender();
```

Method start becomes:

```javascript
    clear() {
        this._cancelScheduledRender();
        this.container.innerHTML = '';
```

- [ ] **Step 6: Chạy JS test**

Run:

```bash
npm run test:js -- tests/transcript-ui-render-queue.test.mjs
```

Expected: PASS.

- [ ] **Step 7: Manual smoke test**

Run:

```bash
npm run tauri dev
```

Expected: start app, transcript vẫn hiện original, translation, provisional. Chuyển view mode `below`, `dual`, `off` không mất nội dung.

- [ ] **Step 8: Commit**

Run:

```bash
git add src/js/ui.js tests/transcript-ui-render-queue.test.mjs
git diff --cached --check
git commit -m "perf(ui): coalesce transcript renders"
```

Expected: commit chỉ chứa `ui.js` và test liên quan.

## Task 3: Tối Ưu Danh Sách Transcript

**Files:**
- Modify: `src-tauri/src/commands/transcript.rs:1-142`

- [ ] **Step 1: Viết failing Rust tests cho frontmatter parser**

Add vào cuối `src-tauri/src/commands/transcript.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::parse_transcript_metadata;

    #[test]
    fn parses_duration_and_languages_from_frontmatter() {
        let input = "---\nduration: 2m 10s\nsource_lang: ja\ntarget_lang: vi\n---\n\nbody";

        let metadata = parse_transcript_metadata(input);

        assert_eq!(metadata.duration.as_deref(), Some("2m 10s"));
        assert_eq!(metadata.source_lang.as_deref(), Some("ja"));
        assert_eq!(metadata.target_lang.as_deref(), Some("vi"));
    }

    #[test]
    fn returns_empty_metadata_without_frontmatter() {
        let input = "plain transcript body";

        let metadata = parse_transcript_metadata(input);

        assert!(metadata.duration.is_none());
        assert!(metadata.source_lang.is_none());
        assert!(metadata.target_lang.is_none());
    }
}
```

- [ ] **Step 2: Chạy test để thấy fail**

Run:

```bash
cd src-tauri && cargo test commands::transcript::tests::parses_duration_and_languages_from_frontmatter
```

Expected: FAIL vì `parse_transcript_metadata` chưa tồn tại.

- [ ] **Step 3: Thêm helper metadata và limited reader**

In `src-tauri/src/commands/transcript.rs`, replace import block:

```rust
use std::fs;
use std::path::PathBuf;
```

with:

```rust
use std::fs;
use std::io::{Read, BufReader};
use std::path::{Path, PathBuf};
```

Then add below `TranscriptEntry` struct:

```rust
const FRONTMATTER_READ_LIMIT: u64 = 16 * 1024;

struct TranscriptMetadata {
    duration: Option<String>,
    source_lang: Option<String>,
    target_lang: Option<String>,
}

fn read_frontmatter_prefix(path: &Path) -> String {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(_) => return String::new(),
    };

    let mut reader = BufReader::new(file.take(FRONTMATTER_READ_LIMIT));
    let mut content = String::new();
    let _ = reader.read_to_string(&mut content);
    content
}

fn parse_transcript_metadata(content: &str) -> TranscriptMetadata {
    let mut metadata = TranscriptMetadata {
        duration: None,
        source_lang: None,
        target_lang: None,
    };

    if !content.starts_with("---") {
        return metadata;
    }

    let Some(end) = content[3..].find("---") else {
        return metadata;
    };

    let yaml = &content[3..3 + end];
    for line in yaml.lines() {
        if let Some(val) = line.strip_prefix("duration:") {
            metadata.duration = Some(val.trim().to_string());
        } else if let Some(val) = line.strip_prefix("source_lang:") {
            metadata.source_lang = Some(val.trim().to_string());
        } else if let Some(val) = line.strip_prefix("target_lang:") {
            metadata.target_lang = Some(val.trim().to_string());
        }
    }

    metadata
}
```

- [ ] **Step 4: Dùng limited reader trong `list_transcripts`**

Replace block trong `list_transcripts`:

```rust
            let (duration, source_lang, target_lang) = {
                let content = fs::read_to_string(entry.path()).unwrap_or_default();
                let mut dur = None;
                let mut src = None;
                let mut tgt = None;
                if content.starts_with("---") {
                    if let Some(end) = content[3..].find("---") {
                        let yaml = &content[3..3 + end];
                        for line in yaml.lines() {
                            if let Some(val) = line.strip_prefix("duration:") {
                                dur = Some(val.trim().to_string());
                            } else if let Some(val) = line.strip_prefix("source_lang:") {
                                src = Some(val.trim().to_string());
                            } else if let Some(val) = line.strip_prefix("target_lang:") {
                                tgt = Some(val.trim().to_string());
                            }
                        }
                    }
                }
                (dur, src, tgt)
            };
```

with:

```rust
            let metadata = parse_transcript_metadata(&read_frontmatter_prefix(&entry.path()));
            let duration = metadata.duration;
            let source_lang = metadata.source_lang;
            let target_lang = metadata.target_lang;
```

- [ ] **Step 5: Chạy Rust test**

Run:

```bash
cd src-tauri && cargo test commands::transcript::tests
```

Expected: both tests pass.

- [ ] **Step 6: Chạy compile**

Run:

```bash
cd src-tauri && cargo check
```

Expected: `Finished dev profile`.

- [ ] **Step 7: Commit**

Run:

```bash
git add src-tauri/src/commands/transcript.rs
git diff --cached --check
git commit -m "perf(transcript): read only frontmatter when listing sessions"
```

Expected: commit chỉ chứa Rust transcript command.

## Task 4: Giới Hạn Hàng Đợi TTS

**Files:**
- Create: `tests/tts-queue-limit.test.mjs`
- Modify: `src/js/edge-tts.js:9-84`
- Modify: `src/js/google-tts.js:21-135`
- Modify: `src/js/elevenlabs-tts.js:6-225`

- [ ] **Step 1: Viết failing tests cho queue limit**

Create `tests/tts-queue-limit.test.mjs`:

```javascript
import assert from 'node:assert/strict';
import test from 'node:test';

global.window = {
    __TAURI__: {
        core: {
            invoke: async () => 'base64-audio',
        },
    },
};

test('Edge TTS drops oldest queued text when queue is full', async () => {
    const { edgeTTSRust } = await import(`../src/js/edge-tts.js?edge=${Date.now()}`);
    edgeTTSRust.disconnect();
    edgeTTSRust._isSpeaking = true;

    for (let i = 0; i < 8; i++) {
        edgeTTSRust.speak(`edge ${i}`);
    }

    assert.equal(edgeTTSRust._queue.length, 5);
    assert.deepEqual(edgeTTSRust._queue, ['edge 3', 'edge 4', 'edge 5', 'edge 6', 'edge 7']);
});

test('Google TTS drops oldest queued text when queue is full', async () => {
    const { googleTTS } = await import(`../src/js/google-tts.js?google=${Date.now()}`);
    googleTTS.disconnect();
    googleTTS._isSpeaking = true;

    for (let i = 0; i < 8; i++) {
        googleTTS.speak(`google ${i}`);
    }

    assert.equal(googleTTS._queue.length, 5);
    assert.deepEqual(googleTTS._queue, ['google 3', 'google 4', 'google 5', 'google 6', 'google 7']);
});

test('ElevenLabs TTS limits queued text before websocket connects', async () => {
    const { elevenLabsTTS } = await import(`../src/js/elevenlabs-tts.js?eleven=${Date.now()}`);
    elevenLabsTTS.disconnect();
    elevenLabsTTS.ws = { readyState: WebSocket.CLOSED };
    elevenLabsTTS.connect = () => {};

    for (let i = 0; i < 12; i++) {
        elevenLabsTTS.speak(`eleven ${i}`);
    }

    assert.equal(elevenLabsTTS._textQueue.length, 10);
    assert.deepEqual(elevenLabsTTS._textQueue[0], 'eleven 2');
    assert.deepEqual(elevenLabsTTS._textQueue[9], 'eleven 11');
});
```

- [ ] **Step 2: Chạy test để thấy fail**

Run:

```bash
npm run test:js -- tests/tts-queue-limit.test.mjs
```

Expected: FAIL vì queue chưa bị giới hạn.

- [ ] **Step 3: Sửa Edge TTS queue**

In `src/js/edge-tts.js`, inside constructor after `_isSpeaking`:

```javascript
        this._maxQueueSize = 5;
```

Replace `speak(text)`:

```javascript
    speak(text) {
        if (!text?.trim()) return;
        this._queue.push(text.trim());
        if (!this._isSpeaking) {
            this._processQueue();
        }
    }
```

with:

```javascript
    speak(text) {
        const cleanText = text?.trim();
        if (!cleanText) return;
        this._enqueueText(cleanText);
        if (!this._isSpeaking) {
            this._processQueue();
        }
    }

    _enqueueText(text) {
        this._queue.push(text);
        if (this._queue.length > this._maxQueueSize) {
            this._queue.splice(0, this._queue.length - this._maxQueueSize);
        }
    }
```

- [ ] **Step 4: Sửa Google TTS queue**

In `src/js/google-tts.js`, inside constructor after `_isSpeaking`:

```javascript
        this._maxQueueSize = 5;
```

Replace `speak(text)`:

```javascript
    speak(text) {
        if (!text?.trim()) return;
        this._queue.push(text.trim());
        if (!this._isSpeaking) {
            this._processQueue();
        }
    }
```

with:

```javascript
    speak(text) {
        const cleanText = text?.trim();
        if (!cleanText) return;
        this._enqueueText(cleanText);
        if (!this._isSpeaking) {
            this._processQueue();
        }
    }

    _enqueueText(text) {
        this._queue.push(text);
        if (this._queue.length > this._maxQueueSize) {
            this._queue.splice(0, this._queue.length - this._maxQueueSize);
        }
    }
```

- [ ] **Step 5: Sửa ElevenLabs queued text**

In `src/js/elevenlabs-tts.js`, inside constructor after `_textQueue`:

```javascript
        this._maxTextQueueSize = 10;
```

Replace offline branch in `speak(text)`:

```javascript
        } else {
            // Queue and connect if needed
            this._textQueue.push(text);
            if (!this.ws || this.ws.readyState === WebSocket.CLOSED) {
                this.connect();
            }
        }
```

with:

```javascript
        } else {
            this._enqueueText(text);
            if (!this.ws || this.ws.readyState === WebSocket.CLOSED) {
                this.connect();
            }
        }
```

Add method before `_sendText(text)`:

```javascript
    _enqueueText(text) {
        this._textQueue.push(text);
        if (this._textQueue.length > this._maxTextQueueSize) {
            this._textQueue.splice(0, this._textQueue.length - this._maxTextQueueSize);
        }
    }
```

- [ ] **Step 6: Chạy TTS tests**

Run:

```bash
npm run test:js -- tests/tts-queue-limit.test.mjs
```

Expected: PASS.

- [ ] **Step 7: Chạy toàn bộ JS tests**

Run:

```bash
npm run test:js
```

Expected: all JS tests pass.

- [ ] **Step 8: Commit**

Run:

```bash
git add src/js/edge-tts.js src/js/google-tts.js src/js/elevenlabs-tts.js tests/tts-queue-limit.test.mjs
git diff --cached --check
git commit -m "perf(tts): bound pending narration queues"
```

Expected: commit chỉ chứa 3 TTS modules và test.

## Task 5: Giảm Chi Phí Audio Local Qua IPC JS

**Files:**
- Modify: `src-tauri/src/commands/local_pipeline.rs:1-227`
- Modify: `src-tauri/src/commands/audio.rs:1-179`
- Modify: `src-tauri/src/lib.rs:36-63`
- Modify: `src/js/app.js:1358-1379`

- [ ] **Step 1: Chạy impact trước khi sửa Rust audio**

Run:

```bash
npx gitnexus impact --repo my-translator start_capture --direction upstream
npx gitnexus impact --repo my-translator start_local_pipeline --direction upstream
npx gitnexus impact --repo my-translator send_audio_to_pipeline --direction upstream
```

Expected: LOW hoặc MEDIUM. Nếu HIGH hoặc CRITICAL, dừng và báo user.

- [ ] **Step 2: Đổi `LocalPipelineState` sang Arc**

In `src-tauri/src/commands/local_pipeline.rs`, replace imports:

```rust
use std::sync::Mutex;
```

with:

```rust
use std::sync::{Arc, Mutex};
```

Replace state struct:

```rust
pub struct LocalPipelineState {
    pub process: Mutex<Option<Child>>,
}
```

with:

```rust
pub struct LocalPipelineState {
    pub process: Arc<Mutex<Option<Child>>>,
}
```

No other line in `local_pipeline.rs` needs to change because `Arc<Mutex<T>>` derefs to `Mutex<T>`.

- [ ] **Step 3: Khởi tạo Arc state trong lib**

In `src-tauri/src/lib.rs`, replace imports:

```rust
use std::sync::Mutex;
```

with:

```rust
use std::sync::{Arc, Mutex};
```

Replace state init:

```rust
        .manage(LocalPipelineState {
            process: Mutex::new(None),
        })
```

with:

```rust
        .manage(LocalPipelineState {
            process: Arc::new(Mutex::new(None)),
        })
```

- [ ] **Step 4: Refactor receiver start helper trong audio command**

In `src-tauri/src/commands/audio.rs`, add imports near top:

```rust
use crate::commands::local_pipeline::LocalPipelineState;
use std::io::Write;
use std::process::Child;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
```

Replace current import:

```rust
use std::sync::mpsc;
use std::sync::Mutex;
```

with:

```rust
use std::sync::mpsc;
use std::sync::Mutex;
```

Then add helper below `PermissionStatus`:

```rust
fn start_receiver_for_source(
    source: &str,
    state: &AudioState,
) -> Result<mpsc::Receiver<Vec<u8>>, String> {
    match source {
        "system" => {
            let sys = state.system_audio.lock().map_err(|e| e.to_string())?;
            sys.start()
        }
        "microphone" => {
            let mut mic = state.microphone.lock().map_err(|e| e.to_string())?;
            mic.start()
        }
        "both" => {
            let sys = state.system_audio.lock().map_err(|e| e.to_string())?;
            let sys_rx = sys.start()?;
            let mut mic = state.microphone.lock().map_err(|e| e.to_string())?;
            let mic_rx = mic.start()?;

            let (merged_tx, merged_rx) = mpsc::channel::<Vec<u8>>();
            let tx1 = merged_tx.clone();
            let tx2 = merged_tx;

            std::thread::spawn(move || {
                while let Ok(data) = sys_rx.recv() {
                    if tx1.send(data).is_err() {
                        break;
                    }
                }
            });

            std::thread::spawn(move || {
                while let Ok(data) = mic_rx.recv() {
                    if tx2.send(data).is_err() {
                        break;
                    }
                }
            });

            Ok(merged_rx)
        }
        _ => Err(format!("Unknown source: {}", source)),
    }
}
```

- [ ] **Step 5: Simplify `start_capture` bằng helper**

In `src-tauri/src/commands/audio.rs`, replace source match inside `start_capture`:

```rust
    let receiver: mpsc::Receiver<Vec<u8>> = match source.as_str() {
        "system" => {
            let sys = state.system_audio.lock().map_err(|e| e.to_string())?;
            sys.start()?
        }
        "microphone" => {
            let mut mic = state.microphone.lock().map_err(|e| e.to_string())?;
            mic.start()?
        }
        "both" => {
            // Start both sources and merge into a single receiver
            let sys = state.system_audio.lock().map_err(|e| e.to_string())?;
            let sys_rx = sys.start()?;
            let mut mic = state.microphone.lock().map_err(|e| e.to_string())?;
            let mic_rx = mic.start()?;

            let (merged_tx, merged_rx) = mpsc::channel::<Vec<u8>>();
            let tx1 = merged_tx.clone();
            let tx2 = merged_tx;

            // Forward system audio to merged channel
            std::thread::spawn(move || {
                while let Ok(data) = sys_rx.recv() {
                    if tx1.send(data).is_err() { break; }
                }
            });
            // Forward mic audio to merged channel
            std::thread::spawn(move || {
                while let Ok(data) = mic_rx.recv() {
                    if tx2.send(data).is_err() { break; }
                }
            });

            merged_rx
        }
        _ => return Err(format!("Unknown source: {}", source)),
    };
```

with:

```rust
    let receiver = start_receiver_for_source(&source, &state)?;
```

- [ ] **Step 6: Thêm direct pipeline command**

Add below `start_capture` in `src-tauri/src/commands/audio.rs`:

```rust
#[tauri::command]
pub fn start_capture_to_pipeline(
    source: String,
    state: State<'_, AudioState>,
    pipeline_state: State<'_, LocalPipelineState>,
) -> Result<(), String> {
    stop_capture_inner(&state);

    let receiver = start_receiver_for_source(&source, &state)?;
    let process = pipeline_state.process.clone();
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_clone = stop_flag.clone();

    std::thread::spawn(move || {
        let mut buffer: Vec<u8> = Vec::with_capacity(32000);
        let batch_interval = std::time::Duration::from_millis(200);
        let mut last_flush = std::time::Instant::now();

        loop {
            if stop_flag_clone.load(Ordering::SeqCst) {
                if !buffer.is_empty() {
                    let _ = write_audio_to_pipeline(&process, &buffer);
                }
                break;
            }

            match receiver.recv_timeout(std::time::Duration::from_millis(10)) {
                Ok(data) => {
                    buffer.extend_from_slice(&data);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    if !buffer.is_empty() {
                        let _ = write_audio_to_pipeline(&process, &buffer);
                    }
                    break;
                }
            }

            if last_flush.elapsed() >= batch_interval && !buffer.is_empty() {
                if write_audio_to_pipeline(&process, &buffer).is_err() {
                    break;
                }
                buffer.clear();
                last_flush = std::time::Instant::now();
            }
        }
    });

    let forwarder = AudioForwarder { stop_flag };
    let mut active = state.active_receiver.lock().map_err(|e| e.to_string())?;
    *active = Some(forwarder);

    Ok(())
}

fn write_audio_to_pipeline(
    process: &Arc<Mutex<Option<Child>>>,
    data: &[u8],
) -> Result<(), String> {
    let mut proc = process.lock().map_err(|e| e.to_string())?;
    if let Some(ref mut child) = *proc {
        if let Some(ref mut stdin) = child.stdin {
            stdin.write_all(data).map_err(|e| e.to_string())?;
            stdin.flush().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
```

- [ ] **Step 7: Đăng ký command mới**

In `src-tauri/src/lib.rs`, add to `invoke_handler` after `start_capture`:

```rust
            commands::audio::start_capture_to_pipeline,
```

The audio command group becomes:

```rust
            commands::audio::start_capture,
            commands::audio::start_capture_to_pipeline,
            commands::audio::stop_capture,
```

- [ ] **Step 8: Đổi local mode JS sang direct command**

In `src/js/app.js`, replace block:

```javascript
            const audioChannel = new window.__TAURI__.core.Channel();
            let audioChunkCount = 0;

            audioChannel.onmessage = async (pcmData) => {
                audioChunkCount++;
                if (audioChunkCount <= 3 || audioChunkCount % 50 === 0) {
                    console.log(`[Local] Audio batch #${audioChunkCount}, size:`, pcmData?.length || 0);
                }
                try {
                    await invoke('send_audio_to_pipeline', { data: Array.from(new Uint8Array(pcmData)) });
                } catch (e) {
                    // Pipeline may not be ready yet
                }
            };

            await invoke('start_capture', {
                source: this.currentSource,
                channel: audioChannel,
            });
```

with:

```javascript
            await invoke('start_capture_to_pipeline', {
                source: this.currentSource,
            });
```

Keep the existing `console.log('[App] Audio capture started');` line after it.

- [ ] **Step 9: Chạy Rust compile**

Run:

```bash
cd src-tauri && cargo check
```

Expected: `Finished dev profile`.

- [ ] **Step 10: Manual local mode smoke test**

Run:

```bash
npm run tauri dev
```

Expected:

- Chọn local mode.
- Start session.
- Không còn log `[Local] Audio batch` từ JS.
- Pipeline vẫn nhận audio và trả result.
- Stop session không treo process Python.

- [ ] **Step 11: Commit**

Run:

```bash
git add src-tauri/src/commands/local_pipeline.rs src-tauri/src/commands/audio.rs src-tauri/src/lib.rs src/js/app.js
git diff --cached --check
git commit -m "perf(local): stream captured audio directly to pipeline"
```

Expected: commit chứa Rust direct bridge và thay đổi local JS.

## Task 6: Final Verification Và Scope Check

**Files:**
- Verify only

- [ ] **Step 1: Chạy Python tests**

Run:

```bash
~/.venv/claude/bin/python -m pytest tests/test_local_pipeline_buffer.py -q
```

Expected: `3 passed`.

- [ ] **Step 2: Chạy JS tests**

Run:

```bash
npm run test:js
```

Expected: all JS tests pass.

- [ ] **Step 3: Chạy Rust tests**

Run:

```bash
cd src-tauri && cargo test commands::transcript::tests
```

Expected: transcript tests pass.

- [ ] **Step 4: Chạy Rust compile**

Run:

```bash
cd src-tauri && cargo check
```

Expected: `Finished dev profile`.

- [ ] **Step 5: Chạy app smoke test**

Run:

```bash
npm run tauri dev
```

Expected:

- Soniox mode start, pause, resume, stop vẫn hoạt động.
- Local mode start, stop vẫn hoạt động.
- Transcript view `below`, `dual`, `off` vẫn hiển thị đúng.
- Sessions view mở nhanh với nhiều transcript.
- TTS bật tắt được, không đọc backlog quá dài.

- [ ] **Step 6: Kiểm tra GitNexus affected scope**

Run:

```bash
npx gitnexus detect_changes
```

Nếu CLI không có command này trong bản đang cài, run fallback:

```bash
git diff --stat HEAD
git status --short
```

Expected: thay đổi chỉ nằm trong file của plan. Không có file ngoài phạm vi bị sửa.

- [ ] **Step 7: Re-run GitNexus analyze sau commit cuối nếu cần**

Run:

```bash
npx gitnexus analyze
```

Expected: index cập nhật. Nếu `.gitnexus` là file tracked, commit riêng bằng:

```bash
git add .gitnexus AGENTS.md .gitignore
git diff --cached --check
git commit -m "chore: update gitnexus index"
```

Nếu `.gitnexus` không cần commit, không stage.

## Self Review

Spec coverage:

- Local pipeline memory growth: Task 1.
- Local audio IPC overhead: Task 5.
- Transcript render churn: Task 2.
- Transcript session listing I/O: Task 3.
- TTS queue growth: Task 4.
- Verification and GitNexus scope: Task 0 and Task 6.

Placeholder scan:

- No task uses placeholder markers.
- Every code-changing step includes exact code or exact replacement block.
- Every test step has command and expected result.

Type consistency:

- `LocalPipeline._compact_audio_buffer(processed_pos)` returns new `processed_pos`.
- `TranscriptUI._renderNow()` is the immediate render entry used by tests.
- `start_capture_to_pipeline(source)` is registered in Rust and called from JS with only `source`.
