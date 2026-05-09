# UI Redesign: Light/Clean Subtitle Strip — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Chuyển toàn bộ UI từ dark glassmorphism sang light/clean theme (Linear/Arc Browser style), đổi overlay từ fixed control bar sang floating toolbar xuất hiện khi hover.

**Architecture:** 3 file thay đổi: `main.css` (viết lại hoàn toàn), `index.html` (restructure overlay view, giữ nguyên Settings/Sessions structure), `ui.js` (cập nhật hover logic + inline style colors). Giữ nguyên tất cả button IDs để `app.js` và các JS khác không cần thay đổi. Design tokens mới trong CSS `:root`.

**Tech Stack:** HTML, CSS (vanilla, CSS custom properties), vanilla JS (no framework)

---

## File Structure

| File | Thay đổi | Trách nhiệm |
|------|----------|-------------|
| `src/styles/main.css` | Viết lại hoàn toàn (~1900 dòng) | Tất cả styling: design tokens, overlay, toolbar, transcript, floating controls, settings, sessions, modal, animations |
| `src/index.html` | Restructure overlay view | Bỏ fixed `.control-bar` trong `#drag-region`, thêm drag handle + floating toolbar + status label. Settings/Sessions giữ nguyên HTML structure |
| `src/js/ui.js` | Cập nhật 2 chỗ | 1) Inline style color trong `showStatusMessage()` cần đổi sang light theme. 2) Transcript flow colors match light theme |

**Không thay đổi:** `app.js`, `settings.js`, `soniox.js`, TTS modules, `updater.js`, `audio-player.js`, Rust backend, Tauri config.

**Ràng buộc quan trọng:** Tất cả button IDs (`#btn-settings`, `#btn-start`, `#btn-source-system`, `#btn-source-mic`, `#btn-source-both`, `#btn-tts`, `#btn-clear`, `#btn-copy`, `#btn-open-transcripts`, `#btn-sessions`, `#btn-compact`, `#btn-pin`, `#btn-minimize`, `#btn-close`, v.v.) PHẢI giữ nguyên. `app.js` dùng 192+ `getElementById` calls.

---

### Task 1: CSS Design Tokens + Base Reset

**Files:**
- Modify: `src/styles/main.css:1-63` (thay toàn bộ `:root` và base styles)

- [ ] **Step 1: Thay thế toàn bộ `:root` variables**

Xóa toàn bộ nội dung `main.css` hiện tại. Viết mới từ đầu, bắt đầu với reset và design tokens:

```css
/* My Translator — Light/Clean Theme */

*,
*::before,
*::after {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

:root {
  /* Backgrounds */
  --bg-primary: rgba(255, 255, 255, 0.95);
  --bg-secondary: #f9fafb;
  --bg-hover: #f5f5f7;
  --bg-active: #eef2ff;
  --bg-toolbar: #ffffff;
  --bg-desktop: #eef0f4;

  /* Borders */
  --border-subtle: rgba(0, 0, 0, 0.04);
  --border-light: #e5e7eb;
  --border-input: #e5e7eb;
  --border-focus: #c7d2fe;

  /* Text */
  --text-primary: #111827;
  --text-secondary: #6b7280;
  --text-muted: #9ca3af;
  --text-disabled: #b0b5c0;
  --text-original: #b0b5c0;

  /* Accent */
  --accent: #4f46e5;
  --accent-light: #eef2ff;
  --accent-border: #c7d2fe;

  /* Status */
  --success: #10b981;
  --warning: #f59e0b;
  --error: #ef4444;
  --error-light: #fef2f2;

  /* Sizing */
  --radius-sm: 6px;
  --radius-md: 8px;
  --radius-lg: 12px;
  --toolbar-btn: 30px;
  --toolbar-gap-group: 12px;
  --toolbar-gap-item: 4px;
  --toolbar-padding: 6px 12px;
  --icon-size: 14px;
  --icon-stroke: 2;

  /* Shadows */
  --shadow-overlay: 0 1px 8px rgba(0, 0, 0, 0.07), 0 0 0 1px rgba(0, 0, 0, 0.04);
  --shadow-toolbar: 0 2px 12px rgba(0, 0, 0, 0.08), 0 0 0 1px rgba(0, 0, 0, 0.04);

  /* Transitions */
  --transition-fast: 150ms cubic-bezier(0.4, 0, 0.2, 1);
  --transition-normal: 200ms cubic-bezier(0.4, 0, 0.2, 1);
}

html,
body {
  width: 100%;
  height: 100%;
  overflow: hidden;
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
  font-size: 14px;
  color: var(--text-primary);
  background: transparent;
  -webkit-font-smoothing: antialiased;
  user-select: none;
}

/* Views */
.view {
  display: none;
  width: 100%;
  height: 100%;
  flex-direction: column;
}

.view.active {
  display: flex;
  animation: viewFadeIn 0.2s ease;
}

@keyframes viewFadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}
```

- [ ] **Step 2: Mở app xác nhận không bị lỗi parse CSS**

Run: `cargo tauri dev` (hoặc dev server đang chạy)
Expected: App mở được, styles trắng/broken nhưng không crash

- [ ] **Step 3: Commit**

```bash
git add src/styles/main.css
git commit -m "refactor(ui): replace dark theme tokens with light/clean design tokens"
```

---

### Task 2: HTML Overlay Restructure

**Files:**
- Modify: `src/index.html:16-178` (toàn bộ `#overlay-view`)

Thay đổi chính:
1. `#drag-region` chỉ còn drag handle (thanh nhỏ 32x3px) + chấm status ẩn
2. Thêm `#floating-toolbar` nằm absolute phía trên content
3. Thêm `#status-label` trong content card
4. Giữ nguyên `#transcript-container`, `.floating-controls`, `#resize-handle`
5. Tất cả button IDs giữ nguyên, chỉ dời vị trí trong DOM

- [ ] **Step 1: Restructure `#overlay-view` trong `index.html`**

Thay toàn bộ `#overlay-view` (line 16-178) bằng:

```html
  <!-- OVERLAY VIEW -->
  <div id="overlay-view" class="view active">
    <!-- Drag handle (top center) -->
    <div id="drag-region" data-tauri-drag-region>
      <div class="drag-handle" data-tauri-drag-region></div>
    </div>

    <!-- Status dot (top right corner, visible when translating) -->
    <div id="status-indicator" class="status-dot disconnected"></div>

    <!-- Status label (top left, visible on hover when translating) -->
    <div id="status-label" class="status-label">
      <span class="status-label-dot"></span>
      <span id="status-text" class="status-text"></span>
    </div>

    <!-- Floating toolbar (appears on hover, above content) -->
    <div id="floating-toolbar" class="floating-toolbar">
      <!-- Group: Source -->
      <div class="toolbar-group source-controls">
        <button id="btn-source-system" class="toolbar-btn active" title="System Audio (⌘1)">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
            <path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07" />
          </svg>
        </button>
        <button id="btn-source-mic" class="toolbar-btn" title="Microphone (⌘2)">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z" />
            <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
            <line x1="12" y1="19" x2="12" y2="23" />
            <line x1="8" y1="23" x2="16" y2="23" />
          </svg>
        </button>
        <button id="btn-source-both" class="toolbar-btn" title="System + Mic (⌘3)">
          <svg width="16" height="14" viewBox="0 0 28 24" fill="none" stroke="currentColor" stroke-width="2">
            <polygon points="9 6 5 9 2 9 2 15 5 15 9 18 9 6" />
            <path d="M12.5 9.5a3 3 0 0 1 0 5" />
            <path d="M20 4a2.5 2.5 0 0 0-2.5 2.5v5a2.5 2.5 0 0 0 5 0v-5A2.5 2.5 0 0 0 20 4z" />
            <path d="M25 10v1.5a5 5 0 0 1-10 0V10" />
          </svg>
        </button>
      </div>

      <div class="toolbar-separator"></div>

      <!-- Group: Display -->
      <div class="toolbar-group">
        <button id="btn-tts" class="toolbar-btn" title="TTS Narration (⌘T)">
          <svg id="icon-tts-off" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
            <line x1="9" y1="10" x2="15" y2="10" />
          </svg>
          <svg id="icon-tts-on" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="display:none">
            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
            <path d="M9 10h2" /><path d="M13 8v4" /><path d="M15 10h1" />
          </svg>
        </button>
        <button id="btn-font-size-toolbar" class="toolbar-btn" title="Font size">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M4 7V4h16v3" /><path d="M9 20h6" /><path d="M12 4v16" />
          </svg>
        </button>
        <button id="btn-view-mode" class="toolbar-btn" title="Toggle dual view">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="7" height="18" rx="1" />
            <rect x="14" y="3" width="7" height="18" rx="1" />
          </svg>
        </button>
      </div>

      <div class="toolbar-separator"></div>

      <!-- Group: Action -->
      <div class="toolbar-group">
        <button id="btn-copy" class="toolbar-btn" title="Copy transcript">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
          </svg>
        </button>
        <button id="btn-sessions" class="toolbar-btn" title="View saved sessions">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10" />
            <polyline points="12 6 12 12 16 14" />
          </svg>
        </button>
        <button id="btn-settings" class="toolbar-btn" title="Settings">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
          </svg>
          <span id="settings-badge" class="settings-badge" style="display:none"></span>
        </button>
      </div>

      <div class="toolbar-separator"></div>

      <!-- Group: Control -->
      <div class="toolbar-group">
        <button id="btn-start" class="toolbar-btn toolbar-btn-stop" title="Stop (Space)">
          <svg id="icon-play" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polygon points="5 3 19 12 5 21 5 3" fill="currentColor" />
          </svg>
          <svg id="icon-stop" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="display:none">
            <rect x="4" y="4" width="16" height="16" rx="2" fill="currentColor" />
          </svg>
        </button>
        <button id="btn-pin" class="toolbar-btn active" title="Pin on top">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 17v5" />
            <path d="M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V6h1a2 2 0 0 0 0-4H8a2 2 0 0 0 0 4h1v4.76z" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Idle state overlay (shown when not translating) -->
    <div id="idle-overlay" class="idle-overlay">
      <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.4">
        <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z" />
        <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
        <line x1="12" y1="19" x2="12" y2="23" />
        <line x1="8" y1="23" x2="16" y2="23" />
      </svg>
      <p class="idle-text">Nhấn ▶ để bắt đầu dịch</p>
      <button id="btn-idle-start" class="idle-start-btn">Bắt đầu</button>
    </div>

    <!-- Transcript Area -->
    <div id="transcript-container" data-tauri-drag-region>
      <div id="transcript-content" data-tauri-drag-region>
        <div class="transcript-placeholder">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.4">
            <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z" />
            <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
            <line x1="12" y1="19" x2="12" y2="23" />
            <line x1="8" y1="23" x2="16" y2="23" />
          </svg>
          <p>Press ▶ to start translating</p>
        </div>
      </div>
    </div>

    <!-- Floating controls (bottom-right, shown on hover) -->
    <div class="floating-controls">
      <div class="font-controls">
        <button id="btn-font-down" class="float-btn" title="Decrease font size">A−</button>
        <span id="font-size-display" class="font-size-label">16</span>
        <button id="btn-font-up" class="float-btn" title="Increase font size">A+</button>
      </div>
      <div class="color-controls">
        <button class="color-dot active" data-color="#111827" title="Dark" style="background:#111827;"></button>
        <button class="color-dot" data-color="#92400e" title="Yellow" style="background:#92400e;"></button>
        <button class="color-dot" data-color="#164e63" title="Cyan" style="background:#164e63;"></button>
      </div>
      <button id="btn-view-mode-float" class="float-btn" title="Toggle dual view">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="3" width="7" height="18" rx="1" />
          <rect x="14" y="3" width="7" height="18" rx="1" />
        </svg>
      </button>
    </div>

    <!-- Hidden buttons (keep IDs for app.js, but not visible in new design) -->
    <div style="display:none">
      <button id="btn-clear"></button>
      <button id="btn-open-transcripts"></button>
      <button id="btn-compact"></button>
      <button id="btn-minimize"></button>
      <button id="btn-close"></button>
    </div>

    <!-- Resize Handle -->
    <div id="resize-handle"></div>
  </div>
```

**Ghi chú quan trọng:**
- Các button không còn trong toolbar mới (`btn-clear`, `btn-open-transcripts`, `btn-compact`, `btn-minimize`, `btn-close`) được giữ ẩn trong hidden div để `app.js` không lỗi getElementById.
- `#btn-idle-start` cần wire trong `app.js` (hoặc cùng handler với `#btn-start`).
- `.source-btn` đổi thành `.toolbar-btn` cho thống nhất. `app.js` tham chiếu qua ID nên không ảnh hưởng.
- Color dots đổi màu phù hợp light theme: `#111827` (dark text), `#92400e` (amber), `#164e63` (teal).

- [ ] **Step 2: Xác nhận app.js không lỗi**

Run: Mở app, kiểm tra console không có null reference errors.
Expected: Tất cả `getElementById` trả về element (không null).

- [ ] **Step 3: Commit**

```bash
git add src/index.html
git commit -m "refactor(ui): restructure overlay HTML for floating toolbar design"
```

---

### Task 3: CSS Overlay View (card, drag handle, status)

**Files:**
- Modify: `src/styles/main.css` (append sau base styles)

- [ ] **Step 1: Thêm CSS cho overlay card, drag handle, status**

Append vào `main.css` sau phần base styles:

```css
/* ══════════════ OVERLAY VIEW ══════════════ */

#overlay-view {
  position: relative;
  background: var(--bg-primary);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-overlay);
  overflow: visible;
}

/* Drag Region */
#drag-region {
  flex-shrink: 0;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.drag-handle {
  width: 32px;
  height: 3px;
  border-radius: 2px;
  background: rgba(0, 0, 0, 0.1);
  transition: background var(--transition-fast);
}

.drag-handle:hover {
  background: rgba(0, 0, 0, 0.2);
}

/* Status dot (top right) */
#overlay-view > .status-dot {
  position: absolute;
  top: 8px;
  right: 10px;
  width: 5px;
  height: 5px;
  border-radius: 50%;
  z-index: 5;
  transition: background var(--transition-normal);
}

.status-dot.disconnected {
  background: var(--text-muted);
}

.status-dot.connecting {
  background: var(--warning);
  animation: pulse 1.5s infinite;
}

.status-dot.connected {
  background: var(--success);
  box-shadow: 0 0 4px rgba(16, 185, 129, 0.4);
}

.status-dot.error {
  background: var(--error);
  box-shadow: 0 0 4px rgba(239, 68, 68, 0.3);
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

/* Status label (top left, visible on hover when translating) */
.status-label {
  position: absolute;
  top: 6px;
  left: 12px;
  display: flex;
  align-items: center;
  gap: 4px;
  opacity: 0;
  transition: opacity var(--transition-fast);
  z-index: 5;
  pointer-events: none;
}

#overlay-view.is-recording:hover .status-label {
  opacity: 1;
}

.status-label-dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--success);
}

.status-label .status-text {
  font-size: 10px;
  color: var(--text-muted);
  font-weight: 500;
}

/* Idle overlay */
.idle-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--text-muted);
  z-index: 2;
}

.idle-overlay.hidden {
  display: none;
}

.idle-text {
  font-size: 13px;
  font-weight: 400;
  color: var(--text-muted);
}

.idle-start-btn {
  padding: 8px 24px;
  border: none;
  border-radius: var(--radius-md);
  background: var(--accent);
  color: #fff;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
  -webkit-app-region: no-drag;
}

.idle-start-btn:hover {
  background: #4338ca;
  box-shadow: 0 2px 8px rgba(79, 70, 229, 0.3);
}

.idle-start-btn:active {
  transform: scale(0.96);
}
```

- [ ] **Step 2: Verify visual — card trắng, drag handle visible, idle state centered**

Run: Mở app
Expected: Card nền trắng, drag handle nhỏ ở top center, idle overlay hiện giữa card

- [ ] **Step 3: Commit**

```bash
git add src/styles/main.css
git commit -m "style(ui): add overlay card, drag handle, status dot, and idle state styles"
```

---

### Task 4: CSS Floating Toolbar

**Files:**
- Modify: `src/styles/main.css` (append)

- [ ] **Step 1: Thêm CSS cho floating toolbar**

Append vào `main.css`:

```css
/* ══════════════ FLOATING TOOLBAR ══════════════ */

.floating-toolbar {
  position: absolute;
  top: -52px;
  left: 50%;
  transform: translateX(-50%);
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
  transform: translateX(-50%) translateY(4px);
}

#overlay-view.is-recording:hover .floating-toolbar {
  opacity: 1;
  pointer-events: auto;
  transform: translateX(-50%) translateY(0);
}

.toolbar-separator {
  width: 1px;
  height: 20px;
  background: var(--border-light);
  margin: 0 4px;
  flex-shrink: 0;
}

.toolbar-group {
  display: flex;
  align-items: center;
  gap: var(--toolbar-gap-item);
}

/* Source controls pill group */
.toolbar-group.source-controls {
  background: var(--bg-hover);
  border-radius: var(--radius-sm);
  padding: 2px;
}

.toolbar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: var(--toolbar-btn);
  height: var(--toolbar-btn);
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-disabled);
  cursor: pointer;
  transition: all var(--transition-fast);
  -webkit-app-region: no-drag;
  padding: 0;
}

.toolbar-btn svg {
  width: var(--icon-size);
  height: var(--icon-size);
}

.toolbar-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.toolbar-btn:active {
  transform: scale(0.92);
}

.toolbar-btn.active {
  background: var(--accent-light);
  color: var(--accent);
}

/* Stop button specific style */
.toolbar-btn-stop.recording {
  color: var(--error);
}

.toolbar-btn-stop.recording:hover {
  background: var(--error-light);
}

/* Pin active */
#btn-pin.active {
  color: var(--accent);
}

/* TTS active */
#btn-tts.active {
  color: var(--accent);
  background: var(--accent-light);
}
```

- [ ] **Step 2: Verify — hover card, toolbar slides down above it**

Run: Mở app, hover lên overlay card
Expected: Toolbar trắng xuất hiện phía trên card với animation slideDown nhẹ

- [ ] **Step 3: Commit**

```bash
git add src/styles/main.css
git commit -m "style(ui): add floating toolbar with hover reveal animation"
```

---

### Task 5: CSS Transcript Content + Dual View

**Files:**
- Modify: `src/styles/main.css` (append)

- [ ] **Step 1: Thêm CSS cho transcript container, text styles, dual view**

Append vào `main.css`:

```css
/* ══════════════ TRANSCRIPT ══════════════ */

#transcript-container {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 8px 16px 12px;
}

#transcript-container::-webkit-scrollbar {
  width: 4px;
}

#transcript-container::-webkit-scrollbar-track {
  background: transparent;
}

#transcript-container::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.08);
  border-radius: 2px;
}

#transcript-container::-webkit-scrollbar-thumb:hover {
  background: rgba(0, 0, 0, 0.15);
}

/* Placeholder */
.transcript-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 12px;
  color: var(--text-muted);
}

.transcript-placeholder p {
  font-size: 13px;
  font-weight: 400;
}

.shortcut-hint {
  font-size: 11px !important;
  color: var(--text-muted) !important;
  opacity: 0.6;
  background: var(--bg-hover);
  padding: 2px 10px !important;
  border-radius: 4px;
  border: 1px solid var(--border-subtle);
  font-family: -apple-system, monospace;
}

/* Listening Indicator */
.listening-indicator {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 16px;
  color: var(--text-muted);
  animation: fadeInUp 0.3s ease;
}

.listening-indicator p {
  font-size: 12px;
  font-weight: 500;
  letter-spacing: 0.04em;
}

.listening-waves {
  display: flex;
  align-items: center;
  gap: 3px;
  height: 32px;
}

.listening-waves span {
  display: block;
  width: 3px;
  height: 8px;
  background: var(--accent);
  border-radius: 2px;
  animation: wave 1.2s ease-in-out infinite;
}

.listening-waves span:nth-child(1) { animation-delay: 0s; }
.listening-waves span:nth-child(2) { animation-delay: 0.15s; }
.listening-waves span:nth-child(3) { animation-delay: 0.3s; }
.listening-waves span:nth-child(4) { animation-delay: 0.45s; }
.listening-waves span:nth-child(5) { animation-delay: 0.6s; }

@keyframes wave {
  0%, 100% { height: 8px; opacity: 0.5; }
  50% { height: 24px; opacity: 1; }
}

/* Transcript Flow */
.transcript-flow {
  font-size: var(--transcript-font-size, 16px);
  color: var(--transcript-font-color, var(--text-primary));
  line-height: 1.7;
  word-wrap: break-word;
  overflow-wrap: break-word;
}

/* Segment block */
.seg-block {
  display: block;
  margin-bottom: 4px;
}

/* Translated text */
.seg-translated {
  display: block;
  color: var(--transcript-font-color, var(--text-primary));
  font-weight: 450;
  font-size: 16px;
}

/* Original text */
.seg-original {
  display: block;
  color: var(--text-original);
  font-weight: 400;
  font-size: 12px;
  margin-top: 8px;
  margin-bottom: 2px;
}

/* Provisional text */
.seg-provisional {
  display: block;
  color: var(--text-muted);
  font-weight: 400;
  font-style: italic;
}

/* Speaker label */
.speaker-label {
  display: block;
  color: var(--accent);
  font-weight: 600;
  font-size: 0.85em;
  letter-spacing: 0.02em;
  margin-top: 12px;
  margin-bottom: 2px;
}

/* Language badge */
.lang-badge {
  display: inline-block;
  font-size: 0.7em;
  font-weight: 600;
  letter-spacing: 0.04em;
  color: var(--text-secondary);
  background: var(--accent-light);
  border: 1px solid var(--accent-border);
  border-radius: 4px;
  padding: 1px 6px;
  margin-right: 4px;
  vertical-align: middle;
}

/* Low confidence */
.low-confidence {
  border-left: 2px solid rgba(245, 158, 11, 0.5);
  padding-left: 6px;
  color: var(--text-secondary);
}

.low-confidence::after {
  content: ' ⚠';
  font-size: 0.7em;
  color: var(--warning);
  opacity: 0.6;
}

/* Cursor blink */
.cursor-blink {
  color: var(--accent);
  font-weight: 300;
  animation: blink 1s step-end infinite;
}

@keyframes blink {
  0%, 50% { opacity: 1; }
  51%, 100% { opacity: 0; }
}

@keyframes fadeInUp {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}

/* ─── Dual View Mode ─── */
.dual-view #transcript-content {
  display: flex;
  gap: 0;
  height: 100%;
}

.dual-view .transcript-flow {
  display: flex;
  gap: 0;
  height: 100%;
  min-height: 0;
}

.dual-view .panel-source,
.dual-view .panel-translation {
  flex: 1;
  overflow-y: auto;
  padding: 0 12px;
  min-height: 0;
}

.dual-view .panel-source {
  border-right: 1px solid #f0f1f3;
}

.dual-view .panel-source::-webkit-scrollbar,
.dual-view .panel-translation::-webkit-scrollbar {
  width: 3px;
}

.dual-view .panel-source::-webkit-scrollbar-thumb,
.dual-view .panel-translation::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.08);
  border-radius: 2px;
}

.dual-view .panel-source .seg-text {
  color: var(--text-secondary);
  font-size: 14px;
  margin-bottom: 6px;
  line-height: 1.5;
}

.dual-view .panel-translation .seg-text {
  color: var(--transcript-font-color, var(--text-primary));
  font-weight: 450;
  font-size: 14px;
  margin-bottom: 6px;
  line-height: 1.5;
}

.dual-view .panel-source .seg-text.pending {
  color: var(--text-muted);
  font-style: italic;
}

.dual-view .seg-block {
  display: none;
}
```

- [ ] **Step 2: Verify transcript text hiển thị đúng light theme colors**

Run: Mở app, bắt đầu dịch
Expected: Text dịch = `#111827`, text gốc = `#b0b5c0`, provisional = `#9ca3af` italic

- [ ] **Step 3: Commit**

```bash
git add src/styles/main.css
git commit -m "style(ui): add transcript, dual view, and text segment styles for light theme"
```

---

### Task 6: CSS Floating Controls (font, color, view mode)

**Files:**
- Modify: `src/styles/main.css` (append)

- [ ] **Step 1: Thêm CSS cho floating controls bottom-right**

Append vào `main.css`:

```css
/* ══════════════ FLOATING CONTROLS (bottom-right) ══════════════ */

.floating-controls {
  position: absolute;
  bottom: 24px;
  right: 8px;
  display: flex;
  align-items: center;
  gap: 6px;
  z-index: 10;
  opacity: 0;
  transition: opacity 0.2s;
}

#overlay-view.is-recording:hover .floating-controls {
  opacity: 1;
}

.font-controls {
  display: flex;
  align-items: center;
  gap: 2px;
  background: var(--bg-toolbar);
  border-radius: var(--radius-sm);
  padding: 2px 4px;
  box-shadow: var(--shadow-overlay);
}

.float-btn {
  width: 26px;
  height: 26px;
  border-radius: 5px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 600;
  transition: all 0.15s;
}

.float-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.float-btn.active {
  background: var(--accent-light);
  color: var(--accent);
}

.font-size-label {
  color: var(--text-muted);
  font-size: 10px;
  min-width: 20px;
  text-align: center;
  font-variant-numeric: tabular-nums;
}

.color-controls {
  display: flex;
  align-items: center;
  gap: 4px;
  background: var(--bg-toolbar);
  border-radius: var(--radius-sm);
  padding: 4px 6px;
  box-shadow: var(--shadow-overlay);
}

.color-dot {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  transition: all 0.15s;
  padding: 0;
  outline: none;
}

.color-dot:hover {
  transform: scale(1.2);
}

.color-dot.active {
  border-color: var(--accent-border);
  box-shadow: 0 0 0 2px var(--accent-light);
}

/* Resize Handle */
#resize-handle {
  height: 6px;
  cursor: ns-resize;
  background: transparent;
  display: flex;
  align-items: center;
  justify-content: center;
  -webkit-app-region: no-drag;
}

#resize-handle::after {
  content: '';
  width: 40px;
  height: 3px;
  border-radius: 2px;
  background: rgba(0, 0, 0, 0.06);
  transition: background var(--transition-fast);
}

#resize-handle:hover::after {
  background: rgba(0, 0, 0, 0.15);
}
```

- [ ] **Step 2: Commit**

```bash
git add src/styles/main.css
git commit -m "style(ui): add floating controls and resize handle styles"
```

---

### Task 7: CSS Compact Mode

**Files:**
- Modify: `src/styles/main.css` (append)

- [ ] **Step 1: Thêm compact mode CSS**

Append vào `main.css`:

```css
/* ══════════════ COMPACT MODE ══════════════ */

#drag-region.compact-hidden {
  display: none !important;
}

.compact-mode #overlay-view > .status-dot {
  display: none;
}

.compact-mode::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 6px;
  z-index: 100;
  cursor: pointer;
}

.compact-mode:hover #drag-region.compact-hidden {
  display: flex !important;
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  z-index: 99;
  background: var(--bg-toolbar);
  border-bottom: 1px solid var(--border-subtle);
  height: 16px;
}

.compact-mode #transcript-container {
  padding-top: 4px;
}
```

- [ ] **Step 2: Commit**

```bash
git add src/styles/main.css
git commit -m "style(ui): add compact mode styles for light theme"
```

---

### Task 8: CSS Settings View

**Files:**
- Modify: `src/styles/main.css` (append)

Settings HTML structure giữ nguyên, chỉ reskin CSS sang light theme.

- [ ] **Step 1: Thêm toàn bộ CSS cho settings view**

Append vào `main.css`:

```css
/* ══════════════ SETTINGS VIEW ══════════════ */

#settings-view {
  background: var(--bg-primary);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-overlay);
  overflow: hidden;
}

#settings-drag {
  flex-shrink: 0;
  border-bottom: 1px solid var(--border-light);
  background: var(--bg-toolbar);
}

.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  height: 42px;
}

.settings-header h2 {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: 0.01em;
}

.settings-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.settings-body::-webkit-scrollbar {
  width: 4px;
}

.settings-body::-webkit-scrollbar-track {
  background: transparent;
}

.settings-body::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.08);
  border-radius: 2px;
}

/* Tabs */
.settings-tabs {
  display: flex;
  gap: 0;
  border-bottom: 1px solid var(--border-light);
  margin-bottom: 16px;
}

.settings-tab {
  flex: 1;
  padding: 8px 0;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--text-muted);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: color 0.15s, border-color 0.15s;
}

.settings-tab:hover {
  color: var(--text-primary);
}

.settings-tab.active {
  color: var(--accent);
  border-bottom-color: var(--accent);
}

.settings-tab-content {
  display: none;
}

.settings-tab-content.active {
  display: block;
}

/* Sections */
.settings-section {
  margin-bottom: 20px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--border-subtle);
}

.settings-section:last-of-type {
  border-bottom: none;
  margin-bottom: 8px;
}

.section-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 10px;
}

/* Badges */
.optional-badge {
  font-size: 10px;
  font-weight: 400;
  color: var(--text-muted);
  text-transform: none;
  letter-spacing: 0;
  background: var(--bg-hover);
  padding: 1px 6px;
  border-radius: 4px;
}

.required-badge {
  font-size: 10px;
  font-weight: 500;
  color: var(--error);
  text-transform: none;
  letter-spacing: 0;
  background: var(--error-light);
  padding: 1px 6px;
  border-radius: 4px;
}

.save-btn-top {
  color: var(--accent) !important;
}

.save-btn-top:hover {
  background: var(--accent-light) !important;
  color: var(--accent) !important;
}

/* Settings Badge */
.settings-badge {
  position: absolute;
  top: 2px;
  right: 2px;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--error);
}

/* Icon Buttons (shared between settings/sessions) */
.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-direction: column;
  min-width: 32px;
  height: 32px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
  -webkit-app-region: no-drag;
  padding: 0 4px;
  gap: 1px;
}

.icon-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.icon-btn:active {
  transform: scale(0.92);
}

.icon-btn.small {
  width: 28px;
  height: 28px;
}

/* Form Elements */
.input-group {
  display: flex;
  gap: 4px;
}

.input-group input {
  flex: 1;
}

input[type="text"],
input[type="password"] {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--border-input);
  border-radius: var(--radius-md);
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-family: inherit;
  font-size: 13px;
  outline: none;
  transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
}

input[type="text"]:focus,
input[type="password"]:focus {
  border-color: var(--border-focus);
  box-shadow: 0 0 0 3px rgba(79, 70, 229, 0.08);
}

input::placeholder {
  color: var(--text-muted);
}

select {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--border-input);
  border-radius: var(--radius-md);
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-family: inherit;
  font-size: 13px;
  outline: none;
  cursor: pointer;
  -webkit-appearance: none;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg width='10' height='6' viewBox='0 0 10 6' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M1 1L5 5L9 1' stroke='%239ca3af' stroke-width='1.5' stroke-linecap='round'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 10px center;
  padding-right: 28px;
  transition: border-color var(--transition-fast);
}

select:focus {
  border-color: var(--border-focus);
}

textarea {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--border-input);
  border-radius: var(--radius-md);
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-family: inherit;
  font-size: 13px;
  outline: none;
  resize: vertical;
  min-height: 36px;
  transition: border-color var(--transition-fast);
}

textarea:focus {
  border-color: var(--border-focus);
  box-shadow: 0 0 0 3px rgba(79, 70, 229, 0.08);
}

/* Field layout */
.field-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.field-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.hint {
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 4px;
}

.hint a {
  color: var(--accent);
  text-decoration: none;
}

.hint a:hover {
  text-decoration: underline;
}

.hint-dim {
  opacity: 0.5;
  font-size: 0.9em;
}

/* Radio Group */
.radio-group {
  display: flex;
  gap: 4px;
  background: var(--bg-hover);
  border-radius: var(--radius-sm);
  padding: 3px;
}

.radio-option {
  flex: 1;
  text-align: center;
  cursor: pointer;
}

.radio-option input[type="radio"] {
  display: none;
}

.radio-label {
  display: block;
  padding: 6px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  transition: all var(--transition-fast);
}

.radio-option input[type="radio"]:checked + .radio-label {
  background: var(--accent-light);
  color: var(--accent);
  border: 1px solid var(--accent-border);
}

.radio-option:hover .radio-label {
  color: var(--text-primary);
}

/* Checkbox */
.checkbox-option {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-primary);
}

.checkbox-option input[type="checkbox"] {
  accent-color: var(--accent);
}

.checkbox-label {
  font-size: 13px;
  color: var(--text-primary);
}

/* Sliders */
.slider-field {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}

.slider-field .field-label {
  min-width: 65px;
}

input[type="range"] {
  flex: 1;
  -webkit-appearance: none;
  appearance: none;
  height: 4px;
  background: var(--border-light);
  border-radius: 2px;
  outline: none;
}

input[type="range"]::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--accent);
  cursor: pointer;
  border: 2px solid white;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.15);
}

.range-value {
  font-size: 11px;
  color: var(--text-muted);
  min-width: 32px;
  text-align: right;
  font-variant-numeric: tabular-nums;
}

/* Context rows */
.terms-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.btn-icon-sm {
  width: 24px;
  height: 24px;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-sm);
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 16px;
  line-height: 1;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background var(--transition-fast);
}

.btn-icon-sm:hover {
  background: var(--bg-hover);
}

.term-row {
  display: flex;
  gap: 6px;
  margin-bottom: 4px;
  align-items: center;
}

.term-row input {
  flex: 1;
  min-width: 0;
}

.term-row .btn-remove-term {
  width: 22px;
  height: 22px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--text-muted);
  font-size: 14px;
  cursor: pointer;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.term-row .btn-remove-term:hover {
  color: var(--error);
  background: var(--error-light);
}

.general-row {
  display: flex;
  gap: 6px;
  margin-bottom: 4px;
  align-items: center;
}

.general-row input {
  flex: 1;
  min-width: 0;
}

.general-row .general-key {
  flex: 0.4;
}

.general-row .general-value {
  flex: 0.6;
}

.general-row .btn-remove-general {
  width: 22px;
  height: 22px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--text-muted);
  font-size: 14px;
  cursor: pointer;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.general-row .btn-remove-general:hover {
  color: var(--error);
  background: var(--error-light);
}

.context-subsection {
  margin-bottom: 10px;
}

.context-subsection textarea {
  width: 100%;
  resize: vertical;
  min-height: 36px;
  margin-top: 4px;
}

/* Save Button */
.settings-actions {
  padding: 12px 0 4px;
}

.primary-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  width: 100%;
  padding: 12px;
  border: none;
  border-radius: 10px;
  background: var(--accent);
  color: #fff;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.primary-btn:hover {
  background: #4338ca;
  box-shadow: 0 2px 8px rgba(79, 70, 229, 0.3);
}

.primary-btn:active {
  transform: scale(0.98);
}

.secondary-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 10px 20px;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-md);
  background: var(--bg-toolbar);
  color: var(--text-secondary);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.secondary-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
```

- [ ] **Step 2: Verify settings view**

Run: Mở app, click Settings
Expected: Nền trắng, tabs hoạt động, form elements styled đúng (input nền #f9fafb, border #e5e7eb, accent indigo)

- [ ] **Step 3: Commit**

```bash
git add src/styles/main.css
git commit -m "style(ui): add settings view styles for light theme"
```

---

### Task 9: CSS Sessions View + About Tab

**Files:**
- Modify: `src/styles/main.css` (append)

- [ ] **Step 1: Thêm CSS cho sessions view và about tab**

Append vào `main.css`:

```css
/* ══════════════ SESSIONS VIEW ══════════════ */

#sessions-view {
  background: var(--bg-primary);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-overlay);
  overflow: hidden;
}

.sessions-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  height: 42px;
  border-bottom: 1px solid var(--border-light);
  background: var(--bg-toolbar);
  flex-shrink: 0;
}

.sessions-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
}

.sessions-body {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
}

.sessions-body::-webkit-scrollbar {
  width: 4px;
}

.sessions-body::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.08);
  border-radius: 2px;
}

/* Session items */
.session-item {
  padding: 10px 12px;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: background var(--transition-fast);
  margin-bottom: 2px;
}

.session-item:hover {
  background: #f8f9fb;
}

.session-item-langs {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.session-item-date {
  font-size: 11px;
  color: var(--text-muted);
  float: right;
}

.session-item-preview {
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 2px;
}

.session-item-meta {
  font-size: 10px;
  color: var(--text-disabled);
  margin-top: 2px;
}

/* Session viewer */
.session-viewer-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border-light);
}

.session-viewer-title-text {
  flex: 1;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.session-copy-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  font-size: 12px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.session-copy-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.session-content-scroll {
  padding: 12px 16px;
  overflow-y: auto;
  flex: 1;
  font-size: 13px;
  line-height: 1.6;
  color: var(--text-primary);
}

.session-content-scroll::-webkit-scrollbar {
  width: 4px;
}

.session-content-scroll::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.08);
  border-radius: 2px;
}

/* About tab */
.about-info {
  text-align: center;
  padding: 16px 0;
}

.about-app-header {
  display: flex;
  align-items: baseline;
  justify-content: center;
  gap: 8px;
  margin-bottom: 4px;
}

.about-app-name {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.about-app-version {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-muted);
  background: var(--bg-hover);
  padding: 2px 8px;
  border-radius: 4px;
}

.about-links {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.about-links a {
  color: var(--accent);
  text-decoration: none;
  font-size: 13px;
}

.about-links a:hover {
  text-decoration: underline;
}

/* Update status */
.update-status {
  display: flex;
  align-items: center;
  gap: 8px;
}

#update-status-text {
  flex: 1;
  font-size: 12px;
  color: var(--text-secondary);
}

.check-update-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.check-update-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.update-btn {
  width: auto;
}

.update-progress {
  display: flex;
  align-items: center;
  gap: 8px;
}

.progress-bar {
  flex: 1;
  height: 4px;
  background: var(--border-light);
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--accent);
  border-radius: 2px;
  transition: width 0.3s ease;
}

.progress-pct {
  font-size: 11px;
  color: var(--text-muted);
  min-width: 32px;
  text-align: right;
}
```

- [ ] **Step 2: Commit**

```bash
git add src/styles/main.css
git commit -m "style(ui): add sessions view, about tab, and update progress styles"
```

---

### Task 10: CSS Modal + Toast + Misc

**Files:**
- Modify: `src/styles/main.css` (append)

- [ ] **Step 1: Thêm CSS cho modal, toast, và misc styles**

Append vào `main.css`:

```css
/* ══════════════ MODAL ══════════════ */

.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
  backdrop-filter: blur(4px);
}

.modal-card {
  background: var(--bg-toolbar);
  border-radius: var(--radius-lg);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
  padding: 24px;
  width: 360px;
  max-width: 90%;
}

.modal-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 16px;
}

.modal-header h3 {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
}

.modal-desc {
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.5;
  margin-bottom: 16px;
}

.modal-body {
  margin-bottom: 16px;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
}

.progress-container {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 16px;
}

.setup-steps {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 12px;
}

.setup-step {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--text-secondary);
}

.step-icon {
  font-size: 14px;
  width: 20px;
  text-align: center;
}

.step-text {
  font-size: 12px;
}

.setup-status {
  text-align: center;
  font-size: 12px;
  color: var(--text-muted);
}

/* ══════════════ TOAST ══════════════ */

.toast {
  position: fixed;
  bottom: 20px;
  left: 50%;
  transform: translateX(-50%) translateY(20px);
  padding: 8px 16px;
  border-radius: var(--radius-md);
  background: var(--text-primary);
  color: white;
  font-size: 12px;
  font-weight: 500;
  z-index: 300;
  opacity: 0;
  transition: all 0.3s ease;
  pointer-events: none;
  white-space: nowrap;
}

.toast.show {
  opacity: 1;
  transform: translateX(-50%) translateY(0);
}

.toast.success {
  background: var(--success);
}

.toast.error {
  background: var(--error);
}

/* ══════════════ MISC ══════════════ */

/* Action button (legacy compat — used by hidden #btn-start alternate) */
.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 30px;
  border: none;
  border-radius: var(--radius-sm);
  background: var(--accent);
  color: #fff;
  cursor: pointer;
  transition: all var(--transition-fast);
  -webkit-app-region: no-drag;
}

.action-btn.recording {
  background: var(--error);
  animation: recordPulse 2s infinite;
}

@keyframes recordPulse {
  0%, 100% { box-shadow: 0 0 0 0 rgba(239, 68, 68, 0.3); }
  50% { box-shadow: 0 0 10px 3px rgba(239, 68, 68, 0.2); }
}

/* TTS button compat */
.tts-action-btn {
  display: none;
}

.tts-label {
  display: none;
}

/* Source btn compat (now toolbar-btn but keep class for any JS that adds it) */
.source-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 30px;
  height: 28px;
  border: none;
  border-radius: 4px;
  background: transparent;
  cursor: pointer;
  transition: all var(--transition-fast);
}

/* Close btn compat */
.close-btn:hover {
  background: var(--error-light);
  color: var(--error);
}

/* Pipeline status inline style override */
.pipeline-status {
  text-align: center;
  padding: 8px;
  color: var(--text-muted);
  font-size: 13px;
}
```

- [ ] **Step 2: Commit**

```bash
git add src/styles/main.css
git commit -m "style(ui): add modal, toast, and misc compatibility styles"
```

---

### Task 11: JS Updates — ui.js inline styles + app.js idle button

**Files:**
- Modify: `src/js/ui.js:199-209` (showStatusMessage inline color)
- Modify: `src/js/app.js` (wire `#btn-idle-start` + update source-btn class references)

- [ ] **Step 1: Fix inline style trong ui.js showStatusMessage**

Trong `src/js/ui.js`, line 206, thay đổi inline style color:

Tìm:
```javascript
statusEl.style.cssText = 'text-align:center; padding:8px; color:rgba(255,255,255,0.5); font-size:13px;';
```

Thay bằng:
```javascript
statusEl.style.cssText = 'text-align:center; padding:8px; color:var(--text-muted); font-size:13px;';
```

- [ ] **Step 2: Wire `#btn-idle-start` trong app.js**

Trong `src/js/app.js`, tìm đoạn bind event cho `#btn-start` (trong method `_bindEvents`), thêm ngay sau đó:

```javascript
const btnIdleStart = document.getElementById('btn-idle-start');
if (btnIdleStart) {
    btnIdleStart.addEventListener('click', () => this._toggleRecording());
}
```

Và thêm logic ẩn/hiện idle overlay + toggle toolbar visibility khi bắt đầu/dừng recording. Tìm trong method nơi `isRunning` thay đổi, thêm:

```javascript
const idleOverlay = document.getElementById('idle-overlay');
const overlayView = document.getElementById('overlay-view');
if (idleOverlay) {
    idleOverlay.classList.toggle('hidden', this.isRunning);
}
if (overlayView) {
    overlayView.classList.toggle('is-recording', this.isRunning);
}
```

**Quan trọng:** Floating toolbar chỉ hiện khi hover + đang recording. Thêm CSS rule cho điều này:

Trong `main.css`, thay rule hiện toolbar:
```css
/* Toolbar chỉ hiện khi recording + hover */
#overlay-view.is-recording:hover .floating-toolbar {
  opacity: 1;
  pointer-events: auto;
  transform: translateX(-50%) translateY(0);
}
```

Và bỏ rule cũ `#overlay-view:hover .floating-toolbar` (thay bằng rule trên). Tương tự cho status label:
```css
#overlay-view.is-recording:hover .status-label {
  opacity: 1;
}
```

- [ ] **Step 3: Update source button class references trong app.js**

`app.js` dùng `querySelector('.source-btn.active')` hoặc class toggle `active` trên source buttons. Vì HTML đổi class từ `source-btn` sang `toolbar-btn` nhưng parent `.source-controls` vẫn giữ, kiểm tra `app.js` xem có reference class `source-btn`:

Tìm trong `app.js` mọi chỗ reference `source-btn`:
- Nếu dùng `classList.add/remove/toggle('active')` qua ID: OK, không cần đổi
- Nếu dùng `querySelectorAll('.source-btn')`: cần đổi thành `querySelectorAll('.source-controls .toolbar-btn')` hoặc giữ class `source-btn` trên buttons

**Giải pháp an toàn hơn:** Giữ thêm class `source-btn` trên 3 source buttons trong HTML bên cạnh `toolbar-btn`. Sửa HTML:

```html
<button id="btn-source-system" class="toolbar-btn source-btn active" ...>
<button id="btn-source-mic" class="toolbar-btn source-btn" ...>
<button id="btn-source-both" class="toolbar-btn source-btn" ...>
```

- [ ] **Step 4: Verify functional — Start/Stop, source switch, TTS toggle, Settings, Sessions**

Run: Mở app, test:
1. Click "Bắt đầu" trên idle overlay → recording bắt đầu, idle overlay ẩn
2. Hover → floating toolbar hiện
3. Switch source buttons
4. TTS toggle
5. Open Settings/Sessions và quay lại
6. Stop recording → idle overlay hiện lại

Expected: Tất cả hoạt động, không console errors

- [ ] **Step 5: Commit**

```bash
git add src/js/ui.js src/js/app.js src/index.html
git commit -m "fix(ui): wire idle overlay, fix inline styles, keep source-btn class compat"
```

---

### Task 12: Visual Polish + Verification

**Files:**
- Có thể modify: `src/styles/main.css`, `src/index.html`

- [ ] **Step 1: So sánh visual với mockups**

Mở mockups tại `.superpowers/brainstorm/` và so sánh:
- `overlay-v6.html`: Toolbar + content layout
- `settings-design.html`: Settings tabs
- `sessions-design.html`: Sessions list + viewer

Ghi chú mọi khác biệt cần fix.

- [ ] **Step 2: Fix spacing, colors, sizes nếu lệch mockup**

Ví dụ có thể cần:
- Adjust toolbar button spacing
- Tweak font sizes
- Fix border-radius inconsistencies
- Adjust shadow values

- [ ] **Step 3: Test responsive — resize window nhỏ/lớn**

Kéo resize handle, đảm bảo:
- Toolbar không bị tràn
- Transcript content scroll đúng
- Floating controls không che content

- [ ] **Step 4: Test compact mode**

Toggle compact mode (⌘D):
- Drag handle ẩn
- Status dot ẩn
- Hover top 6px hiện drag region

- [ ] **Step 5: Test dual view**

Toggle dual view:
- 2 cột bằng nhau
- Separator dọc #f0f1f3
- Scroll independent mỗi cột

- [ ] **Step 6: Final commit**

```bash
git add -A
git commit -m "style(ui): final polish and visual alignment for light theme redesign"
```
