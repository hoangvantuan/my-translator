# Color & Font Size UX Overhaul — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix font color persistence, expand preset colors with popup palette, adjust font step to 2px, and make secondary text follow user-chosen color at 75% opacity.

**Architecture:** Add `font_color` field to Rust Settings struct and JS DEFAULT_SETTINGS. Replace 3 inline color dots with 1 trigger dot + popup palette. When font color changes, set both `--transcript-font-color` and `--transcript-font-color-muted` (75% opacity variant) CSS variables. Secondary text (`.seg-original`, `.seg-provisional`) uses the muted variant.

**Tech Stack:** Rust (Tauri backend), vanilla JS, CSS custom properties

---

## File Map

| File | Action | Responsibility |
|------|--------|----------------|
| `src-tauri/src/settings.rs` | Modify | Add `font_color: String` field |
| `src/js/settings.js` | Modify | Add `font_color` to DEFAULT_SETTINGS |
| `src/styles/main.css` | Modify | Popup palette styles, muted color variable usage |
| `src/index.html` | Modify | Replace 3 color dots with trigger + popup |
| `src/js/app.js` | Modify | Popup handlers, persist/restore, font step, muted color |
| `src/js/ui.js` | Modify | Set `--transcript-font-color-muted` in configure() |

---

### Task 1: Rust backend — add `font_color` field

**Files:**
- Modify: `src-tauri/src/settings.rs:54-98` (Settings struct)
- Modify: `src-tauri/src/settings.rs:100-126` (Default impl)

- [ ] **Step 1: Add `font_color` field to Settings struct**

In `src-tauri/src/settings.rs`, add after line 66 (`pub font_size: u32`):

```rust
    /// Font color hex string
    pub font_color: String,
```

- [ ] **Step 2: Add default value in Default impl**

In the `Default for Settings` impl, add after `font_size: 16,` (line 108):

```rust
            font_color: "#111827".to_string(),
```

- [ ] **Step 3: Verify Rust compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles with no errors. `#[serde(default)]` on the struct handles backward compatibility with existing settings.json files that lack `font_color`.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/settings.rs
git commit -m "feat(settings): add font_color field to Settings struct"
```

---

### Task 2: JS settings — add `font_color` default

**Files:**
- Modify: `src/js/settings.js:8-27` (DEFAULT_SETTINGS)

- [ ] **Step 1: Add `font_color` to DEFAULT_SETTINGS**

In `src/js/settings.js`, add after line 14 (`font_size: 16,`):

```javascript
  font_color: '#111827',
```

- [ ] **Step 2: Commit**

```bash
git add src/js/settings.js
git commit -m "feat(settings): add font_color to JS DEFAULT_SETTINGS"
```

---

### Task 3: CSS — popup palette styles + muted color variable

**Files:**
- Modify: `src/styles/main.css:724-752` (color-controls section)
- Modify: `src/styles/main.css:517-519` (.seg-translated)
- Modify: `src/styles/main.css:525-532` (.seg-original)
- Modify: `src/styles/main.css:535-540` (.seg-provisional)

- [ ] **Step 1: Replace `.color-controls` and `.color-dot` styles with trigger + palette**

Replace the existing `.color-controls` block (lines 724-752) with:

```css
.color-controls {
  position: relative;
}

.color-trigger {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  padding: 0;
  outline: none;
  background: var(--transcript-font-color, var(--text-primary));
  box-shadow: var(--shadow-overlay);
  transition: all 0.15s;
}

.color-trigger:hover {
  transform: scale(1.15);
}

.color-palette {
  position: absolute;
  bottom: calc(100% + 6px);
  right: -4px;
  background: var(--bg-toolbar);
  border-radius: var(--radius-md);
  padding: 8px;
  box-shadow: var(--shadow-toolbar);
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 6px;
  z-index: 20;
}

.color-palette.hidden {
  display: none;
}

.color-palette .color-dot {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  padding: 0;
  outline: none;
  transition: all 0.15s;
}

.color-palette .color-dot:hover {
  transform: scale(1.2);
}

.color-palette .color-dot.active {
  border-color: var(--accent-border);
  box-shadow: 0 0 0 2px var(--accent-light);
}
```

- [ ] **Step 2: Update `.seg-original` to use muted font color variable**

Replace the `.seg-original` block (lines 525-532):

```css
.seg-original {
  display: block;
  color: var(--transcript-font-color-muted, var(--text-original));
  font-weight: 400;
  font-size: 0.75em;
  margin-top: 8px;
  margin-bottom: 2px;
}
```

- [ ] **Step 3: Update `.seg-provisional` to use muted font color variable**

Replace the `.seg-provisional` block (lines 535-540):

```css
.seg-provisional {
  display: block;
  color: var(--transcript-font-color-muted, var(--text-muted));
  font-weight: 400;
  font-style: italic;
}
```

- [ ] **Step 4: Commit**

```bash
git add src/styles/main.css
git commit -m "feat(css): popup palette styles, muted color variable for secondary text"
```

---

### Task 4: HTML — replace color dots with trigger + popup palette

**Files:**
- Modify: `src/index.html:162-166` (color-controls section)

- [ ] **Step 1: Replace the 3 color dots with trigger + popup**

Replace lines 162-166:

```html
      <div class="color-controls">
        <button class="color-dot active" data-color="#111827" title="Dark" style="background:#111827;"></button>
        <button class="color-dot" data-color="#92400e" title="Yellow" style="background:#92400e;"></button>
        <button class="color-dot" data-color="#164e63" title="Cyan" style="background:#164e63;"></button>
      </div>
```

With:

```html
      <div class="color-controls">
        <button class="color-trigger" title="Font color"></button>
        <div class="color-palette hidden">
          <button class="color-dot active" data-color="#111827" title="Dark" style="background:#111827;"></button>
          <button class="color-dot" data-color="#92400e" title="Amber" style="background:#92400e;"></button>
          <button class="color-dot" data-color="#164e63" title="Teal" style="background:#164e63;"></button>
          <button class="color-dot" data-color="#14532d" title="Forest" style="background:#14532d;"></button>
          <button class="color-dot" data-color="#7f1d1d" title="Wine" style="background:#7f1d1d;"></button>
          <button class="color-dot" data-color="#e5e7eb" title="Light" style="background:#e5e7eb;"></button>
        </div>
      </div>
```

- [ ] **Step 2: Commit**

```bash
git add src/index.html
git commit -m "feat(html): color trigger dot with popup palette (6 presets)"
```

---

### Task 5: ui.js — set muted color variable in configure()

**Files:**
- Modify: `src/js/ui.js:53-56` (fontColor handling in configure)

- [ ] **Step 1: Add `--transcript-font-color-muted` alongside font color**

Replace the fontColor block in `configure()` (lines 53-56):

```javascript
        if (fontColor !== undefined) {
            this.fontColor = fontColor;
            this.container.style.setProperty('--transcript-font-color', fontColor);
        }
```

With:

```javascript
        if (fontColor !== undefined) {
            this.fontColor = fontColor;
            this.container.style.setProperty('--transcript-font-color', fontColor);
            this.container.style.setProperty('--transcript-font-color-muted', this._hexToRgba(fontColor, 0.75));
        }
```

- [ ] **Step 2: Add `_hexToRgba` helper method**

Add this method to the `TranscriptUI` class (after the `configure` method, around line 58):

```javascript
    _hexToRgba(hex, alpha) {
        const r = parseInt(hex.slice(1, 3), 16);
        const g = parseInt(hex.slice(3, 5), 16);
        const b = parseInt(hex.slice(5, 7), 16);
        return `rgba(${r}, ${g}, ${b}, ${alpha})`;
    }
```

- [ ] **Step 3: Commit**

```bash
git add src/js/ui.js
git commit -m "feat(ui): set muted font color variable (75% opacity) for secondary text"
```

---

### Task 6: app.js — popup handlers, persist, restore, font step

**Files:**
- Modify: `src/js/app.js:211-212` (font step callers)
- Modify: `src/js/app.js:214-222` (color dot event listeners)
- Modify: `src/js/app.js:693-709` (_saveSettingsFromForm)
- Modify: `src/js/app.js:768-792` (_applySettings)

- [ ] **Step 1: Change font step from 4 to 2**

Replace lines 211-212:

```javascript
        document.getElementById('btn-font-up').addEventListener('click', () => this._adjustFontSize(4));
        document.getElementById('btn-font-down').addEventListener('click', () => this._adjustFontSize(-4));
```

With:

```javascript
        document.getElementById('btn-font-up').addEventListener('click', () => this._adjustFontSize(2));
        document.getElementById('btn-font-down').addEventListener('click', () => this._adjustFontSize(-2));
```

- [ ] **Step 2: Replace color dot listeners with popup palette logic**

Replace lines 214-222:

```javascript
        // Color dot controls
        document.querySelectorAll('.color-dot').forEach(dot => {
            dot.addEventListener('click', () => {
                document.querySelectorAll('.color-dot').forEach(d => d.classList.remove('active'));
                dot.classList.add('active');
                const color = dot.dataset.color;
                this.transcriptUI.configure({ fontColor: color });
            });
        });
```

With:

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

- [ ] **Step 3: Add `font_color` to _saveSettingsFromForm**

In `_saveSettingsFromForm()`, after line 708 (`show_original: ...`):

```javascript
            font_color: settingsManager.settings.font_color || '#111827',
```

- [ ] **Step 4: Restore font color in _applySettings**

In `_applySettings(settings)`, after the `this.transcriptUI.configure({` block (line 777-781), add `fontColor` to the configure call. Replace:

```javascript
            this.transcriptUI.configure({
                maxLines: settings.max_lines || 5,
                showOriginal: showOriginal,
                fontSize: settings.font_size || 16,
            });
```

With:

```javascript
            const fontColor = settings.font_color || '#111827';
            this.transcriptUI.configure({
                maxLines: settings.max_lines || 5,
                showOriginal: showOriginal,
                fontSize: settings.font_size || 16,
                fontColor: fontColor,
            });

            // Sync color trigger dot and palette active state
            const trigger = document.querySelector('.color-trigger');
            if (trigger) trigger.style.background = fontColor;
            document.querySelectorAll('.color-palette .color-dot').forEach(d => {
                d.classList.toggle('active', d.dataset.color === fontColor);
            });
```

- [ ] **Step 5: Commit**

```bash
git add src/js/app.js
git commit -m "feat(app): color popup handlers, font_color persist/restore, font step 2px"
```

---

### Task 7: Manual verification

- [ ] **Step 1: Build and launch app**

Run: `cd src-tauri && cargo tauri dev`

- [ ] **Step 2: Test color popup**

1. Hover floating controls (bottom-right) — see 1 color dot instead of 3
2. Click color dot — popup palette opens with 6 colors (grid 2x3)
3. Click a color (Wine) — popup closes, dot updates, text changes color
4. Click outside popup — popup closes

- [ ] **Step 3: Test color persistence**

1. Choose Teal color
2. Open Settings → Save Settings
3. Quit and relaunch app
4. Verify text is Teal, trigger dot shows Teal

- [ ] **Step 4: Test secondary text opacity**

1. Set show_original to "below"
2. Start transcription — verify original text is same color as translation but at 75% opacity
3. Set show_original to "off"
4. Start transcription — verify provisional text (đang transcript) is 75% opacity
5. Set show_original to "dual"
6. Verify both panels use full color, no opacity difference

- [ ] **Step 5: Test font step**

1. Click A+ button — font increases by 2px (not 4)
2. Click A− button — font decreases by 2px
3. Verify display shows correct size
4. Verify slider in Settings syncs

- [ ] **Step 6: Test backward compatibility**

1. Delete `font_color` from `~/Library/Application Support/com.personal.translator/settings.json`
2. Relaunch app — should default to Dark (#111827) without crash

- [ ] **Step 7: Commit any fixes from testing**

```bash
git add -A
git commit -m "fix: adjustments from manual testing"
```
