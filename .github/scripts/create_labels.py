#!/usr/bin/env python3
"""
Script pour créer tous les labels nécessaires sur GitHub
"""

import subprocess
import sys

LABELS = [
    ("enhancement", "a2eeef", "Nouvelle fonctionnalité ou amélioration"),
    ("feature", "0e8a16", "Nouvelle fonctionnalité"),
    ("bug", "d73a4a", "Quelque chose ne fonctionne pas"),
    ("documentation", "0075ca", "Amélioration de la documentation"),
    ("testing", "bfe5bf", "Tests"),
    ("security", "ee0701", "Sécurité"),
    ("performance", "fbca04", "Performance"),
    ("ux", "c5def5", "Expérience utilisateur"),
    ("i18n", "bfd4f2", "Internationalisation"),
    ("technical", "7057ff", "Amélioration technique"),
    ("refactoring", "e4e669", "Refactoring"),
    ("maintenance", "ffffff", "Maintenance"),
    ("priority:high", "b60205", "Priorité haute"),
    ("priority:medium", "fbca04", "Priorité moyenne"),
    ("priority:low", "0e8a16", "Priorité basse"),
]

def create_label(repo, name, color, description):
    """Crée un label sur GitHub"""
    try:
        subprocess.run(
            ['gh', 'label', 'create', name, '--repo', repo, '--color', color, '--description', description],
            capture_output=True,
            check=True
        )
        print(f"  ✓ Label '{name}' créé")
        return True
    except subprocess.CalledProcessError as e:
        # Le label existe peut-être déjà
        if 'already exists' in e.stderr.decode() or 'already_exists' in e.stderr.decode():
            print(f"  - Label '{name}' existe déjà")
            return True
        else:
            print(f"  ✗ Erreur pour '{name}': {e.stderr.decode()}")
            return False

def main():
    # Détecter le repo
    try:
        result = subprocess.run(
            ['gh', 'repo', 'view', '--json', 'nameWithOwner', '-q', '.nameWithOwner'],
            capture_output=True,
            text=True,
            check=True
        )
        repo = result.stdout.strip()
    except:
        repo = sys.argv[1] if len(sys.argv) > 1 else None
        if not repo:
            print("Erreur: Impossible de détecter le repository")
            sys.exit(1)
    
    print(f"Création des labels sur {repo}...\n")
    
    created = 0
    for name, color, description in LABELS:
        if create_label(repo, name, color, description):
            created += 1
    
    print(f"\n✓ {created}/{len(LABELS)} labels créés/vérifiés")

if __name__ == '__main__':
    main()
