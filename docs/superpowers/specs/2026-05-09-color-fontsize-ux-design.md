# Rà soát UX: Màu chữ & Font size

## Vấn đề hiện tại

| # | Vấn đề | Loại |
|---|--------|------|
| 1 | Màu chữ không persist, restart mất | Bug |
| 2 | Rust backend thiếu field `font_color` | Bug |
| 3 | `_applySettings()` không restore màu | Bug |
| 4 | 3 màu hardcode, không mở rộng được | Hạn chế |
| 5 | Font step 4px, thô ở cỡ nhỏ | UX |
| 6 | 3 color dots chiếm chỗ floating bar | UX |

## Quyết định

- Font size range: giữ 12-140px
- Font step: 4px → 2px
- Color: preset mở rộng 6 màu, không custom color picker
- Color UI: 1 dot trigger + popup palette thay 3 dots nằm hàng
- Color controls: chỉ floating, không thêm vào Settings panel
- Contrast: bỏ White, dùng Light Gray cho an toàn

## Thiết kế

### 1. Bảng màu preset (6 màu)

| Tên | Hex | Ghi chú |
|-----|-----|---------|
| Dark | `#111827` | Mặc định |
| Amber | `#92400e` | Giữ |
| Teal | `#164e63` | Giữ |
| Forest | `#14532d` | Mới |
| Wine | `#7f1d1d` | Mới |
| Light Gray | `#e5e7eb` | Mới, cho nền tối xuyên qua |

Toàn bộ đọc rõ trên overlay trắng bán trong suốt. Light Gray dùng khi opacity thấp, nền tối phía sau xuyên qua. Không cần logic contrast đặc biệt.

### 1b. Màu text phụ theo font color

Khi user đổi font color, text phụ cũng đổi theo nhưng ở opacity 75%:

| Chế độ | Text chính (full color) | Text phụ (opacity 75%) |
|--------|------------------------|----------------------|
| **Below** | `.seg-translated` (bản dịch) | `.seg-original` (bản gốc) |
| **Off** | Bản dịch | `.seg-provisional` (đang transcript) |
| **Dual** | Cả 2 panel full color | Không áp dụng, phân biệt bằng layout |

Cách triển khai: khi set `--transcript-font-color`, đồng thời set `--transcript-font-color-muted` = cùng hex + opacity 75%. CSS `.seg-original` và `.seg-provisional` dùng biến muted này thay vì hardcode.

### 2. Color dot + popup palette

**Floating controls** (thay thế 3 dots hiện tại):

```
┌─────────────────────┐
│ ● ● ●               │  ← popup palette (grid 2x3)
│ ● ● ●               │
└─────────────────────┘
   [A−] 16 [A+]  [●]  [⫼]   ← floating bar
```

**HTML**:

```html
<div class="color-controls">
  <button class="color-trigger" title="Font color">
    <!-- fill = màu hiện tại -->
  </button>
  <div class="color-palette">
    <!-- 6 preset dots, grid 2x3 -->
  </div>
</div>
```

**Hành vi**:
- `.color-trigger`: 1 dot hiển thị màu đang active
- Click trigger → toggle `.color-palette` (popup phía trên)
- Chọn màu trong palette → đóng popup, đổi màu, trigger dot cập nhật
- Click ngoài popup → đóng
- Không cần nút đóng riêng

**CSS**:
- `.color-palette`: `position: absolute; bottom: 100%; right: 0`
- Background trắng, shadow overlay, border-radius
- Grid 2x3, gap 6px, padding 8px
- Dot active: viền accent + glow (giữ style `.color-dot.active` hiện tại)
- Transition fade-in 150ms

### 3. Backend persist

**Rust** (`src-tauri/src/settings.rs`):
- Thêm field `font_color: String` vào struct `Settings`
- Default: `"#111827"`
- Backward compatible qua `#[serde(default)]`

**Không cần thay đổi** `get_settings` / `save_settings` commands vì chúng serialize/deserialize toàn bộ Settings struct.

### 4. Frontend restore + save

**`_applySettings(settings)`** (`src/js/app.js`):
- Thêm `fontColor: settings.font_color || '#111827'` vào `transcriptUI.configure()` call
- Highlight đúng dot trong palette matching `font_color`

**Color dot click handler**:
- Gọi `transcriptUI.configure({ fontColor: color })`
- Cập nhật `settingsManager.settings.font_color` in-memory (truy cập trực tiếp hoặc qua helper)
- Chưa save disk, save khi user bấm Save Settings

**`src/js/settings.js`**:
- Thêm `font_color: '#111827'` vào `DEFAULT_SETTINGS`

**Save settings flow**:
- Thu thập `font_color` từ state hiện tại vào payload gửi Tauri `save_settings`

### 5. Font step

**`_adjustFontSize(delta)`** (`src/js/app.js`):
- Đổi caller từ `_adjustFontSize(4)` thành `_adjustFontSize(2)`
- Logic clamp giữ nguyên `Math.max(12, Math.min(140, current + delta))`

## Files thay đổi

| File | Thay đổi |
|------|----------|
| `src-tauri/src/settings.rs` | Thêm field `font_color: String` |
| `src/index.html` | Thay 3 color dots → 1 trigger + popup palette |
| `src/js/app.js` | Persist logic, restore, font step 2px, popup handlers |
| `src/js/settings.js` | Thêm `font_color: '#111827'` vào `DEFAULT_SETTINGS` |
| `src/styles/main.css` | Style popup palette, color-trigger |

## Không thay đổi

- Font size range (giữ 12-140px)
- Settings panel Display tab (không thêm color vào đây)
- Slider font size trong Settings (giữ nguyên)
- Dual view: cả 2 panel dùng full font color, không áp dụng opacity

## Thay đổi bổ sung

- `TranscriptUI.configure()`: khi set `fontColor`, đồng thời set CSS variable `--transcript-font-color-muted` (cùng hex, opacity 75%)
- `.seg-original` CSS: đổi từ `var(--text-original)` sang `var(--transcript-font-color-muted, var(--text-original))`
- `.seg-provisional` CSS: đổi từ `var(--text-muted)` sang `var(--transcript-font-color-muted, var(--text-muted))`
