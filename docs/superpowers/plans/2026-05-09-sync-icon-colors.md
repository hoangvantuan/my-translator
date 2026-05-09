# Đồng bộ màu icon giữa các control area

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Đồng bộ màu icon mặc định, hover, và active state giữa 4 vùng control: floating toolbar, floating controls (bottom-right), settings header, sessions header.

**Architecture:** Chỉ sửa CSS. Thống nhất tất cả icon buttons về cùng bộ color tokens: default `--text-secondary`, hover `--text-primary` + `--bg-hover`, active `--accent` + `--accent-light`.

**Tech Stack:** CSS custom properties

---

## Phân tích hiện trạng

4 vùng chứa icon buttons, mỗi vùng dùng color tokens khác nhau:

| # | Selector | Vùng | Default | Hover bg | Active bg |
|---|----------|------|---------|----------|-----------|
| 1 | `.toolbar-btn` | Floating toolbar (trên) | `--text-disabled` #b0b5c0 | `--bg-hover` | `--accent-light` |
| 2 | `.float-btn` | Float controls (dưới-phải) | `--text-secondary` #6b7280 | `--bg-hover` | `--accent-light` |
| 3 | `.opacity-trigger` | Float controls (dưới-phải) | `--text-secondary` #6b7280 | **KHÔNG** | — |
| 4 | `.icon-btn` | Settings/Sessions header | `--text-secondary` #6b7280 | `--bg-hover` | — |
| 5 | `#btn-pin.active` | Floating toolbar | — | — | **KHÔNG** (thiếu bg) |

### 3 bất đồng bộ cần sửa:

1. **Default color**: `.toolbar-btn` dùng `--text-disabled` (nhạt hơn), 3 vùng còn lại dùng `--text-secondary` (đậm hơn)
2. **`#btn-pin.active`**: Thiếu `background: var(--accent-light)` so với `.toolbar-btn.active` và `#btn-tts.active`
3. **`.opacity-trigger:hover`**: Thiếu `background: var(--bg-hover)` so với `.float-btn:hover` cùng vùng

---

### Task 1: Đồng bộ default color của `.toolbar-btn`

**Files:**
- Modify: `src/styles/main.css:352`

- [ ] **Step 1: Đổi default color từ `--text-disabled` sang `--text-secondary`**

```css
/* Dòng 352: đổi từ */
color: var(--text-disabled);
/* thành */
color: var(--text-secondary);
```

File `src/styles/main.css`, trong selector `.toolbar-btn` (dòng 343-357):

```css
.toolbar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: var(--toolbar-btn);
  height: var(--toolbar-btn);
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);  /* ← đổi từ --text-disabled */
  cursor: pointer;
  transition: all var(--transition-fast);
  -webkit-app-region: no-drag;
  padding: 0;
}
```

- [ ] **Step 2: Verify trên trình duyệt**

Mở app, hover vào toolbar. Tất cả icon phải cùng mức đậm với float controls bên dưới (#6b7280).

- [ ] **Step 3: Commit**

```bash
git add src/styles/main.css
git commit -m "fix(css): unify toolbar-btn default color to --text-secondary"
```

---

### Task 2: Thêm active background cho `#btn-pin`

**Files:**
- Modify: `src/styles/main.css:387-390`

- [ ] **Step 1: Thêm `background: var(--accent-light)` cho `#btn-pin.active`**

```css
/* Dòng 387-390: đổi từ */
#btn-pin.active {
  color: var(--accent);
}
/* thành */
#btn-pin.active {
  color: var(--accent);
  background: var(--accent-light);
}
```

Giờ `#btn-pin.active` khớp với `.toolbar-btn.active` và `#btn-tts.active` (cả hai đều có cặp `accent` + `accent-light`).

- [ ] **Step 2: Verify trên trình duyệt**

Pin button khi active phải có nền tím nhạt (`#eef2ff`) giống TTS button khi active.

- [ ] **Step 3: Commit**

```bash
git add src/styles/main.css
git commit -m "fix(css): add accent-light background to #btn-pin.active"
```

---

### Task 3: Thêm hover background cho `.opacity-trigger`

**Files:**
- Modify: `src/styles/main.css:808-811`

- [ ] **Step 1: Thêm `background: var(--bg-hover)` cho `.opacity-trigger:hover`**

```css
/* Dòng 808-811: đổi từ */
.opacity-trigger:hover {
  transform: scale(1.15);
  color: var(--text-primary);
}
/* thành */
.opacity-trigger:hover {
  transform: scale(1.15);
  color: var(--text-primary);
  background: var(--bg-hover);
}
```

Giờ opacity icon hover giống `.float-btn:hover` cùng vùng.

- [ ] **Step 2: Verify trên trình duyệt**

Hover vào opacity icon (hình tròn SVG), phải có nền xám nhạt giống các nút A−/A+ bên cạnh.

- [ ] **Step 3: Commit**

```bash
git add src/styles/main.css
git commit -m "fix(css): add bg-hover to opacity-trigger hover state"
```

---

## Bảng tổng hợp sau khi sửa

| Selector | Default | Hover color | Hover bg | Active color | Active bg |
|----------|---------|------------|----------|-------------|-----------|
| `.toolbar-btn` | `--text-secondary` | `--text-primary` | `--bg-hover` | `--accent` | `--accent-light` |
| `#btn-pin.active` | — | — | — | `--accent` | `--accent-light` |
| `.float-btn` | `--text-secondary` | `--text-primary` | `--bg-hover` | `--accent` | `--accent-light` |
| `.opacity-trigger` | `--text-secondary` | `--text-primary` | `--bg-hover` | — | — |
| `.icon-btn` | `--text-secondary` | `--text-primary` | `--bg-hover` | — | — |

Tất cả đồng bộ.
