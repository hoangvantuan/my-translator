# Speaker-Aware Transcript UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Thêm color-coded border bên trái phân biệt speaker trong transcript UI, tự kích hoạt khi ≥2 speakers.

**Architecture:** Truyền `speaker`/`language` từ soniox → app → ui qua `onTranslation` callback. UI đếm unique speakers, gán CSS class `speaker-border-{N}` cho `seg-block`/`seg-text`. Palette 4 màu cố định + 1 default. `_speakerMap` quản lý mapping speaker→index, reset khi `clear()`/`showPlaceholder()`.

**Tech Stack:** Vanilla JS (ES modules), CSS custom properties, node:test + JSDOM.

---

### Task 1: CSS speaker border styles

**Files:**
- Modify: `src/styles/main.css:625-664`

- [ ] **Step 1: Thêm CSS custom properties và border classes**

Thêm ngay sau `.speaker-label` block (sau line 635), trước `.lang-badge`:

```css
/* Speaker color palette */
:root {
    --speaker-1: #6366f1;
    --speaker-2: #10b981;
    --speaker-3: #f59e0b;
    --speaker-4: #f43f5e;
    --speaker-default: #64748b;
}

/* Speaker color borders — active when ≥2 speakers */
.speaker-border-1 { border-left: 3px solid var(--speaker-1); padding-left: 8px; }
.speaker-border-2 { border-left: 3px solid var(--speaker-2); padding-left: 8px; }
.speaker-border-3 { border-left: 3px solid var(--speaker-3); padding-left: 8px; }
.speaker-border-4 { border-left: 3px solid var(--speaker-4); padding-left: 8px; }
.speaker-border-default { border-left: 3px solid var(--speaker-default); padding-left: 8px; }
```

- [ ] **Step 2: Cập nhật `.speaker-label` dùng speaker color khi có border**

Thêm ngay sau block border classes:

```css
/* Speaker label color matches border */
.speaker-border-1 .speaker-label { color: var(--speaker-1); }
.speaker-border-2 .speaker-label { color: var(--speaker-2); }
.speaker-border-3 .speaker-label { color: var(--speaker-3); }
.speaker-border-4 .speaker-label { color: var(--speaker-4); }
.speaker-border-default .speaker-label { color: var(--speaker-default); }
```

- [ ] **Step 3: Cập nhật `.lang-badge` color theo speaker**

Thêm tiếp:

```css
/* Lang badge color matches speaker */
.speaker-border-1 .lang-badge {
    background: rgba(99, 102, 241, 0.1);
    color: var(--speaker-1);
    border-color: rgba(99, 102, 241, 0.3);
}
.speaker-border-2 .lang-badge {
    background: rgba(16, 185, 129, 0.1);
    color: var(--speaker-2);
    border-color: rgba(16, 185, 129, 0.3);
}
.speaker-border-3 .lang-badge {
    background: rgba(245, 158, 11, 0.1);
    color: var(--speaker-3);
    border-color: rgba(245, 158, 11, 0.3);
}
.speaker-border-4 .lang-badge {
    background: rgba(244, 63, 94, 0.1);
    color: var(--speaker-4);
    border-color: rgba(244, 63, 94, 0.3);
}
.speaker-border-default .lang-badge {
    background: rgba(100, 116, 139, 0.1);
    color: var(--speaker-default);
    border-color: rgba(100, 116, 139, 0.3);
}
```

- [ ] **Step 4: Commit**

```bash
git add src/styles/main.css
git commit -m "feat(ui): add speaker color palette CSS variables and border classes"
```

---

### Task 2: soniox.js truyền speaker/language vào onTranslation

**Files:**
- Modify: `src/js/soniox.js:333`

- [ ] **Step 1: Viết test xác nhận onTranslation nhận speaker và language**

Tạo file `tests/soniox-translation-callback.test.mjs`:

```javascript
import assert from 'node:assert/strict';
import test from 'node:test';

test('_handleResponse passes speaker and language to onTranslation', () => {
    // Simulate the callback contract: onTranslation receives (text, speaker, language)
    let captured = null;

    const fakeClient = {
        onOriginal: null,
        onTranslation: (text, speaker, language) => {
            captured = { text, speaker, language };
        },
        onProvisional: null,
        onConfidence: null,
        _addToHistory: () => {},
    };

    // Simulate what _handleResponse does with translation tokens
    // This tests the CONTRACT, not the internals
    const translationText = 'Xin chào';
    const speaker = '1';
    const language = 'ja';

    fakeClient.onTranslation(translationText, speaker, language);

    assert.deepStrictEqual(captured, {
        text: 'Xin chào',
        speaker: '1',
        language: 'ja',
    });
});
```

- [ ] **Step 2: Chạy test, confirm pass**

```bash
node --test tests/soniox-translation-callback.test.mjs
```

Expected: PASS (test chỉ validate contract shape)

- [ ] **Step 3: Sửa soniox.js line 333**

Hiện tại:
```javascript
this.onTranslation?.(translationText);
```

Đổi thành:
```javascript
this.onTranslation?.(translationText, speaker, language);
```

`speaker` và `language` đã được extract từ tokens ở line 288-294, sẵn có trong scope.

- [ ] **Step 4: Commit**

```bash
git add src/js/soniox.js tests/soniox-translation-callback.test.mjs
git commit -m "feat(soniox): pass speaker and language to onTranslation callback"
```

---

### Task 3: app.js cập nhật onTranslation callback signature

**Files:**
- Modify: `src/js/app.js:569-572`

- [ ] **Step 1: Sửa callback signature**

Hiện tại (line 569-572):
```javascript
sonioxClient.onTranslation = (text) => {
    this.transcriptUI.addTranslation(text);
    this._speakIfEnabled(text);
};
```

Đổi thành:
```javascript
sonioxClient.onTranslation = (text, speaker, language) => {
    this.transcriptUI.addTranslation(text, speaker, language);
    this._speakIfEnabled(text);
};
```

- [ ] **Step 2: Commit**

```bash
git add src/js/app.js
git commit -m "feat(app): forward speaker and language from onTranslation to transcriptUI"
```

---

### Task 4: ui.js addTranslation nhận speaker/language

**Files:**
- Modify: `src/js/ui.js:119-144`
- Test: `tests/transcript-ui-speaker.test.mjs`

- [ ] **Step 1: Viết test cho addTranslation với speaker/language**

Tạo file `tests/transcript-ui-speaker.test.mjs`:

```javascript
import assert from 'node:assert/strict';
import test from 'node:test';
import { JSDOM } from 'jsdom';
import { TranscriptUI } from '../src/js/ui.js';

function setupDom() {
    const dom = new JSDOM('<!doctype html><body><div id="overlay-view"></div></body>');
    global.document = dom.window.document;
    global.window = dom.window;
    global.requestAnimationFrame = (cb) => { cb(); return 1; };

    const scrollHost = document.createElement('div');
    const container = document.createElement('div');
    scrollHost.appendChild(container);
    document.body.appendChild(scrollHost);
    return { container };
}

test('addTranslation matches existing original segment (speaker already set)', () => {
    const { container } = setupDom();
    const ui = new TranscriptUI(container);

    ui.addOriginal('こんにちは', '1', 'ja');
    ui.addTranslation('Hello', '1', 'ja');

    assert.equal(ui.segments.length, 1);
    assert.equal(ui.segments[0].translation, 'Hello');
    assert.equal(ui.segments[0].speaker, '1');
    assert.equal(ui.segments[0].language, 'ja');
    assert.equal(ui.segments[0].status, 'translated');
});

test('addTranslation creates new segment with speaker/language when no original match', () => {
    const { container } = setupDom();
    const ui = new TranscriptUI(container);

    ui.addTranslation('Orphan translation', '2', 'en');

    assert.equal(ui.segments.length, 1);
    assert.equal(ui.segments[0].speaker, '2');
    assert.equal(ui.segments[0].language, 'en');
    assert.equal(ui.segments[0].translation, 'Orphan translation');
    assert.equal(ui.segments[0].original, '');
});

test('addTranslation without speaker/language still works (backward compat)', () => {
    const { container } = setupDom();
    const ui = new TranscriptUI(container);

    ui.addOriginal('テスト', '1', 'ja');
    ui.addTranslation('Test');

    assert.equal(ui.segments[0].translation, 'Test');
    assert.equal(ui.segments[0].speaker, '1');
});
```

- [ ] **Step 2: Chạy test, confirm fail**

```bash
node --test tests/transcript-ui-speaker.test.mjs
```

Expected: FAIL ở test 2 (orphan segment không có speaker/language vì `addTranslation` chưa nhận params).

- [ ] **Step 3: Sửa addTranslation**

Hiện tại (line 119-144):
```javascript
addTranslation(text) {
    const seg = this.segments.find(s => s.status === 'original');
    if (seg) {
        seg.translation = text;
        seg.status = 'translated';
        const logSeg = this.sessionLog.find(
            s => s.status === 'original' && s.createdAt === seg.createdAt
        );
        if (logSeg) {
            logSeg.translation = text;
            logSeg.status = 'translated';
        }
    } else {
        const newSeg = {
            original: '',
            translation: text,
            status: 'translated',
            speaker: null,
            createdAt: Date.now(),
        };
        this.segments.push(newSeg);
        this.sessionLog.push({ ...newSeg });
    }
    this._render();
}
```

Đổi thành:
```javascript
addTranslation(text, speaker = null, language = null) {
    const seg = this.segments.find(s => s.status === 'original');
    if (seg) {
        seg.translation = text;
        seg.status = 'translated';
        const logSeg = this.sessionLog.find(
            s => s.status === 'original' && s.createdAt === seg.createdAt
        );
        if (logSeg) {
            logSeg.translation = text;
            logSeg.status = 'translated';
        }
    } else {
        const newSeg = {
            original: '',
            translation: text,
            status: 'translated',
            speaker: speaker || null,
            language: language || null,
            createdAt: Date.now(),
        };
        this.segments.push(newSeg);
        this.sessionLog.push({ ...newSeg });
    }
    this._render();
}
```

Thay đổi:
- Signature: `(text)` → `(text, speaker = null, language = null)`
- Fallback segment: thêm `language: language || null`

- [ ] **Step 4: Chạy test, confirm pass**

```bash
node --test tests/transcript-ui-speaker.test.mjs
```

Expected: 3/3 PASS

- [ ] **Step 5: Chạy toàn bộ test suite**

```bash
node --test tests/*.test.mjs
```

Expected: tất cả pass (backward compat với test cũ `transcript-ui-render-queue.test.mjs` gọi `addTranslation('Xin chào')` không args).

- [ ] **Step 6: Commit**

```bash
git add src/js/ui.js tests/transcript-ui-speaker.test.mjs
git commit -m "feat(ui): addTranslation accepts speaker and language parameters"
```

---

### Task 5: ui.js _speakerIndex helper và reset logic

**Files:**
- Modify: `src/js/ui.js` (thêm method mới, sửa `clear()` và `showPlaceholder()`)
- Test: `tests/transcript-ui-speaker.test.mjs` (thêm test cases)

- [ ] **Step 1: Thêm test cho _speakerIndex**

Append vào `tests/transcript-ui-speaker.test.mjs`:

```javascript
test('_speakerIndex assigns sequential indices starting from 1', () => {
    const { container } = setupDom();
    const ui = new TranscriptUI(container);

    assert.equal(ui._speakerIndex('1'), 1);
    assert.equal(ui._speakerIndex('2'), 2);
    assert.equal(ui._speakerIndex('1'), 1); // same speaker, same index
    assert.equal(ui._speakerIndex('3'), 3);
    assert.equal(ui._speakerIndex('4'), 4);
    assert.equal(ui._speakerIndex('5'), 'default'); // 5th+ speaker
});

test('_speakerIndex resets after clear()', () => {
    const { container } = setupDom();
    const ui = new TranscriptUI(container);

    ui._speakerIndex('1');
    ui._speakerIndex('2');
    ui.clear();

    assert.equal(ui._speakerIndex('1'), 1); // re-assigned from 1
});

test('_speakerIndex resets after showPlaceholder()', () => {
    const { container } = setupDom();
    const ui = new TranscriptUI(container);

    ui._speakerIndex('1');
    ui._speakerIndex('2');
    ui.showPlaceholder();

    assert.equal(ui._speakerIndex('3'), 1); // new speaker gets index 1
});
```

- [ ] **Step 2: Chạy test, confirm fail**

```bash
node --test tests/transcript-ui-speaker.test.mjs
```

Expected: FAIL (`_speakerIndex` not defined)

- [ ] **Step 3: Thêm `_speakerIndex` method**

Thêm sau `_ensureContent()` method (sau line 385):

```javascript
_speakerIndex(speaker) {
    if (!this._speakerMap) this._speakerMap = new Map();
    if (!this._speakerMap.has(speaker)) {
        this._speakerMap.set(speaker, this._speakerMap.size + 1);
    }
    const idx = this._speakerMap.get(speaker);
    return idx <= 4 ? idx : 'default';
}
```

- [ ] **Step 4: Reset `_speakerMap` trong `clear()`**

Thêm `this._speakerMap = null;` vào method `clear()` (line 356-367), sau `this.lastConfidence = null;`:

```javascript
clear() {
    this._cancelScheduledRender();
    this.container.innerHTML = '';
    this.segments = [];
    this.provisionalText = '';
    this.provisionalSpeaker = null;
    this.provisionalLanguage = null;
    this.currentSpeaker = null;
    this.currentLanguage = null;
    this.lastConfidence = null;
    this._speakerMap = null;
    this.contentEl = null;
}
```

- [ ] **Step 5: Reset `_speakerMap` trong `showPlaceholder()`**

Thêm `this._speakerMap = null;` vào `showPlaceholder()` (line 178-200), sau `this.lastConfidence = null;`:

```javascript
showPlaceholder() {
    this._cancelScheduledRender();
    this.container.innerHTML = `
  <div class="transcript-placeholder">
    <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.4">
      <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"/>
      <path d="M19 10v2a7 7 0 0 1-14 0v-2"/>
      <line x1="12" y1="19" x2="12" y2="23"/>
      <line x1="8" y1="23" x2="16" y2="23"/>
    </svg>
    <p>Press ▶ to start translating</p>
    <p class="shortcut-hint">⌘ Enter</p>
  </div>
`;
    this.segments = [];
    this.provisionalText = '';
    this.provisionalSpeaker = null;
    this.provisionalLanguage = null;
    this.currentSpeaker = null;
    this.currentLanguage = null;
    this.lastConfidence = null;
    this._speakerMap = null;
    this.contentEl = null;
}
```

- [ ] **Step 6: Chạy test, confirm pass**

```bash
node --test tests/transcript-ui-speaker.test.mjs
```

Expected: 6/6 PASS

- [ ] **Step 7: Commit**

```bash
git add src/js/ui.js tests/transcript-ui-speaker.test.mjs
git commit -m "feat(ui): add _speakerIndex helper with reset on clear/showPlaceholder"
```

---

### Task 6: ui.js _renderSingle() color-coded border

**Files:**
- Modify: `src/js/ui.js:426-470` (`_renderSingle`)
- Test: `tests/transcript-ui-speaker.test.mjs` (thêm test)

- [ ] **Step 1: Thêm test cho speaker border rendering**

Append vào `tests/transcript-ui-speaker.test.mjs`:

```javascript
test('_renderSingle adds speaker-border class when ≥2 speakers', () => {
    const { container } = setupDom();
    const ui = new TranscriptUI(container);
    ui.transcriptOnly = true;

    ui.addOriginal('Hello', '1', 'en');
    ui.addOriginal('Bonjour', '2', 'fr');

    const blocks = container.querySelectorAll('.seg-block');
    assert.equal(blocks.length, 2);
    assert.ok(blocks[0].classList.contains('speaker-border-1'));
    assert.ok(blocks[1].classList.contains('speaker-border-2'));
});

test('_renderSingle no border class when only 1 speaker', () => {
    const { container } = setupDom();
    const ui = new TranscriptUI(container);
    ui.transcriptOnly = true;

    ui.addOriginal('Hello', '1', 'en');
    ui.addOriginal('World', '1', 'en');

    const blocks = container.querySelectorAll('.seg-block');
    assert.equal(blocks.length, 2);
    assert.ok(!blocks[0].classList.contains('speaker-border-1'));
    assert.ok(!blocks[1].classList.contains('speaker-border-1'));
});

test('_renderSingle provisional text gets speaker border in multi-speaker', () => {
    const { container } = setupDom();
    const ui = new TranscriptUI(container);
    ui.transcriptOnly = true;

    ui.addOriginal('Hello', '1', 'en');
    ui.addOriginal('Bonjour', '2', 'fr');
    ui.setProvisional('Hola', '2', 'es');

    const blocks = container.querySelectorAll('.seg-block');
    assert.equal(blocks.length, 3);
    assert.ok(blocks[2].classList.contains('speaker-border-2'));
});
```

- [ ] **Step 2: Chạy test, confirm fail**

```bash
node --test tests/transcript-ui-speaker.test.mjs
```

Expected: FAIL (no `speaker-border-*` classes in rendered HTML)

- [ ] **Step 3: Sửa `_renderSingle()`**

Thay toàn bộ `_renderSingle()` (line 426-470):

```javascript
_renderSingle() {
    const uniqueSpeakers = new Set(
        this.segments.map(s => s.speaker).filter(Boolean)
    );
    const multiSpeaker = uniqueSpeakers.size >= 2;

    let html = '';
    let lastRenderedSpeaker = null;
    let lastRenderedLang = null;

    for (const seg of this.segments) {
        const borderClass = (multiSpeaker && seg.speaker)
            ? ` speaker-border-${this._speakerIndex(seg.speaker)}`
            : '';

        if (seg.speaker && seg.speaker !== lastRenderedSpeaker) {
            html += `<span class="speaker-label">Speaker ${seg.speaker}:</span> `;
            lastRenderedSpeaker = seg.speaker;
        }

        if (seg.language && seg.language !== lastRenderedLang) {
            html += `<span class="lang-badge">${this._langEmoji(seg.language)}</span> `;
            lastRenderedLang = seg.language;
        }

        if (seg.status === 'translated' && seg.translation) {
            const confidenceClass = (seg.confidence !== null && seg.confidence < 0.7) ? ' low-confidence' : '';
            html += `<div class="seg-block${borderClass}">`;
            html += `<div class="seg-translated${confidenceClass}">${this._esc(seg.translation)}</div>`;
            if (this.showOriginal === 'below' && seg.original) {
                html += `<div class="seg-original">${this._esc(seg.original)}</div>`;
            }
            html += `</div>`;
        } else if (this.transcriptOnly && seg.status === 'original' && seg.original) {
            const confidenceClass = (seg.confidence !== null && seg.confidence < 0.7) ? ' low-confidence' : '';
            html += `<div class="seg-block${borderClass}">`;
            html += `<div class="seg-translated${confidenceClass}">${this._esc(seg.original)}</div>`;
            html += `</div>`;
        }
    }

    if (this.provisionalText) {
        const provBorderClass = (multiSpeaker && this.provisionalSpeaker)
            ? ` speaker-border-${this._speakerIndex(this.provisionalSpeaker)}`
            : '';

        if (this.provisionalSpeaker && this.provisionalSpeaker !== lastRenderedSpeaker) {
            html += `<span class="speaker-label">Speaker ${this.provisionalSpeaker}:</span> `;
        }
        if (this.provisionalLanguage && this.provisionalLanguage !== lastRenderedLang) {
            html += `<span class="lang-badge">${this._langEmoji(this.provisionalLanguage)}</span> `;
        }
        html += `<div class="seg-block${provBorderClass}"><div class="seg-provisional">${this._esc(this.provisionalText)}</div></div>`;
    }

    this.contentEl.innerHTML = html;
    this._smartScroll(this.container.parentElement || this.container);
}
```

Thay đổi so với gốc:
- Thêm `uniqueSpeakers` + `multiSpeaker` check ở đầu
- Mỗi `seg-block` div nhận `borderClass` khi multi-speaker
- Provisional block cũng nhận border class

- [ ] **Step 4: Chạy test, confirm pass**

```bash
node --test tests/transcript-ui-speaker.test.mjs
```

Expected: 9/9 PASS

- [ ] **Step 5: Chạy toàn bộ test suite**

```bash
node --test tests/*.test.mjs
```

Expected: tất cả pass

- [ ] **Step 6: Commit**

```bash
git add src/js/ui.js tests/transcript-ui-speaker.test.mjs
git commit -m "feat(ui): _renderSingle adds color-coded speaker borders"
```

---

### Task 7: ui.js _renderDual() color-coded border

**Files:**
- Modify: `src/js/ui.js:472-542` (`_renderDual`)
- Test: `tests/transcript-ui-speaker.test.mjs` (thêm test)

- [ ] **Step 1: Thêm test cho dual view speaker border**

Append vào `tests/transcript-ui-speaker.test.mjs`:

```javascript
test('_renderDual adds speaker-border class to seg-text when ≥2 speakers', () => {
    const { container } = setupDom();
    const ui = new TranscriptUI(container);
    ui.showOriginal = 'dual';

    ui.addOriginal('こんにちは', '1', 'ja');
    ui.addTranslation('Hello', '1', 'ja');
    ui.addOriginal('Bonjour', '2', 'fr');
    ui.addTranslation('Xin chào', '2', 'fr');

    const srcTexts = container.querySelectorAll('.panel-source .seg-text');
    const tgtTexts = container.querySelectorAll('.panel-translation .seg-text');

    assert.equal(srcTexts.length, 2);
    assert.ok(srcTexts[0].classList.contains('speaker-border-1'));
    assert.ok(srcTexts[1].classList.contains('speaker-border-2'));
    assert.ok(tgtTexts[0].classList.contains('speaker-border-1'));
    assert.ok(tgtTexts[1].classList.contains('speaker-border-2'));
});

test('_renderDual no border when only 1 speaker', () => {
    const { container } = setupDom();
    const ui = new TranscriptUI(container);
    ui.showOriginal = 'dual';

    ui.addOriginal('Hello', '1', 'en');
    ui.addTranslation('Xin chào', '1', 'en');

    const srcTexts = container.querySelectorAll('.panel-source .seg-text');
    assert.equal(srcTexts.length, 1);
    assert.ok(!srcTexts[0].classList.contains('speaker-border-1'));
});
```

- [ ] **Step 2: Chạy test, confirm fail**

```bash
node --test tests/transcript-ui-speaker.test.mjs
```

Expected: FAIL (no `speaker-border-*` trên `.seg-text`)

- [ ] **Step 3: Sửa `_renderDual()`**

Thay phần loop và HTML generation trong `_renderDual()` (line 479-514). Giữ nguyên scroll logic ở đầu (line 472-478) và cuối (line 516-541).

```javascript
_renderDual() {
    const oldSrcPanel = this.contentEl.querySelector('.panel-source');
    const oldTgtPanel = this.contentEl.querySelector('.panel-translation');
    const srcScrollState = oldSrcPanel ? this._getScrollState(oldSrcPanel) : { nearBottom: true, scrollTop: 0 };
    const tgtScrollState = oldTgtPanel ? this._getScrollState(oldTgtPanel) : { nearBottom: true, scrollTop: 0 };

    const uniqueSpeakers = new Set(
        this.segments.map(s => s.speaker).filter(Boolean)
    );
    const multiSpeaker = uniqueSpeakers.size >= 2;

    let srcHtml = '';
    let tgtHtml = '';
    let lastSpeaker = null;
    let lastLang = null;

    for (const seg of this.segments) {
        const borderClass = (multiSpeaker && seg.speaker)
            ? ` speaker-border-${this._speakerIndex(seg.speaker)}`
            : '';

        let speakerHtml = '';
        if (seg.speaker && seg.speaker !== lastSpeaker) {
            speakerHtml = `<div class="speaker-label">Speaker ${seg.speaker}:</div>`;
            lastSpeaker = seg.speaker;
        }

        let langHtml = '';
        if (seg.language && seg.language !== lastLang) {
            langHtml = `<span class="lang-badge">${this._langEmoji(seg.language)}</span> `;
            lastLang = seg.language;
        }

        if (seg.status === 'translated' && seg.translation) {
            const confidenceClass = (seg.confidence !== null && seg.confidence < 0.7) ? ' low-confidence' : '';
            srcHtml += speakerHtml + langHtml;
            srcHtml += `<div class="seg-text${borderClass}">${this._esc(seg.original || '')}</div>`;
            tgtHtml += speakerHtml ? '<div class="speaker-label">&nbsp;</div>' : '';
            tgtHtml += `<div class="seg-text${borderClass}${confidenceClass}">${this._esc(seg.translation)}</div>`;
        } else if (seg.status === 'original' && seg.original) {
            srcHtml += speakerHtml + langHtml;
            srcHtml += `<div class="seg-text${borderClass} pending">${this._esc(seg.original)}</div>`;
            tgtHtml += speakerHtml ? '<div class="speaker-label">&nbsp;</div>' : '';
            tgtHtml += `<div class="seg-text${borderClass} pending">...</div>`;
        }
    }

    if (this.provisionalText) {
        const provBorderClass = (multiSpeaker && this.provisionalSpeaker)
            ? ` speaker-border-${this._speakerIndex(this.provisionalSpeaker)}`
            : '';
        srcHtml += `<div class="seg-text${provBorderClass} pending">${this._esc(this.provisionalText)}</div>`;
        tgtHtml += `<div class="seg-text${provBorderClass} pending">...</div>`;
    }

    this.contentEl.innerHTML = `
        <div class="panel-source">${srcHtml}</div>
        <div class="panel-translation">${tgtHtml}</div>
    `;

    const srcPanel = this.contentEl.querySelector('.panel-source');
    const tgtPanel = this.contentEl.querySelector('.panel-translation');
    if (srcPanel) {
        if (srcScrollState.nearBottom) {
            srcPanel.scrollTop = srcPanel.scrollHeight;
        } else {
            srcPanel.scrollTop = srcScrollState.scrollTop;
        }
    }
    if (tgtPanel) {
        if (tgtScrollState.nearBottom) {
            tgtPanel.scrollTop = tgtPanel.scrollHeight;
        } else {
            tgtPanel.scrollTop = tgtScrollState.scrollTop;
        }
    }

    if (srcPanel && tgtPanel) {
        this._setupScrollSync(srcPanel, tgtPanel);
    }
}
```

Thay đổi so với gốc:
- Thêm `uniqueSpeakers` + `multiSpeaker` check
- Mỗi `.seg-text` div nhận `borderClass`
- Provisional text cũng nhận border class

- [ ] **Step 4: Chạy test, confirm pass**

```bash
node --test tests/transcript-ui-speaker.test.mjs
```

Expected: 11/11 PASS

- [ ] **Step 5: Chạy toàn bộ test suite**

```bash
node --test tests/*.test.mjs
```

Expected: tất cả pass

- [ ] **Step 6: Commit**

```bash
git add src/js/ui.js tests/transcript-ui-speaker.test.mjs
git commit -m "feat(ui): _renderDual adds color-coded speaker borders"
```

---

### Task 8: Integration test và manual verification

**Files:**
- Test: `tests/transcript-ui-speaker.test.mjs` (thêm integration test)

- [ ] **Step 1: Thêm integration test: full flow 2 speakers**

Append vào `tests/transcript-ui-speaker.test.mjs`:

```javascript
test('integration: full 2-speaker flow with translation', () => {
    const { container } = setupDom();
    const ui = new TranscriptUI(container);

    // Speaker 1 nói tiếng Nhật
    ui.addOriginal('こんにちは', '1', 'ja');
    ui.addTranslation('Xin chào', '1', 'ja');

    // Speaker 2 nói tiếng Anh
    ui.addOriginal('Hello', '2', 'en');
    ui.addTranslation('Xin chào bạn', '2', 'en');

    // Speaker 1 nói tiếp
    ui.addOriginal('元気ですか', '1', 'ja');
    ui.addTranslation('Khỏe không?', '1', 'ja');

    assert.equal(ui.segments.length, 3);

    // All segments have correct speaker
    assert.equal(ui.segments[0].speaker, '1');
    assert.equal(ui.segments[1].speaker, '2');
    assert.equal(ui.segments[2].speaker, '1');

    // Rendered with borders
    const blocks = container.querySelectorAll('.seg-block');
    assert.equal(blocks.length, 3);
    assert.ok(blocks[0].classList.contains('speaker-border-1'));
    assert.ok(blocks[1].classList.contains('speaker-border-2'));
    assert.ok(blocks[2].classList.contains('speaker-border-1'));
});

test('integration: single speaker has no borders', () => {
    const { container } = setupDom();
    const ui = new TranscriptUI(container);

    ui.addOriginal('Hello', '1', 'en');
    ui.addTranslation('Xin chào');

    ui.addOriginal('World', '1', 'en');
    ui.addTranslation('Thế giới');

    const blocks = container.querySelectorAll('.seg-block');
    for (const block of blocks) {
        assert.ok(!block.className.includes('speaker-border'));
    }
});

test('integration: sessionLog preserves speaker through translation', () => {
    const { container } = setupDom();
    const ui = new TranscriptUI(container);

    ui.addOriginal('Test', '1', 'en');
    ui.addTranslation('Thử', '1', 'en');

    assert.equal(ui.sessionLog.length, 1);
    assert.equal(ui.sessionLog[0].speaker, '1');
    assert.equal(ui.sessionLog[0].language, 'en');
    assert.equal(ui.sessionLog[0].translation, 'Thử');
});
```

- [ ] **Step 2: Chạy toàn bộ test suite**

```bash
node --test tests/*.test.mjs
```

Expected: tất cả pass (bao gồm test cũ `transcript-ui-render-queue.test.mjs`)

- [ ] **Step 3: Commit**

```bash
git add tests/transcript-ui-speaker.test.mjs
git commit -m "test(ui): add integration tests for speaker-aware transcript flow"
```

- [ ] **Step 4: Test thủ công**

Chạy app, kiểm tra:

1. **Two-way mode, 2 ngôn ngữ**: Viền màu khác nhau cho mỗi speaker
2. **One-way mode, nhiều người nói**: Viền xuất hiện khi phát hiện speaker 2
3. **1 speaker duy nhất**: Không viền, UI giữ nguyên
4. **Clear/restart**: Viền reset, bắt đầu gán lại
5. **Transcript-only mode**: Viền vẫn hoạt động
6. **Dual view**: Cả 2 panel có viền matching

---

## Tổng kết files thay đổi

| File | Loại | Mô tả |
|------|------|-------|
| `src/js/soniox.js` | MODIFY | Line 333: thêm `speaker, language` vào `onTranslation` |
| `src/js/app.js` | MODIFY | Line 569-572: cập nhật callback signature |
| `src/js/ui.js` | MODIFY | `addTranslation` nhận params, `_speakerIndex` helper, `_renderSingle`/`_renderDual` thêm border class, reset trong `clear`/`showPlaceholder` |
| `src/styles/main.css` | MODIFY | CSS variables, border classes, speaker-label/lang-badge color matching |
| `tests/soniox-translation-callback.test.mjs` | CREATE | Test contract onTranslation |
| `tests/transcript-ui-speaker.test.mjs` | CREATE | Test speaker index, border rendering, integration |
