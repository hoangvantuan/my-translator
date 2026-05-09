# Pause/Resume Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Thêm trạng thái Paused phân biệt với Stop. Pause giữ transcript + session, Stop kết thúc session + auto-save.

**Architecture:** Thêm `isPaused` flag vào App class. Khi running, nút chính chuyển thành Pause (thay vì Stop). Khi paused, hiện overlay 2 nút Resume/Stop giữa màn hình. `start()` đã handle resume tự nhiên (skip clear nếu có content).

**Tech Stack:** Vanilla JS, CSS, HTML (Tauri app, không framework)

---

### Task 1: HTML — Thêm icon-pause SVG + paused-overlay

**Files:**
- Modify: `src/index.html:127-134` (btn-start icons)
- Modify: `src/index.html:144-147` (sau idle-overlay, thêm paused-overlay)

- [ ] **Step 1: Thêm icon-pause SVG vào btn-start**

Trong `src/index.html`, tìm block btn-start (dòng 127-134). Thêm icon-pause SVG sau icon-stop. icon-pause mặc định `display:none`.

Thay toàn bộ block btn-start:

```html
<button id="btn-start" class="toolbar-btn toolbar-btn-stop" title="Start (Space)">
  <svg id="icon-play" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
    <polygon points="5 3 19 12 5 21 5 3" fill="currentColor" />
  </svg>
  <svg id="icon-stop" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="display:none">
    <rect x="4" y="4" width="16" height="16" rx="2" fill="currentColor" />
  </svg>
  <svg id="icon-pause" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="display:none">
    <rect x="5" y="3" width="4" height="18" rx="1" fill="currentColor" />
    <rect x="15" y="3" width="4" height="18" rx="1" fill="currentColor" />
  </svg>
</button>
```

- [ ] **Step 2: Thêm paused-overlay HTML**

Ngay sau `idle-overlay` (dòng 147), thêm:

```html
<!-- Paused state overlay (shown when paused, Resume or Stop) -->
<div id="paused-overlay" class="paused-overlay hidden" data-tauri-drag-region>
  <div class="paused-buttons">
    <button id="btn-paused-resume" class="paused-btn paused-btn-resume">Tiếp tục</button>
    <button id="btn-paused-stop" class="paused-btn paused-btn-stop">Dừng lại</button>
  </div>
</div>
```

- [ ] **Step 3: Commit**

```bash
git add src/index.html
git commit -m "feat(html): add pause icon and paused-overlay with resume/stop buttons"
```

---

### Task 2: CSS — Style paused-overlay

**Files:**
- Modify: `src/styles/main.css:297-299` (sau idle-start-btn:active, trước floating toolbar)

- [ ] **Step 1: Thêm CSS cho paused-overlay**

Trong `src/styles/main.css`, chèn ngay sau `.idle-start-btn:active` (dòng 296-298) và trước comment `/* ══════════════ FLOATING TOOLBAR ══════════════ */` (dòng 300):

```css
/* Paused overlay */
.paused-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.3);
  backdrop-filter: blur(2px);
  z-index: 2;
}

.paused-overlay.hidden {
  display: none;
}

.paused-buttons {
  display: flex;
  gap: 16px;
  -webkit-app-region: no-drag;
}

.paused-btn {
  padding: 14px 32px;
  border: none;
  border-radius: var(--radius-lg);
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.paused-btn-resume {
  background: var(--accent);
  color: #fff;
}

.paused-btn-resume:hover {
  background: #4338ca;
  box-shadow: 0 2px 8px rgba(79, 70, 229, 0.3);
}

.paused-btn-resume:active {
  transform: scale(0.96);
}

.paused-btn-stop {
  background: rgba(255, 255, 255, 0.15);
  color: var(--text-primary);
}

.paused-btn-stop:hover {
  background: rgba(239, 68, 68, 0.8);
  color: #fff;
}

.paused-btn-stop:active {
  transform: scale(0.96);
}
```

- [ ] **Step 2: Commit**

```bash
git add src/styles/main.css
git commit -m "feat(css): add paused-overlay and button styles"
```

---

### Task 3: JS — Thêm isPaused flag + pause() method

**Files:**
- Modify: `src/js/app.js:35` (constructor, thêm isPaused)
- Modify: `src/js/app.js:1479-1524` (sau stop(), thêm pause())

- [ ] **Step 1: Thêm isPaused vào constructor**

Trong `src/js/app.js`, tìm dòng `this.isCompact = false;` (dòng 35 trong constructor). Thêm ngay sau:

```js
this.isPaused = false;    // Paused state (session alive, connections closed)
```

- [ ] **Step 2: Thêm method pause()**

Trong `src/js/app.js`, chèn method `pause()` ngay SAU method `stop()` (sau dòng 1524):

```js
async pause() {
    this.isRunning = false;
    this.isPaused = true;
    this._updateStartButton();

    // Stop audio capture
    try {
        await invoke('stop_capture');
    } catch (err) {
        console.error('Failed to stop audio capture:', err);
    }

    if (this.translationMode === 'local') {
        try {
            await invoke('stop_local_pipeline');
        } catch (err) {
            console.error('Failed to stop local pipeline:', err);
        }
        this.localPipelineReady = false;
        this.transcriptUI.removeStatusMessage();
    } else {
        sonioxClient.disconnect();
    }

    // Keep transcript visible — don't clear, don't save
    this.transcriptUI.clearProvisional();

    // Stop TTS
    elevenLabsTTS.disconnect();
    edgeTTSRust.disconnect();
    googleTTS.disconnect();
    audioPlayer.stop();

    this._updateStatus('paused');
}
```

- [ ] **Step 3: Commit**

```bash
git add src/js/app.js
git commit -m "feat(app): add isPaused flag and pause() method"
```

---

### Task 4: JS — Thêm resume() method + sửa stop()

**Files:**
- Modify: `src/js/app.js` (thêm resume() sau pause(), sửa stop())

- [ ] **Step 1: Thêm method resume()**

Chèn ngay SAU method `pause()`:

```js
async resume() {
    this.isPaused = false;
    this._updateStartButton();
    await this.start();
}
```

Giải thích: `start()` đã handle resume tự nhiên. Nó kiểm tra `this.transcriptUI.hasContent()` trước khi clear, nên transcript cũ giữ nguyên. `sessionStartTime` vẫn còn (pause không reset), nên session metadata tiếp tục.

- [ ] **Step 2: Sửa stop() — reset isPaused + ẩn paused-overlay**

Tìm method `stop()`. Thêm 2 dòng ở đầu method, ngay sau `this.isRunning = false;` (dòng 1481):

```js
this.isPaused = false;
```

Dòng `this._updateStartButton()` đã có sẵn ở dòng tiếp theo, nó sẽ tự ẩn paused-overlay (sẽ sửa ở Task 5).

- [ ] **Step 3: Commit**

```bash
git add src/js/app.js
git commit -m "feat(app): add resume() method, reset isPaused in stop()"
```

---

### Task 5: JS — Sửa _updateStartButton() cho 3 trạng thái

**Files:**
- Modify: `src/js/app.js:1526-1543` (_updateStartButton)

- [ ] **Step 1: Thay thế _updateStartButton()**

Tìm method `_updateStartButton()` (dòng 1526-1543). Thay toàn bộ body:

```js
_updateStartButton() {
    const btn = document.getElementById('btn-start');
    const iconPlay = document.getElementById('icon-play');
    const iconPause = document.getElementById('icon-pause');
    const iconStop = document.getElementById('icon-stop');
    const idleOverlay = document.getElementById('idle-overlay');
    const pausedOverlay = document.getElementById('paused-overlay');
    const overlayView = document.getElementById('overlay-view');

    // Hide all icons first
    if (iconPlay) iconPlay.style.display = 'none';
    if (iconPause) iconPause.style.display = 'none';
    if (iconStop) iconStop.style.display = 'none';

    if (this.isRunning) {
        // Running: show pause icon, hide both overlays
        if (iconPause) iconPause.style.display = 'block';
        if (btn) btn.classList.add('recording');
        if (idleOverlay) idleOverlay.classList.add('hidden');
        if (pausedOverlay) pausedOverlay.classList.add('hidden');
        if (overlayView) overlayView.classList.add('is-recording');
    } else if (this.isPaused) {
        // Paused: show play icon (ready to resume), show paused-overlay
        if (iconPlay) iconPlay.style.display = 'block';
        if (btn) btn.classList.remove('recording');
        if (idleOverlay) idleOverlay.classList.add('hidden');
        if (pausedOverlay) pausedOverlay.classList.remove('hidden');
        if (overlayView) overlayView.classList.remove('is-recording');
    } else {
        // Idle: show play icon, show idle-overlay
        if (iconPlay) iconPlay.style.display = 'block';
        if (btn) btn.classList.remove('recording');
        if (idleOverlay) idleOverlay.classList.remove('hidden');
        if (pausedOverlay) pausedOverlay.classList.add('hidden');
        if (overlayView) overlayView.classList.remove('is-recording');
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src/js/app.js
git commit -m "feat(app): update _updateStartButton for idle/running/paused states"
```

---

### Task 6: JS — Sửa _updateStatus() thêm case 'paused'

**Files:**
- Modify: `src/js/app.js:1587-1611` (_updateStatus)

- [ ] **Step 1: Thêm case 'paused' vào switch**

Tìm `_updateStatus()` (dòng 1587). Thêm case mới vào switch, ngay trước case `'error'`:

```js
case 'paused':
    dot.classList.add('connecting');  // reuse yellow/amber dot
    text.textContent = 'Paused';
    break;
```

- [ ] **Step 2: Commit**

```bash
git add src/js/app.js
git commit -m "feat(app): add paused status indicator"
```

---

### Task 7: JS — Sửa btn-start handler + bind paused-overlay buttons

**Files:**
- Modify: `src/js/app.js:288-309` (btn-start click handler)
- Modify: `src/js/app.js` (trong `_bindEvents()`, thêm paused-overlay button listeners)

- [ ] **Step 1: Sửa btn-start click handler**

Tìm block `// Start/Stop button` (dòng 288). Thay toàn bộ event listener:

```js
// Start/Pause/Resume button
document.getElementById('btn-start').addEventListener('click', async () => {
    if (this.isStarting) return;
    try {
        if (this.isRunning) {
            await this.pause();
        } else if (this.isPaused) {
            this.isStarting = true;
            await this.resume();
        } else {
            this.isStarting = true;
            await this.start();
        }
    } catch (err) {
        console.error('[App] Start/Pause/Resume error:', err);
        this._showToast(`Error: ${err}`, 'error');
        this.isRunning = false;
        this.isPaused = false;
        this._updateStartButton();
        this._updateStatus('error');
        this.transcriptUI.clear();
        this.transcriptUI.showPlaceholder();
    } finally {
        this.isStarting = false;
    }
});
```

- [ ] **Step 2: Thêm paused-overlay button listeners**

Tìm block `// Idle start button` (dòng 311). Chèn ngay SAU block idle-start button (sau dòng 333):

```js
// Paused overlay buttons
document.getElementById('btn-paused-resume').addEventListener('click', async () => {
    if (this.isStarting) return;
    try {
        this.isStarting = true;
        await this.resume();
    } catch (err) {
        console.error('[App] Resume error:', err);
        this._showToast(`Error: ${err}`, 'error');
        this.isRunning = false;
        this.isPaused = false;
        this._updateStartButton();
        this._updateStatus('error');
    } finally {
        this.isStarting = false;
    }
});

document.getElementById('btn-paused-stop').addEventListener('click', async () => {
    await this.stop();
});
```

- [ ] **Step 3: Commit**

```bash
git add src/js/app.js
git commit -m "feat(app): wire btn-start for pause/resume, bind paused-overlay buttons"
```

---

### Task 8: JS — Sửa keyboard shortcuts

**Files:**
- Modify: `src/js/app.js:555-610` (_bindKeyboardShortcuts)

- [ ] **Step 1: Thêm Space shortcut**

Tìm `_bindKeyboardShortcuts()` (dòng 555). Ngay sau block `Cmd/Ctrl + Enter` (dòng 584), thêm:

```js
// Space: Start/Pause/Resume
if (e.key === ' ' || e.code === 'Space') {
    e.preventDefault();
    if (this.isStarting) return;
    (async () => {
        try {
            if (this.isRunning) {
                await this.pause();
            } else if (this.isPaused) {
                this.isStarting = true;
                await this.resume();
            } else {
                this.isStarting = true;
                await this.start();
            }
        } catch (err) {
            console.error('[App] Space shortcut error:', err);
            this._showToast(`Error: ${err}`, 'error');
            this.isRunning = false;
            this.isPaused = false;
            this._updateStartButton();
            this._updateStatus('error');
        } finally {
            this.isStarting = false;
        }
    })();
}
```

- [ ] **Step 2: Sửa Cmd/Ctrl+Enter handler**

Tìm block `Cmd/Ctrl + Enter` (dòng 562-584). Thay logic bên trong:

```js
// Cmd/Ctrl + Enter: Start/Pause/Resume (same as Space)
if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
    e.preventDefault();
    if (this.isStarting) return;
    (async () => {
        try {
            if (this.isRunning) {
                await this.pause();
            } else if (this.isPaused) {
                this.isStarting = true;
                await this.resume();
            } else {
                this.isStarting = true;
                await this.start();
            }
        } catch (err) {
            console.error('[App] Keyboard start/pause error:', err);
            this._showToast(`Error: ${err}`, 'error');
            this.isRunning = false;
            this.isPaused = false;
            this._updateStartButton();
            this._updateStatus('error');
        } finally {
            this.isStarting = false;
        }
    })();
}
```

- [ ] **Step 3: Sửa Escape handler — thêm stop khi paused**

Tìm block `// Escape:` (dòng 586-593). Thay toàn bộ:

```js
// Escape: Stop session (when paused) or close settings
if (e.key === 'Escape') {
    e.preventDefault();
    if (this.isPaused) {
        (async () => await this.stop())();
    } else {
        const settingsVisible = document.getElementById('settings-view').classList.contains('active');
        if (settingsVisible) {
            this._showView('overlay');
        }
    }
}
```

- [ ] **Step 4: Commit**

```bash
git add src/js/app.js
git commit -m "feat(app): add Space shortcut, update Cmd+Enter and Escape for pause/resume"
```

---

### Task 9: Kiểm tra thủ công + sửa lỗi

**Files:** Không có file mới. Kiểm tra tất cả file đã sửa.

- [ ] **Step 1: Chạy dev server**

```bash
cd /Users/tuanhv/Desktop/git_projects/my-translator/.claude/worktrees/ui-redesign-light
npm run tauri dev
```

- [ ] **Step 2: Test flow chính**

Kiểm tra từng bước:

1. App khởi động → idle-overlay hiện, nút Play → OK
2. Bấm Play hoặc Space → running, icon chuyển Pause, idle-overlay ẩn → OK
3. Bấm Pause hoặc Space → paused, paused-overlay hiện 2 nút, transcript giữ nguyên → OK
4. Bấm "Tiếp tục" → resume, transcript tiếp tục nhận, paused-overlay ẩn → OK
5. Bấm Pause lần nữa → paused overlay hiện lại → OK
6. Bấm "Dừng lại" → stop, auto-save, idle-overlay hiện, transcript clear session → OK
7. Escape khi paused → stop → OK
8. Cmd+Enter cycle qua start/pause/resume → OK

- [ ] **Step 3: Test edge cases**

1. Đổi source (system/mic) khi paused rồi resume → dùng source mới
2. Mở Settings khi paused, thay đổi, đóng Settings rồi resume → dùng settings mới
3. Pause → Resume nhiều lần liên tiếp → transcript nối liền mạch, không mất data
4. Pause khi đang reconnect → disconnect ngay, không crash

- [ ] **Step 4: Commit bất kỳ fix nào**

```bash
git add -A
git commit -m "fix(app): address issues found during pause/resume testing"
```

Nếu không có fix, skip step này.
