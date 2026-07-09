+++
title = "Configure every repo to a consistent standard."

[extra]
tagline = "A CLI auto-configurator for GitHub & GitLab repositories."
lede = "RepoLens audits repository hygiene and configuration, then turns every finding into a reviewable, applicable action — so you review the plan before anything changes."
cta = "View on GitHub"
cta2 = "Install"
+++

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
  <div class="callout">{{ img(src="img/portrait.webp") }}<div>🛡️ <strong>RepoLens configures a repository as well as it can, automatically</strong> — the plan/apply split means you always review before anything changes. <em>Not a secret or dependency scanner; it focuses on structure, docs &amp; hosting-platform configuration.</em></div></div>
</div>
