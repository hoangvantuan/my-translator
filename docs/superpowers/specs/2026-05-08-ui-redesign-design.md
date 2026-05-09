# UI Redesign: Light/Clean Subtitle Strip

## Tổng quan

Redesign toàn bộ UI của My Translator từ dark glassmorphism sang light/clean theme, tham khảo Linear/Arc Browser. Mục tiêu: cải thiện cả thẩm mỹ lẫn UX. App vẫn là overlay floating nhỏ trên các app khác.

## Quyết định thiết kế

| Quyết định | Chọn | Lý do |
|-----------|------|-------|
| Theme | Light/Clean | Tham khảo Linear/Arc, thoát dark glassmorphism |
| Overlay style | Subtitle Strip | Tối giản cực độ, chỉ hiện text khi không tương tác |
| Controls | Float toolbar phía trên khi hover | Content không thay đổi kích thước |
| Icons | SVG line icons, stroke #b0b5c0 inactive / #4f46e5 active | Đồng bộ, không emoji |
| Settings | Màn hình riêng, light redesign | Giữ flow hiện tại, làm đẹp hơn |
| Sessions | Màn hình riêng, light redesign | List + viewer |

## Design tokens (mới)

```css
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
```

## Overlay View

### 3 trạng thái

**1. Idle (chưa bắt đầu)**
- Card trắng bo 12px, shadow nhẹ
- Drag handle (thanh nhỏ 32x3px) ở top center
- Icon mic + text "Nhấn ▶ để bắt đầu dịch" ở giữa
- Nút "Bắt đầu" accent color ở giữa

**2. Đang dịch (không hover)**
- Card trắng, chỉ hiện:
  - Drag handle top center
  - Chấm xanh (5px) góc phải trên: trạng thái kết nối
  - Text dịch: 16px, #111827, font-weight 450
  - Text gốc: 12px, #b0b5c0, margin-top 8px
- Không có control bar, không có nút nào

**3. Đang dịch (hover)**
- Content card giữ nguyên kích thước (width + height không đổi)
- Toolbar ngang float phía trên bằng `position: absolute; top: -52px`
- Toolbar cách content 10px
- Status label xuất hiện góc trái trên card: chấm xanh + "JA → VI" (10px, #9ca3af)
- Animation: slideDown 150ms

### Floating toolbar (hover)

Thanh ngang bo 12px, nền trắng, shadow. Nằm giữa (centered) phía trên content.

4 nhóm nút, phân cách bằng separator dọc (1px x 20px, #e5e7eb), gap giữa nhóm 12px:

| Nhóm | Nút | Ghi chú |
|------|-----|---------|
| Source | System, Mic, Both | Pill group nền #f5f5f7, active nền #eef2ff stroke #4f46e5 |
| Display | TTS, Font (Aa), Dual view | Inactive stroke #b0b5c0 |
| Action | Copy, Sessions, Settings | Inactive stroke #b0b5c0 |
| Control | Stop, Pin | Stop stroke #ef4444, Pin active stroke #4f46e5 |

Mỗi nút: 30x30px, border-radius 6px. Icon: 14px, stroke-width 2.

### Dual view mode

Khi bật dual view:
- Content chia 2 cột bằng nhau, separator dọc 1px #f0f1f3 ở giữa
- Cột trái: text gốc, 14px, #6b7280
- Cột phải: text dịch, 14px, #111827, font-weight 450

### Floating controls (font size, color)

Giữ nguyên logic hiện tại, chuyển sang style light:
- Nền trắng, bo tròn, shadow nhẹ
- Xuất hiện bottom-right khi hover
- Font controls: A- / size display / A+
- Color dots: white (#111827 text), yellow (#92400e text), cyan (#164e63 text) cho text dịch
- Nút dual view toggle

### Compact mode

Giữ nguyên logic: toolbar ẩn hoàn toàn, hover vùng top 6px để hiện lại. Vì design mới mặc định đã ẩn toolbar, compact mode chỉ cần ẩn thêm drag handle và status dot.

## Settings View

Màn hình riêng thay thế overlay khi mở Settings.

### Header
- Nút Back (arrow left) bên trái
- Title "Settings" ở giữa, 15px, font-weight 600
- Nút Save (checkmark) bên phải, stroke accent

### Tab bar
- 4 tab: Translation, Display, TTS, About
- Tab active: text #4f46e5, underline 2px solid #4f46e5
- Tab inactive: text #9ca3af

### Form elements (chung)
- Label: 12px, uppercase, letter-spacing 0.5px, font-weight 600, color #6b7280
- Select/Input: nền #f9fafb, border 1px #e5e7eb, border-radius 8px, padding 10px 12px
- Hint text: 11px, #9ca3af
- Required badge: 10px, nền #fef2f2, text #ef4444, border-radius 4px

### Tab Translation
- Engine: select dropdown
- API Key: password input + toggle eye button
- Translation Type: button group (One-way / Two-way), active nền #eef2ff border #c7d2fe
- Languages: 2 select cạnh nhau, mũi tên → ở giữa
- Audio Source: button group 3 nút (System / Mic / Both)
- Endpoint Delay: slider, value hiển thị bên phải label
- Context sections: giữ nguyên logic, style light

### Tab Display
- Opacity: slider
- Font Size: slider
- Max Lines: slider
- Show original: checkbox

### Tab TTS
- Provider: select
- Settings theo provider (Edge/Google/ElevenLabs): giữ nguyên logic, style light

### Tab About
- App name + version
- Update checker
- Links

### Save button
- Full width, nền accent, text white, bo 10px
- Icon checkmark + "Save & Close"

## Sessions View

Màn hình riêng.

### Danh sách sessions
- Header: Back + "Sessions" + spacer
- Mỗi session item:
  - Language pair (13px, bold): "JA → VI"
  - Thời gian (11px, muted) góc phải: "14:30"
  - Preview text (12px, #6b7280): 1 dòng, ellipsis
  - Metadata (10px, #b0b5c0): "8 May 2026 · 12 phút"
  - Hover: nền #f8f9fb, bo 8px

### Session viewer
- Header: Back + language pair + date/time + nút Copy
- Nội dung: xen kẽ text gốc (#6b7280) và text dịch (#111827, weight 450)
- Mỗi cặp cách nhau 14px

## Scope không thay đổi

- Rust backend: không thay đổi
- Logic JS (app.js, soniox.js, TTS modules, settings.js): giữ nguyên logic, chỉ cập nhật DOM selectors nếu cần
- Tauri config: giữ nguyên window settings
- Font: giữ Inter

## Scope thay đổi

| File | Thay đổi |
|------|----------|
| src/styles/main.css | Viết lại hoàn toàn: light theme, design tokens mới |
| src/index.html | Restructure overlay view (bỏ control bar cố định, thêm floating toolbar), giữ nguyên Settings/Sessions structure, cập nhật class names |
| src/js/ui.js | Cập nhật hover logic cho floating toolbar, animation slideDown |

## Mockups

Mockups interactive tại `.superpowers/brainstorm/` (không commit):
- `overlay-v6.html`: Overlay final (toolbar + content)
- `settings-design.html`: Settings tab Translation
- `sessions-design.html`: Sessions list + viewer
