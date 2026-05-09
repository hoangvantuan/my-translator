# Settings Fixes: Opacity, Show Original Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix 2 broken settings: make opacity affect only background (not text), and make show_original actually work with 3 display modes (off/below/dual).

**Architecture:** Opacity changes from `style.opacity` (whole element) to `style.backgroundColor` with rgba alpha. Show_original changes from unused boolean to tri-state string (`off`/`below`/`dual`) that controls rendering in TranscriptUI. Rust settings struct gets custom serde deserializer for backward compat (bool → string migration).

**Tech Stack:** Rust (Tauri backend, serde), JavaScript (vanilla, no framework), CSS, HTML

---

## File Map

| File | Action | Responsibility |
|------|--------|---------------|
| `src-tauri/src/settings.rs` | Modify | Change `show_original` type, add custom deserializer, fix opacity default |
| `src/js/settings.js` | Modify | Update default values |
| `src/js/ui.js` | Modify | Replace `viewMode` with `showOriginal`, wire up rendering |
| `src/js/app.js` | Modify | Update settings application, view mode cycling, form read/write |
| `src/styles/main.css` | Modify | Make `#overlay-view` background transparent |
| `src/index.html` | Modify | Checkbox → radio group for show_original |

---

### Task 1: Rust settings — type migration + opacity default

**Files:**
- Modify: `src-tauri/src/settings.rs:24-95`

- [ ] **Step 1: Add custom deserializer for `show_original`**

Add this function above the `Settings` struct:

```rust
fn deserialize_show_original<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;

    struct ShowOriginalVisitor;

    impl<'de> de::Visitor<'de> for ShowOriginalVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a boolean or string (\"off\", \"below\", \"dual\")")
        }

        fn visit_bool<E: de::Error>(self, v: bool) -> Result<String, E> {
            Ok(if v { "below".to_string() } else { "off".to_string() })
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<String, E> {
            match v {
                "off" | "below" | "dual" => Ok(v.to_string()),
                _ => Ok("below".to_string()),
            }
        }
    }

    deserializer.deserialize_any(ShowOriginalVisitor)
}
```

- [ ] **Step 2: Change `show_original` field type and add annotation**

In the `Settings` struct, change:

```rust
    /// How to show original text: "off" | "below" | "dual"
    #[serde(deserialize_with = "deserialize_show_original")]
    pub show_original: String,
```

- [ ] **Step 3: Update default impl**

In `impl Default for Settings`, change:

```rust
            show_original: "below".to_string(),
```

And change:

```rust
            overlay_opacity: 0.85,
```

- [ ] **Step 4: Verify Rust compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles with no errors

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/settings.rs
git commit -m "feat(settings): migrate show_original bool→string, fix opacity default"
```

---

### Task 2: CSS — transparent overlay background

**Files:**
- Modify: `src/styles/main.css:96-104`

- [ ] **Step 1: Change `#overlay-view` background to transparent**

Current code at line 99:

```css
#overlay-view {
  position: relative;
  background: var(--bg-primary);
```

Change to:

```css
#overlay-view {
  position: relative;
  background: transparent;
```

The `--bg-primary` CSS variable stays in `:root` (line 13) because `#settings-view` and `#sessions-view` still use it.

- [ ] **Step 2: Commit**

```bash
git add src/styles/main.css
git commit -m "fix(css): overlay background transparent for opacity control"
```

---

### Task 3: JS settings defaults

**Files:**
- Modify: `src/js/settings.js:16`

- [ ] **Step 1: Update default `show_original`**

Change line 16:

```js
  show_original: 'below',
```

- [ ] **Step 2: Commit**

```bash
git add src/js/settings.js
git commit -m "fix(settings): update show_original default to 'below'"
```

---

### Task 4: TranscriptUI — wire `showOriginal` into rendering

**Files:**
- Modify: `src/js/ui.js:14-55` (constructor + configure)
- Modify: `src/js/ui.js:360-410` (_render + _renderSingle)

- [ ] **Step 1: Replace `viewMode` with `showOriginal` in constructor**

Change line 19:

```js
        this.showOriginal = 'below'; // 'off' | 'below' | 'dual'
```

Remove the line `this.viewMode = 'single';` (line 19 currently).

- [ ] **Step 2: Update `configure()` method**

Replace the current `configure` method (lines 36-54) with:

```js
    configure({ maxLines, showOriginal, fontSize, fontColor }) {
        if (maxLines !== undefined) this.maxChars = maxLines * 160;
        if (showOriginal !== undefined) {
            this.showOriginal = showOriginal;
            const overlay = document.getElementById('overlay-view');
            if (overlay) {
                overlay.classList.toggle('dual-view', showOriginal === 'dual');
            }
            this._render();
        }
        if (fontSize !== undefined) {
            this.fontSize = fontSize;
            this.container.style.setProperty('--transcript-font-size', `${fontSize}px`);
        }
        if (fontColor !== undefined) {
            this.fontColor = fontColor;
            this.container.style.setProperty('--transcript-font-color', fontColor);
        }
    }
```

Key changes: removed `viewMode` param, added `showOriginal` handling. The `dual-view` CSS class toggle stays because `main.css` uses `.dual-view` for panel layout.

- [ ] **Step 3: Update `_render()` dispatch**

Replace `_render()` method (line 360-369):

```js
    _render() {
        this._ensureContent();
        this._trimSegments();

        if (this.showOriginal === 'dual') {
            this._renderDual();
        } else {
            this._renderSingle();
        }
    }
```

- [ ] **Step 4: Update `_renderSingle()` to show original text when `showOriginal === 'below'`**

Replace the segment rendering block inside `_renderSingle()` (lines 389-396). The full updated method:

```js
    _renderSingle() {
        let html = '';
        let lastRenderedSpeaker = null;
        let lastRenderedLang = null;

        for (const seg of this.segments) {
            if (seg.speaker && seg.speaker !== lastRenderedSpeaker) {
                html += `<span class="speaker-label">Speaker ${seg.speaker}:</span> `;
                lastRenderedSpeaker = seg.speaker;
            }

            if (seg.language && seg.language !== lastRenderedLang) {
                html += `<span class="lang-badge">${this._langEmoji(seg.language)}</span> `;
                lastRenderedLang = seg.language;
            }

            if (seg.status === 'translated' && seg.translation) {
                const confidenceClass = (seg.confidence !== null && seg.confidence < 0.7) ? ' low-confidence' : '';
                html += `<div class="seg-block">`;
                html += `<div class="seg-translated${confidenceClass}">${this._esc(seg.translation)}</div>`;
                if (this.showOriginal === 'below' && seg.original) {
                    html += `<div class="seg-original">${this._esc(seg.original)}</div>`;
                }
                html += `</div>`;
            }
        }

        if (this.provisionalText) {
            if (this.provisionalSpeaker && this.provisionalSpeaker !== lastRenderedSpeaker) {
                html += `<span class="speaker-label">Speaker ${this.provisionalSpeaker}:</span> `;
            }
            if (this.provisionalLanguage && this.provisionalLanguage !== lastRenderedLang) {
                html += `<span class="lang-badge">${this._langEmoji(this.provisionalLanguage)}</span> `;
            }
            html += `<div class="seg-block"><div class="seg-provisional">${this._esc(this.provisionalText)}</div></div>`;
        }

        this.contentEl.innerHTML = html;
        this._smartScroll(this.container.parentElement || this.container);
    }
```

The only addition vs. current code: the `if (this.showOriginal === 'below' && seg.original)` block that adds `<div class="seg-original">` after the translated text. The CSS class `.seg-original` is already defined in main.css (line 525-532) with correct styling.

- [ ] **Step 5: Verify no remaining references to `viewMode`**

Run: `grep -n 'viewMode' src/js/ui.js`
Expected: no matches (all replaced with `showOriginal`)

- [ ] **Step 6: Commit**

```bash
git add src/js/ui.js
git commit -m "feat(ui): wire showOriginal tri-state into rendering"
```

---

### Task 5: App — opacity application + view mode cycling + settings form

**Files:**
- Modify: `src/js/app.js:766-777` (_applySettings)
- Modify: `src/js/app.js:1602-1608` (_toggleViewMode)
- Modify: `src/js/app.js:620-632` (_populateSettingsForm, show_original section)
- Modify: `src/js/app.js:703-706` (_saveSettingsFromForm, show_original)
- Modify: `src/js/app.js:200-208` (view mode button bindings)

- [ ] **Step 1: Fix opacity in `_applySettings()`**

Replace lines 768-769:

```js
        const opacity = settings.overlay_opacity !== undefined ? settings.overlay_opacity : 0.85;
        overlayView.style.backgroundColor = `rgba(255, 255, 255, ${opacity})`;
```

Remove any existing `overlayView.style.opacity = ...` line.

- [ ] **Step 2: Update `showOriginal` in `_applySettings()`**

Replace lines 773-776 (the `transcriptUI.configure` call):

```js
        if (this.transcriptUI) {
            const showOriginal = settings.show_original || 'below';
            this.transcriptUI.configure({
                maxLines: settings.max_lines || 5,
                showOriginal: showOriginal,
                fontSize: settings.font_size || 16,
            });
            this._updateViewModeButton(showOriginal);
        }
```

- [ ] **Step 3: Replace `_toggleViewMode()` with 3-state cycle**

Replace the method (lines 1602-1608):

```js
    _toggleViewMode() {
        const cycle = { off: 'below', below: 'dual', dual: 'off' };
        const current = this.transcriptUI.showOriginal || 'below';
        const next = cycle[current] || 'below';
        this.transcriptUI.configure({ showOriginal: next });
        this._updateViewModeButton(next);
    }

    _updateViewModeButton(mode) {
        const btn = document.getElementById('btn-view-mode');
        const btnFloat = document.getElementById('btn-view-mode-float');
        const titles = { off: 'Original: off', below: 'Original: below translation', dual: 'Original: dual panel' };
        const title = titles[mode] || titles.below;
        if (btn) {
            btn.classList.toggle('active', mode !== 'off');
            btn.title = title;
        }
        if (btnFloat) {
            btnFloat.classList.toggle('active', mode !== 'off');
            btnFloat.title = title;
        }
    }
```

- [ ] **Step 4: Update `_populateSettingsForm()` for show_original**

Replace lines 631-632:

```js
        const showOriginalVal = s.show_original || 'below';
        const showOriginalRadio = document.querySelector(`input[name="show-original"][value="${showOriginalVal}"]`);
        if (showOriginalRadio) showOriginalRadio.checked = true;
```

- [ ] **Step 5: Update `_saveSettingsFromForm()` for show_original**

Replace line 706:

```js
            show_original: document.querySelector('input[name="show-original"]:checked')?.value || 'below',
```

- [ ] **Step 6: Commit**

```bash
git add src/js/app.js
git commit -m "feat(app): background-only opacity, tri-state show_original, view mode cycle"
```

---

### Task 6: HTML — checkbox → radio group

**Files:**
- Modify: `src/index.html:508-511`

- [ ] **Step 1: Replace show_original checkbox with radio group**

Replace lines 508-511:

```html
          <label class="checkbox-option" style="margin-top: 8px;">
            <input type="checkbox" id="check-show-original" checked />
            <span class="checkbox-label">Show original text</span>
          </label>
```

With:

```html
          <div style="margin-top: 8px;">
            <span class="field-label">Original Text</span>
            <div class="radio-group" style="margin-top: 4px;">
              <label class="radio-option"><input type="radio" name="show-original" value="off" /><span class="radio-label">Off</span></label>
              <label class="radio-option"><input type="radio" name="show-original" value="below" checked /><span class="radio-label">Below</span></label>
              <label class="radio-option"><input type="radio" name="show-original" value="dual" /><span class="radio-label">Dual</span></label>
            </div>
          </div>
```

This follows the existing radio group pattern used for Audio Source (lines 447-451).

- [ ] **Step 2: Commit**

```bash
git add src/index.html
git commit -m "feat(html): show_original checkbox to radio group (off/below/dual)"
```

---

### Task 7: Manual verification

- [ ] **Step 1: Build and run**

```bash
cd src-tauri && cargo tauri dev
```

- [ ] **Step 2: Test opacity**

1. Open Settings → Display tab
2. Drag opacity slider to ~50%
3. Save settings
4. Verify: background becomes semi-transparent (can see desktop through it), but transcript text stays fully opaque and readable
5. Drag opacity to 100%, verify fully opaque white background

- [ ] **Step 3: Test show_original — Off mode**

1. Open Settings → Display tab
2. Select "Off" radio for Original Text
3. Save settings
4. Start recording, wait for translated segments
5. Verify: only translated text appears, no original text visible

- [ ] **Step 4: Test show_original — Below mode**

1. Open Settings, select "Below" radio
2. Save, start recording
3. Verify: translated text appears with original text smaller and dimmer underneath each translation

- [ ] **Step 5: Test show_original — Dual mode**

1. Open Settings, select "Dual" radio
2. Save, start recording
3. Verify: two side-by-side panels appear (source | translation)

- [ ] **Step 6: Test toolbar view mode button**

1. Click view mode button in toolbar
2. Verify it cycles: off → below → dual → off
3. Check tooltip updates with each state
4. Floating view mode button (bottom-right) should also cycle

- [ ] **Step 7: Test backward compat**

1. Manually edit `~/Library/Application Support/com.personal.translator/settings.json`
2. Set `"show_original": true` (old boolean format)
3. Restart app
4. Verify app loads without error and treats `true` as "below" mode
5. Set `"show_original": false`, restart, verify treated as "off" mode

- [ ] **Step 8: Final commit (if any fixups needed)**

```bash
git add -A
git commit -m "fix: settings fixes polish"
```
