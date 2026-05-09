# Opacity Popover Control Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Cho phép user điều chỉnh opacity background trực tiếp trên màn transcript qua popover slider, thay vì phải vào Settings.

**Architecture:** Thêm opacity trigger button vào `floating-controls`. Click trigger → hiện popover chứa mini range slider. Kéo slider → thay đổi real-time background overlay. Auto-save qua `settingsManager`. Dùng cùng pattern với `color-trigger` / `color-palette` đã có.

**Tech Stack:** HTML, CSS, vanilla JS (codebase hiện tại không dùng framework)

---

### Task 1: Thêm HTML cho opacity control

**Files:**
- Modify: `src/index.html:172` (sau `.color-controls`, trước `btn-view-mode-float`)

- [ ] **Step 1: Thêm opacity-control HTML vào floating-controls**

Chèn block sau giữa `</div>` đóng `.color-controls` (dòng 172) và `<button id="btn-view-mode-float"` (dòng 173):

```html
      <div class="opacity-control">
        <button class="opacity-trigger" title="Background opacity">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="1.5" fill="currentColor" fill-opacity="0.85"/>
            <circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="1.5" fill="none"/>
          </svg>
        </button>
        <div class="opacity-popover hidden">
          <input type="range" id="range-opacity-live" min="20" max="100" value="85" />
          <span class="opacity-label">85%</span>
        </div>
      </div>
```

SVG dùng 2 circle chồng nhau: circle trong có `fill-opacity` phản ánh mức opacity hiện tại, circle ngoài viền cố định.

- [ ] **Step 2: Verify HTML bằng cách mở app**

Run: `cargo tauri dev` (nếu chưa chạy)

Expected: Thấy icon circle mới xuất hiện trong floating-controls khi hover overlay. Chưa có style đẹp, chưa có interaction.

- [ ] **Step 3: Commit**

```bash
git add src/index.html
git commit -m "feat(ui): add opacity trigger and popover HTML to floating-controls"
```

---

### Task 2: CSS cho opacity trigger và popover

**Files:**
- Modify: `src/styles/main.css:787` (trước `/* Resize Handle */`)

- [ ] **Step 1: Thêm CSS cho opacity-control**

Chèn trước dòng `/* Resize Handle */` (dòng 789):

```css
/* Opacity control */
.opacity-control {
  position: relative;
}

.opacity-trigger {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  padding: 0;
  outline: none;
  background: transparent;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  box-shadow: var(--shadow-overlay);
  transition: all 0.15s;
}

.opacity-trigger:hover {
  transform: scale(1.15);
  color: var(--text-primary);
}

.opacity-popover {
  position: absolute;
  bottom: calc(100% + 8px);
  left: 50%;
  transform: translateX(-50%);
  background: var(--bg-toolbar);
  border-radius: var(--radius-md);
  padding: 8px 12px;
  box-shadow: var(--shadow-toolbar);
  display: flex;
  align-items: center;
  gap: 6px;
  z-index: 20;
  white-space: nowrap;
}

.opacity-popover::after {
  content: '';
  position: absolute;
  top: 100%;
  left: 50%;
  transform: translateX(-50%);
  border: 5px solid transparent;
  border-top-color: var(--bg-toolbar);
}

.opacity-popover.hidden {
  display: none;
}

.opacity-popover input[type="range"] {
  width: 100px;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  background: var(--border-color);
  border-radius: 2px;
  outline: none;
  cursor: pointer;
}

.opacity-popover input[type="range"]::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: var(--accent);
  cursor: pointer;
  border: none;
}

.opacity-label {
  font-size: 10px;
  color: var(--text-muted);
  min-width: 26px;
  text-align: right;
  font-variant-numeric: tabular-nums;
}
```

Pattern giống `color-palette`: position absolute phía trên trigger, cùng `var(--bg-toolbar)`, `var(--shadow-toolbar)`.

- [ ] **Step 2: Verify visual trong browser**

Expected: Opacity trigger hiển thị đúng kích thước cạnh color trigger. Popover vẫn hidden (chưa có JS).

- [ ] **Step 3: Commit**

```bash
git add src/styles/main.css
git commit -m "feat(css): add opacity trigger and popover styles"
```

---

### Task 3: JS event listeners cho popover toggle và click-outside

**Files:**
- Modify: `src/js/app.js:235-237` (sau color palette click-outside handler)

- [ ] **Step 1: Thêm opacity trigger toggle và click-outside**

Tìm đoạn code sau (dòng 213-237):

```javascript
        // Color trigger + palette
        const colorTrigger = document.querySelector('.color-trigger');
        const colorPalette = document.querySelector('.color-palette');

        colorTrigger?.addEventListener('click', (e) => {
            e.stopPropagation();
            colorPalette.classList.toggle('hidden');
        });

        document.querySelectorAll('.color-palette .color-dot').forEach(dot => {
            dot.addEventListener('click', (e) => {
                e.stopPropagation();
                document.querySelectorAll('.color-palette .color-dot').forEach(d => d.classList.remove('active'));
                dot.classList.add('active');
                const color = dot.dataset.color;
                this.transcriptUI.configure({ fontColor: color });
                colorTrigger.style.background = color;
                settingsManager.settings.font_color = color;
                colorPalette.classList.add('hidden');
            });
        });

        document.addEventListener('click', () => {
            colorPalette?.classList.add('hidden');
        });
```

Thay thế bằng (thêm opacity logic + hợp nhất click-outside):

```javascript
        // Color trigger + palette
        const colorTrigger = document.querySelector('.color-trigger');
        const colorPalette = document.querySelector('.color-palette');

        colorTrigger?.addEventListener('click', (e) => {
            e.stopPropagation();
            colorPalette.classList.toggle('hidden');
            opacityPopover?.classList.add('hidden');
        });

        document.querySelectorAll('.color-palette .color-dot').forEach(dot => {
            dot.addEventListener('click', (e) => {
                e.stopPropagation();
                document.querySelectorAll('.color-palette .color-dot').forEach(d => d.classList.remove('active'));
                dot.classList.add('active');
                const color = dot.dataset.color;
                this.transcriptUI.configure({ fontColor: color });
                colorTrigger.style.background = color;
                settingsManager.settings.font_color = color;
                colorPalette.classList.add('hidden');
            });
        });

        // Opacity trigger + popover
        const opacityTrigger = document.querySelector('.opacity-trigger');
        const opacityPopover = document.querySelector('.opacity-popover');
        const opacitySlider = document.getElementById('range-opacity-live');
        const opacityLabel = document.querySelector('.opacity-label');

        opacityTrigger?.addEventListener('click', (e) => {
            e.stopPropagation();
            opacityPopover.classList.toggle('hidden');
            colorPalette?.classList.add('hidden');
        });

        opacityPopover?.addEventListener('click', (e) => {
            e.stopPropagation();
        });

        // Click-outside: close both popovers
        document.addEventListener('click', () => {
            colorPalette?.classList.add('hidden');
            opacityPopover?.classList.add('hidden');
        });
```

Khi mở opacity popover → đóng color palette (và ngược lại). Click-outside đóng cả hai.

- [ ] **Step 2: Verify trong browser**

Expected: Click opacity trigger → popover hiện/ẩn. Click ngoài → popover đóng. Mở color palette → opacity popover đóng (và ngược lại).

- [ ] **Step 3: Commit**

```bash
git add src/js/app.js
git commit -m "feat(ui): add opacity popover toggle and click-outside handling"
```

---

### Task 4: Slider real-time preview và auto-save

**Files:**
- Modify: `src/js/app.js` (trong cùng block event listeners, sau opacityPopover click handler)

- [ ] **Step 1: Thêm slider input handler với real-time preview và debounced auto-save**

Thêm ngay sau `opacityPopover?.addEventListener('click', ...)` block:

```javascript
        let opacitySaveTimeout = null;

        opacitySlider?.addEventListener('input', (e) => {
            const pct = parseInt(e.target.value);
            const opacity = pct / 100;

            // Real-time preview
            document.getElementById('overlay-view').style.backgroundColor =
                `rgba(255, 255, 255, ${opacity})`;
            opacityLabel.textContent = `${pct}%`;

            // Update trigger icon fill-opacity
            const fillCircle = opacityTrigger.querySelector('circle[fill-opacity]');
            if (fillCircle) fillCircle.setAttribute('fill-opacity', opacity);

            // Debounced auto-save
            clearTimeout(opacitySaveTimeout);
            opacitySaveTimeout = setTimeout(async () => {
                const settings = settingsManager.get();
                settings.overlay_opacity = opacity;
                await settingsManager.save(settings);

                // Sync settings form slider
                const settingsSlider = document.getElementById('range-opacity');
                if (settingsSlider) settingsSlider.value = pct;
                const settingsValue = document.getElementById('opacity-value');
                if (settingsValue) settingsValue.textContent = `${pct}%`;
            }, 300);
        });
```

Kéo slider → background thay đổi tức thì. Sau 300ms ngừng kéo → save settings + đồng bộ settings form.

- [ ] **Step 2: Verify trong browser**

Expected:
1. Kéo slider → background overlay thay đổi real-time
2. Label % cập nhật
3. Trigger icon fill thay đổi theo
4. Mở Settings → giá trị slider đồng bộ đúng

- [ ] **Step 3: Commit**

```bash
git add src/js/app.js
git commit -m "feat(ui): opacity slider real-time preview and auto-save with debounce"
```

---

### Task 5: Đồng bộ từ Settings → overlay control

**Files:**
- Modify: `src/js/app.js:796-800` (trong `_applySettings`)

- [ ] **Step 1: Cập nhật `_applySettings` để sync opacity trigger và slider**

Tìm đoạn (dòng 796-800):

```javascript
    _applySettings(settings) {
        // Update overlay opacity
        const overlayView = document.getElementById('overlay-view');
        const opacity = settings.overlay_opacity !== undefined ? settings.overlay_opacity : 0.85;
        overlayView.style.backgroundColor = `rgba(255, 255, 255, ${opacity})`;
```

Thay thế bằng:

```javascript
    _applySettings(settings) {
        // Update overlay opacity
        const overlayView = document.getElementById('overlay-view');
        const opacity = settings.overlay_opacity !== undefined ? settings.overlay_opacity : 0.85;
        overlayView.style.backgroundColor = `rgba(255, 255, 255, ${opacity})`;

        // Sync opacity popover controls
        const opacityPct = Math.round(opacity * 100);
        const liveSlider = document.getElementById('range-opacity-live');
        if (liveSlider) liveSlider.value = opacityPct;
        const liveLabel = document.querySelector('.opacity-label');
        if (liveLabel) liveLabel.textContent = `${opacityPct}%`;
        const triggerFill = document.querySelector('.opacity-trigger circle[fill-opacity]');
        if (triggerFill) triggerFill.setAttribute('fill-opacity', opacity);
```

Khi user save trong Settings → `_applySettings` được gọi → đồng bộ slider, label, icon trên overlay.

- [ ] **Step 2: Verify luồng Settings → overlay**

1. Mở Settings, chỉnh opacity slider, Save
2. Quay overlay, hover floating-controls
3. Click opacity trigger

Expected: Slider value và label % khớp giá trị vừa save trong Settings.

- [ ] **Step 3: Commit**

```bash
git add src/js/app.js
git commit -m "feat(ui): sync opacity popover from settings on apply"
```

---

### Task 6: Kiểm tra tổng thể và dọn dẹp

**Files:**
- Review: `src/index.html`, `src/js/app.js`, `src/styles/main.css`

- [ ] **Step 1: Test golden path**

1. Mở app, hover overlay → thấy opacity trigger icon
2. Click trigger → popover hiện, slider đúng giá trị hiện tại
3. Kéo slider → background thay đổi real-time, label % cập nhật
4. Click ngoài → popover đóng
5. Thoát app, mở lại → opacity giữ nguyên giá trị đã chỉnh

- [ ] **Step 2: Test edge cases**

1. Kéo slider về 20% (min) → background gần trong suốt
2. Kéo slider về 100% (max) → background trắng hoàn toàn
3. Mở color palette → opacity popover tự đóng
4. Mở opacity popover → color palette tự đóng
5. Chỉnh opacity trên overlay → mở Settings → giá trị khớp
6. Chỉnh trong Settings → Save → overlay icon + slider khớp

- [ ] **Step 3: Run detect_changes**

```bash
# Verify only expected files changed
```

Dùng `gitnexus_detect_changes({scope: "compare", base_ref: "main"})` để xác nhận phạm vi thay đổi đúng mong đợi.

- [ ] **Step 4: Final commit nếu có dọn dẹp**

```bash
git add -A
git commit -m "chore: cleanup opacity popover implementation"
```
