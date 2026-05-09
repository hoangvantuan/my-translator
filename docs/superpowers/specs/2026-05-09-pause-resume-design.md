# Pause/Resume: Phân biệt Stop vs Tạm dừng

## Bối cảnh

Hiện tại app có 2 trạng thái: idle và running. Nút Start/Stop toggle giữa 2 trạng thái. Khi Stop, session kết thúc: auto-save transcript, clear session metadata, hiện idle-overlay.

User cần tạm dừng mà không mất session. Pause giữ transcript + session metadata, cho phép Resume tiếp tục hoặc Stop kết thúc.

## State Machine

```
IDLE  →(start)→  RUNNING  →(pause)→  PAUSED  →(resume)→  RUNNING
                     ↓                    ↓
                  (stop)               (stop)
                     ↓                    ↓
                   IDLE                 IDLE
```

3 trạng thái quản lý bởi 2 flag: `isRunning` và `isPaused`.

| Trạng thái | isRunning | isPaused |
|------------|-----------|----------|
| Idle       | false     | false    |
| Running    | true      | false    |
| Paused     | false     | true     |

## Hành vi từng trạng thái

| Trạng thái | WebSocket/Pipeline | Audio capture | Transcript | Session metadata | TTS |
|------------|-------------------|---------------|------------|-----------------|-----|
| Running    | Mở                | Đang gửi      | Cập nhật   | Giữ             | Hoạt động |
| Paused     | Đóng              | Dừng          | Frozen     | Giữ             | Dừng |
| Idle       | Đóng              | Dừng          | Auto-saved | Reset           | Dừng |

## UI: Nút bấm

### Toolbar (btn-start)

- **Idle**: icon Play. Bấm = `start()`.
- **Running**: icon Pause (2 thanh dọc, thay icon Stop hình vuông hiện tại). Bấm = `pause()`.
- **Paused**: icon Play (sẵn sàng resume). Bấm = `resume()`. Paused-overlay cũng hiển thị song song.

### Paused Overlay (mới)

Overlay semi-transparent giữa màn hình, giống style idle-overlay. Transcript thấy mờ phía sau.

Chứa 2 nút lớn:
- **Tiếp tục** (icon Play): gọi `resume()`
- **Dừng lại** (icon Stop): gọi `stop()`

HTML mới: `<div id="paused-overlay">` chứa 2 button.

### Idle Overlay

Giữ nguyên. Chỉ hiện khi trạng thái idle (cả `isRunning` và `isPaused` đều false).

## Keyboard Shortcuts

- **Space khi idle**: start
- **Space khi running**: pause
- **Space khi paused**: resume
- **Escape khi paused**: stop (kết thúc session)

Logic hiện tại: Space toggle start/stop. Thay đổi: Space toggle start/pause/resume. Escape thêm mới cho stop khi paused.

## Logic Methods

### `pause()` (mới)

```
1. isPaused = true
2. isRunning = false
3. Dừng audio capture (invoke stop_capture)
4. Đóng WebSocket (sonioxClient.disconnect()) hoặc stop local pipeline
5. Dừng TTS (disconnect + audioPlayer.stop)
6. KHÔNG clear transcript
7. KHÔNG reset sessionStartTime
8. KHÔNG auto-save
9. Hiện paused-overlay
10. Cập nhật toolbar button
11. Cập nhật status = 'paused'
```

### `resume()` (mới)

```
1. isPaused = false
2. Ẩn paused-overlay
3. Gọi start() — start() đã handle: nếu transcript có content thì không clear
4. Transcript mới nối tiếp liền mạch vào cuối transcript cũ
```

### `stop()` (sửa nhỏ)

```
Giữ nguyên logic hiện tại + thêm:
1. isPaused = false (reset flag)
2. Ẩn paused-overlay (nếu đang hiện)
```

### `_updateStartButton()` (sửa)

Xử lý 3 trạng thái:
- Idle: icon Play, ẩn paused-overlay, hiện idle-overlay
- Running: icon Pause, ẩn cả 2 overlay
- Paused: ẩn idle-overlay, hiện paused-overlay

## CSS

### Paused Overlay

Style giống idle-overlay: position absolute, inset 0, flex center, backdrop blur nhẹ, background semi-transparent. 2 nút lớn cạnh nhau với gap.

Nút Resume: xanh lá (accent color). Nút Stop: đỏ hoặc xám.

## File thay đổi

| File | Thay đổi |
|------|----------|
| `src/index.html` | Thêm paused-overlay HTML, thêm icon-pause SVG vào btn-start |
| `src/js/app.js` | Thêm `isPaused` flag, methods `pause()` + `resume()`, sửa `stop()`, sửa `_updateStartButton()`, sửa keyboard shortcuts, sửa btn-start handler |
| `src/styles/main.css` | Thêm styles cho paused-overlay và 2 nút |

## Edge Cases

1. **Pause khi đang reconnect**: Nếu WS đang reconnect, pause vẫn disconnect ngay.
2. **Source change khi paused**: Nếu user đổi source (system/mic/both) khi paused, lưu setting mới. Resume sẽ dùng source mới.
3. **Settings change khi paused**: Cho phép mở Settings và thay đổi. Resume dùng settings mới.
4. **isStarting guard**: Pause chỉ hoạt động khi `isRunning && !isStarting`.
5. **Timer hiển thị**: `recordingStartTime` giữ nguyên khi paused (nếu có timer hiển thị, nó tạm dừng đếm).
