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
  <div class="callout">{{ img(src="img/portrait.webp") }}<div>🛡️ <strong>RepoLens configure un dépôt au mieux, automatiquement</strong> — la séparation plan/apply garantit que vous validez toujours avant tout changement. <em>Ce n'est pas un scanner de secrets ni de dépendances ; il se concentre sur la structure, la documentation &amp; la configuration de la plateforme d'hébergement.</em></div></div>
</div>
