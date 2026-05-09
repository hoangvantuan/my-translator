# View Size Separation Design

## Vấn đề

Transcript (overlay), Settings, Sessions dùng chung 1 size window (600x400). Settings và Sessions cần nhiều không gian hơn để hiển thị form, danh sách.

## Giải pháp

Resize window tự động khi chuyển view. Resize trước, chuyển view sau (await) để tránh flash content.

## Size mỗi view

| View | Width | Height |
|------|-------|--------|
| overlay | 600 | 400 |
| settings | 800 | 600 |
| sessions | 800 | 600 |

## Chi tiết kỹ thuật

### Constant `VIEW_SIZES`

Khai báo ở đầu file hoặc trong class:

```js
const VIEW_SIZES = {
    overlay:  { width: 600, height: 400 },
    settings: { width: 800, height: 600 },
    sessions: { width: 800, height: 600 },
};
```

### Thay đổi `_showView(view)`

Hàm hiện tại chỉ toggle CSS class. Thay đổi:

1. Lấy size từ `VIEW_SIZES[view]`
2. Lấy size hiện tại qua `this.appWindow.innerSize()` + `scaleFactor()`
3. Nếu size khác, `await this.appWindow.setSize(new LogicalSize(width, height))`
4. Sau khi resize xong, toggle CSS class active cho view
5. Gọi `_populateSettingsForm()` hoặc `_showSessions()` như cũ

Hàm trở thành `async`.

### Window state (khởi động)

App luôn khởi động vào overlay view. `_restoreWindowPosition()` không cần thay đổi, sẽ restore size cuối cùng (overlay 600x400).

### Vị trí window

Giữ góc trên-trái cố định khi resize. `setSize()` của Tauri mặc định giữ top-left, nên không cần xử lý thêm.

### Edge cases

- **User tự kéo resize**: khi chuyển view, snap về size quy định. Hành vi nhất quán.
- **minWidth/minHeight**: giữ 400x200 trong tauri.conf.json, nhỏ hơn mọi view size, không gây xung đột.
- **`_showView` được gọi từ nhiều nơi**: tất cả đã dùng pattern `this._showView('settings')`, không cần thay caller.

## File thay đổi

| File | Thay đổi |
|------|----------|
| `src/js/app.js` | Thêm `VIEW_SIZES`, sửa `_showView()` thành async + resize |

## Không thay đổi

- `tauri.conf.json`: giữ nguyên initial size 600x400
- `_saveWindowPosition()`: giữ nguyên logic lưu
- `_restoreWindowPosition()`: giữ nguyên
- CSS: không cần thay đổi
