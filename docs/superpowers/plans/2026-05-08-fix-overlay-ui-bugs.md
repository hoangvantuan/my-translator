# Fix Overlay UI Bugs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Sửa 4 bug UI overlay: window không drag được, thiếu window controls, nút pause không hoạt động, menu toolbar không hiển thị khi hover.

**Architecture:** Root cause chính là floating toolbar đặt ở `top: -52px` (ngoài window bounds bị clip), idle overlay che drag region, và thiếu window control buttons. Sửa bằng cách di chuyển toolbar vào trong window, thêm `data-tauri-drag-region` cho idle overlay, và thêm window controls.

**Tech Stack:** HTML/CSS/JS (Tauri v2 frontend)

---

## Root Cause Analysis

### Bug 1: Window không drag được khi mở app
- `#idle-overlay` có `position: absolute; inset: 0; z-index: 2` che toàn bộ overlay-view
- `#drag-region` (16px, top) bị che bởi idle overlay
- `#transcript-container` có `data-tauri-drag-region` nhưng cũng bị che
- **Fix:** Thêm `data-tauri-drag-region` vào `#idle-overlay`

### Bug 2: Thiếu close/minimize/fullscreen buttons
- `tauri.conf.json` có `decorations: false` → không có native title bar
- HTML có `btn-minimize`, `btn-close` nhưng nằm trong `div style="display:none"`
- Không có custom window controls nào visible
- **Fix:** Thêm window controls vào drag region

### Bug 3: Nút pause không hoạt động
- Thực chất là hệ quả của Bug 4: toolbar không hiển thị nên không click được
- `btn-start` handler hoạt động đúng (toggle start/stop)
- **Fix:** Khi toolbar hiển thị được (Bug 4), nút sẽ hoạt động

### Bug 4: Menu/toolbar không hiển thị khi hover
- `.floating-toolbar` có `top: -52px` → render NGOÀI window bounds
- Tauri window clip content ngoài bounds → toolbar bị ẩn hoàn toàn
- CSS rule `#overlay-view.is-recording:hover .floating-toolbar` đúng logic, nhưng toolbar ở ngoài window nên không bao giờ thấy
- **Fix:** Di chuyển toolbar vào trong window (top: 0 hoặc bên trong overlay)

## File Structure

**Modified files:**
- `src/index.html` — Thêm window controls vào drag region, thêm drag attribute cho idle overlay
- `src/styles/main.css` — Sửa toolbar position, style window controls
- `src/js/app.js` — Wire click handlers cho window controls mới

---

### Task 1: Fix Floating Toolbar Position (Bug 4 + Bug 3)

**Files:**
- Modify: `src/styles/main.css:240-262` (floating toolbar CSS)

Di chuyển toolbar từ ngoài window (`top: -52px`) vào trong window. Toolbar sẽ xuất hiện ở top bên trong overlay-view.

- [ ] **Step 1: Sửa floating toolbar positioning**

Trong `src/styles/main.css`, sửa `.floating-toolbar`:

```css
.floating-toolbar {
  position: absolute;
  top: 4px;                    /* Was: -52px (outside window) */
  left: 50%;
  display: flex;
  align-items: center;
  gap: var(--toolbar-gap-item);
  padding: var(--toolbar-padding);
  background: var(--bg-toolbar);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-toolbar);
  z-index: 50;
  opacity: 0;
  pointer-events: none;
  transition: opacity var(--transition-fast), transform var(--transition-fast);
  transform: translateX(-50%) translateY(-4px);  /* Slide down animation */
}

#overlay-view.is-recording:hover .floating-toolbar {
  opacity: 1;
  pointer-events: auto;
  transform: translateX(-50%) translateY(0);
}
```

Key change: `top: -52px` → `top: 4px`, giữ toolbar bên trong window bounds.

- [ ] **Step 2: Verify toolbar hiển thị khi hover (manual test)**

Build và test: khi app đang recording, hover vào overlay → toolbar phải hiển thị. Click btn-start (stop icon) → app phải dừng.

- [ ] **Step 3: Commit**

```bash
git add src/styles/main.css
git commit -m "fix(ui): move floating toolbar inside window bounds

Toolbar was positioned at top: -52px, rendering outside the Tauri
window bounds and getting clipped. Move to top: 4px so it appears
on hover during recording."
```

---

### Task 2: Fix Window Dragging When Idle (Bug 1)

**Files:**
- Modify: `src/index.html:133` (idle-overlay div)

Thêm `data-tauri-drag-region` vào idle overlay để toàn bộ area idle có thể drag. Button "Bắt đầu" đã có `-webkit-app-region: no-drag` nên không bị ảnh hưởng.

- [ ] **Step 1: Thêm data-tauri-drag-region vào idle-overlay**

Trong `src/index.html`, sửa dòng 133:

```html
<!-- Before -->
<div id="idle-overlay" class="idle-overlay">

<!-- After -->
<div id="idle-overlay" class="idle-overlay" data-tauri-drag-region>
```

- [ ] **Step 2: Verify drag hoạt động (manual test)**

Build và test: mở app ở trạng thái idle → click + drag trên vùng idle (ngoài nút "Bắt đầu") → window phải di chuyển. Click nút "Bắt đầu" → phải start recording, không drag.

- [ ] **Step 3: Commit**

```bash
git add src/index.html
git commit -m "fix(ui): enable window dragging in idle state

Add data-tauri-drag-region to idle overlay so users can drag the
window when the app is not recording. The start button already has
-webkit-app-region: no-drag."
```

---

### Task 3: Add Window Controls (Bug 2)

**Files:**
- Modify: `src/index.html:17-20` (drag region)
- Modify: `src/styles/main.css:105-125` (drag region CSS)
- Modify: `src/js/app.js:152-163` (wire up existing handlers)

Thêm 3 nút (close, minimize, fullscreen) vào drag region. macOS-style: dot buttons bên trái.

- [ ] **Step 1: Thêm window control buttons vào drag region HTML**

Trong `src/index.html`, thay thế drag-region block (dòng 18-20):

```html
<div id="drag-region" data-tauri-drag-region>
  <div class="window-controls">
    <button id="btn-wc-close" class="wc-btn wc-close" title="Close">
      <svg width="6" height="6" viewBox="0 0 6 6" fill="currentColor">
        <path d="M0.5 0.5L5.5 5.5M5.5 0.5L0.5 5.5" stroke="currentColor" stroke-width="1.2" fill="none"/>
      </svg>
    </button>
    <button id="btn-wc-minimize" class="wc-btn wc-minimize" title="Minimize">
      <svg width="6" height="2" viewBox="0 0 6 2" fill="currentColor">
        <line x1="0.5" y1="1" x2="5.5" y2="1" stroke="currentColor" stroke-width="1.2"/>
      </svg>
    </button>
    <button id="btn-wc-fullscreen" class="wc-btn wc-fullscreen" title="Fullscreen">
      <svg width="6" height="6" viewBox="0 0 6 6" fill="none">
        <path d="M0.5 3.5V5.5H2.5M5.5 2.5V0.5H3.5" stroke="currentColor" stroke-width="1"/>
      </svg>
    </button>
  </div>
  <div class="drag-handle" data-tauri-drag-region></div>
</div>
```

- [ ] **Step 2: Thêm CSS cho window controls**

Trong `src/styles/main.css`, thêm sau block `#drag-region` (sau dòng 112):

```css
/* Window Controls (macOS-style dots) */
.window-controls {
  position: absolute;
  left: 8px;
  top: 50%;
  transform: translateY(-50%);
  display: flex;
  align-items: center;
  gap: 6px;
  opacity: 0;
  transition: opacity var(--transition-fast);
  z-index: 10;
}

#overlay-view:hover .window-controls {
  opacity: 1;
}

.wc-btn {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  -webkit-app-region: no-drag;
  transition: all var(--transition-fast);
}

.wc-btn svg {
  opacity: 0;
  transition: opacity var(--transition-fast);
}

.window-controls:hover .wc-btn svg {
  opacity: 1;
}

.wc-close {
  background: #ff5f57;
  color: #4a0002;
}

.wc-minimize {
  background: #febc2e;
  color: #5a3e00;
}

.wc-fullscreen {
  background: #28c840;
  color: #003a00;
}

.wc-close:hover { background: #ff3b30; }
.wc-minimize:hover { background: #f5a623; }
.wc-fullscreen:hover { background: #1aad2e; }

.wc-btn:active {
  transform: scale(0.85);
}
```

- [ ] **Step 3: Wire click handlers cho window controls**

Trong `src/js/app.js`, trong `_bindEvents()`, thêm sau block btn-minimize (khoảng dòng 163):

```javascript
// Window control buttons (custom titlebar)
document.getElementById('btn-wc-close')?.addEventListener('click', async () => {
    await this._saveWindowPosition();
    await this.stop();
    await this.appWindow.close();
});

document.getElementById('btn-wc-minimize')?.addEventListener('click', async () => {
    await this._saveWindowPosition();
    await this.appWindow.minimize();
});

document.getElementById('btn-wc-fullscreen')?.addEventListener('click', async () => {
    const isMaximized = await this.appWindow.isMaximized();
    if (isMaximized) {
        await this.appWindow.unmaximize();
    } else {
        await this.appWindow.maximize();
    }
});
```

- [ ] **Step 4: Xóa hidden buttons block**

Trong `src/index.html`, xóa block hidden buttons (dòng 179-186) vì đã thay thế bằng window controls mới:

```html
<!-- Xóa block này -->
<div style="display:none">
    <button id="btn-close"></button>
    <button id="btn-open-transcripts"></button>
    <button id="btn-compact"></button>
    <button id="btn-minimize"></button>
    <button id="btn-close"></button>
</div>
```

Lưu ý: `btn-open-transcripts` và `btn-compact` vẫn có handler trong app.js. Cần giữ lại ID nếu cần, hoặc nếu handler có null-check (`?.addEventListener`) thì an toàn để xóa. Kiểm tra:
- `btn-close` (dòng 153): `.addEventListener` → sẽ throw nếu null. Cần giữ hoặc thêm `?.`
- `btn-minimize` (dòng 160): `.addEventListener` → sẽ throw nếu null. Cần giữ hoặc thêm `?.`
- `btn-compact` (dòng 172): `.addEventListener` → sẽ throw nếu null
- `btn-open-transcripts` (dòng 275): `.addEventListener` → sẽ throw nếu null

Thay vì xóa, sửa handler trong app.js thêm optional chaining `?.addEventListener` cho cả 4 button cũ. Hoặc đơn giản hơn: giữ hidden div nhưng chỉ giữ `btn-open-transcripts` và `btn-compact`:

```html
<div style="display:none">
    <button id="btn-open-transcripts"></button>
    <button id="btn-compact"></button>
</div>
```

Và sửa handler `btn-close` + `btn-minimize` thành optional:
```javascript
document.getElementById('btn-close')?.addEventListener('click', ...
document.getElementById('btn-minimize')?.addEventListener('click', ...
```

- [ ] **Step 5: Verify window controls (manual test)**

Build và test:
- Hover vào overlay → thấy 3 dots (đỏ, vàng, xanh) ở góc trái
- Hover vào dots → hiện icon bên trong
- Click đỏ → window close
- Click vàng → window minimize
- Click xanh → window maximize/unmaximize

- [ ] **Step 6: Commit**

```bash
git add src/index.html src/styles/main.css src/js/app.js
git commit -m "feat(ui): add macOS-style window controls to drag region

Add close, minimize, and fullscreen buttons as colored dots in the
drag region. Buttons appear on hover with icons. Removes dependency
on hidden button elements for close/minimize."
```

---

### Task 4: Ensure Toolbar Hover Works In All States

**Files:**
- Modify: `src/styles/main.css:258` (toolbar hover rule)

Toolbar chỉ hiện khi `.is-recording:hover`. Nhưng user cũng cần access toolbar khi không recording (ví dụ nhấn play). Cần cho toolbar hiện khi hover bất kể state.

- [ ] **Step 1: Sửa toolbar hover rule cho cả non-recording state**

Trong `src/styles/main.css`, thêm rule cho non-recording state:

```css
/* Show toolbar on hover regardless of recording state */
#overlay-view:hover .floating-toolbar {
  opacity: 1;
  pointer-events: auto;
  transform: translateX(-50%) translateY(0);
}
```

Thay thế rule cũ `#overlay-view.is-recording:hover .floating-toolbar`.

Cũng sửa tương tự cho floating-controls và status-label:

```css
/* Was: #overlay-view.is-recording:hover .floating-controls */
#overlay-view:hover .floating-controls {
  opacity: 1;
}

/* Was: #overlay-view.is-recording:hover .status-label */
#overlay-view.is-recording:hover .status-label {
  opacity: 1;
}
```

Chỉ status-label giữ `.is-recording` vì nó chỉ có ý nghĩa khi recording.

- [ ] **Step 2: Verify toolbar hiển thị cả khi idle và recording**

Build và test:
- Trạng thái idle: hover vào overlay → toolbar hiện (nút play, settings, etc.)
- Trạng thái recording: hover → toolbar hiện (nút stop, source controls, etc.)

- [ ] **Step 3: Commit**

```bash
git add src/styles/main.css
git commit -m "fix(ui): show toolbar on hover in all states, not just recording

Toolbar was only visible during recording state. Users need access
to play button and settings when idle too."
```

---

### Task 5: Integration Test

- [ ] **Step 1: Full flow test**

Build app và test toàn bộ:
1. Mở app → idle state
2. Hover overlay → toolbar hiện ở top, window controls hiện ở trái
3. Click + drag vùng idle → window di chuyển
4. Click nút play trong toolbar → start recording
5. Hover overlay → toolbar hiện (giờ có stop icon)
6. Click stop → dừng recording
7. Click close dot → window đóng
8. Test minimize, fullscreen dots

- [ ] **Step 2: Edge case test**

- Compact mode: toolbar vẫn hiển thị?
- Settings view: window controls vẫn visible?
- Resize handle vẫn hoạt động?

- [ ] **Step 3: Final commit (nếu cần fix thêm)**
