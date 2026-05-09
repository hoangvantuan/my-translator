# Opacity Fixes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix 3 opacity-related issues: hidden text showing at low opacity, extend slider range to 0-100, add visual preview effect to opacity popover.

**Architecture:** All changes in frontend only (HTML/CSS/JS). No Rust backend changes needed since `overlay_opacity: f64` already accepts 0.0-1.0.

**Tech Stack:** Vanilla JS, CSS3, HTML range input

---

## File Map

- Modify: `src/index.html:188,193,520` (slider range, SVG icon)
- Modify: `src/js/app.js:264-290,862-876` (opacity logic, preview effect)
- Modify: `src/styles/main.css:370-384,730-745,128-143,228-243,878-934` (hidden element fixes, slider styling)

---

### Task 1: Fix hidden text showing at low opacity

**Problem:** Elements with `opacity: 0` (floating-toolbar, floating-controls, window-controls, status-label) are rendered but invisible. The floating-toolbar correctly uses both `opacity: 0` + `visibility: hidden`. But `.floating-controls`, `.window-controls`, `.status-label` only use `opacity: 0` without `visibility: hidden`, which may cause rendering artifacts on some platforms at low background opacity.

**Files:**
- Modify: `src/styles/main.css:128-143` (window-controls)
- Modify: `src/styles/main.css:228-243` (status-label)
- Modify: `src/styles/main.css:730-745` (floating-controls)

- [ ] **Step 1: Add `visibility: hidden` to `.window-controls` default state**

In `src/styles/main.css`, find the `.window-controls` block (~line 128):

```css
/* BEFORE */
.window-controls {
  position: absolute;
  left: 12px;
  top: 15px;
  display: flex;
  align-items: center;
  gap: 6px;
  opacity: 0;
  transition: opacity 0.3s ease 1.5s;
  z-index: 10;
}

#overlay-view:hover .window-controls {
  opacity: 1;
  transition: opacity 0.15s ease;
}

/* AFTER */
.window-controls {
  position: absolute;
  left: 12px;
  top: 15px;
  display: flex;
  align-items: center;
  gap: 6px;
  opacity: 0;
  visibility: hidden;
  transition: opacity 0.3s ease 1.5s, visibility 0s 1.8s;
  z-index: 10;
}

#overlay-view:hover .window-controls {
  opacity: 1;
  visibility: visible;
  transition: opacity 0.15s ease, visibility 0s;
}
```

- [ ] **Step 2: Add `visibility: hidden` to `.status-label` default state**

In `src/styles/main.css`, find `.status-label` (~line 228):

```css
/* BEFORE */
.status-label {
  position: absolute;
  top: 12px;
  right: 28px;
  display: flex;
  align-items: center;
  gap: 4px;
  opacity: 0;
  transition: opacity var(--transition-fast);
  z-index: 5;
  pointer-events: none;
}

#overlay-view.is-recording:hover .status-label {
  opacity: 1;
}

/* AFTER */
.status-label {
  position: absolute;
  top: 12px;
  right: 28px;
  display: flex;
  align-items: center;
  gap: 4px;
  opacity: 0;
  visibility: hidden;
  transition: opacity var(--transition-fast), visibility 0s 0.15s;
  z-index: 5;
  pointer-events: none;
}

#overlay-view.is-recording:hover .status-label {
  opacity: 1;
  visibility: visible;
  transition: opacity var(--transition-fast), visibility 0s;
}
```

- [ ] **Step 3: Add `visibility: hidden` to `.floating-controls` default state**

In `src/styles/main.css`, find `.floating-controls` (~line 730):

```css
/* BEFORE */
.floating-controls {
  position: absolute;
  bottom: 24px;
  right: 16px;
  display: flex;
  align-items: center;
  gap: 6px;
  z-index: 10;
  opacity: 0;
  transition: opacity 0.3s ease 1.5s;
}

#overlay-view:hover .floating-controls {
  opacity: 1;
  transition: opacity 0.15s ease;
}

/* AFTER */
.floating-controls {
  position: absolute;
  bottom: 24px;
  right: 16px;
  display: flex;
  align-items: center;
  gap: 6px;
  z-index: 10;
  opacity: 0;
  visibility: hidden;
  transition: opacity 0.3s ease 1.5s, visibility 0s 1.8s;
}

#overlay-view:hover .floating-controls {
  opacity: 1;
  visibility: visible;
  transition: opacity 0.15s ease, visibility 0s;
}
```

- [ ] **Step 4: Test at low opacity**

Run: `npm run tauri dev`

Test:
1. Kéo opacity slider xuống 20%
2. Verify: không còn text/UI artifacts nào hiện ra ở background
3. Hover vào overlay: toolbar, controls, window-controls vẫn hiện bình thường
4. Bỏ hover: tất cả ẩn đi mượt (fade + visibility transition)

- [ ] **Step 5: Commit**

```bash
git add src/styles/main.css
git commit -m "fix: add visibility:hidden to prevent ghost text at low overlay opacity"
```

---

### Task 2: Extend opacity range to 0-100

**Problem:** Slider min=20, chặn user không thể chỉnh xuống dưới 20%. Cần mở rộng về 0%.

**Files:**
- Modify: `src/index.html:193` (live popover slider min)
- Modify: `src/index.html:520` (settings tab slider min)

- [ ] **Step 1: Change live popover slider min to 0**

In `src/index.html` line 193, change `min="20"` to `min="0"`:

```html
<!-- BEFORE -->
<input type="range" id="range-opacity-live" min="20" max="100" value="85" />

<!-- AFTER -->
<input type="range" id="range-opacity-live" min="0" max="100" value="85" />
```

- [ ] **Step 2: Change settings tab slider min to 0**

In `src/index.html` line 520, change `min="20"` to `min="0"`:

```html
<!-- BEFORE -->
<input type="range" id="range-opacity" min="20" max="100" value="85" />

<!-- AFTER -->
<input type="range" id="range-opacity" min="0" max="100" value="85" />
```

- [ ] **Step 3: Test full range**

Run: `npm run tauri dev`

Test:
1. Kéo slider về 0%: overlay hoàn toàn trong suốt, chỉ thấy text + controls
2. Kéo slider về 100%: overlay trắng hoàn toàn
3. Kéo slider về 50%: semi-transparent
4. Vào Settings > Display tab: slider cũng cho phép 0-100
5. Save settings, restart app: giá trị opacity được giữ nguyên

- [ ] **Step 4: Commit**

```bash
git add src/index.html
git commit -m "feat: allow opacity range 0-100% instead of 20-100%"
```

---

### Task 3: Add visual preview effect to opacity popover slider

**Problem:** Slider track là màu đồng nhất, không có fill/gradient để chỉ vị trí hiện tại. User không nhìn thấy mức opacity đang chọn.

**Files:**
- Modify: `src/styles/main.css:907-926` (slider track styling)
- Modify: `src/js/app.js:264-290` (update slider fill on input)
- Modify: `src/js/app.js:862-876` (update slider fill on settings load)

- [ ] **Step 1: Add CSS for gradient fill on slider track**

In `src/styles/main.css`, replace the slider styling block (~line 907-926):

```css
/* BEFORE */
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

/* AFTER */
.opacity-popover input[type="range"] {
  width: 100px;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  border-radius: 2px;
  outline: none;
  cursor: pointer;
  background: linear-gradient(to right, var(--accent) 0%, var(--accent) 85%, var(--border-color) 85%, var(--border-color) 100%);
}

.opacity-popover input[type="range"]::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: var(--accent);
  cursor: pointer;
  border: none;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.15);
}
```

Note: the gradient percentages (85%) are initial values. JS will update them dynamically.

- [ ] **Step 2: Add checkerboard pattern behind the popover to preview opacity**

In `src/styles/main.css`, update `.opacity-popover` (~line 878):

```css
/* BEFORE */
.opacity-popover {
  position: absolute;
  bottom: calc(100% + 8px);
  right: -8px;
  transform: none;
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

/* AFTER */
.opacity-popover {
  position: absolute;
  bottom: calc(100% + 8px);
  right: -8px;
  transform: none;
  border-radius: var(--radius-md);
  padding: 8px 12px;
  box-shadow: var(--shadow-toolbar);
  display: flex;
  align-items: center;
  gap: 6px;
  z-index: 20;
  white-space: nowrap;
  background:
    linear-gradient(rgba(255,255,255, var(--preview-opacity, 0.85)), rgba(255,255,255, var(--preview-opacity, 0.85))),
    repeating-conic-gradient(#d1d5db 0% 25%, #f3f4f6 0% 50%) 0 0 / 8px 8px;
}
```

This adds a checkerboard pattern behind the popover, overlaid with a semi-transparent white layer. The `--preview-opacity` CSS variable will be updated by JS to match the slider value, giving live preview of the chosen opacity.

- [ ] **Step 3: Add JS to update slider fill and popover preview on input**

In `src/js/app.js`, find the opacity slider input handler (~line 264). Add slider fill + popover preview update:

```js
// BEFORE (inside opacitySlider input handler, line 264-290)
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
    ...
});

// AFTER
opacitySlider?.addEventListener('input', (e) => {
    const pct = parseInt(e.target.value);
    const opacity = pct / 100;

    // Real-time preview on overlay
    document.getElementById('overlay-view').style.backgroundColor =
        `rgba(255, 255, 255, ${opacity})`;
    opacityLabel.textContent = `${pct}%`;

    // Update trigger icon fill-opacity
    const fillCircle = opacityTrigger.querySelector('circle[fill-opacity]');
    if (fillCircle) fillCircle.setAttribute('fill-opacity', opacity);

    // Update slider track fill
    opacitySlider.style.background =
        `linear-gradient(to right, var(--accent) 0%, var(--accent) ${pct}%, var(--border-color) ${pct}%, var(--border-color) 100%)`;

    // Update popover preview opacity
    opacityPopover.style.setProperty('--preview-opacity', opacity);

    // Debounced auto-save
    ...
});
```

- [ ] **Step 4: Initialize slider fill + popover preview on settings load**

In `src/js/app.js`, find `_applySettings` (~line 862). After syncing slider value, also set fill + preview:

```js
// BEFORE (after line 873-875)
const liveLabel = document.querySelector('.opacity-label');
if (liveLabel) liveLabel.textContent = `${opacityPct}%`;
const triggerFill = document.querySelector('.opacity-trigger circle[fill-opacity]');
if (triggerFill) triggerFill.setAttribute('fill-opacity', opacity);

// AFTER
const liveLabel = document.querySelector('.opacity-label');
if (liveLabel) liveLabel.textContent = `${opacityPct}%`;
const triggerFill = document.querySelector('.opacity-trigger circle[fill-opacity]');
if (triggerFill) triggerFill.setAttribute('fill-opacity', opacity);

// Initialize slider fill
if (liveSlider) {
    liveSlider.style.background =
        `linear-gradient(to right, var(--accent) 0%, var(--accent) ${opacityPct}%, var(--border-color) ${opacityPct}%, var(--border-color) 100%)`;
}
// Initialize popover preview
const opacityPopover = document.querySelector('.opacity-popover');
if (opacityPopover) {
    opacityPopover.style.setProperty('--preview-opacity', opacity);
}
```

- [ ] **Step 5: Test visual feedback**

Run: `npm run tauri dev`

Test:
1. Mở opacity popover: slider track có fill gradient (accent color bên trái, gray bên phải)
2. Kéo slider: fill cập nhật real-time, popover background thay đổi theo opacity
3. Opacity 0%: popover hiện checkerboard pattern rõ
4. Opacity 100%: popover trắng hoàn toàn, không thấy checkerboard
5. Opacity 50%: checkerboard mờ phía sau
6. Đóng popover rồi mở lại: fill + preview đúng giá trị hiện tại

- [ ] **Step 6: Commit**

```bash
git add src/styles/main.css src/js/app.js
git commit -m "feat: add slider fill gradient and checkerboard preview to opacity popover"
```
