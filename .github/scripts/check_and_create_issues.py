#!/usr/bin/env python3
"""
Script pour vérifier les issues existants et créer uniquement ceux qui manquent
"""

import re
import subprocess
import sys
import time
from pathlib import Path

def parse_issues_file(file_path):
    """Parse le fichier ISSUES.md et extrait tous les issues"""
    with open(file_path, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    
    issues = []
    current_issue = None
    current_section = None
    in_issue = False
    
    i = 0
    while i < len(lines):
        line = lines[i]
        
        # Détecter le début d'un issue
        match = re.match(r'^### Issue #(\d+): (.+)$', line.strip())
        if match:
            # Sauvegarder l'issue précédent s'il existe
            if current_issue:
                issues.append(current_issue)
            
            # Nouvel issue
            current_issue = {
                'number': int(match.group(1)),
                'title': match.group(2).strip(),
                'labels': [],
                'description': '',
                'objectives': '',
                'acceptance': ''
            }
            in_issue = True
            current_section = None
            i += 1
            continue
        
        if not in_issue:
            i += 1
            continue
        
        # Détecter les labels
        if line.strip().startswith('**Labels:**'):
            labels_str = line.replace('**Labels:**', '').strip()
            current_issue['labels'] = [
                l.strip().replace('`', '').replace("'", '')
                for l in labels_str.split(',')
                if l.strip()
            ]
            i += 1
            continue
        
        # Détecter les sections
        if line.strip() == '**Description:**':
            current_section = 'description'
            i += 1
            continue
        elif line.strip() == '**Objectifs:**':
            current_section = 'objectives'
            i += 1
            continue
        elif line.strip() == '**Acceptance Criteria:**':
            current_section = 'acceptance'
            i += 1
            continue
        
        # Détecter la fin d'un issue
        if line.strip() == '---':
            in_issue = False
            current_section = None
            i += 1
            continue
        
        # Ajouter le contenu à la section courante
        if current_section and line.strip():
            if current_section == 'description':
                current_issue['description'] += line
            elif current_section == 'objectives':
                current_issue['objectives'] += line
            elif current_section == 'acceptance':
                current_issue['acceptance'] += line
        
        i += 1
    
    # Ajouter le dernier issue
    if current_issue:
        issues.append(current_issue)
    
    # Nettoyer les champs
    for issue in issues:
        issue['description'] = issue['description'].strip()
        issue['objectives'] = issue['objectives'].strip()
        issue['acceptance'] = issue['acceptance'].strip()
    
    return sorted(issues, key=lambda x: x['number'])

def get_existing_issues(repo):
    """Récupère la liste des issues existants avec leurs titres"""
    try:
        result = subprocess.run(
            ['gh', 'issue', 'list', '--repo', repo, '--limit', '200', '--json', 'number,title'],
            capture_output=True,
            text=True,
            check=True
        )
        
        import json
        issues = json.loads(result.stdout)
        return {issue['title']: issue['number'] for issue in issues}
    except Exception as e:
        print(f"Erreur lors de la récupération des issues: {e}", file=sys.stderr)
        return {}

def create_issue(repo, issue_data):
    """Crée un issue sur GitHub"""
    title = issue_data['title']
    description = issue_data['description']
    objectives = issue_data['objectives']
    acceptance = issue_data['acceptance']
    labels = issue_data['labels']
    number = issue_data['number']
    
    # Construire le body
    body_parts = []
    if description:
        body_parts.append(f"## Description\n\n{description}")
    if objectives:
        body_parts.append(f"## Objectifs\n\n{objectives}")
    if acceptance:
        body_parts.append(f"## Acceptance Criteria\n\n{acceptance}")
    
    body = "\n\n".join(body_parts)
    
    # Préparer la commande gh
    cmd = ['gh', 'issue', 'create', '--repo', repo, '--title', title, '--body', body]
    
    # Ajouter les labels
    for label in labels:
        if label:  # Ignorer les labels vides
            cmd.extend(['--label', label])
    
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            check=True
        )
        
        # Extraire l'URL de l'issue
        output = result.stdout.strip()
        url_match = re.search(r'https://github\.com/[^/]+/[^/]+/issues/\d+', output)
        if url_match:
            return url_match.group(0)
        return output
    except subprocess.CalledProcessError as e:
        print(f"Erreur lors de la création de l'issue #{number}: {e.stderr}", file=sys.stderr)
        return None

def main():
    # Détecter le repo automatiquement
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
            print("Erreur: Impossible de détecter le repository. Spécifiez-le en argument.")
            print("Usage: python check_and_create_issues.py [owner/repo]")
            sys.exit(1)
    
    issues_file = Path(__file__).parent.parent / 'ISSUES.md'
    
    if not issues_file.exists():
        print(f"Erreur: {issues_file} non trouvé", file=sys.stderr)
        sys.exit(1)
    
    print(f"Parsing {issues_file}...")
    issues = parse_issues_file(issues_file)
    print(f"Trouvé {len(issues)} issues dans le fichier\n")
    
    print(f"Récupération des issues existants sur {repo}...")
    existing_issues = get_existing_issues(repo)
    print(f"Trouvé {len(existing_issues)} issues existants\n")
    
    # Identifier les issues manquants
    missing_issues = []
    for issue in issues:
        if issue['title'] not in existing_issues:
            missing_issues.append(issue)
        else:
            existing_num = existing_issues[issue['title']]
            print(f"  ✓ Issue #{issue['number']} '{issue['title']}' existe déjà (#{existing_num})")
    
    if not missing_issues:
        print("\n✓ Tous les issues sont déjà créés!")
        return 0
    
    print(f"\n{len(missing_issues)} issues manquants à créer:")
    for issue in missing_issues:
        print(f"  - #{issue['number']}: {issue['title']}")
    
    response = input(f"\nCréer les {len(missing_issues)} issues manquants? (y/N): ")
    if response.lower() != 'y':
        print("Annulé")
        return 0
    
    print(f"\nCréation des issues manquants...\n")
    
    created = 0
    failed = 0
    urls = []
    
    for issue in missing_issues:
        print(f"[{issue['number']}] Création: {issue['title'][:60]}...")
        url = create_issue(repo, issue)
        
        if url:
            print(f"  ✓ Créée: {url}")
            urls.append(url)
            created += 1
        else:
            print(f"  ✗ Échec")
            failed += 1
        
        # Pause pour éviter le rate limiting
        if issue != missing_issues[-1]:
            time.sleep(1.5)
    
    print(f"\n{'='*60}")
    print(f"✓ {created} issues créés avec succès")
    if failed > 0:
        print(f"✗ {failed} issues ont échoué")
    print(f"{'='*60}\n")
    
    if urls:
        print("URLs des issues créées:")
        for url in urls:
            print(f"  - {url}")
    
    return 0 if failed == 0 else 1

if __name__ == '__main__':
    sys.exit(main())
