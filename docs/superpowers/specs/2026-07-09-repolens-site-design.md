# RepoLens landing site — design

- **Date:** 2026-07-09
- **Status:** approved (brainstorming)
- **Scope:** Add a Zola landing + docs site for RepoLens, deployed to GitHub Pages, ported from the
  sibling `josephine` site and re-themed to RepoLens's own dark "Lens" identity.

## 1. Objective

RepoLens has no web presence today (the `_templates/cli` design ships a Zola landing axis and
`josephine` implements it, but RepoLens's workspace migration explicitly deferred it —
`2026-07-02-workspace-skeleton-migration-design.md:185`). This project adds that site: a single-page
bilingual (EN/FR) landing at `https://systm-d.github.io/repolens`, built with **Zola** and deployed by
a `pages.yml` GitHub Actions workflow, matching josephine's structure but with RepoLens's own dark
visual identity and v3 content (6-category GitHub/GitLab auto-configurator, plan/apply).

Non-goals: multi-page docs (README + docs/ + docs.rs already cover reference material); a JS framework
or Node toolchain (Zola is a single Rust binary); changing any Rust crate.

## 2. Visual direction — "Lens" (dark)

Chosen from three mocked directions (Terminal / Lens / Blueprint). **Lens** = deep-black background with
a cyan accent and gradient headlines, anchored by RepoLens's existing mascot artwork.

**Palette (CSS variables):**

| Token | Value | Use |
|-------|-------|-----|
| `--bg` | `#000000` | page background (pure black — the mascot art is on black, so it composites seamlessly) |
| `--surface` | `#0c1526` | cards |
| `--surface2` | `#0a1220` | category tiles |
| `--line` | `#1b2740` | borders |
| `--text` | `#e2e8f0` | body text |
| `--mut` | `#94a3b8` | secondary text |
| `--cy` / `--cy2` | `#38bdf8` / `#7dd3fc` | accent (brand color), links, headings gradient |
| `--ok` / `--warn` / `--crit` | `#3fb950` / `#d29922` / `#f85149` | severity states (reused from RepoLens's HTML report) |

- Font: system sans-serif for prose; `ui-monospace` for code, labels, and the terminal demo.
- Headlines use a light→cyan gradient (`background-clip:text`).
- **No glow/halo behind the mascot** (removed per review); only a faint cyan radial ambiance at the very
  top of the hero. A 1px cyan gradient rule closes the hero.
- `brand_color = #38bdf8` in `config.toml [extra]`.

## 3. Illustrations (RepoLens mascot)

RepoLens already has a mascot: a low-poly **blue knight holding a shield made of a glowing
constellation/network — "the lens"**, on black. Eight PNGs exist, currently **untracked in git stash
`stash@{1}` (the `claudettes-wip` stash), retrievable via `git show 'stash@{1}^3:docs/illustrations/<f>'`**.

Usage on the site:

| Asset | Source PNG | Role |
|-------|-----------|------|
| Hero mascot | `repolens_illustrations-002.png` (clean, on black) | hero image, ~300px |
| Callout accent | `repolens_illustrations-001.png` (portrait) | small accent in the closing callout |
| Logo / favicon | mascot-head badge (crop from a banner, or 002) | header logo + `favicon` |
| OG / social card | see caveat below | `og:image` / Twitter card |

**Caveats (load-bearing):**
- **Stale text:** the full banners (`repolens.png`, `repolens_illustrations-004.png`) have baked-in text
  that is *wrong for v3* — "audit **GitHub**", "**Secret Detection**", "**License Compliance**" (removed
  categories), and the old `github.com/delfour-co/repolens` URL. **Do not display these banners with
  their text on the site.** Use only the text-free artwork (002/001). For the OG card, use the optimized
  clean mascot on black (no baked-in text) as `og:image`; regenerating a correct text-bearing branded
  banner is a nice-to-have follow-up, out of scope here.
- **Weight:** the source PNGs are 1–2 MB at 1024–1536px — far too heavy for web, and committing multi-MB
  binaries would itself trip RepoLens's own `git`/large-file hygiene. **Only optimized web assets are
  committed:** resize + convert to **WebP** (ImageMagick `magick` is available; `magick in.png -resize 600x
  -quality 82 out.webp`), targeting <150 KB each, in `site/static/img/`. A small PNG favicon is derived
  the same way. The multi-MB originals are **not** committed (their provenance — the stash — is recorded
  here).

## 4. Site structure (Zola, ported from josephine)

```
site/
├── config.toml              # base_url, title, description, default_language=en, [languages.fr], [extra] brand_color/repo_url
├── content/
│   ├── _index.md            # EN landing content (front-matter: tagline/lede/cta + body sections)
│   └── _index.fr.md         # FR landing content (same structure, translated)
├── templates/
│   ├── base.html            # <html>, <head> (meta/OG/favicon/lang), footer, stylesheet link
│   └── index.html           # hero (mascot, gradient H1, tagline, lede, CTAs) + {{ section.content }}
├── sass/
│   └── main.scss            # the Lens dark theme (compile_sass = true)
└── static/
    ├── img/                 # optimized mascot.webp, portrait.webp, og.* 
    └── favicon.png
```

**config.toml** mirrors josephine: `base_url = "https://systm-d.github.io/repolens"`, `title = "RepoLens"`,
`description` (EN + a `[languages.fr]` override), `compile_sass = true`, `[extra] brand_color = "#38bdf8"`,
`repo_url = "https://github.com/systm-d/repolens"`.

**Language switch** sits **top-right** of the hero (absolute), linking EN ⇄ FR via Zola's
`get_url(path='@/_index.md', lang=…)`, exactly like josephine's `.langs` nav but repositioned.

## 5. Page sections (single page)

1. **Hero** — lang switch (top-right) · mascot (002) · gradient H1 "Configure every repo to a consistent
   standard." · tagline "A CLI auto-configurator for GitHub & GitLab repositories." · lede (from README) ·
   CTAs *View on GitHub* / *Install* (anchor to §install).
2. **Why RepoLens** — 4 feature cards: plan/apply split · 6 rule categories · GitHub & GitLab · 5 output
   formats.
3. **See it in action** — a styled terminal block showing `repolens plan` (6 categories with ok/warn/fix
   states) → `repolens apply` (✓ lines). This is the differentiator.
4. **What it checks** — a grid of the 6 categories (files, docs, security, git, codeowners, metadata) with
   one-line descriptions.
5. **Commands** — plan · apply · report · compare · install-hooks (concise list).
6. **Install** — Docker (`ghcr.io/systm-d/repolens`) + `cargo install repolens`, with a note that
   Homebrew/AUR/deb/Scoop and GitHub-release binaries also exist. Closes with the "configures a repository
   as well as it can, automatically / not a secret or dependency scanner" callout + portrait accent.
7. **Footer** — one line + GitHub link, MIT OR Apache-2.0.

Content is **English-first** (per CONVENTIONS/CLAUDE.md); the FR file is a faithful translation. No claim
references a removed category or a non-v3 feature.

## 6. Deployment

`.github/workflows/pages.yml`, ported from josephine:

- Triggers: `push` to `main` touching `site/**` or the workflow itself, + `workflow_dispatch`.
- Job **build**: checkout → `actions/configure-pages@v5` (`enablement: true`) → install `zola@0.19.2` via
  `taiki-e/install-action` → `cd site && zola build` → `actions/upload-pages-artifact` (`site/public`).
- Job **deploy**: `actions/deploy-pages` to the `github-pages` environment.
- Permissions: `pages: write`, `id-token: write`.

**Deployment prerequisites / caveats:**
- GitHub Pages must be enabled with source = "GitHub Actions" (the `configure-pages enablement:true` step
  handles this).
- Adding `pages.yml` touches `.github/workflows/`. The current `gh` OAuth token lacks the `workflow`
  scope, so **merging** a PR that adds this file via the `gh` API is blocked (as with the recent
  dependabot workflow PRs). Land it either by merging via the GitHub web UI or after
  `gh auth refresh -s workflow`. A plain `git push` over SSH is unaffected.
- Bump the pinned action/Zola versions to match what CI already uses where relevant; keep `zola@0.19.2`
  (josephine's pin) unless a newer stable is preferred.

## 7. Testing / acceptance

- `cd site && zola build` succeeds locally with no warnings; `zola check` passes (no broken internal
  links). Both `_index.md` and `_index.fr.md` render; the EN⇄FR switch resolves.
- Committed images are WebP/optimized and each < 150 KB; no multi-MB binary is committed.
- The rendered page shows no removed-category / stale-URL text.
- After merge, the `pages.yml` run is green and the site is live at the Pages URL.

## 8. Provenance

- Ported from `josephine` (`/home/kdelfour/Workspace/Professionel/systm-D/josephine/site` +
  `.github/workflows/pages.yml`).
- Illustrations: `stash@{1}^3:docs/illustrations/*` in this repo.
- Template design that introduced the site axis: `_templates/cli/docs/superpowers/specs/2026-06-27-cli-template-design.md` §8.
