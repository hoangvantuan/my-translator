# Remove Dual View Button from Floating Controls

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Xoá button dual view (`btn-view-mode-float`) khỏi floating controls vì toolbar chính (`btn-view-mode`) đã có sẵn.

**Architecture:** Xoá HTML button, xoá JS event listener + cập nhật hàm `_updateViewModeButton`, giữ nguyên CSS `.float-btn` vì font buttons vẫn dùng.

**Tech Stack:** HTML, vanilla JS, CSS

---

### Task 1: Xoá button HTML và dọn JS references

**Files:**
- Modify: `src/index.html:197-202` (xoá button)
- Modify: `src/js/app.js:205-208` (xoá event listener)
- Modify: `src/js/app.js:1827,1834-1837` (xoá btnFloat logic trong `_updateViewModeButton`)

- [ ] **Step 1: Xoá button `btn-view-mode-float` khỏi HTML**

Trong `src/index.html`, xoá 6 dòng (197-202):

```html
<!-- XOÁ ĐOẠN NÀY -->
      <button id="btn-view-mode-float" class="float-btn" title="Toggle dual view">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="3" width="7" height="18" rx="1" />
          <rect x="14" y="3" width="7" height="18" rx="1" />
        </svg>
      </button>
```

Kết quả: `</div>` đóng `.opacity-control` (line 196) nối thẳng sang `</div>` đóng `.floating-controls` (line cũ 203, nay 197).

- [ ] **Step 2: Xoá event listener cho `btn-view-mode-float` trong `app.js`**

Trong `src/js/app.js`, xoá 4 dòng (205-208):

```js
// XOÁ ĐOẠN NÀY
        // View mode toggle (floating controls duplicate)
        document.getElementById('btn-view-mode-float')?.addEventListener('click', () => {
            this._toggleViewMode();
        });
```

- [ ] **Step 3: Dọn `_updateViewModeButton` trong `app.js`**

Trong `src/js/app.js`, hàm `_updateViewModeButton` (line 1825), xoá references tới `btnFloat`. Trước:

```js
    _updateViewModeButton(mode) {
        const btn = document.getElementById('btn-view-mode');
        const btnFloat = document.getElementById('btn-view-mode-float');
        const titles = { off: 'Original: off', below: 'Original: below translation', dual: 'Original: dual panel' };
        const title = titles[mode] || titles.below;
        if (btn) {
            btn.classList.toggle('active', mode !== 'off');
            btn.title = title;
        }
        if (btnFloat) {
            btnFloat.classList.toggle('active', mode !== 'off');
            btnFloat.title = title;
        }
    }
```

Sau:

```js
    _updateViewModeButton(mode) {
        const btn = document.getElementById('btn-view-mode');
        const titles = { off: 'Original: off', below: 'Original: below translation', dual: 'Original: dual panel' };
        const title = titles[mode] || titles.below;
        if (btn) {
            btn.classList.toggle('active', mode !== 'off');
            btn.title = title;
        }
    }
```

- [ ] **Step 4: Verify app chạy bình thường**

Run: `npm run tauri dev` hoặc mở app, kiểm tra:
1. Floating controls (góc dưới phải) chỉ có font size + color + opacity, không có icon dual view
2. Toolbar chính vẫn có button dual view (`btn-view-mode`) và hoạt động toggle off/below/dual
3. Settings panel radio group `show-original` vẫn hoạt động

- [ ] **Step 5: Commit**

```bash
git add src/index.html src/js/app.js
git commit -m "refactor(ui): remove duplicate dual-view button from floating controls"
```
