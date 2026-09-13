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
import os
import re
import subprocess
import sys

# Le dépôt, DÉRIVÉ du chemin de ce script et jamais écrit en dur.
#
# Il valait `/home/ubuntu/koprogo` — le poste d'une seule personne. Tant que
# ces scripts ne tournaient que là, personne ne l'a vu. Le 2026-09-13, le
# fan-out a appelé `backlog-pret.py --issue` depuis un runner GitHub, et le
# script est mort sur :
#
#     FileNotFoundError: [Errno 2] No such file or directory: '/home/ubuntu/koprogo'
#
# Pire que la panne : le workflow testait `if ! python3 ...` et a donc
# annoncé « #868 ne porte pas les huit éléments » — un VERDICT — là où le
# script n'avait rien pu mesurer. Quatre agents refusés sur un diagnostic
# faux.
#
# `gantt-passes.py` et `rice-produit.py` dérivaient déjà leur chemin. Les
# deux autres non, et rien ne le signalait.
DEPOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
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


def une_issue(numero: int) -> int:
    """Mesure UNE issue, pour que le fan-out emploie CET instrument (#874).

    ── Pourquoi ce mode existe ───────────────────────────────────────────────

    `fanout-stories.yml` cherchait le titre littéral « Story Agent IA Ready »
    dans le corps de l'issue, et refusait de lancer l'agent sans lui. C'était
    une SECONDE définition de « prêt », à côté de celle de ce script, et les
    deux se sont contredites le 2026-09-13 : #877 porte les huit éléments —
    ce script rendait 89/89 — sous les titres `## Récit` et `## Critères`,
    et le workflow l'a refusée.

    Un agent n'a pas tourné pour un désaccord de vocabulaire entre deux
    instruments dont aucun ne savait que l'autre existait. C'est le motif
    dominant de ce dépôt, appliqué cette fois à la mesure elle-même.

    Le remède n'est pas d'ajouter le titre à #877 : ce serait plier l'issue à
    l'instrument le plus grossier des deux. C'est de n'avoir qu'un
    instrument.

    Rend 0 si l'issue porte les huit éléments, 1 sinon, et NOMME ce qui
    manque — un refus qui ne dit pas quoi corriger se lit comme un caprice.
    """
    # Code 2 = « je n'ai PAS PU mesurer », distinct de 1 = « mesuré, et il
    # manque quelque chose ».
    #
    # Sans cette distinction, un appelant qui écrit `if ! script` confond une
    # panne avec un verdict. C'est arrivé le 2026-09-13 : `DEPOT` pointait
    # vers un chemin absent du runner, le script est mort, et le fan-out a
    # annoncé « #868 ne porte pas les huit éléments » à quatre agents dont
    # les issues étaient prêtes. L'absence de mesure s'écrit `null`, pas
    # `zéro` — ici elle s'écrit 2, pas 1.
    try:
        rendu = subprocess.run(
            ["gh", "issue", "view", str(numero), "--json", "number,title,body"],
            capture_output=True, text=True, cwd=DEPOT, check=True)
    except (OSError, subprocess.CalledProcessError) as e:
        print(f"#{numero} n'a PAS PU être mesurée : {e}", file=sys.stderr)
        print("Ce n'est pas un verdict sur la story.", file=sys.stderr)
        return 2
    issue = json.loads(rendu.stdout)
    absents = manquants(issue.get("body") or "")
    if not absents:
        print(f"#{numero} porte les huit éléments.")
        return 0
    libelles = {cle: libelle for cle, libelle, _ in CRITERES}
    print(f"#{numero} — éléments manquants :", file=sys.stderr)
    for cle in absents:
        print(f"  - {libelles[cle]}", file=sys.stderr)
    return 1


def main():
    for i, arg in enumerate(sys.argv):
        if arg == "--issue" and i + 1 < len(sys.argv):
            return une_issue(int(sys.argv[i + 1]))
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
