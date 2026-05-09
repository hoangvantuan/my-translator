# Transcript-only Mode Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a "transcript only" mode that skips translation entirely, showing only the original transcribed text.

**Architecture:** Extend the existing `translation_type` field with a `"transcript_only"` value. When selected, Soniox receives no `translation` config (no translation tokens returned), local pipeline skips LLM translation, and UI hides language target settings.

**Tech Stack:** HTML, JavaScript (frontend), Rust/Tauri (backend commands), Python (local pipeline)

---

### Task 1: Add `transcript_only` option to HTML dropdown

**Files:**
- Modify: `src/index.html:243-246`

- [ ] **Step 1: Add the new option as first in the dropdown**

In `src/index.html`, find the `select-translation-type` dropdown (line 243-246) and add `transcript_only` as the first option:

```html
<select id="select-translation-type">
  <option value="transcript_only">Transcript only — no translation</option>
  <option value="one_way">One-way → translate all to target</option>
  <option value="two_way">Two-way ↔ translate between two languages</option>
</select>
```

- [ ] **Step 2: Commit**

```bash
git add src/index.html
git commit -m "feat(html): add transcript_only option to translation type dropdown"
```

---

### Task 2: Update `_updateTranslationTypeUI()` to handle `transcript_only`

**Files:**
- Modify: `src/js/app.js:918-944`

- [ ] **Step 1: Rewrite `_updateTranslationTypeUI` with three branches**

In `src/js/app.js`, replace the `_updateTranslationTypeUI` method (lines 918-944) with:

```javascript
_updateTranslationTypeUI(type) {
    const oneway = document.getElementById('section-oneway-langs');
    const twoway = document.getElementById('section-twoway-langs');
    const hintTwoway = document.getElementById('hint-twoway');
    const strictLang = document.getElementById('section-strict-lang');

    if (type === 'transcript_only') {
        if (oneway) oneway.style.display = 'none';
        if (twoway) twoway.style.display = 'none';
        if (hintTwoway) hintTwoway.style.display = 'none';
        if (strictLang) strictLang.style.display = 'none';
        this._updateTTSButton();
    } else if (type === 'two_way') {
        if (oneway) oneway.style.display = 'none';
        if (twoway) twoway.style.display = 'flex';
        if (hintTwoway) hintTwoway.style.display = 'block';
        if (strictLang) strictLang.style.display = 'none';
        if (this.ttsEnabled) {
            this.ttsEnabled = false;
            this._getActiveTTS().disconnect();
            audioPlayer.stop();
        }
        this._updateTTSButton();
    } else {
        if (oneway) oneway.style.display = 'flex';
        if (twoway) twoway.style.display = 'none';
        if (hintTwoway) hintTwoway.style.display = 'none';
        if (strictLang) strictLang.style.display = 'flex';
        this._updateTTSButton();
    }
}
```

- [ ] **Step 2: Verify in browser**

Run the dev server, open Settings, switch the Translation Type dropdown through all three values. Confirm:
- `transcript_only`: source/target language sections hidden
- `one_way`: source + target visible, strict lang visible
- `two_way`: two-way lang section visible, TTS disabled

- [ ] **Step 3: Commit**

```bash
git add src/js/app.js
git commit -m "feat(ui): handle transcript_only in translation type UI"
```

---

### Task 3: Skip Soniox translation config when `transcript_only`

**Files:**
- Modify: `src/js/soniox.js:117-131`

- [ ] **Step 1: Add guard for `transcript_only` in `_doConnect`**

In `src/js/soniox.js`, find the translation config block (lines 117-131). Replace it with:

```javascript
// Translation config (#5: support one-way, two-way, and transcript-only)
if (translationType === 'transcript_only') {
    // No translation config — Soniox returns original tokens only
} else if (translationType === 'two_way' && languageA && languageB) {
    configMsg.translation = {
        type: 'two_way',
        language_a: languageA,
        language_b: languageB,
    };
    configMsg.language_hints = [languageA, languageB];
} else if (targetLanguage) {
    configMsg.translation = {
        type: 'one_way',
        target_language: targetLanguage,
    };
}
```

- [ ] **Step 2: Commit**

```bash
git add src/js/soniox.js
git commit -m "feat(soniox): skip translation config for transcript_only mode"
```

---

### Task 4: Wire TTS to speak original text when `transcript_only`

**Files:**
- Modify: `src/js/app.js:443-449` (Soniox callbacks)
- Modify: `src/js/app.js:1234-1244` (`_handleLocalPipelineResult`)

- [ ] **Step 1: Update Soniox `onOriginal` callback to speak when transcript-only**

In `src/js/app.js`, find the Soniox callback wiring (lines 443-449). Replace with:

```javascript
sonioxClient.onOriginal = (text, speaker, language) => {
    this.transcriptUI.addOriginal(text, speaker, language);
    const translationType = settingsManager.get().translation_type || 'one_way';
    if (translationType === 'transcript_only') {
        this._speakIfEnabled(text);
    }
};

sonioxClient.onTranslation = (text) => {
    this.transcriptUI.addTranslation(text);
    this._speakIfEnabled(text);
};
```

- [ ] **Step 2: Update `_handleLocalPipelineResult` to speak original when transcript-only**

In `src/js/app.js`, find the `case 'result':` handler (lines 1234-1245). Replace with:

```javascript
case 'result':
    if (data.original) {
        this.transcriptUI.addOriginal(data.original);
    }
    setTimeout(() => {
    if (data.translated) {
        this.transcriptUI.addTranslation(data.translated);
        this._speakIfEnabled(data.translated);
    } else if (data.original) {
        this._speakIfEnabled(data.original);
    }
    }, 80);
    break;
```

This handles both cases: when translation exists (speak translation), when it doesn't (transcript-only, speak original).

- [ ] **Step 3: Commit**

```bash
git add src/js/app.js
git commit -m "feat(tts): speak original text in transcript_only mode"
```

---

### Task 5: Add `--transcript-only` flag to local pipeline

**Files:**
- Modify: `scripts/local_pipeline.py:48-54` (constructor)
- Modify: `scripts/local_pipeline.py:329-348` (`_process_chunk` translation section)
- Modify: `scripts/local_pipeline.py:402-414` (argparse)
- Modify: `scripts/local_pipeline.py:416-453` (main, pass arg to pipeline)

- [ ] **Step 1: Add `transcript_only` param to `LocalPipeline.__init__`**

In `scripts/local_pipeline.py`, modify the constructor (line 48-55) to accept `transcript_only`:

```python
class LocalPipeline:
    def __init__(
        self,
        asr_model="whisper",
        source_lang="ja",
        target_lang="vi",
        chunk_seconds=7,
        stride_seconds=5,
        transcript_only=False,
    ):
```

Add after line 58 (`self.target_lang = target_lang`):

```python
        self.transcript_only = transcript_only
```

- [ ] **Step 2: Skip LLM loading and translation when `transcript_only`**

In the `setup()` method, find where LLM is loaded (around lines 120-128). Wrap it:

```python
        if not self.transcript_only:
            log("Loading LLM (gemma-3-4b)...")
            emit({"type": "status", "message": "Loading translation model..."})
            t = time.time()
            from mlx_lm import load
            self.llm_model, self.llm_tokenizer = load("mlx-community/gemma-3-4b-it-qat-4bit")
            log(f"LLM loaded in {time.time()-t:.1f}s")

            log("Warming up LLM...")
            emit({"type": "status", "message": "Warming up translator..."})
            self._translate("テスト")
```

In `_process_chunk`, find the translation block (lines 329-348). Replace with:

```python
            if self.transcript_only:
                total = time.time() - t_start
                log(f"ASR={t_asr:.2f}s total={total:.2f}s (transcript only)")
                emit({
                    "type": "result",
                    "original": new_text,
                    "translated": None,
                    "language": lang if isinstance(lang, str) else (lang[0] if lang else "ja"),
                    "timing": {
                        "asr": round(t_asr, 2),
                        "translate": 0,
                        "total": round(total, 2),
                    },
                })
            else:
                t2 = time.time()
                translated = self._translate(new_text)
                t_llm = time.time() - t2

                total = time.time() - t_start
                log(f"ASR={t_asr:.2f}s LLM={t_llm:.2f}s total={total:.2f}s")

                emit({
                    "type": "result",
                    "original": new_text,
                    "translated": translated,
                    "language": lang if isinstance(lang, str) else (lang[0] if lang else "ja"),
                    "timing": {
                        "asr": round(t_asr, 2),
                        "translate": round(t_llm, 2),
                        "total": round(total, 2),
                    },
                })
```

- [ ] **Step 3: Add `--transcript-only` CLI argument**

In `main()` (line 405-413), add after the `--test-file` argument:

```python
    parser.add_argument("--transcript-only", action="store_true",
                        help="Transcribe only, skip translation")
```

- [ ] **Step 4: Pass `transcript_only` to `LocalPipeline` in both test and normal modes**

In `main()`, update both pipeline constructors (lines 418-423 and 446-452) to include `transcript_only=args.transcript_only`:

Test mode (line 418):
```python
        pipeline = LocalPipeline(
            asr_model=args.asr_model,
            source_lang=args.source_lang,
            target_lang=args.target_lang,
            chunk_seconds=args.chunk_seconds,
            stride_seconds=args.stride_seconds,
            transcript_only=args.transcript_only,
        )
```

Normal mode (line 446):
```python
        pipeline = LocalPipeline(
            asr_model=args.asr_model,
            source_lang=args.source_lang,
            target_lang=args.target_lang,
            chunk_seconds=args.chunk_seconds,
            stride_seconds=args.stride_seconds,
            transcript_only=args.transcript_only,
        )
```

- [ ] **Step 5: Commit**

```bash
git add scripts/local_pipeline.py
git commit -m "feat(pipeline): add --transcript-only flag to skip translation"
```

---

### Task 6: Pass `transcript_only` from Rust command to Python pipeline

**Files:**
- Modify: `src-tauri/src/commands/local_pipeline.rs:33-36` (function signature)
- Modify: `src-tauri/src/commands/local_pipeline.rs:98-111` (Command builder)

- [ ] **Step 1: Add `transcript_only` param to `start_local_pipeline`**

In `src-tauri/src/commands/local_pipeline.rs`, update the function signature (lines 33-36):

```rust
pub fn start_local_pipeline(
    source_lang: String,
    target_lang: String,
    transcript_only: bool,
    channel: Channel<String>,
    state: tauri::State<'_, LocalPipelineState>,
) -> Result<(), String> {
    log_to_file(&format!("start_local_pipeline called: src={}, tgt={}, transcript_only={}", source_lang, target_lang, transcript_only));
```

- [ ] **Step 2: Pass `--transcript-only` flag to Python child process**

In the same file, find the `Command::new` builder (lines 98-111). Add the flag conditionally before `.env("PATH", path_env)`:

```rust
    let mut child = Command::new(&python)
        .arg(&script_path)
        .arg("--asr-model")
        .arg("whisper")
        .arg("--source-lang")
        .arg(&source_lang)
        .arg("--target-lang")
        .arg(&target_lang)
        .env("PATH", path_env)
        .env("HOME", &home)
        .env("TOKENIZERS_PARALLELISM", "false")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if transcript_only {
        child = child.arg("--transcript-only");
    }

    let mut child = child
        .spawn()
        .map_err(|e| {
            let msg = format!("Failed to start pipeline: {}", e);
```

Note: The `child` variable needs restructuring. The current code chains `.spawn()` directly. Split into builder + conditional arg + spawn. Read the actual code around line 98-114 carefully and adjust.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/commands/local_pipeline.rs
git commit -m "feat(rust): pass transcript_only flag to local pipeline"
```

---

### Task 7: Pass `transcript_only` from JS to Rust invoke

**Files:**
- Modify: `src/js/app.js:1184-1188` (`_startLocalMode` invoke call)

- [ ] **Step 1: Add `transcriptOnly` to the invoke call**

In `src/js/app.js`, find the `invoke('start_local_pipeline', ...)` call (lines 1184-1188). Replace with:

```javascript
            const translationType = settings.translation_type || 'one_way';
            await invoke('start_local_pipeline', {
                sourceLang: sourceLang,
                targetLang: settings.target_language || 'vi',
                transcriptOnly: translationType === 'transcript_only',
                channel: this.localPipelineChannel,
            });
```

- [ ] **Step 2: Commit**

```bash
git add src/js/app.js
git commit -m "feat(app): pass transcriptOnly flag to local pipeline invoke"
```

---

### Task 8: End-to-end verification

- [ ] **Step 1: Test Soniox transcript-only mode**

1. Open app, go to Settings
2. Set Translation Type to "Transcript only"
3. Confirm source/target language sections are hidden
4. Start recording
5. Speak something
6. Verify: only original text appears, no translation text

- [ ] **Step 2: Test Soniox one-way mode still works**

1. Switch Translation Type back to "One-way"
2. Confirm source + target language sections reappear
3. Start recording, verify translations appear normally

- [ ] **Step 3: Test TTS in transcript-only**

1. Switch to "Transcript only"
2. Enable TTS
3. Speak something
4. Verify TTS reads the original transcribed text aloud

- [ ] **Step 4: Test local mode transcript-only (if Apple Silicon)**

1. Set Translation Mode to "Local"
2. Set Translation Type to "Transcript only"
3. Start recording
4. Verify: original text appears without translation, LLM model not loaded

- [ ] **Step 5: Final commit**

```bash
git add -A
git commit -m "test: verify transcript-only mode end-to-end"
```
