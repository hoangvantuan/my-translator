# Settings Fixes: Opacity, MaxLines, Show Original

## Tổng quan

Rà soát 3 settings: overlay_opacity, max_lines, show_original. Phát hiện 3 vấn đề, fix 2, giữ nguyên 1.

## Fix 1: Opacity → Background-only

### Vấn đề

1. **Default mismatch**: JS default = 0.85, Rust default = 1.0
2. **Áp dụng sai**: `style.opacity` ảnh hưởng toàn bộ element (text + background). Khi opacity < 1, text cũng bị mờ.

### Giải pháp

Dùng `background-color: rgba(255, 255, 255, opacity)` thay vì `style.opacity`. Text luôn rõ nét, chỉ background trong suốt.

### Thay đổi

| File | Dòng | Thay đổi |
|------|------|----------|
| `src-tauri/src/settings.rs` | 76 | `overlay_opacity: 1.0` → `0.85` |
| `src/js/app.js` | 769 | `overlayView.style.opacity = ...` → `overlayView.style.backgroundColor = rgba(255,255,255,${opacity})` |
| `src/styles/main.css` | 99 | `#overlay-view` background đổi sang `transparent` (giữ nguyên `--bg-primary` trong `:root` vì settings view và sessions view vẫn dùng) |

## Fix 2: Max Lines

### Vấn đề

Tên "max lines" nhưng logic dùng character count (`maxLines * 160`).

### Quyết định

Giữ nguyên. Character-based trimming hoạt động đủ tốt làm proxy cho visible content. Default đã khớp (5 ở cả JS và Rust). Không cần đổi.

## Fix 3: Show Original → 3 chế độ hiển thị

### Vấn đề

`show_original` setting hoàn toàn không hoạt động:
- `configure()` nhận param `showOriginal` nhưng không lưu, không dùng
- `_renderSingle()` không bao giờ hiện `seg.original` kèm bản dịch
- CSS class `.seg-original` đã define nhưng không có code nào tạo element dùng class này

### Giải pháp

Chuyển `show_original` từ boolean sang tri-state string:

| Giá trị | Hiển thị |
|---------|----------|
| `"off"` | Chỉ bản dịch |
| `"below"` | Bản dịch + gốc nhỏ/mờ phía dưới (single flow) |
| `"dual"` | 2 panel song song (source \| translation) |

### Backward compat

Khi đọc settings cũ:
- `show_original: true` → `"below"`
- `show_original: false` → `"off"`

### Thay đổi

#### settings.rs

- `show_original: bool` → `show_original: String`
- Default: `"below".to_string()`
- Serde deserialize cần handle cả bool (cũ) lẫn string (mới). Dùng custom deserializer hoặc `#[serde(deserialize_with)]` để map: `true` → `"below"`, `false` → `"off"`, string giữ nguyên

#### settings.js

- Default: `show_original: 'below'`

#### ui.js

- Constructor: thêm `this.showOriginal = 'below'`; xóa `this.viewMode`
- `configure()`: lưu `this.showOriginal = showOriginal` khi truyền vào
- `_render()`: dispatch theo `this.showOriginal`:
  - `'off'` → `_renderSingle()` (no original)
  - `'below'` → `_renderSingle()` (with original, dùng class `.seg-original`)
  - `'dual'` → `_renderDual()`
- `_renderSingle()`: khi `this.showOriginal === 'below'` và segment có `seg.original`, thêm:
  ```html
  <div class="seg-original">{original}</div>
  ```
  phía dưới `<div class="seg-translated">` trong `.seg-block`

#### app.js

- `_applySettings()`: truyền `showOriginal: settings.show_original || 'below'`
- `_toggleViewMode()`: cycle `off → below → dual → off` thay vì toggle single/dual
- Tooltip nút view mode cập nhật theo chế độ hiện tại
- `_populateSettingsForm()`: đọc string, set radio/select
- `_saveSettingsFromForm()`: lưu string

#### index.html (settings form)

- Checkbox "Show original" → radio group 3 option:
  - Off (chỉ bản dịch)
  - Below (gốc dưới bản dịch)
  - Dual (2 panel song song)

#### main.css

- Class `.seg-original` đã có sẵn, style phù hợp:
  - `color: var(--text-original)` (#b0b5c0)
  - `font-size: 0.75em`
  - `font-weight: 400`
- Không cần thêm CSS mới

## Ngoài phạm vi

- Không thay đổi max_lines logic
- Không thay đổi dual view layout
- Không thay đổi settings UI ngoài field show_original
