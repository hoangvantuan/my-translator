# Font Size: Sửa đối tượng chịu ảnh hưởng

## Vấn đề

CSS var `--transcript-font-size` set trên `.transcript-flow` (container), nhưng các phần tử con hardcode pixel. Khi user chỉnh font size qua A+/A- hoặc slider, chỉ `.seg-provisional` và `.speaker-label` thay đổi. Phần quan trọng nhất (bản dịch, nguyên bản, dual-view) không đổi.

## Quyết định

- **Bản dịch + nguyên bản** thay đổi theo font size
- **Speaker label, provisional** giữ nguyên (đã hoạt động đúng)
- **Tỉ lệ:** nguyên bản = 75% bản dịch
- **Dual-view:** cả hai panel cùng font size (không áp tỉ lệ 75%)

## Thay đổi

File duy nhất: `src/styles/main.css`

| Selector | Hiện tại | Sửa thành | Lý do |
|----------|----------|-----------|-------|
| `.seg-translated` (line 525) | `font-size: 16px` | `font-size: 1em` | Kế thừa container, scale theo var |
| `.seg-original` (line 533) | `font-size: 12px` | `font-size: 0.75em` | 75% bản dịch, scale tỉ lệ |
| `.dual-view .panel-source .seg-text` (line 642) | `font-size: 14px` | `font-size: 1em` | Cùng size với translation |
| `.dual-view .panel-translation .seg-text` (line 650) | `font-size: 14px` | `font-size: 1em` | Theo CSS var |

## Không thay đổi

- JS logic (`_adjustFontSize`, slider handler)
- Rust backend (`font_size` field)
- Range (12-140px) và bước nhảy (4px)
- Floating controls persistence

## Ví dụ

Font size = 24px:
- Bản dịch: 24px
- Nguyên bản: 18px (24 * 0.75)
- Dual-view source: 24px
- Dual-view translation: 24px

Font size = 32px:
- Bản dịch: 32px
- Nguyên bản: 24px
- Dual-view: 32px cả hai
