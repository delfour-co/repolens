#!/usr/bin/env python3
"""
Script pour fermer les issues en double (#56-110 qui sont des doublons de #1-55)
"""

import subprocess
import sys

def close_duplicate_issues(repo, start_num=56, end_num=110):
    """Ferme les issues en double"""
    closed = 0
    failed = 0
    
    print(f"Fermeture des issues dupliqués #{start_num} à #{end_num}...\n")
    
    for issue_num in range(start_num, end_num + 1):
        try:
            # Fermer l'issue avec un commentaire
            comment = "Issue fermé car doublon d'un issue existant (#1-55)."
            subprocess.run(
                ['gh', 'issue', 'close', str(issue_num), '--repo', repo, '--comment', comment],
                capture_output=True,
                check=True
            )
            print(f"  ✓ Issue #{issue_num} fermé")
            closed += 1
        except subprocess.CalledProcessError as e:
            print(f"  ✗ Erreur pour l'issue #{issue_num}: {e.stderr.decode()}")
            failed += 1
    
    print(f"\n✓ {closed} issues fermés")
    if failed > 0:
        print(f"✗ {failed} erreurs")
    
    return closed, failed

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
    
    print(f"Repository: {repo}\n")
    print("ATTENTION: Ce script va fermer les issues #56-110 qui sont des doublons.")
    response = input("Continuer? (y/N): ")
    if response.lower() != 'y':
        print("Annulé")
        return 0
    
    closed, failed = close_duplicate_issues(repo)
    return 0 if failed == 0 else 1

if __name__ == '__main__':
    sys.exit(main())
