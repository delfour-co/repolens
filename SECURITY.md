# Security Policy

## Supported versions

RepoLens is at **2.0.2**; security fixes target the latest released version and `main`.

| Version | Supported |
| ------- | --------- |
| 2.0.x   | ✅        |
| < 2.0   | ❌        |

## Reporting a vulnerability

Please report security issues **privately** — do not open a public issue.

- Email: **k@levilainpetit.dev**
- Or use GitHub's private vulnerability reporting:
  <https://github.com/systm-d/repolens/security/advisories/new>

Include a description, reproduction steps, and the affected version. We aim to
acknowledge reports within 7 days and to coordinate a fix and disclosure
timeline with you.

RepoLens reads repository contents and calls the GitHub API on your behalf (via
`octocrab` or the `gh` CLI); it does not transmit repository data to any other
third party. `apply` can execute user-defined custom rule commands from
`.repolens.toml` — treat config files from untrusted repositories with the same
caution as any other executable script.
