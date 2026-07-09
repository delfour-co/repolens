# RepoLens Landing Site Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a bilingual (EN/FR) Zola landing page for RepoLens, dark "Lens" themed, deployed to GitHub Pages.

**Architecture:** A single-page Zola site under `site/`, ported structurally from the sibling `josephine` site and re-themed to RepoLens's own black + cyan identity, anchored by the existing knight/shield-lens mascot artwork. A `pages.yml` GitHub Actions workflow builds the site with Zola and publishes it to GitHub Pages. No Rust crate is touched; the site is self-contained.

**Tech Stack:** Zola 0.21.0 (static site generator, single Rust binary), SCSS (compiled by Zola), ImageMagick `magick` (image optimization), GitHub Actions + GitHub Pages.

## Global Constraints

- Spec: `docs/superpowers/specs/2026-07-09-repolens-site-design.md` (authoritative).
- Branch: `feat/site` (already based on `origin/main`, spec committed at `cdfd76c`).
- Palette (exact): `--bg #000000` · `--surface #0c1526` · `--surface2 #0a1220` · `--line #1b2740` · `--text #e2e8f0` · `--mut #94a3b8` · `--cy #38bdf8` · `--cy2 #7dd3fc` · `--ok #3fb950` · `--warn #d29922` · `--crit #f85149`.
- Background is **pure black** `#000`; **no glow/halo behind the mascot** (only a faint cyan radial at the very top of the hero).
- Language switch sits **top-right** of the hero.
- **English-first**; FR is a faithful translation. **No copy may reference a removed category or non-v3 feature** (no "secret detection", "license compliance", "dependency scanning"; provider is "GitHub & GitLab"; repo is `github.com/systm-d/repolens`).
- **Do NOT commit multi-MB PNGs.** Only optimized web assets (WebP/JPG, each **< 150 KB**) go in `site/static/`. Source PNGs come from `git show 'stash@{1}^3:docs/illustrations/<file>'`.
- `brand_color = "#38bdf8"`, `base_url = "https://systm-d.github.io/repolens"`, `repo_url = "https://github.com/systm-d/repolens"`.
- Approved visual reference: `.superpowers/brainstorm/4027486-1783580342/content/landing-b-v4.html` (git-ignored).
- Local binaries available: `zola` (0.21.0, at `~/.cargo/bin/zola`), `magick` (ImageMagick 7).
- "Tests" for a static site = `zola build` succeeds with no warnings + `zola check` passes (no broken links) + a visual check against the mockup. There are no unit tests.

---

### Task 1: Optimized illustration assets

**Files:**
- Create: `site/static/img/mascot.webp`, `site/static/img/portrait.webp`, `site/static/img/og.jpg`, `site/static/favicon.png`

**Interfaces:**
- Produces: four optimized assets referenced by later tasks — `/img/mascot.webp` (hero), `/img/portrait.webp` (callout accent), `/img/og.jpg` (`og:image`), `/favicon.png`.

- [ ] **Step 1: Create the asset directory and extract + optimize the source PNGs**

The source PNGs are untracked in `stash@{1}` and read via its untracked parent `^3`. Extract to `/tmp`, then optimize with ImageMagick. Run from the repo root:

```bash
mkdir -p site/static/img

git show 'stash@{1}^3:docs/illustrations/repolens_illustrations-002.png' > /tmp/rl-mascot.png
git show 'stash@{1}^3:docs/illustrations/repolens_illustrations-001.png' > /tmp/rl-portrait.png

# Hero mascot: 600px wide WebP
magick /tmp/rl-mascot.png -resize 600x600 -quality 82 -define webp:method=6 site/static/img/mascot.webp
# Callout portrait accent: 520px tall WebP
magick /tmp/rl-portrait.png -resize x520 -quality 80 -define webp:method=6 site/static/img/portrait.webp
# Favicon: 64px PNG from the mascot
magick /tmp/rl-mascot.png -resize 64x64 site/static/favicon.png
# OG card: mascot centered on a 1200x630 black canvas, no baked-in text
magick /tmp/rl-mascot.png -resize 560x560 -background black -gravity center -extent 1200x630 -quality 85 site/static/img/og.jpg
```

- [ ] **Step 2: Verify every asset exists and is under 150 KB**

Run:

```bash
ls -l site/static/img/mascot.webp site/static/img/portrait.webp site/static/img/og.jpg site/static/favicon.png
find site/static -type f \( -name '*.webp' -o -name '*.jpg' -o -name '*.png' \) -size +150k -print
```

Expected: all four files listed by `ls`; the `find` prints **nothing** (no file over 150 KB). If any WebP exceeds 150 KB, re-run its `magick` line with `-quality 72`.

- [ ] **Step 3: Commit**

```bash
git add site/static/img/mascot.webp site/static/img/portrait.webp site/static/img/og.jpg site/static/favicon.png
git commit -m "feat(site): add optimized RepoLens mascot assets"
```

---

### Task 2: Zola config + base template + dark theme (buildable skeleton)

**Files:**
- Create: `site/config.toml`, `site/templates/base.html`, `site/sass/main.scss`, `site/content/_index.md` (minimal placeholder), `site/templates/index.html` (minimal)

**Interfaces:**
- Consumes: assets from Task 1 (`/favicon.png`, `/img/og.jpg`).
- Produces: a working Zola site that builds; `main.scss` compiles to `main.css`; `base.html` provides `{% block content %}` + head/footer; `config.toml` defines EN default + FR language and `[extra]`.

- [ ] **Step 1: Write `site/config.toml`**

```toml
base_url = "https://systm-d.github.io/repolens"
title = "RepoLens"
description = "A CLI auto-configurator for GitHub and GitLab repositories."
default_language = "en"
compile_sass = true
build_search_index = false

[markdown]
highlight_code = true

[languages.fr]
title = "RepoLens"
description = "Un auto-configurateur CLI pour les dépôts GitHub et GitLab."

[extra]
brand_color = "#38bdf8"
repo_url = "https://github.com/systm-d/repolens"
```

- [ ] **Step 2: Write `site/templates/base.html`**

```html
<!DOCTYPE html>
<html lang="{{ lang }}">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>{% block title %}{{ config.title }}{% endblock title %}</title>
    <meta name="description" content="{{ config.description }}" />
    <meta property="og:title" content="{{ config.title }}" />
    <meta property="og:description" content="{{ config.description }}" />
    <meta property="og:type" content="website" />
    <meta property="og:image" content="{{ get_url(path='img/og.jpg') }}" />
    <meta name="twitter:card" content="summary_large_image" />
    <link rel="icon" href="{{ get_url(path='favicon.png') }}" />
    <link rel="stylesheet" href="{{ get_url(path='main.css') }}" />
  </head>
  <body>
    {% block content %}{% endblock content %}
    <footer>
      🛡️ RepoLens — auto-configurator for GitHub &amp; GitLab · MIT OR Apache-2.0 ·
      <a href="{{ config.extra.repo_url }}">GitHub</a>
    </footer>
  </body>
</html>
```

- [ ] **Step 3: Write `site/sass/main.scss` (the dark "Lens" theme)**

Global de-scoped port of the approved mockup (`landing-b-v4.html`). Create the file with exactly this content:

```scss
:root {
  --bg:#000000; --surface:#0c1526; --surface2:#0a1220; --line:#1b2740;
  --text:#e2e8f0; --mut:#94a3b8; --cy:#38bdf8; --cy2:#7dd3fc;
  --ok:#3fb950; --warn:#d29922; --crit:#f85149;
}
* { box-sizing:border-box; }
body {
  margin:0; background:var(--bg); color:var(--text); line-height:1.6;
  font-family:system-ui,-apple-system,"Segoe UI",Roboto,sans-serif;
}
a { color:var(--cy2); }
code, pre, .mono { font-family:ui-monospace,"SF Mono",Menlo,monospace; }

/* hero */
.hero {
  position:relative; text-align:center; padding:3.4rem 1.25rem 3.5rem; overflow:hidden;
  background:radial-gradient(58% 40% at 50% -6%, rgba(56,189,248,.10), transparent 62%);
}
.hero::after {
  content:""; position:absolute; inset:auto 0 0 0; height:1px;
  background:linear-gradient(90deg,transparent,rgba(56,189,248,.4),transparent);
}
.langs {
  position:absolute; top:1rem; right:1rem; z-index:3; display:inline-flex; gap:.25rem; padding:.2rem;
  border:1px solid rgba(56,189,248,.3); border-radius:999px; background:rgba(56,189,248,.06); backdrop-filter:blur(4px);
}
.langs a {
  font-size:.72rem; font-weight:700; letter-spacing:.05em; padding:.2rem .7rem; border-radius:999px;
  color:var(--mut); text-decoration:none;
}
.langs a.on { background:rgba(56,189,248,.18); color:var(--cy2); }
.mascot { display:block; width:min(300px,72vw); height:auto; margin:.2rem auto; }
.hero h1 {
  font-size:clamp(2.1rem,5vw,3.2rem); line-height:1.12; font-weight:820; margin:.4rem 0 0; letter-spacing:-.5px;
  background:linear-gradient(94deg,#f1f5f9 20%,var(--cy) 95%);
  -webkit-background-clip:text; background-clip:text; color:transparent;
}
.hero .tag { font-size:1.15rem; color:#cbd5e1; margin:.75rem 0 0; font-weight:500; }
.hero .lede { max-width:40rem; margin:.9rem auto 0; color:var(--mut); font-size:1rem; }
.cta { margin-top:1.7rem; display:flex; gap:.6rem; justify-content:center; flex-wrap:wrap; }
.btn { padding:.7rem 1.35rem; border-radius:9px; font-weight:700; font-size:.9rem; text-decoration:none; display:inline-block; }
.btn.b1 { background:linear-gradient(94deg,var(--cy),#0ea5e9); color:#04121e; box-shadow:0 8px 24px rgba(56,189,248,.3); }
.btn.b2 { border:1.5px solid rgba(56,189,248,.5); color:var(--cy2); }

/* layout */
.wrap { max-width:860px; margin:0 auto; padding:0 1.25rem; }
.sec { padding:3rem 0 .5rem; }
.lbl { font-size:.66rem; letter-spacing:.18em; text-transform:uppercase; color:var(--cy); }
.wrap h2 { font-size:1.5rem; margin:.3rem 0 1.3rem; font-weight:750; }

/* features */
.feats { display:grid; grid-template-columns:1fr; gap:.9rem; }
@media(min-width:640px){ .feats { grid-template-columns:1fr 1fr; } }
.feat { background:var(--surface); border:1px solid var(--line); border-radius:12px; padding:1.1rem 1.2rem; }
.feat .i { font-size:1.4rem; }
.feat h3 { margin:.5rem 0 .3rem; font-size:1.02rem; }
.feat p { margin:0; color:var(--mut); font-size:.88rem; }

/* terminal */
.term { background:#05090f; border:1px solid var(--line); border-radius:11px; overflow:hidden; box-shadow:0 18px 44px rgba(0,0,0,.6); }
.term .bar { display:flex; gap:6px; padding:9px 12px; background:#0b1424; border-bottom:1px solid var(--line); }
.term .bar i { width:11px; height:11px; border-radius:50%; }
.term .b { padding:14px 16px; font-size:12.5px; line-height:1.85; color:#cbd5e1; overflow-x:auto; }
.term .p { color:var(--cy); } .term .mut { color:#64748b; } .term .cy { color:var(--cy2); }
.term .ok { color:var(--ok); } .term .wa { color:var(--warn); } .term .cr { color:var(--crit); } .term .g { color:var(--ok); }

/* categories */
.cats { display:grid; grid-template-columns:repeat(2,1fr); gap:.6rem; }
@media(min-width:640px){ .cats { grid-template-columns:repeat(3,1fr); } }
.cat { background:var(--surface2); border:1px solid var(--line); border-radius:10px; padding:.8rem .9rem; }
.cat b { font-size:.92rem; }
.cat p { margin:.15rem 0 0; font-size:.76rem; color:var(--mut); }

/* commands */
.cmds { list-style:none; margin:0; padding:0; }
.cmds li { display:flex; justify-content:space-between; gap:1rem; padding:.55rem 0; border-bottom:1px solid var(--line); font-size:.88rem; }
.cmds code { color:var(--cy2); }
.cmds span { color:var(--mut); text-align:right; }

/* install + callout */
pre { background:#05090f; border:1px solid var(--line); border-radius:10px; padding:.9rem 1rem; overflow-x:auto; font-size:12.5px; color:#cbd5e1; }
.callout { margin:2rem 0 0; display:flex; gap:1rem; align-items:center; background:rgba(56,189,248,.07); border:1px solid rgba(56,189,248,.25); border-radius:12px; padding:1rem 1.2rem; color:#cbd5e1; font-size:.92rem; }
.callout img { width:74px; height:auto; flex:none; }
footer { margin-top:3rem; padding:2rem 1.25rem; text-align:center; color:var(--mut); font-size:.82rem; border-top:1px solid var(--line); }
```

- [ ] **Step 4: Write a minimal `site/templates/index.html`**

```html
{% extends "base.html" %}
{% block content %}
<main class="wrap">{{ section.content | safe }}</main>
{% endblock content %}
```

- [ ] **Step 5: Write a minimal `site/content/_index.md`**

```markdown
+++
title = "RepoLens"
+++

Placeholder — replaced in Task 3/4.
```

- [ ] **Step 6: Build and verify the skeleton compiles**

Run:

```bash
cd site && zola build && zola check; cd ..
```

Expected: `zola build` prints `Done in …` with **no warnings**; `zola check` reports success (no broken links). `site/public/index.html` and `site/public/main.css` exist:

```bash
ls site/public/index.html site/public/main.css
```

- [ ] **Step 7: Commit**

```bash
git add site/config.toml site/templates/base.html site/templates/index.html site/sass/main.scss site/content/_index.md
git commit -m "feat(site): Zola skeleton + dark Lens theme"
```

---

### Task 3: Hero (mascot, headline, CTAs, language switch)

**Files:**
- Modify: `site/templates/index.html`, `site/content/_index.md`

**Interfaces:**
- Consumes: `main.scss` classes (`.hero`, `.langs`, `.mascot`, `.cta`, `.btn`) from Task 2; `/img/mascot.webp` from Task 1.
- Produces: the hero block; `section.extra.tagline/lede/cta/cta2` front-matter fields that `index.html` reads.

- [ ] **Step 1: Rewrite `site/templates/index.html` with the hero**

```html
{% extends "base.html" %}
{% block title %}RepoLens — {{ section.extra.tagline }}{% endblock title %}
{% block content %}
<header class="hero">
  <nav class="langs">
    <a class="{% if lang == 'en' %}on{% endif %}" href="{{ get_url(path='@/_index.md', lang='en') }}">EN</a>
    <a class="{% if lang == 'fr' %}on{% endif %}" href="{{ get_url(path='@/_index.md', lang='fr') }}">FR</a>
  </nav>
  <img class="mascot" src="{{ get_url(path='img/mascot.webp') }}" alt="RepoLens guardian" />
  <h1>{{ section.title }}</h1>
  <p class="tag">{{ section.extra.tagline }}</p>
  <p class="lede">{{ section.extra.lede }}</p>
  <div class="cta">
    <a class="btn b1" href="{{ config.extra.repo_url }}">{{ section.extra.cta }}</a>
    <a class="btn b2" href="#install">{{ section.extra.cta2 }}</a>
  </div>
</header>
<main class="wrap">{{ section.content | safe }}</main>
{% endblock content %}
```

- [ ] **Step 2: Set the EN hero front-matter in `site/content/_index.md`**

Replace the placeholder front-matter (keep the body as a temporary single line for now):

```markdown
+++
title = "Configure every repo to a consistent standard."

[extra]
tagline = "A CLI auto-configurator for GitHub & GitLab repositories."
lede = "RepoLens audits repository hygiene and configuration, then turns every finding into a reviewable, applicable action — so you review the plan before anything changes."
cta = "View on GitHub"
cta2 = "Install"
+++

Body sections added in Task 4.
```

- [ ] **Step 3: Build and verify the hero renders**

Run:

```bash
cd site && zola build; cd ..
grep -c 'class="mascot"' site/public/index.html
grep -c 'Configure every repo' site/public/index.html
```

Expected: `zola build` clean; both `grep -c` print `1`. Optionally open `site/public/index.html` in a browser and confirm: black background, mascot centered (no halo), gradient headline, EN/FR switch top-right, two CTA buttons.

- [ ] **Step 4: Commit**

```bash
git add site/templates/index.html site/content/_index.md
git commit -m "feat(site): hero with mascot, headline, CTAs, language switch"
```

---

### Task 4: Body sections (EN content)

**Files:**
- Modify: `site/content/_index.md`

**Interfaces:**
- Consumes: `main.scss` classes (`.sec`, `.lbl`, `.feats`, `.feat`, `.term`, `.cats`, `.cat`, `.cmds`, `.callout`) from Task 2; `/img/portrait.webp` from Task 1.
- Produces: the full EN landing body (6 content sections).

- [ ] **Step 1: Replace the `_index.md` body (everything after the `+++` front-matter) with the sections**

Keep the front-matter from Task 3; replace the body line with:

```html
<div class="sec">
  <div class="lbl mono">why repolens</div>
  <div class="feats">
    <div class="feat"><div class="i">📋</div><h3>Plan / apply split</h3><p>Every fix is computed as a typed action plan you review, then apply selectively. Never a surprise mutation.</p></div>
    <div class="feat"><div class="i">🗂️</div><h3>6 rule categories</h3><p>files · docs · security · git · codeowners · metadata — each mapped to an applicable fix.</p></div>
    <div class="feat"><div class="i">🐙</div><h3>GitHub &amp; GitLab</h3><p>One tool, two providers behind a single interface. Auto-detected from your <code>origin</code> remote.</p></div>
    <div class="feat"><div class="i">📤</div><h3>5 output formats</h3><p>Terminal, JSON, Markdown, SARIF &amp; HTML — drop it straight into CI.</p></div>
  </div>
</div>

<div class="sec">
  <div class="lbl mono">see it in action</div>
  <h2>Review the plan. Then apply.</h2>
  <div class="term mono">
    <div class="bar"><i style="background:#ff5f56"></i><i style="background:#ffbd2e"></i><i style="background:#27c93f"></i></div>
    <div class="b">
      <div><span class="p">$</span> repolens plan <span class="mut">--preset opensource</span></div>
      <div class="mut">auditing 6 categories · provider: github (auto)…</div>
      <div>&nbsp;</div>
      <div>&nbsp;&nbsp;<span class="cy">docs</span>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;README, LICENSE, SECURITY&nbsp;&nbsp;&nbsp;&nbsp;<span class="ok">ok</span></div>
      <div>&nbsp;&nbsp;<span class="cy">security</span>&nbsp;&nbsp;branch protection missing&nbsp;&nbsp;&nbsp;&nbsp;<span class="wa">warn</span></div>
      <div>&nbsp;&nbsp;<span class="cy">metadata</span>&nbsp;&nbsp;no description / topics&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;<span class="cr">fix</span></div>
      <div>&nbsp;</div>
      <div><span class="p">▸</span> 4 actions planned <span class="mut">— run</span> <span class="cy">repolens apply</span> <span class="mut">to execute</span></div>
      <div>&nbsp;</div>
      <div><span class="p">$</span> repolens apply</div>
      <div><span class="g">✓</span> updated repository description &amp; topics</div>
      <div><span class="g">✓</span> enabled branch protection on <span class="cy">main</span></div>
    </div>
  </div>
</div>

<div class="sec">
  <div class="lbl mono">what it checks</div>
  <h2>Six categories, every one auto-fixable.</h2>
  <div class="cats">
    <div class="cat"><b>📁 files</b><p>.gitignore presence &amp; entries</p></div>
    <div class="cat"><b>📚 docs</b><p>README, LICENSE, CONTRIBUTING…</p></div>
    <div class="cat"><b>🛡️ security</b><p>branch protection, settings.yml</p></div>
    <div class="cat"><b>🔧 git</b><p>.gitattributes, sensitive files</p></div>
    <div class="cat"><b>👥 codeowners</b><p>CODEOWNERS presence &amp; syntax</p></div>
    <div class="cat"><b>📊 metadata</b><p>description, topics, homepage</p></div>
  </div>
</div>

<div class="sec">
  <div class="lbl mono">commands</div>
  <ul class="cmds">
    <li><code>repolens plan</code><span>compute a reviewable action plan</span></li>
    <li><code>repolens apply</code><span>execute the plan (or --dry-run)</span></li>
    <li><code>repolens report</code><span>audit report — json / sarif / html…</span></li>
    <li><code>repolens compare</code><span>diff two audit reports</span></li>
    <li><code>repolens install-hooks</code><span>git hooks with a secret scan</span></li>
  </ul>
</div>

<div class="sec">
  <div class="lbl mono">install</div>
  <h2 id="install">Get RepoLens</h2>
  <pre># Docker (recommended)
docker run --rm -v "$(pwd)":/repo ghcr.io/systm-d/repolens plan

# Cargo
cargo install repolens</pre>
  <p style="color:var(--mut);font-size:.85rem;margin-top:.7rem">Also on Homebrew, AUR, Debian .deb and Scoop — or grab a binary from the latest GitHub release.</p>
  <div class="callout"><img src="img/portrait.webp" alt="" /><div>🛡️ <strong>RepoLens configures a repository as well as it can, automatically</strong> — the plan/apply split means you always review before anything changes. <em>Not a secret or dependency scanner; it focuses on structure, docs &amp; hosting-platform configuration.</em></div></div>
</div>
```

Note: the callout `<img src="img/portrait.webp">` is a relative path; Zola rewrites/serves it from `static/`. If `zola check` flags it, change to `{{ get_url(path='img/portrait.webp') }}` (but Markdown body is not a Tera template, so keep the relative `img/portrait.webp` — Zola copies `static/` to the site root, so `img/portrait.webp` resolves).

- [ ] **Step 2: Build and verify all sections render**

Run:

```bash
cd site && zola build && zola check; cd ..
for s in "why repolens" "see it in action" "what it checks" "commands" "Get RepoLens"; do
  grep -qi "$s" site/public/index.html && echo "OK: $s" || echo "MISSING: $s"
done
grep -c 'portrait.webp' site/public/index.html
```

Expected: `zola build`/`check` clean; all five sections print `OK`; `portrait.webp` count is `1`. Confirm no removed-category words leaked:

```bash
grep -Ei 'secret detection|license compliance|dependency scan|delfour-co' site/public/index.html || echo "clean — no stale copy"
```

Expected: prints `clean — no stale copy`.

- [ ] **Step 3: Commit**

```bash
git add site/content/_index.md
git commit -m "feat(site): EN landing body sections"
```

---

### Task 5: French translation

**Files:**
- Create: `site/content/_index.fr.md`

**Interfaces:**
- Consumes: the `index.html` template + `config.toml [languages.fr]` from Tasks 2–3.
- Produces: the FR page at `/fr/`; the EN⇄FR switch resolves both ways.

- [ ] **Step 1: Create `site/content/_index.fr.md` (front-matter + translated body)**

```markdown
+++
title = "Alignez chaque dépôt sur un standard constant."

[extra]
tagline = "Un auto-configurateur CLI pour les dépôts GitHub & GitLab."
lede = "RepoLens audite l'hygiène et la configuration d'un dépôt, puis transforme chaque écart en action applicable et vérifiable — vous validez le plan avant tout changement."
cta = "Voir sur GitHub"
cta2 = "Installer"
+++

<div class="sec">
  <div class="lbl mono">pourquoi repolens</div>
  <div class="feats">
    <div class="feat"><div class="i">📋</div><h3>Séparation plan / apply</h3><p>Chaque correctif est calculé comme un plan d'actions typé que vous relisez, puis appliquez sélectivement. Jamais de modification surprise.</p></div>
    <div class="feat"><div class="i">🗂️</div><h3>6 catégories de règles</h3><p>files · docs · security · git · codeowners · metadata — chacune associée à un correctif applicable.</p></div>
    <div class="feat"><div class="i">🐙</div><h3>GitHub &amp; GitLab</h3><p>Un seul outil, deux providers derrière une interface unique. Détecté automatiquement depuis votre remote <code>origin</code>.</p></div>
    <div class="feat"><div class="i">📤</div><h3>5 formats de sortie</h3><p>Terminal, JSON, Markdown, SARIF &amp; HTML — intégrable directement en CI.</p></div>
  </div>
</div>

<div class="sec">
  <div class="lbl mono">en action</div>
  <h2>Relisez le plan. Puis appliquez.</h2>
  <div class="term mono">
    <div class="bar"><i style="background:#ff5f56"></i><i style="background:#ffbd2e"></i><i style="background:#27c93f"></i></div>
    <div class="b">
      <div><span class="p">$</span> repolens plan <span class="mut">--preset opensource</span></div>
      <div class="mut">audit de 6 catégories · provider : github (auto)…</div>
      <div>&nbsp;</div>
      <div>&nbsp;&nbsp;<span class="cy">docs</span>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;README, LICENSE, SECURITY&nbsp;&nbsp;&nbsp;&nbsp;<span class="ok">ok</span></div>
      <div>&nbsp;&nbsp;<span class="cy">security</span>&nbsp;&nbsp;protection de branche absente&nbsp;&nbsp;<span class="wa">warn</span></div>
      <div>&nbsp;&nbsp;<span class="cy">metadata</span>&nbsp;&nbsp;description / topics manquants&nbsp;&nbsp;<span class="cr">fix</span></div>
      <div>&nbsp;</div>
      <div><span class="p">▸</span> 4 actions planifiées <span class="mut">— lancez</span> <span class="cy">repolens apply</span> <span class="mut">pour exécuter</span></div>
      <div>&nbsp;</div>
      <div><span class="p">$</span> repolens apply</div>
      <div><span class="g">✓</span> description &amp; topics du dépôt mis à jour</div>
      <div><span class="g">✓</span> protection de branche activée sur <span class="cy">main</span></div>
    </div>
  </div>
</div>

<div class="sec">
  <div class="lbl mono">ce qu'il vérifie</div>
  <h2>Six catégories, toutes auto-corrigeables.</h2>
  <div class="cats">
    <div class="cat"><b>📁 files</b><p>présence &amp; entrées .gitignore</p></div>
    <div class="cat"><b>📚 docs</b><p>README, LICENSE, CONTRIBUTING…</p></div>
    <div class="cat"><b>🛡️ security</b><p>protection de branche, settings.yml</p></div>
    <div class="cat"><b>🔧 git</b><p>.gitattributes, fichiers sensibles</p></div>
    <div class="cat"><b>👥 codeowners</b><p>présence &amp; syntaxe CODEOWNERS</p></div>
    <div class="cat"><b>📊 metadata</b><p>description, topics, homepage</p></div>
  </div>
</div>

<div class="sec">
  <div class="lbl mono">commandes</div>
  <ul class="cmds">
    <li><code>repolens plan</code><span>calcule un plan d'actions relisable</span></li>
    <li><code>repolens apply</code><span>exécute le plan (ou --dry-run)</span></li>
    <li><code>repolens report</code><span>rapport d'audit — json / sarif / html…</span></li>
    <li><code>repolens compare</code><span>compare deux rapports d'audit</span></li>
    <li><code>repolens install-hooks</code><span>hooks git avec scan de secrets</span></li>
  </ul>
</div>

<div class="sec">
  <div class="lbl mono">installation</div>
  <h2 id="install">Installer RepoLens</h2>
  <pre># Docker (recommandé)
docker run --rm -v "$(pwd)":/repo ghcr.io/systm-d/repolens plan

# Cargo
cargo install repolens</pre>
  <p style="color:var(--mut);font-size:.85rem;margin-top:.7rem">Également sur Homebrew, AUR, Debian .deb et Scoop — ou téléchargez un binaire depuis la dernière release GitHub.</p>
  <div class="callout"><img src="img/portrait.webp" alt="" /><div>🛡️ <strong>RepoLens configure un dépôt au mieux, automatiquement</strong> — la séparation plan/apply garantit que vous validez toujours avant tout changement. <em>Ce n'est pas un scanner de secrets ni de dépendances ; il se concentre sur la structure, la documentation &amp; la configuration de la plateforme d'hébergement.</em></div></div>
</div>
```

- [ ] **Step 2: Build and verify both languages + the switch**

Run:

```bash
cd site && zola build && zola check; cd ..
ls site/public/index.html site/public/fr/index.html
grep -c 'Alignez chaque dépôt' site/public/fr/index.html
grep -c "@/_index.md" site/public/index.html  # switch links present (2 langs)
```

Expected: `zola build`/`check` clean; both `index.html` files exist; the FR headline count is `1`; the EN page contains the language-switch links. Confirm the FR page has no stale copy:

```bash
grep -Ei 'secret detection|license compliance|delfour-co' site/public/fr/index.html || echo "clean"
```

Expected: prints `clean`.

- [ ] **Step 3: Commit**

```bash
git add site/content/_index.fr.md
git commit -m "feat(site): French translation of the landing page"
```

---

### Task 6: GitHub Pages deploy workflow

**Files:**
- Create: `.github/workflows/pages.yml`

**Interfaces:**
- Consumes: the `site/` directory built by Zola.
- Produces: a workflow that builds `site/` and deploys `site/public` to GitHub Pages on push to `main`.

- [ ] **Step 1: Create `.github/workflows/pages.yml`**

```yaml
name: Pages

on:
  push:
    branches: [main]
    paths:
      - "site/**"
      - ".github/workflows/pages.yml"
  workflow_dispatch:

permissions:
  contents: read

concurrency:
  group: pages
  cancel-in-progress: true

jobs:
  build:
    runs-on: ubuntu-24.04
    permissions:
      contents: read
      pages: write
      id-token: write
    steps:
      - uses: actions/checkout@v4
      - name: Configure Pages (enable if needed)
        uses: actions/configure-pages@v5
        with:
          enablement: true
      - uses: taiki-e/install-action@v2
        with:
          tool: zola@0.21.0
      - name: Build site
        run: cd site && zola build
      - uses: actions/upload-pages-artifact@v3
        with:
          path: site/public

  deploy:
    needs: build
    runs-on: ubuntu-24.04
    permissions:
      pages: write
      id-token: write
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - id: deployment
        uses: actions/deploy-pages@v4
```

- [ ] **Step 2: Validate the workflow YAML**

Run (uses Python's YAML parser, always available):

```bash
python3 -c "import yaml,sys; yaml.safe_load(open('.github/workflows/pages.yml')); print('yaml ok')"
```

Expected: prints `yaml ok`. (If `actionlint` is installed, also run `actionlint .github/workflows/pages.yml`.)

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/pages.yml
git commit -m "ci(site): build and deploy the site to GitHub Pages"
```

> **Deployment note (do NOT attempt to merge via the `gh` API):** this file lives under `.github/workflows/`. The current `gh` OAuth token lacks the `workflow` scope, so `gh pr merge` on a PR that adds it is blocked (same as the recent dependabot workflow PRs). The branch is pushed over SSH (unaffected); the PR must be **merged via the GitHub web UI**, or after `gh auth refresh -s workflow`. This is a human step, surfaced at hand-off — not a task step.

---

### Task 7: Final full-site verification

**Files:** none (verification only)

- [ ] **Step 1: Clean build from scratch + link check**

```bash
rm -rf site/public
cd site && zola build && zola check; cd ..
```

Expected: both succeed, zero warnings, zero broken links.

- [ ] **Step 2: Asset-weight + stale-copy gate across the whole built site**

```bash
find site/static -type f \( -name '*.webp' -o -name '*.jpg' -o -name '*.png' \) -size +150k -print
grep -REil 'secret detection|license compliance|dependency scan|delfour-co|audit github repositories' site/public && echo "STALE COPY FOUND" || echo "clean — no oversized asset, no stale copy"
```

Expected: the `find` prints nothing; the final line is `clean — no oversized asset, no stale copy`.

- [ ] **Step 3: Confirm no multi-MB binary is tracked**

```bash
git ls-files site | xargs -I{} du -h {} | sort -rh | head -5
```

Expected: the largest tracked file under `site/` is well under 1 MB.

- [ ] **Step 4: (Manual) Visual check** — open `site/public/index.html` and `site/public/fr/index.html` in a browser. Confirm against the mockup: pure-black background, mascot centered with no halo, gradient headline, EN/FR switch top-right that navigates between languages, all six sections styled, terminal demo colored (ok/warn/fix), callout with portrait.

---

## Self-Review

**Spec coverage:**
- §2 visual direction / palette → Task 2 (main.scss with exact tokens). ✓
- §3 illustrations (mascot hero, portrait callout, favicon, OG; stale-text avoidance; WebP <150 KB; stash source) → Task 1 (assets) + Tasks 3–4 (usage) + Task 7 (gates). ✓
- §4 Zola structure (config, content EN+FR, templates, sass, static) → Tasks 2–5. ✓
- §5 seven sections → hero (Task 3) + 6 body sections (Task 4/5). ✓
- §6 deployment (pages.yml, triggers, jobs, permissions, caveats) → Task 6 (+ workflow-scope note). ✓
- §7 acceptance (`zola build`/`check`, optimized images, no stale copy) → Task 7. ✓

**Placeholder scan:** every code step contains complete file content; no "TBD"/"handle later"; the two `_index.md` placeholders in Task 2 are explicitly replaced in Tasks 3–4. ✓

**Type/name consistency:** `section.extra.tagline/lede/cta/cta2` defined in Task 3 front-matter and consumed by the Task 3 `index.html`; the FR file (Task 5) uses the identical `[extra]` keys; CSS class names in `main.scss` (Task 2) match those used in the hero (Task 3) and body (Tasks 4–5). Asset paths `/img/mascot.webp`, `/img/portrait.webp`, `/img/og.jpg`, `/favicon.png` are produced in Task 1 and referenced consistently thereafter. ✓

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-07-09-repolens-site.md`.
