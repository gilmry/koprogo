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
  [765,770,772,774,777,779,780,
   790,791,792,793,794,796,804,814]),

 ("U", "Refonte UX/UI", """Revue de design du 2026-09-06. Entrée en 0.1.0 le même jour : un financeur ne
lit pas du code, et l'ASBL se fonde sur ce que le produit montre. **U2 ne
commence qu'une fois #772 fermée** — remanier le périmètre applicatif avant
d'avoir fermé la dette de cloisonnement serait l'ordre inverse du bon.""",
  [556,818,820,821,822,823,824,825,826,827,797,798,802,803]),

 ("D", "Documentation vivante multi-persona", """Six personas, quatre workflows transverses, et la restructuration des cent
specs e2e pour que les vidéos racontent le produit plutôt que ses modules.
**Tout le track vient après #803 et après Track U** : filmer des écrans qui
vont changer produit une documentation périmée le jour de sa livraison.""",
  [805,806,807,808,809,810,811,812,813,815,816,817,595]),

 ("M", "Modularité par ACP et RBAC communautaire", """Slice 5 de l'épopée #556. Une ACP active les modules dont elle a besoin ; le
reste répond 403, pas 404. Ce track porte aussi les deux arbitrages ouverts
sur les droits communautaires — le syndic peut-il réserver au nom de l'ACP
(#781, #588), et le comptable doit-il être exclu du communautaire (#589).""",
  [585,586,587,588,589,590,591,592,694,781]),

 ("S", "Gouvernance d'assemblée avancée", """Slice 4 de #556 : assemblée hybride, vote à distance authentifié fort,
procès-verbal signé eIDAS, conseil de copropriété élu, commissaire aux
comptes. C'est le track dont dépend la crédibilité juridique du produit
au-delà du strict Art. 3.87.""",
  [576,577,578,579,581,582,583]),

 ("T", "Dette d'infrastructure de test", """Ce qui empêche la CI de dire la vérité. **Quatre jobs sur dix sont rouges en
continu depuis le 2026-09-04 au moins** : `prettier`, le contrat OpenAPI,
`oasdiff` et la suite BDD. Une CI rouge en permanence ne garde rien — elle
apprend seulement à ne plus la regarder.""",
  [443,540,548,696]),

 ("K", "Dette de code et de contrat", """Les erreurs typées plutôt que classées par sous-chaîne, le contrat OpenAPI
complet, et la suppression du repli qui fabrique une ACP inexistante.""",
  [555,761,762]),

 ("F", "Ops et infrastructure", """Sauvegardes, TLS, GitOps, et les vulnérabilités de dépendances. F3 a été joué
le 2026-09-04 et son résultat est **négatif sur deux volets sur trois** : le
rollback échoue dès qu'une version a migré, et les sauvegardes GPG+S3 du
runbook n'existent pas sur la machine.""",
  [354,355,425,429,432,453,466,515,718,731]),

 ("G", "Gate humain et gouvernance documentaire", """Les deux actes non délégables — la revue humaine et la pose du tag — et ce
qui les prépare : la taxonomie des tests comme gate de release, et le
désencombrement de la documentation.""",
  [426,427]),

 ("?", "À arbitrer — présence en 0.1.0 douteuse", """Une issue dont l'étiquette et le titre se contredisent. Il faut trancher, pas
laisser le doute dans le périmètre.""",
  [635]),
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

print("\n".join(lignes))
