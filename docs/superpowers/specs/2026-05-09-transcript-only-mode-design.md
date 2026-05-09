# Transcript-only Mode

Thêm chế độ "chỉ transcript" vào app, bỏ qua bước dịch. Giảm latency, tiết kiệm API cost, phục vụ use case ghi nhận lời nói nguyên bản.

## Approach

Mở rộng field `translation_type` hiện có: thêm giá trị `"transcript_only"` bên cạnh `"one_way"` và `"two_way"`. Không cần field settings mới.

## Thay đổi chi tiết

### 1. HTML: Dropdown translation type

File: `src/index.html`

Thêm option đầu tiên vào `select-translation-type`:

```html
<option value="transcript_only">Transcript only — no translation</option>
<option value="one_way">One-way → translate all to target</option>
<option value="two_way">Two-way ↔ translate between two languages</option>
```

### 2. JS: UI settings logic

File: `src/js/app.js`, hàm `_updateTranslationTypeUI(type)`

Khi `type === 'transcript_only'`:
- Ẩn `section-oneway-langs` (source + target language dropdowns)
- Ẩn `section-twoway-langs`
- Ẩn `hint-twoway`
- Ẩn `section-strict-lang`
- TTS button vẫn enabled (đọc transcript gốc)

### 3. Soniox mode: Bỏ translation config

File: `src/js/soniox.js`, hàm `_doConnect()`

Khi `translationType === 'transcript_only'`: không gửi `configMsg.translation` trong WebSocket config message. Soniox chỉ trả `original` tokens, không có `translation` tokens.

File: `src/js/app.js`, hàm `_startSonioxMode()`

Truyền `translationType: 'transcript_only'` sang `sonioxClient.connect()`. Logic hiện tại đã truyền `settings.translation_type`, không cần thay đổi.

### 4. Local mode (MLX): Skip translation

File: `src/js/app.js`, hàm `_startLocalMode()`

Truyền flag cho local pipeline biết không cần dịch. Pipeline chỉ chạy speech-to-text, bỏ qua bước translation.

File: `src-tauri/src/commands/` (Rust pipeline commands)

Nhận thêm param `transcript_only: bool`. Khi `true`, pipeline không gọi translation model.

### 5. TTS: Đọc transcript gốc

File: `src/js/app.js`

Hiện tại TTS đọc `translationText` (bản dịch). Khi `transcript_only`, cần đọc `originalText` thay thế. Điều chỉnh callback `onTranslation` hoặc logic `_speakIfEnabled()` để xử lý.

### 6. UI hiển thị transcript

Không cần thay đổi logic render. Khi transcript-only:
- `translationText` luôn rỗng
- UI chỉ hiển thị original text
- `show_original` setting không ảnh hưởng (chỉ có 1 loại text)

### 7. Session metadata

`sessionMode` ghi nhận `'transcript_only'` cho auto-save transcript. Không cần thay đổi logic save.

## Files thay đổi

| File | Thay đổi |
|------|----------|
| `src/index.html` | Thêm option `transcript_only` vào dropdown |
| `src/js/app.js` | `_updateTranslationTypeUI()`, `_startLocalMode()`, TTS logic |
| `src/js/soniox.js` | `_doConnect()`: skip `translation` config khi transcript-only |
| `src-tauri/src/commands/` | Local pipeline: nhận `transcript_only` param |

## Không thay đổi

- `src-tauri/src/settings.rs`: không cần field mới, `translation_type` là dynamic JS field
- `src/js/ui.js`: render logic giữ nguyên
- `src/js/settings.js`: không cần migration
