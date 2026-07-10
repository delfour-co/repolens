+++
title = "Ne livrez plus de dépôts incomplets."

[extra]
eyebrow = "audit · plan · apply"
subline = "Pas de LICENSE, pas de protection de branche, pas de topics, métadonnées obsolètes… RepoLens détecte chaque lacune — puis la corrige, après votre relecture du plan."
cta = "Voir sur GitHub"
cta2 = "Installer"
oneliner = "cargo install repolens"
found_label = "détecté par"
done_label = "corrigé par"
found = [
  { c = "cr", g = "✗", t = "LICENSE manquante" },
  { c = "wa", g = "!", t = "protection de branche absente" },
  { c = "cr", g = "✗", t = "ni description ni topics" },
]
done = [
  "LICENSE ajoutée",
  "protection de branche activée",
  "description & topics définis",
]
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
  <div class="lbl mono">en action · sortie réelle</div>
  <h2>Six commandes, capturées en vrai.</h2>

<div class="term flag mono">
<div class="bar"><i style="background:#ff5f56"></i><i style="background:#ffbd2e"></i><i style="background:#27c93f"></i><span class="ttl">plan — audit + plan d'actions relisable</span></div>
<div class="b">
<div><span class="p">$</span> repolens plan</div>
<div class="mut">auditing 6 categories · provider: github (auto)…</div>
<div class="sp"></div>
<div><span class="cy">━━━━━━ AUDIT RESULTS ━━━━━━</span></div>
<div><span class="cr">❌ CRITICAL (1)</span></div>
<div>  • [DOC004] LICENSE file is missing</div>
<div><span class="wa">⚠️  WARNING (6)</span></div>
<div>  • [FILE002] .gitignore file is missing</div>
<div>  • [DOC001] README file is missing</div>
<div>  • [DOC005] CONTRIBUTING file is missing</div>
<div class="mut">  … +3 (CODE_OF_CONDUCT, SECURITY, CHANGELOG)</div>
<div><span class="cy">ℹ️  INFO (4)</span></div>
<div>  • [SEC007] .github/settings.yml is absent</div>
<div>  • [GIT002] .gitattributes file is missing</div>
<div>  • [CODE001] CODEOWNERS file is missing</div>
<div class="mut">  … +1 (runtime version file)</div>
<div class="sp"></div>
<div><span class="cy">━━━━━━ PLANNED ACTIONS ━━━━━━</span></div>
<div><span class="ok">+</span> [file]   Create LICENSE file  <span class="mut">└─ MIT</span></div>
<div><span class="ok">+</span> [file]   Create README.md</div>
<div><span class="ok">+</span> [file]   Create .gitignore</div>
<div><span class="ok">+</span> [github] Enable branch protection on 'main'</div>
<div><span class="ok">+</span> [github] Update repository settings</div>
<div><span class="ok">+</span> [issues] Open issues for 6 warnings</div>
<div class="mut">  … +8 more</div>
<div class="sp"></div>
<div><span class="cy">━━━━━━ SUMMARY ━━━━━━</span></div>
<div>Critical: 1 │ Warnings: 6 │ Info: 4</div>
<div><span class="p">▸</span> Run <span class="cy">repolens apply</span> to execute planned actions.</div>
</div>
</div>

<div class="pano">

<div class="term mono">
<div class="bar"><i style="background:#ff5f56"></i><i style="background:#ffbd2e"></i><i style="background:#27c93f"></i><span class="ttl">apply --dry-run — prévisualiser chaque changement</span></div>
<div class="b">
<div><span class="p">$</span> repolens apply <span class="mut">--dry-run</span></div>
<div class="sp"></div>
<div><span class="cy">════ ACTION SUMMARY ════</span></div>
<div class="sp"></div>
<div>[*] FILE (10 actions)</div>
<div>    <span class="ok">+</span> Create LICENSE file  <span class="mut">— MIT</span></div>
<div>    <span class="ok">+</span> Create README.md</div>
<div>    <span class="ok">+</span> Create .gitignore</div>
<div>    <span class="ok">+</span> Create CONTRIBUTING.md</div>
<div class="mut">    … +6 more</div>
<div class="sp"></div>
<div>[G] GITHUB (3 actions)</div>
<div>    <span class="ok">+</span> Enable branch protection on 'main'</div>
<div>    <span class="ok">+</span> Update repository settings</div>
<div>    <span class="ok">+</span> Update GitHub Actions &amp; security settings</div>
<div class="sp"></div>
<div class="mut">dry run — nothing was changed.</div>
</div>
</div>

<div class="term mono">
<div class="bar"><i style="background:#ff5f56"></i><i style="background:#ffbd2e"></i><i style="background:#27c93f"></i><span class="ttl">report — json · markdown · sarif · html</span></div>
<div class="b">
<div><span class="p">$</span> repolens report <span class="mut">--format markdown</span></div>
<div><span class="ok">✓</span> Report written to: repolens-report.md</div>
<div class="sp"></div>
<div class="mut"># RepoLens Audit Report</div>
<div class="mut">Preset: opensource · Version: 3.0.0</div>
<div class="sp"></div>
<div>## Summary</div>
<div>| Severity | Count |</div>
<div>|----------|-------|</div>
<div>| Critical | 1     |</div>
<div>| Warning  | 6     |</div>
<div>| Info     | 4     |</div>
<div class="sp"></div>
<div class="mut">also: json · sarif · html · -o &lt;file&gt;</div>
</div>
</div>

<div class="term mono">
<div class="bar"><i style="background:#ff5f56"></i><i style="background:#ffbd2e"></i><i style="background:#27c93f"></i><span class="ttl">compare — diff de deux audits (gate CI)</span></div>
<div class="b">
<div><span class="p">$</span> repolens compare <span class="mut">--base-file before.json --head-file after.json</span></div>
<div class="sp"></div>
<div><span class="cy">━━ SCORE ━━</span></div>
<div>32 → 26   <span class="ok">(-6, improved)</span></div>
<div class="sp"></div>
<div><span class="cy">━━ RESOLVED (3) ━━</span></div>
<div>  <span class="ok">-</span> [DOC004]  <span class="cr">CRITICAL</span>  LICENSE file is missing</div>
<div>  <span class="ok">-</span> [DOC001]  <span class="wa">WARNING</span>   README file is missing</div>
<div>  <span class="ok">-</span> [FILE002] <span class="wa">WARNING</span>   .gitignore file is missing</div>
<div class="sp"></div>
<div><span class="cy">━━ NEW ISSUES (8) ━━</span></div>
<div>  <span class="wa">+</span> [DOC002] README is too short (3 lines)</div>
<div class="mut">  + [DOC003] README missing section: Installation</div>
<div class="mut">  … +6 more</div>
</div>
</div>

<div class="term mono">
<div class="bar"><i style="background:#ff5f56"></i><i style="background:#ffbd2e"></i><i style="background:#27c93f"></i><span class="ttl">install-hooks — pre-commit &amp; pre-push</span></div>
<div class="b">
<div><span class="p">$</span> repolens install-hooks</div>
<div class="sp"></div>
<div>Installing Git hooks…</div>
<div>  <span class="ok">→</span> Installed pre-commit hook</div>
<div>  <span class="ok">→</span> Installed pre-push hook</div>
<div class="sp"></div>
<div><span class="ok">Done!</span></div>
</div>
</div>

<div class="term mono">
<div class="bar"><i style="background:#ff5f56"></i><i style="background:#ffbd2e"></i><i style="background:#27c93f"></i><span class="ttl">--help — toute la surface</span></div>
<div class="b">
<div><span class="p">$</span> repolens --help</div>
<div class="sp"></div>
<div class="mut">Commands:</div>
<div>  <span class="cy">init</span>           Initialize a new configuration file</div>
<div>  <span class="cy">plan</span>           Analyze repository and show planned actions</div>
<div>  <span class="cy">apply</span>          Apply planned changes to the repository</div>
<div>  <span class="cy">report</span>         Generate an audit report</div>
<div>  <span class="cy">compare</span>        Compare two audit reports</div>
<div>  <span class="cy">install-hooks</span>  Install or remove Git hooks</div>
<div>  <span class="cy">schema</span>         Display the report JSON Schema</div>
<div class="sp"></div>
<div class="mut">Options:</div>
<div>  <span class="cy">-C</span> &lt;dir&gt;     working directory</div>
<div>  <span class="cy">-c</span> &lt;file&gt;    config file</div>
<div>  <span class="cy">-v</span>           verbose (-v, -vv, -vvv)</div>
<div class="mut">  --provider github|gitlab on plan/report/apply (else auto-detected)</div>
</div>
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
  <div class="lbl mono">installation</div>
  <h2 id="install">Installer RepoLens</h2>
  <div class="installs">
    <div class="ins"><h3>🐳 Docker <span class="mut">· recommandé</span></h3><pre>docker run --rm -v "$(pwd)":/repo \
  ghcr.io/systm-d/repolens plan</pre><p>Aucune installation locale. Image amd64 sur GHCR.</p></div>
    <div class="ins"><h3>📦 Cargo</h3><pre>cargo install repolens</pre><p>Depuis crates.io — nécessite Rust 1.85+.</p></div>
    <div class="ins"><h3>🍺 Homebrew</h3><pre>brew tap systm-d/repolens
brew install repolens</pre><p>macOS &amp; Linux.</p></div>
    <div class="ins"><h3>🪟 Scoop</h3><pre>scoop bucket add systm-d https://github.com/systm-d/scoop-bucket
scoop install repolens</pre><p>Windows.</p></div>
    <div class="ins"><h3>🏛️ AUR</h3><pre>yay -S repolens</pre><p>Arch Linux.</p></div>
    <div class="ins"><h3>⬇️ Binaire pré-compilé</h3><pre>curl -LO https://github.com/systm-d/repolens/releases/latest/download/repolens-linux-x86_64.tar.gz
tar xzf repolens-linux-x86_64.tar.gz
sudo mv repolens /usr/local/bin/</pre><p>Linux x86_64/arm64, macOS, Windows.</p></div>
    <div class="ins"><h3>🔨 Depuis les sources</h3><pre>git clone https://github.com/systm-d/repolens
cargo build --release -p repolens</pre><p>Rust 1.85+, edition 2024.</p></div>
  </div>
  <p class="verify">Vérifiez avec <code>repolens --version</code> → <code>repolens 3.0.0</code>. Le packaging Debian est sous <code>packaging/debian</code>.</p>
  <div class="callout">{{ img(src="img/portrait.webp") }}<div>🛡️ <strong>RepoLens configure un dépôt au mieux, automatiquement</strong> — la séparation plan/apply garantit que vous validez toujours avant tout changement. <em>Ce n'est pas un scanner de secrets ni de dépendances ; il se concentre sur la structure, la documentation &amp; la configuration de la plateforme d'hébergement.</em></div></div>
</div>
