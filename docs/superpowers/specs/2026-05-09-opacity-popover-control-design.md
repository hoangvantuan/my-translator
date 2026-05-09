# Opacity Popover Control trên màn Transcript

## Vấn đề

Hiện tại opacity chỉ chỉnh được trong Settings page. User phải: mở Settings → kéo slider → Save → quay lại overlay mới thấy kết quả. Vòng lặp dài, không có feedback trực quan tức thì.

## Giải pháp

Thêm opacity dot vào `floating-controls` (góc dưới phải overlay). Click dot → hiện popover chứa mini slider. Kéo slider → background thay đổi real-time. Auto-save khi chỉnh xong.

## Thiết kế chi tiết

### Vị trí

Trong `floating-controls`, giữa `color-controls` và `btn-view-mode-float`.

### HTML

```html
<div class="opacity-control">
  <button class="opacity-dot" title="Background opacity">
    <!-- Circle với fill opacity tương ứng giá trị hiện tại -->
  </button>
  <div class="opacity-popover" style="display:none">
    <input type="range" id="range-opacity-live" min="20" max="100" value="85" />
    <span class="opacity-label">85%</span>
  </div>
</div>
```

### Opacity dot icon

Circle hiển thị mức opacity hiện tại bằng fill opacity. User nhận biết mức opacity bằng mắt mà không cần mở popover.

### Popover behavior

- Click dot → toggle popover phía trên
- Popover vị trí: phía trên dot, căn giữa, có mũi tên nhỏ chỉ xuống
- Click bất kỳ đâu ngoài popover → đóng
- Kéo slider → background thay đổi real-time (`rgba(255,255,255, value)`)
- Thả slider hoặc debounce 300ms → auto-save qua `settingsManager.save()`

### Đồng bộ với Settings page

- Chỉnh trên overlay → cập nhật giá trị `range-opacity` trong settings form
- Chỉnh trong settings → khi save, `_applySettings()` cập nhật dot icon
- Một nguồn sự thật: `settingsManager` giữ giá trị, cả hai UI đọc/ghi cùng chỗ

### CSS

- `.opacity-dot`: circle button, kích thước tương tự color dots
- `.opacity-popover`: position absolute, phía trên dot, nền trắng, border-radius, shadow nhẹ, mũi tên chỉ xuống
- `.opacity-popover input[type=range]`: slider nhỏ, chiều ngang ~100px
- `.opacity-label`: font-size nhỏ, hiển thị giá trị %

## Phạm vi thay đổi

| File | Thay đổi |
|------|----------|
| `src/index.html` | Thêm opacity dot + popover vào floating-controls |
| `src/js/app.js` | Event listeners: click dot toggle popover, slider input thay đổi real-time, click-outside đóng popover, auto-save debounce, đồng bộ hai chiều với settings form |
| CSS (inline style block trong index.html) | Style cho opacity-dot, popover, slider mini |

### Không thay đổi

- Settings page giữ nguyên slider `range-opacity` hiện tại
- Backend `overlay_opacity` trong `settings.rs` không đổi
- `settingsManager` API không đổi
