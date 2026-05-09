# Session Viewer Format Fix

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Render session transcript content as formatted HTML thay vì raw markdown text

**Architecture:** Parse YAML frontmatter thành metadata card, parse body thành styled segments (speaker label + original text + translation). Không dùng thư viện markdown bên ngoài, chỉ string parsing đơn giản vì format cố định.

**Tech Stack:** Vanilla JS, CSS

## Vấn đề hiện tại

`_openSession()` tại [app.js:1886](src/js/app.js#L1886) dùng `content.textContent = text` gán raw markdown. Kết quả: YAML frontmatter, `**Speaker 1:**`, `> blockquote` hiện nguyên dạng text thô, không có line break giữa segments.

## File structure

| File | Thay đổi |
|------|----------|
| `src/js/app.js` | Modify: thêm `_renderSessionContent()`, sửa `_openSession()` |
| `src/styles/main.css` | Modify: thêm CSS cho session content elements |

## Markdown format cần parse

```markdown
---
date: 2026-05-09
time: 07:08:17
duration: 2m 2s
source_lang: auto
target_lang: vi
mode: one_way
audio_source: system
model: Soniox Cloud API
segments: 90
---

**Speaker 1:**
> In their body, right?
Trong cơ thể họ, đúng không?

**Speaker 1:**
> And so when you try to get yourself
Vì vậy khi bạn cố gắng tự khiến mình
```

---

### Task 1: Thêm method `_renderSessionContent(rawText)` vào App

**Files:**
- Modify: `src/js/app.js:1873-1890`

- [ ] **Step 1: Thêm method `_renderSessionContent`**

Thêm sau method `_openSession` (khoảng dòng 1890):

```javascript
_renderSessionContent(raw) {
    let body = raw;
    let metaHtml = '';

    // Parse YAML frontmatter
    if (raw.startsWith('---')) {
        const endIdx = raw.indexOf('---', 3);
        if (endIdx !== -1) {
            const yaml = raw.slice(3, endIdx).trim();
            body = raw.slice(endIdx + 3).trim();

            const meta = {};
            for (const line of yaml.split('\n')) {
                const colonIdx = line.indexOf(':');
                if (colonIdx === -1) continue;
                const key = line.slice(0, colonIdx).trim();
                const val = line.slice(colonIdx + 1).trim();
                if (val) meta[key] = val;
            }

            const chips = [];
            if (meta.duration) chips.push(meta.duration);
            if (meta.source_lang && meta.target_lang) {
                chips.push(`${meta.source_lang} → ${meta.target_lang}`);
            }
            if (meta.mode) {
                const modeLabel = meta.mode === 'one_way' ? 'One-way' : 'Two-way';
                chips.push(modeLabel);
            }
            if (meta.audio_source) chips.push(meta.audio_source);
            if (meta.model) chips.push(meta.model);
            if (meta.segments) chips.push(`${meta.segments} segments`);

            if (chips.length) {
                metaHtml = `<div class="session-meta">${chips.map(c =>
                    `<span class="session-meta-chip">${this._esc(c)}</span>`
                ).join('')}</div>`;
            }
        }
    }

    // Parse transcript body
    const lines = body.split('\n');
    const parts = [];
    let i = 0;

    while (i < lines.length) {
        const line = lines[i];

        // Speaker label: **Speaker N:**
        const speakerMatch = line.match(/^\*\*(.+?):\*\*$/);
        if (speakerMatch) {
            parts.push(`<div class="session-speaker">${this._esc(speakerMatch[1])}</div>`);
            i++;
            continue;
        }

        // Original text: > text
        if (line.startsWith('> ')) {
            parts.push(`<div class="session-original">${this._esc(line.slice(2))}</div>`);
            i++;
            continue;
        }

        // Empty line = segment separator
        if (line.trim() === '') {
            i++;
            continue;
        }

        // Translation text (anything else)
        parts.push(`<div class="session-translation">${this._esc(line)}</div>`);
        i++;
    }

    return metaHtml + '<div class="session-segments">' + parts.join('') + '</div>';
}
```

- [ ] **Step 2: Thêm helper `_esc` nếu chưa có**

Kiểm tra class App đã có `_esc()` chưa. Nếu chưa, thêm:

```javascript
_esc(str) {
    const d = document.createElement('div');
    d.textContent = str;
    return d.innerHTML;
}
```

Lưu ý: class App đã có `_escAttr()` (dòng 1852), nhưng cần `_esc()` riêng cho HTML content.

- [ ] **Step 3: Sửa `_openSession` dùng innerHTML**

Thay dòng 1886:
```javascript
// Trước:
if (content) content.textContent = text;

// Sau:
if (content) content.innerHTML = this._renderSessionContent(text);
```

Giữ dòng 1882 `content.textContent = 'Loading...'` vì đó là loading state, textContent OK.

- [ ] **Step 4: Sửa Copy button giữ raw text**

Tại dòng 153-158, Copy button dùng `textContent` từ rendered HTML. Để copy sạch hơn, lưu raw text vào dataset:

Sửa `_openSession` thêm 1 dòng sau khi set innerHTML:
```javascript
if (content) {
    content.innerHTML = this._renderSessionContent(text);
    content.dataset.rawText = text;
}
```

Sửa Copy handler (dòng 154-155):
```javascript
// Trước:
const content = document.getElementById('session-viewer-content')?.textContent || '';

// Sau:
const el = document.getElementById('session-viewer-content');
const content = el?.dataset.rawText || el?.textContent || '';
```

- [ ] **Step 5: Verify build không lỗi**

Run: mở app, vào Sessions view, click 1 session, kiểm tra hiển thị.

- [ ] **Step 6: Commit**

```bash
git add src/js/app.js
git commit -m "fix(sessions): render transcript content as formatted HTML instead of raw markdown"
```

---

### Task 2: Thêm CSS styles cho session content

**Files:**
- Modify: `src/styles/main.css:1697` (sau `.session-content-scroll` scrollbar styles)

- [ ] **Step 1: Thêm CSS cho metadata card và transcript segments**

Thêm sau dòng 1698 (sau scrollbar-thumb rule):

```css
/* Session content: metadata */
.session-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding-bottom: 12px;
  margin-bottom: 12px;
  border-bottom: 1px solid var(--border-light);
}

.session-meta-chip {
  font-size: 11px;
  color: var(--text-secondary);
  background: var(--bg-hover);
  border-radius: var(--radius-sm);
  padding: 2px 8px;
}

/* Session content: transcript */
.session-segments {
  display: flex;
  flex-direction: column;
}

.session-speaker {
  color: var(--accent);
  font-weight: 600;
  font-size: 12px;
  margin-top: 14px;
  margin-bottom: 2px;
}

.session-speaker:first-child {
  margin-top: 0;
}

.session-original {
  color: var(--text-secondary);
  font-size: 13px;
  font-style: italic;
  padding-left: 8px;
  border-left: 2px solid var(--border-light);
  margin: 2px 0;
}

.session-translation {
  color: var(--text-primary);
  font-size: 13px;
  margin: 2px 0 8px 0;
}
```

- [ ] **Step 2: Verify visual result**

Mở app, kiểm tra:
- Metadata chips hiển thị gọn ở trên
- Speaker labels có màu accent, bold
- Original text nghiêng, có border-left
- Translation text rõ ràng, spacing tốt

- [ ] **Step 3: Commit**

```bash
git add src/styles/main.css
git commit -m "style(sessions): add CSS for formatted session transcript viewer"
```

---

### Task 3: Cải thiện session list hiển thị metadata từ frontmatter

**Files:**
- Modify: `src-tauri/src/commands/transcript.rs:54-101`
- Modify: `src/js/app.js:1850-1897`

Hiện tại `_parseSessionMeta` chỉ parse filename, không đọc frontmatter. Kết quả: duration và langPair luôn empty trong list view.

- [ ] **Step 1: Thêm metadata fields vào TranscriptEntry (Rust)**

Sửa `TranscriptEntry` struct:

```rust
#[derive(Serialize)]
pub struct TranscriptEntry {
    filename: String,
    path: String,
    created_at: String,
    size_bytes: u64,
    duration: Option<String>,
    source_lang: Option<String>,
    target_lang: Option<String>,
}
```

- [ ] **Step 2: Parse frontmatter trong `list_transcripts`**

Trong closure `filter_map`, sau khi tạo `created_at`, thêm frontmatter parsing:

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

Some(TranscriptEntry {
    filename,
    path,
    created_at,
    size_bytes,
    duration,
    source_lang,
    target_lang,
})
```

- [ ] **Step 3: Cập nhật `_parseSessionMeta` trong JS**

Sửa method:

```javascript
_parseSessionMeta(session) {
    const parts = (session.created_at || '').split(' ');
    const date = parts[0] || '';
    const time = parts[1] ? parts[1].slice(0, 5) : '';
    const duration = session.duration || '';
    const langPair = (session.source_lang && session.target_lang)
        ? `${session.source_lang} → ${session.target_lang}`
        : '';
    return { date, time, duration, langPair };
}
```

- [ ] **Step 4: Verify list view hiển thị metadata**

Mở app, vào Sessions view, kiểm tra list items hiển thị duration và language pair.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/transcript.rs src/js/app.js
git commit -m "feat(sessions): show duration and language pair in session list"
```
