# View Size Separation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Window tự resize khi chuyển view: overlay 600x400, settings/sessions 800x600.

**Architecture:** Thêm constant `VIEW_SIZES` map view name → size. Sửa `_showView()` thành async: resize trước (await), toggle view sau. Giữ góc trên-trái, snap ngay.

**Tech Stack:** Tauri Window API (`LogicalSize`, `setSize`, `innerSize`, `scaleFactor`)

---

### Task 1: Thêm VIEW_SIZES constant và sửa _showView() thành async + resize

**Files:**
- Modify: `src/js/app.js:711-722` (hàm `_showView`)

- [ ] **Step 1: Thêm constant VIEW_SIZES ở đầu class hoặc trước hàm _showView**

Thêm ngay trên hàm `_showView` (dòng 709):

```js
// ─── Views ──────────────────────────────────────────────

static VIEW_SIZES = {
    overlay:  { width: 600, height: 400 },
    settings: { width: 800, height: 600 },
    sessions: { width: 800, height: 600 },
};
```

Nếu class không hỗ trợ static field (check cú pháp class hiện tại), khai báo constant ngoài class:

```js
const VIEW_SIZES = {
    overlay:  { width: 600, height: 400 },
    settings: { width: 800, height: 600 },
    sessions: { width: 800, height: 600 },
};
```

- [ ] **Step 2: Sửa _showView thành async với resize logic**

Thay toàn bộ hàm `_showView` (dòng 711-722) bằng:

```js
async _showView(view) {
    const target = VIEW_SIZES[view];
    if (target) {
        const { LogicalSize } = window.__TAURI__.window;
        const factor = await this.appWindow.scaleFactor();
        const current = await this.appWindow.innerSize();
        const currentW = Math.round(current.width / factor);
        const currentH = Math.round(current.height / factor);

        if (currentW !== target.width || currentH !== target.height) {
            await this.appWindow.setSize(new LogicalSize(target.width, target.height));
        }
    }

    document.getElementById('overlay-view').classList.toggle('active', view === 'overlay');
    document.getElementById('settings-view').classList.toggle('active', view === 'settings');
    document.getElementById('sessions-view').classList.toggle('active', view === 'sessions');

    if (view === 'settings') {
        this._populateSettingsForm();
    }
    if (view === 'sessions') {
        this._showSessions();
    }
}
```

Logic:
1. Lấy target size từ `VIEW_SIZES`
2. So sánh size hiện tại (logical) với target
3. Nếu khác, `await setSize()` trước
4. Sau khi resize xong, toggle CSS class
5. Populate form/sessions như cũ

- [ ] **Step 3: Test thủ công**

Chạy app (`cargo tauri dev`). Kiểm tra:
1. Click Settings → window resize lên 800x600
2. Click Back → window resize về 600x400
3. Click Sessions → window resize lên 800x600
4. Click Back → window resize về 600x400
5. Ấn Cmd+, → settings mở + resize
6. Ấn Escape từ settings → overlay + resize về
7. Kéo resize window bằng tay, chuyển view → snap về size quy định

- [ ] **Step 4: Commit**

```bash
git add src/js/app.js
git commit -m "feat(ui): auto-resize window when switching views

Overlay stays 600x400. Settings and sessions resize to 800x600.
Resize happens before view toggle to avoid content flash."
```
