#!/usr/bin/env python3
"""
Script robuste pour créer automatiquement les issues GitHub depuis ISSUES.md
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

def create_issue(repo, issue_data, dry_run=False):
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
    
    if dry_run:
        print(f"[DRY RUN] Issue #{number}: {title}")
        print(f"  Labels: {', '.join(labels)}")
        print(f"  Body length: {len(body)} chars")
        return None
    
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
        # Si c'est juste un problème de label, créer l'issue sans labels
        if 'label' in e.stderr.lower() and 'not found' in e.stderr.lower():
            print(f"  ⚠ Labels non trouvés, création sans labels...")
            cmd_no_labels = ['gh', 'issue', 'create', '--repo', repo, '--title', title, '--body', body]
            try:
                result = subprocess.run(
                    cmd_no_labels,
                    capture_output=True,
                    text=True,
                    check=True
                )
                output = result.stdout.strip()
                url_match = re.search(r'https://github\.com/[^/]+/[^/]+/issues/\d+', output)
                if url_match:
                    # Ajouter les labels après création
                    for label in labels:
                        if label:
                            try:
                                subprocess.run(
                                    ['gh', 'issue', 'edit', url_match.group(0), '--add-label', label],
                                    capture_output=True,
                                    check=True
                                )
                            except:
                                pass  # Ignorer les erreurs de labels
                    return url_match.group(0)
                return output
            except subprocess.CalledProcessError as e2:
                print(f"Erreur lors de la création de l'issue #{number}: {e2.stderr}", file=sys.stderr)
                return None
        else:
            print(f"Erreur lors de la création de l'issue #{number}: {e.stderr}", file=sys.stderr)
            if e.stdout:
                print(f"Stdout: {e.stdout}", file=sys.stderr)
            return None

def main():
    # Parser les arguments
    auto_yes = '--yes' in sys.argv or '-y' in sys.argv
    repo_arg = None
    for arg in sys.argv[1:]:
        if arg not in ['--yes', '-y'] and not arg.startswith('-'):
            repo_arg = arg
            break
    
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
        repo = repo_arg
        if not repo:
            print("Erreur: Impossible de détecter le repository. Spécifiez-le en argument.")
            print("Usage: python create_issues_robust.py [owner/repo] [--yes]")
            sys.exit(1)
    
    issues_file = Path(__file__).parent.parent / 'ISSUES.md'
    
    if not issues_file.exists():
        print(f"Erreur: {issues_file} non trouvé", file=sys.stderr)
        sys.exit(1)
    
    print(f"Parsing {issues_file}...")
    issues = parse_issues_file(issues_file)
    print(f"Trouvé {len(issues)} issues\n")
    
    # Afficher un aperçu
    print("Aperçu des 5 premiers issues:")
    for issue in issues[:5]:
        print(f"  #{issue['number']}: {issue['title']}")
    print(f"  ... et {len(issues) - 5} autres\n")
    
    # Demander confirmation sauf si --yes
    if not auto_yes:
        print(f"Ce script va créer {len(issues)} issues sur {repo}")
        response = input("Continuer? (y/N): ")
        if response.lower() != 'y':
            print("Annulé")
            sys.exit(0)
    else:
        print(f"Création de {len(issues)} issues sur {repo} (mode automatique)\n")
    
    print(f"\nCréation des issues sur {repo}...\n")
    
    created = 0
    failed = 0
    urls = []
    
    for issue in issues:
        print(f"[{issue['number']}/{len(issues)}] Création: {issue['title'][:60]}...")
        url = create_issue(repo, issue)
        
        if url:
            print(f"  ✓ Créée: {url}")
            urls.append(url)
            created += 1
        else:
            print(f"  ✗ Échec")
            failed += 1
        
        # Pause pour éviter le rate limiting
        if issue['number'] < len(issues):
            time.sleep(1.5)  # 1.5 secondes entre chaque issue
    
    print(f"\n{'='*60}")
    print(f"✓ {created} issues créées avec succès")
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
