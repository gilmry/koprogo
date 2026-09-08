# -*- coding: utf-8 -*-
import subprocess, json, collections

brut = subprocess.run(
    ["gh","issue","list","--state","open","--limit","300","--label","release:0.1.0",
     "--json","number,title,labels"], capture_output=True, text=True, cwd="/home/ubuntu/koprogo").stdout
issues = {i["number"]: i for i in json.loads(brut)}

# Affectation explicite. Toute issue non citée ici tombe dans « non classé »,
# ce qui fait échouer la génération : le classement doit être exhaustif.
TRACKS = [
 ("R", "Défauts de recette navigateur", """Six recettes menées au navigateur entre le 2026-09-04 et le 2026-09-06. Le
motif dominant, confirmé six fois : **une capacité écrite, testée, et
inatteignable**. Nos tests prouvent que le code marche tout en masquant qu'on
ne peut pas y arriver.""",
  [779,780,840,842]),

 ("U", "Refonte UX/UI", """Revue de design du 2026-09-06. Entrée en 0.1.0 le même jour : un financeur ne
lit pas du code, et l'ASBL se fonde sur ce que le produit montre. **U2 ne
commençait qu'une fois #772 fermée. **Elle l'est depuis le 2026-09-08** : la
dette de cloisonnement est passée de 109 routes imbriquées sans identité à
trois, et un test end-to-end prouve qu'aucune donnée ne traverse d'une
organisation à l'autre, en lecture comme en écriture. U2 n'est plus bloquée.

Les quatre maquettes U0a à U0d dépendaient de #803. **Sa dette est à zéro
depuis le 2026-09-08** : les 755 éléments interactifs sans ancrage relevés au
départ sont tous ancrés, et le cliquet est devenu une interdiction. La
refonte peut désormais déplacer n'importe quel écran sans qu'on perde le
moyen de vérifier qu'il a survécu — ce qui était l'argument entier de
l'issue. Le seul critère de fin qu'elle porte encore est une règle de
processus pour les lots à venir, qu'aucun commit ne peut satisfaire.""",
  [556,818,820,821,822,823,824,825,826,827,797,798,802,803,834,841]),

 ("D", "Documentation vivante multi-persona", """Six personas, quatre workflows transverses, et la restructuration des cent
specs e2e pour que les vidéos racontent le produit plutôt que ses modules.
**Tout le track vient après #803 et après Track U** : filmer des écrans qui
vont changer produit une documentation périmée le jour de sa livraison.""",
  [805,806,807,808,809,810,811,812,813,815,816,817,595,835]),

 ("M", "Modularité par ACP et RBAC communautaire", """Slice 5 de l'épopée #556. Une ACP active les modules dont elle a besoin ; le
reste répond 403, pas 404. Ce track porte aussi les deux arbitrages ouverts
sur les droits communautaires — le syndic peut-il réserver au nom de l'ACP
(#781, #588), et le comptable doit-il être exclu du communautaire (#589).""",
  [585,586,587,588,589,590,591,592,781]),

 ("S", "Gouvernance d'assemblée avancée", """Slice 4 de #556 : assemblée hybride, vote à distance authentifié fort,
procès-verbal signé eIDAS, conseil de copropriété élu, commissaire aux
comptes. C'est le track dont dépend la crédibilité juridique du produit
au-delà du strict Art. 3.87.

S'y ajoute depuis le 2026-09-08 un défaut qui, lui, porte sur le strict
Art. 3.87 § 1er : **un lot détenu à deux ne peut jamais voter** (#848). La
suspension du vote d'une indivision est implémentée et câblée au portique ;
la désignation d'un représentant, qui est le remède prévu par la loi, n'existe
nulle part — `is_voting_representative` n'est écrite par aucun code. Un couple
propriétaire de son appartement ne peut voter à aucune assemblée, et une
décision prise sans son vote est attaquable.""",
  [576,577,578,579,581,582,583,848]),

 ("T", "Dette d'infrastructure de test", """Ce qui empêche la CI de dire la vérité. Les quatre jobs rouges en continu du
2026-09-04 — `prettier`, le contrat OpenAPI, `oasdiff` et la suite BDD — sont
verts depuis le 2026-09-06. Ce qui reste est plus insidieux : **un job qui
n'est ni vert ni rouge**, Playwright s'exécutant en `skipped` (#828), et **un
garde-fou qui affiche sans bloquer**, `svelte-check --threshold warning` dont
la CLI dit qu'il « filtre les diagnostics à AFFICHER » là où le commentaire
de la CI prétendait qu'il bloquait. Sa référence de 0 warning avait dérivé à
29 sans que rien ne l'annonce, masquant quinze variables non réactives et
trois modals qui ne pouvaient pas s'ouvrir (#832). Corrigé en
`--fail-on-warnings` le 2026-09-07.

Une CI rouge en permanence n'apprend qu'à ne plus la regarder ; une CI qui
affiche sans bloquer apprend à croire qu'on regarde.

S'y ajoute depuis le 2026-09-08 ce qui empêche la RECETTE de dire la vérité :
soixante dialogues natifs du navigateur (#844). Un navigateur piloté les
supprime, et l'action prend alors la forme exacte d'une panne. C'est ce qui a
fait déclarer mort le bouton « Reporter » d'une assemblée pendant deux
recettes, alors que sa source était correcte.

Et le cas le plus retors du track : **une attestation qui ne vérifie rien**
(#847). Le registre légal déclare l'Art. 3.89 § 1er couvert et cite un test
à l'appui ; la citation désignait une méthode de production, un mot courant
et un module `tests`, pas une vérification de la durée de mandat. La garde
constatait que la preuve EXISTE, jamais qu'elle est PERTINENTE. C'est la
forme la plus coûteuse du motif : non pas un test absent, mais un test
présent qui atteste d'autre chose que ce qu'il prétend.""",
  [696,832,847]),

 ("K", "Dette de code et de contrat", """Les erreurs typées plutôt que classées par sous-chaîne, le contrat OpenAPI
complet, et la suppression du repli qui fabrique une ACP inexistante.

Deux constats du 2026-09-08 s'y ajoutent, et ils portent plus loin que le
confort d'écriture. **Trente routes sur 604 ne vérifiaient aucune identité**
(#845) — ni `AuthenticatedUser`, ni jeton lu à la main : supprimer une
assemblée générale ne demandait qu'un UUID. Et **onze tables existent en base
que rien ne lit** (#846), dont celle dont la vue se déclare outil de
vérification des procurations et renvoie toujours zéro ligne.

S'y ajoute le 2026-09-08 une confusion d'identifiants qui rendait une
fonctionnalité entière inatteignable : **aucun copropriétaire ne pouvait
voter à une consultation** (#849), parce que le handler comparait un
`users.id` à des `owners.id`. Le commentaire du code avouait le provisoire
— « for now, we use the authenticated user's ID » — et il n'a jamais été
remplacé.""",
  [555,762,845,846,849]),

 ("F", "Ops et infrastructure", """Sauvegardes, TLS, GitOps, et les vulnérabilités de dépendances. F3 a été joué
le 2026-09-04 et son résultat est **négatif sur deux volets sur trois** : le
rollback échoue dès qu'une version a migré, et les sauvegardes GPG+S3 du
runbook n'existent pas sur la machine.""",
  [354,355,425,429,432,453,466,515,718,731]),

 ("G", "Gate humain et gouvernance documentaire", """Les deux actes non délégables — la revue humaine et la pose du tag — et ce
qui les prépare : la taxonomie des tests comme gate de release, et le
désencombrement de la documentation.""",
  [427]),

 ("?", "À arbitrer — présence en 0.1.0 douteuse", """Deux issues qui se contredisent elles-mêmes. #635 porte « (v0.2.0) » dans son
titre et l'étiquette `release:0.1.0`. #694 écrit noir sur blanc dans son corps
« Non bloquant pour v0.1.0 (bêta fermée) », et demande par ailleurs un brief
Maury signé avant tout code.

Il faut trancher, pas laisser le doute dans le périmètre : chacune pèse sur le
décompte de ce qui reste, et donc sur la date du tag.""",
  [635,694]),
]

classees = set()
for _, _, _, nums in TRACKS:
    for n in nums:
        assert n not in classees, f"#{n} classé deux fois"
        classees.add(n)

manquantes = set(issues) - classees
fantomes = classees - set(issues)
assert not manquantes, f"non classées : {sorted(manquantes)}"
assert not fantomes, f"classées mais hors périmètre : {sorted(fantomes)}"

def priorite(i):
    for l in i["labels"]:
        if l["name"].startswith("priority:"):
            return l["name"].split(":")[1]
    return "—"

RANG = {"critical":0,"high":1,"medium":2,"low":3,"—":4}

lignes = []
lignes.append("## Inventaire complet du périmètre 0.1.0")
lignes.append("")
lignes.append(f"**{len(issues)} issues ouvertes** portent l'étiquette `release:0.1.0`. Elles sont")
lignes.append("toutes ci-dessous, sans exception : une issue du périmètre absente du WBS est")
lignes.append("une issue que personne ne planifie.")
lignes.append("")
lignes.append("Le tableau est engendré depuis GitHub par `scripts/inventaire-wbs.py`, et un")
lignes.append("test refuse qu'une issue du périmètre n'y figure pas. Il ne peut donc pas")
lignes.append("se désynchroniser en silence, comme l'a fait `docs/api/openapi.json` pendant")
lignes.append("cinq jours.")
lignes.append("")
compte = collections.Counter()
for _, _, _, nums in TRACKS:
    for n in nums:
        compte[priorite(issues[n])] += 1
lignes.append("| Priorité | Nombre |")
lignes.append("|---|---|")
for p in ["critical","high","medium","low","—"]:
    if compte[p]:
        lignes.append(f"| {p} | {compte[p]} |")
lignes.append("")

for code, titre, chapeau, nums in TRACKS:
    nums = sorted(nums, key=lambda n: (RANG[priorite(issues[n])], n))
    lignes.append(f"### Track {code} — {titre} ({len(nums)})")
    lignes.append("")
    lignes.append(chapeau)
    lignes.append("")
    lignes.append("| Issue | Prio | Intitulé |")
    lignes.append("|---|---|---|")
    for n in nums:
        t = issues[n]["title"].replace("|", "\\|")
        if len(t) > 100:
            t = t[:99] + "…"
        lignes.append(f"| #{n} | {priorite(issues[n])} | {t} |")
    lignes.append("")

# ── Écriture en place ──────────────────────────────────────────────────────
#
# La section était recopiée à la main depuis la sortie standard. Une copie
# manuelle se périme dès qu'on oublie de la refaire, et c'est précisément ce
# que ce fichier reproche à `docs/api/openapi.json`. Le script substitue donc
# lui-même, entre deux balises, et n'imprime plus rien à recopier.
DEBUT = "<!-- INVENTAIRE:DEBUT — engendré par scripts/inventaire-wbs.py, ne pas éditer à la main -->"
FIN = "<!-- INVENTAIRE:FIN -->"

import pathlib as _pathlib

wbs = _pathlib.Path("/home/ubuntu/koprogo/docs/WBS_v0_1_0.md")
texte = wbs.read_text()
assert DEBUT in texte and FIN in texte, (
    "balises absentes du WBS : la section ne peut pas être remplacée sans risque"
)
avant = texte.split(DEBUT)[0]
apres = texte.split(FIN)[1]
wbs.write_text(avant + DEBUT + "\n\n" + "\n".join(lignes).strip() + "\n\n" + FIN + apres)
print(f"docs/WBS_v0_1_0.md : {len(issues)} issues inventoriées")
