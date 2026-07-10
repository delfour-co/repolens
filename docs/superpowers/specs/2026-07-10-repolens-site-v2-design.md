# RepoLens landing site v2 — enriched landing (design)

**Status:** approved (2026-07-10). Supersedes the v1 landing content shipped in PR #251.
**Scope:** Sub-project A of the "improve the site" request — enrich the *existing single-page
landing*. Sub-project B (a full multi-page documentation site generated from `docs/*.md`) is
deferred to a follow-up.

## Problem

The v1 landing (PR #251, live at systm-d.github.io/repolens) is correct but thin: a generic
centered hero, a single hand-mocked terminal block, and a two-line install snippet. The user asked
to improve it with **(1) real command captures, (2) a more compelling hero, (3) a fuller install
guide,** and (4) complete documentation (→ Sub-project B).

## Decisions (approved)

1. **Hero → "hero C".** Replace the generic centered hero with a problem→resolution hero:
   - transparent mascot on top (background removed);
   - eyebrow `◇ audit · plan · apply` (cyan, mono, uppercase);
   - H1 **"Stop shipping repos with gaps."** (gradient text, unchanged theme);
   - one subline naming the concrete gaps (LICENSE, branch protection, topics, stale metadata);
   - a two-card **✗ → ✓ flow**: left card "found by `repolens plan`" (red/amber markers), right card
     "done by `repolens apply`" (green checks), arrow between;
   - CTAs (View on GitHub / Install) + a `$ cargo install repolens` one-liner.
   - Lang switch stays top-right; pure-black bg; NO halo box behind the mascot.

2. **Real-capture panorama.** A new section of styled terminal blocks showing **actual** output
   captured from the built binary against a bare demo repo (git-init, `src/main.rs` only, no docs):
   `plan`, `apply --dry-run`, `report`, `compare`, `install-hooks`, `--help`. Content is the real
   tool output, lightly trimmed of the French progress lines (Chargement / Analyse / Exécution /
   → category / Génération) that are operational noise. Severity markers are theme-coloured
   (critical `#f85149`, warning `#d29922`, info/ok `#3fb950`, accent `#38bdf8`).

3. **Enriched install guide.** Replace the two-line snippet with the full distribution matrix from
   CLAUDE.md → *Distribution*: Docker (ghcr.io/systm-d/repolens), Cargo (`cargo install repolens`),
   Homebrew, AUR, Debian `.deb`, Scoop, prebuilt GitHub-release binaries, and from-source; plus a
   `repolens --version` verification step.

4. **Transparent mascot + refreshed OG card.** `site/static/img/mascot.webp` becomes the
   background-removed cutout (574×600, 59 KB). `og.jpg` regenerated on pure black with the mascot +
   "RepoLens" wordmark + tagline (1200×630, 64 KB).

5. **Bilingual EN/FR** parity preserved. Captured tool output is literal (English, as the binary
   emits it) and identical in both languages; only the surrounding prose translates.

## Non-goals / constraints (unchanged from v1)

- Pure black `#000`, cyan `#38bdf8`; dark "Lens" theme in `site/sass/main.scss`.
- No removed-category copy (no secret detection / dependency/license scanning / stale-issue / docker
  or CI linting); providers are GitHub & GitLab; repo is `systm-d/repolens`.
- Do NOT commit multi-MB PNGs — only optimized WebP/JPG < 150 KB in `site/static/`.
- `zola build` (no warnings) + `zola check` (no broken links) + visual check are the gates.
- Pushing / opening a PR is the human hand-off (pages.yml already lives on main; the workflow-scope
  caveat only applies if that file changes — it does not here).

## Out of scope (Sub-project B, follow-up)

A multi-page docs site generated from `docs/` (installation, usage, configuration, rules,
providers, output formats…). Tracked separately.

## Aside — product bug found while capturing

`crates/repolens/src/cli/output/terminal.rs:47,61,74` emit a **literal `\n`** (escaped `\\n`) in the
severity headers instead of a real newline. Cosmetic, TTY-independent, out of scope for the site;
recommended as a tiny standalone `fix(output)` PR. The panorama shows the intended (newline)
rendering.
