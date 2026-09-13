# -*- coding: utf-8 -*-
"""Mesure combien d'issues du périmètre 0.1.0 sont « Agent IA Ready ».

── Pourquoi ce script existe ──────────────────────────────────────────────

La Méthode Foyer (BMAD, phase E de TOGAF) pose qu'une story n'entre pas en
fabrication sans ses huit éléments. Le Scrum Master de conception l'écrit sans
détour : « **aucune story n'est prête sans elles** ».

Un chiffre de préparation écrit à la main dans un markdown vieillit en trois
jours et personne ne s'en aperçoit. Celui-ci se relève.

── Ce qu'il ne prétend pas faire ─────────────────────────────────────────

Il cherche des **marqueurs de forme**, pas du sens. Une issue qui écrit
`@security` au-dessus d'un critère creux sera comptée comme portant sa classe
de tests. C'est une borne haute de la préparation, jamais un verdict : le
jugement reste humain, c'est le point entier du *répondre-de*.

Usage :
    python3 scripts/backlog-pret.py            # le tableau
    python3 scripts/backlog-pret.py --detail   # + ce qui manque, issue par issue
"""
import json
import re
import subprocess
import sys

DEPOT = "/home/ubuntu/koprogo"
JALON = "release:0.1.0"

# Les huit éléments du gabarit `bmad/livrables/epics-stories.template.md`.
# L'ordre suit celui du gabarit, pour qu'on puisse les lire en vis-à-vis.
CRITERES = [
    ("recit", "Récit — En tant que… je veux… afin de…",
     lambda c: re.search(r"en tant qu", c, re.I) is not None),
    ("gherkin", "Critères Gherkin — Étant donné / Quand / Alors",
     lambda c: (re.search(r"étant donné|given\b", c, re.I) is not None
                and re.search(r"\balors\b|\bthen\b", c, re.I) is not None)),
    ("happy", "Classe @happy — le chemin nominal",
     lambda c: "@happy" in c.lower()),
    ("negative", "Classe @negative — entrées invalides, échecs attendus",
     lambda c: "@negative" in c.lower()),
    ("edge", "Classe @edge — bornes : vide, max, concurrence",
     lambda c: "@edge" in c.lower()),
    ("security", "Classe @security — abus, injection, autorisation",
     lambda c: "@security" in c.lower()),
    ("couche", "Couche — Domain / Application / Infra / Frontend / IaC",
     # Le gabarit écrit littéralement `**Couche(s)**`, parenthèses comprises.
     # Un `couches?` ne les lit pas : c'est ce qui a fait rendre « manquant »
     # à une story qui portait le champ.
     lambda c: re.search(r"^\s*[-*]?\s*\*{0,2}couche(?:s|\(s\))?\*{0,2}\s*:", c,
                         re.I | re.M) is not None),
    ("taille", "Taille — S (0,5 j) / M (0,75 j) / L (1 j)",
     lambda c: re.search(r"\*{0,2}taille\*{0,2}\s*:?\s*\*{0,2}[SML]\b", c,
                         re.I) is not None),
]


def issues_ouvertes():
    brut = subprocess.run(
        ["gh", "issue", "list", "--state", "open", "--limit", "300",
         "--label", JALON, "--json", "number,title,body"],
        capture_output=True, text=True, cwd=DEPOT, check=True).stdout
    return json.loads(brut)


def manquants(corps):
    return [cle for cle, _, teste in CRITERES if not teste(corps)]


def main():
    detail = "--detail" in sys.argv
    issues = issues_ouvertes()
    total = len(issues)
    if total == 0:
        print(f"Aucune issue ouverte en {JALON} : rien à mesurer.")
        return 0

    print(f"{total} issues ouvertes en {JALON}\n")
    print("  présent  élément")
    print("  ───────  ─────────────────────────────────────────────────────")
    for cle, libelle, teste in CRITERES:
        n = sum(1 for i in issues if teste(i["body"] or ""))
        print(f"  {n:3}/{total:<3}  {libelle}")

    pretes = [i for i in issues if not manquants(i["body"] or "")]
    quatre = [i for i in issues
              if not [c for c in manquants(i["body"] or "")
                      if c in ("happy", "negative", "edge", "security")]]

    print()
    print(f"  {len(quatre):3}/{total:<3}  portent les QUATRE classes de tests")
    print(f"  {len(pretes):3}/{total:<3}  **Agent IA Ready** — les huit éléments")

    if detail:
        print("\n── Ce qui manque, issue par issue ──\n")
        for i in sorted(issues, key=lambda x: len(manquants(x["body"] or ""))):
            trous = manquants(i["body"] or "")
            etat = "PRÊTE" if not trous else "manque " + ", ".join(trous)
            print(f"  #{i['number']:<5} {etat}")
            print(f"         {i['title'][:78]}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
