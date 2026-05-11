# Speaker-Aware Transcript UI

Cải tiến UI transcript để phân biệt trực quan giữa các người nói bằng color-coded timeline. Áp dụng cho cả one-way và two-way mode khi phát hiện ≥2 speakers.

## Bối cảnh

UI hiện tại xử lý transcript tuần tự, speaker label chỉ là text nhỏ "Speaker 1:" không có visual distinction. Trong two-way mode (2 người nói 2 ngôn ngữ khác nhau), rất khó phân biệt ai đang nói gì. Ngoài ra, `onTranslation` callback không truyền speaker/language metadata, khiến translation segment mất context.

## Quyết định thiết kế

- **Layout**: Color-coded Timeline (viền màu bên trái phân biệt speaker)
- **Scope**: Tự động kích hoạt khi ≥2 speakers được phát hiện, cả one-way và two-way
- **Màu sắc**: Palette cố định theo speaker number, không theo language
- **Backward compatible**: 1 speaker → UI giữ nguyên, không viền màu

## Speaker Color Palette

| Speaker | Color | Hex | CSS Variable |
|---------|-------|-----|--------------|
| Speaker 1 | Indigo | `#6366f1` | `--speaker-1` |
| Speaker 2 | Emerald | `#10b981` | `--speaker-2` |
| Speaker 3 | Amber | `#f59e0b` | `--speaker-3` |
| Speaker 4 | Rose | `#f43f5e` | `--speaker-4` |
| Speaker 5+ | Slate | `#64748b` | `--speaker-default` |

## Thay đổi chi tiết

### 1. soniox.js: Truyền speaker/language vào onTranslation

**Vấn đề**: `_handleResponse()` emit `onTranslation(text)` không có speaker, language. Translation segment mất hoàn toàn context ai là người nói.

**Giải pháp**: Emit `onTranslation(text, speaker, language)`. Biến `speaker` và `language` đã được extract từ original tokens trong cùng batch, chỉ cần truyền thêm.

```js
// Trước
this.onTranslation?.(translationText);

// Sau
this.onTranslation?.(translationText, speaker, language);
```

Callback signature trong `app.js` cũng cần cập nhật:
```js
// Trước
sonioxClient.onTranslation = (text) => { ... };

// Sau
sonioxClient.onTranslation = (text, speaker, language) => { ... };
```

### 2. ui.js: addTranslation nhận speaker/language

**Vấn đề**: `addTranslation(text)` chỉ nhận text, ghép vào original cũ nhất chưa dịch. Nếu original đã bị cleanup (stale 10s), translation tạo segment mới không có speaker.

**Giải pháp**: `addTranslation(text, speaker, language)`. Khi tạo segment mới (không tìm thấy original match), gắn speaker/language từ parameter.

```js
addTranslation(text, speaker, language) {
    const seg = this.segments.find(s => s.status === 'original');
    if (seg) {
        seg.translation = text;
        seg.status = 'translated';
        // speaker/language đã có từ addOriginal, không cần override
    } else {
        // Fallback: tạo segment mới VỚI speaker info
        this.segments.push({
            original: '',
            translation: text,
            status: 'translated',
            speaker: speaker || null,
            language: language || null,
            createdAt: Date.now(),
        });
    }
}
```

### 3. ui.js: _renderSingle() thêm color-coded border

**Logic chính**:
1. Đếm số unique speakers trong `this.segments`
2. Nếu ≥2 → kích hoạt color mode
3. Mỗi `seg-block` thêm class `speaker-border-{N}` dựa trên `seg.speaker`
4. Speaker label chỉ hiện khi speaker thay đổi so với segment trước (giữ nguyên logic hiện tại)

```js
_renderSingle() {
    const uniqueSpeakers = new Set(
        this.segments.map(s => s.speaker).filter(Boolean)
    );
    const multiSpeaker = uniqueSpeakers.size >= 2;

    // ... trong vòng lặp segments:
    const borderClass = (multiSpeaker && seg.speaker)
        ? ` speaker-border-${this._speakerIndex(seg.speaker)}`
        : '';
    html += `<div class="seg-block${borderClass}">`;
}
```

Helper method `_speakerIndex(speaker)`:
```js
_speakerIndex(speaker) {
    if (!this._speakerMap) this._speakerMap = new Map();
    if (!this._speakerMap.has(speaker)) {
        this._speakerMap.set(speaker, this._speakerMap.size + 1);
    }
    const idx = this._speakerMap.get(speaker);
    return idx <= 4 ? idx : 'default';
}
```

`_speakerMap` reset khi `clear()` hoặc `showPlaceholder()`.

### 4. ui.js: _renderDual() cập nhật tương tự

Áp dụng cùng logic color-coded border cho dual view. Mỗi `.seg-text` trong cả panel-source và panel-translation nhận border class tương ứng.

### 5. main.css: Thêm speaker border styles

```css
/* Speaker color borders */
:root {
    --speaker-1: #6366f1;
    --speaker-2: #10b981;
    --speaker-3: #f59e0b;
    --speaker-4: #f43f5e;
    --speaker-default: #64748b;
}

.speaker-border-1 { border-left: 3px solid var(--speaker-1); padding-left: 8px; }
.speaker-border-2 { border-left: 3px solid var(--speaker-2); padding-left: 8px; }
.speaker-border-3 { border-left: 3px solid var(--speaker-3); padding-left: 8px; }
.speaker-border-4 { border-left: 3px solid var(--speaker-4); padding-left: 8px; }
.speaker-border-default { border-left: 3px solid var(--speaker-default); padding-left: 8px; }
```

Cập nhật `.speaker-label` để dùng màu matching:
```css
.speaker-border-1 .speaker-label { color: var(--speaker-1); }
.speaker-border-2 .speaker-label { color: var(--speaker-2); }
.speaker-border-3 .speaker-label { color: var(--speaker-3); }
.speaker-border-4 .speaker-label { color: var(--speaker-4); }
.speaker-border-default .speaker-label { color: var(--speaker-default); }
```

Cập nhật `.lang-badge` tương tự:
```css
.speaker-border-1 .lang-badge {
    background: rgba(99, 102, 241, 0.1);
    color: var(--speaker-1);
    border-color: rgba(99, 102, 241, 0.3);
}
/* ... tương tự cho speaker 2, 3, 4 */
```

## Session log & Copy

`sessionLog` đã lưu `speaker` và `language` per segment. Không cần thay đổi `getFullSessionText()` hay `getPlainText()`.

## Edge Cases

| Case | Hành vi |
|------|---------|
| 1 speaker duy nhất | Không viền màu, UI y nguyên hiện tại |
| Speaker xuất hiện giữa chừng | `_speakerMap` gán index mới, viền bắt đầu từ segment đó |
| transcript_only mode | Viền màu vẫn hoạt động (dựa trên speaker, không phụ thuộc translation) |
| `clear()` gọi giữa session | Reset `_speakerMap`, bắt đầu gán lại từ đầu |
| Provisional text | Dùng viền màu của speaker hiện tại nếu biết, mặc định slate nếu chưa rõ |

## Files thay đổi

| File | Loại | Mô tả |
|------|------|-------|
| `src/js/soniox.js` | MODIFY | `onTranslation` thêm speaker, language params |
| `src/js/app.js` | MODIFY | Cập nhật `onTranslation` callback signature |
| `src/js/ui.js` | MODIFY | `addTranslation` nhận speaker/lang, render color borders, `_speakerIndex` helper |
| `src/styles/main.css` | MODIFY | Speaker border CSS, color variables |

## Verification

- Chạy existing tests: `npm test`
- Test thủ công: bật two-way mode với 2 ngôn ngữ, kiểm tra viền màu hiển thị đúng
- Test one-way mode với video có nhiều người nói
- Test 1 speaker → không viền
- Test clear/restart → speaker map reset
