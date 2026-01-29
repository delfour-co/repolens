#!/usr/bin/env python3
"""
Script pour créer automatiquement les issues GitHub depuis ISSUES.md
"""

import re
import subprocess
import sys
import time
from pathlib import Path

def parse_issues_file(file_path):
    """Parse le fichier ISSUES.md et extrait tous les issues"""
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Pattern pour trouver chaque issue
    pattern = r'### Issue #(\d+): (.+?)\n\n\*\*Labels:\*\* (.+?)\n\n\*\*Description:\*\*\n\n(.+?)\n\n\*\*Objectifs:\*\*\n\n(.+?)\n\n\*\*Acceptance Criteria:\*\*\n\n(.+?)\n\n---'
    
    issues = []
    for match in re.finditer(pattern, content, re.DOTALL):
        issue_num = int(match.group(1))
        title = match.group(2).strip()
        labels = [l.strip().replace('`', '') for l in match.group(3).split(',')]
        description = match.group(4).strip()
        objectives = match.group(5).strip()
        acceptance = match.group(6).strip()
        
        # Construire le body complet
        body = f"""## Description

{description}

## Objectifs

{objectives}

## Acceptance Criteria

{acceptance}"""
        
        issues.append({
            'number': issue_num,
            'title': title,
            'body': body,
            'labels': labels
        })
    
    return sorted(issues, key=lambda x: x['number'])

def create_issue(repo, issue_data, dry_run=False):
    """Crée un issue sur GitHub"""
    title = issue_data['title']
    body = issue_data['body']
    labels = issue_data['labels']
    number = issue_data['number']
    
    if dry_run:
        print(f"[DRY RUN] Issue #{number}: {title}")
        print(f"  Labels: {', '.join(labels)}")
        print(f"  Body length: {len(body)} chars")
        return None
    
    # Préparer la commande gh
    cmd = ['gh', 'issue', 'create', '--repo', repo, '--title', title, '--body', body]
    
    # Ajouter les labels
    for label in labels:
        cmd.extend(['--label', label])
    
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            check=True
        )
        
        # Extraire l'URL de l'issue
        url_match = re.search(r'https://github\.com/[^/]+/[^/]+/issues/\d+', result.stdout)
        if url_match:
            url = url_match.group(0)
            return url
        return result.stdout.strip()
    except subprocess.CalledProcessError as e:
        print(f"Erreur lors de la création de l'issue #{number}: {e.stderr}", file=sys.stderr)
        return None

def main():
    repo = sys.argv[1] if len(sys.argv) > 1 else None
    
    if not repo:
        # Essayer de détecter le repo automatiquement
        try:
            result = subprocess.run(
                ['gh', 'repo', 'view', '--json', 'nameWithOwner', '-q', '.nameWithOwner'],
                capture_output=True,
                text=True,
                check=True
            )
            repo = result.stdout.strip()
        except:
            print("Erreur: Impossible de détecter le repository. Spécifiez-le en argument.")
            print("Usage: python create_issues.py [owner/repo]")
            sys.exit(1)
    
    issues_file = Path(__file__).parent.parent / 'ISSUES.md'
    
    if not issues_file.exists():
        print(f"Erreur: {issues_file} non trouvé", file=sys.stderr)
        sys.exit(1)
    
    print(f"Parsing {issues_file}...")
    issues = parse_issues_file(issues_file)
    print(f"Trouvé {len(issues)} issues\n")
    
    # Demander confirmation
    print(f"Ce script va créer {len(issues)} issues sur {repo}")
    response = input("Continuer? (y/N): ")
    if response.lower() != 'y':
        print("Annulé")
        sys.exit(0)
    
    print(f"\nCréation des issues sur {repo}...\n")
    
    created = 0
    failed = 0
    
    for issue in issues:
        print(f"[{issue['number']}/{len(issues)}] Création: {issue['title']}")
        url = create_issue(repo, issue)
        
        if url:
            print(f"  ✓ Créée: {url}")
            created += 1
        else:
            print(f"  ✗ Échec")
            failed += 1
        
        # Pause pour éviter le rate limiting (5000 requêtes/heure pour les issues)
        if issue['number'] < len(issues):
            time.sleep(1)
    
    print(f"\n✓ {created} issues créées avec succès")
    if failed > 0:
        print(f"✗ {failed} issues ont échoué")
    
    return 0 if failed == 0 else 1

if __name__ == '__main__':
    sys.exit(main())
