# Plan: Bỏ text selection, thêm drag anywhere cho overlay card

## Bối cảnh
- App Tauri overlay (floating window), transcript hiển thị text dịch
- Hiện có `data-tauri-drag-region` trên `#transcript-container` và `#transcript-content` nhưng text nodes bên trong vẫn bắt sự kiện select, chặn drag
- Copy đã có 2 button riêng (`btn-copy`, `btn-session-copy`) nên text selection không cần
- Settings view đã dùng `appWindow.startDragging()` trên mousedown (app.js:316-322) hoạt động tốt

## Thay đổi

### Task 1: Thêm drag anywhere cho overlay-view (app.js)
- Thêm mousedown listener trên `#overlay-view` tương tự settings-view
- Khi mousedown trên vùng không phải interactive element (button, input, select...), gọi `appWindow.startDragging()`
- Exclude thêm: `.floating-controls`, `.floating-toolbar`, `.window-controls`, `#resize-handle`

**File**: `src/js/app.js`
**Vị trí**: Sau block settings-view mousedown (line ~322)
**Code**:
```js
document.getElementById('overlay-view')?.addEventListener('mousedown', (e) => {
    const interactive = e.target.closest('button, input, select, label, a, textarea, .floating-controls, .floating-toolbar, .window-controls, #resize-handle');
    if (!interactive && e.buttons === 1) {
        e.preventDefault();
        this.appWindow.startDragging();
    }
});
```

### Task 2: Xóa `data-tauri-drag-region` khỏi transcript elements (index.html)
- Bỏ `data-tauri-drag-region` khỏi `#transcript-container` (line 150) và `#transcript-content` (line 151)
- Giữ `data-tauri-drag-region` trên `#drag-region` (line 18), `.drag-handle` (line 36), `#idle-overlay` (line 145) vì chúng hoạt động đúng

**File**: `src/index.html`

### Task 3: Verify không có `user-select: text` override (CSS)
- Kiểm tra không có rule nào override `user-select` cho transcript text
- Body đã có `user-select: none` (line 74), đủ rồi

## Verify
- `gitnexus_impact` cho symbols bị sửa
- Build kiểm tra lỗi
- Test: click-drag trên text transcript phải drag window, không select text
- Test: buttons/controls vẫn clickable
- Test: scroll transcript vẫn hoạt động (scroll !== drag)
