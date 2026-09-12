# -*- coding: utf-8 -*-
"""Structure le périmètre 0.1.0 en épopées et capacités, et la maintient.

── Pourquoi ce script existe ──────────────────────────────────────────────

Le WBS classe les 84 issues par **provenance** : « recette navigateur »,
« revue de design », « documentation vivante », « ops ». C'est l'histoire de
comment on les a trouvées, pas de ce qu'elles livrent. Mesuré le 2026-09-12 :
28 issues ne portaient aucune étiquette de domaine, et 54 aucune étiquette de
track.

Or le domaine, lui, est déjà découpé — et **gardé en CI**. `tests/architecture.rs`
déclare quatre contextes bornés et interdit les dépendances entre eux :

    copropriete         ← ne dépend de rien (le noyau légal)
    comptabilite        ← connaît copropriete
    economie_circulaire ← connaît copropriete
    plateforme          ← ne dépend de rien

Ce sont les épopées. Les y rattacher, c'est faire dire au backlog ce que le
code dit déjà, au lieu de raconter d'où vient chaque ticket.

── Ce que le script garantit ──────────────────────────────────────────────

Le classement est **exhaustif et exclusif** : chaque issue ouverte appartient à
exactement une capacité. Une issue nouvelle, non citée ici, fait ÉCHOUER la
génération. C'est délibéré — un classement qui tolère un reliquat cesse d'être
un classement au troisième ticket.

Usage :
    python3 scripts/backlog-structure.py              # le livrable, sur stdout
    python3 scripts/backlog-structure.py --verifier   # classement seul, code 1 si trou
    python3 scripts/backlog-structure.py --appliquer  # pose les étiquettes GitHub
"""
import importlib.util
import json
import os
import subprocess
import sys

# Les huit éléments du gabarit BMAD vivent dans `backlog-pret.py` : les
# redéfinir ici ferait deux définitions qui divergeraient au premier ajout.
_spec = importlib.util.spec_from_file_location(
    "backlog_pret", os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                 "backlog-pret.py"))
_pret = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_pret)

DEPOT = "/home/ubuntu/koprogo"
JALON = "release:0.1.0"

# ── Les épopées ───────────────────────────────────────────────────────────
# (id, slug d'étiquette, titre, ce que l'épopée tient)
EPOPEES = [
    ("E1", "copropriete", "Copropriété — le jumeau juridique",
     "Le contexte borné qui ne dépend de rien : ce que la loi belge dit d'une "
     "assemblée, d'un lot, d'une voix. Une erreur ici n'est pas un défaut "
     "d'affichage, c'est une décision annulable."),
    ("E2", "comptabilite", "Comptabilité — la charge et sa répartition",
     "Connaît `copropriete` : une charge se répartit sur des quotités, elle ne "
     "peut pas les ignorer. L'inverse est faux."),
    ("E3", "economie-circulaire", "Économie circulaire — les modules communautaires",
     "Connaît `copropriete`. SEL, sondages, objets partagés, énergie : la "
     "partie du produit qui distingue KoproGo d'un logiciel de syndic."),
    ("E4", "plateforme", "Plateforme — identité, périmètre, droits",
     "Ne dépend de rien. Qui est l'appelant, ce qu'il a le droit de voir, et ce "
     "qu'il peut exiger qu'on efface."),
    ("T1", "ux", "Refonte UX — épopée habilitante",
     "Ne livre aucune capacité métier : elle rend les autres atteignables. "
     "BMAD la range parmi les stories transverses, pas parmi les épopées de "
     "domaine, et l'ordre compte."),
    ("T2", "accessibilite", "Accessibilité et mobile",
     "La cible est mobile-first et l'application est écrite desktop-first. "
     "L'écart n'est pas cosmétique : il rend des capacités inatteignables."),
    ("T3", "recette", "Harnais de recette",
     "Sprint 0 continué. Ce qui permet de BOUCLER : sans lui, aucune autre "
     "capacité ne peut être déclarée tenue."),
    ("T4", "doc-vivante", "Documentation vivante multi-persona",
     "Vient après que les parcours fonctionnent : filmer un écran qui casse "
     "produit une documentation périmée le jour de sa livraison."),
    ("T5", "ops", "Ops et infrastructure",
     "Le déploiement, l'IaC, les vulnérabilités, et les garde-fous des agents."),
    ("T6", "arbitrage", "Arbitrages produit en attente",
     "Des questions ouvertes, pas des défauts. La Méthode Foyer les veut en "
     "RFC, discutées avant d'être tranchées — jamais tranchées en silence "
     "dans le code."),
]

# ── Les capacités ─────────────────────────────────────────────────────────
# (id, titre, priorité MoSCoW, issues, taille par issue)
#
# La priorité ORDONNE, elle ne retire rien. La décision du 2026-09-06 a mis
# les 84 au périmètre du tag ; seul le superviseur peut la défaire.
CAPACITES = [
    ("C1.1", "E1", "Une assemblée générale aboutit, de la convocation au PV", "Must",
     {780: "L", 840: "M", 848: "L", 850: "L", 576: "L", 577: "L", 581: "M"}),
    ("C1.2", "E1", "Le procès-verbal fait foi", "Should",
     {578: "L", 579: "L"}),
    ("C1.3", "E1", "Le registre légal atteste ce qu'il déclare", "Must",
     {847: "L", 846: "M"}),
    ("C1.4", "E1", "Les organes de contrôle existent", "Could",
     {582: "M", 583: "L"}),
    ("C1.5", "E1", "L'état daté a un destinataire identifié", "Should",
     {855: "L"}),

    ("C2.1", "E2", "Une quote-part se saisit sans ambiguïté", "Should",
     {852: "S"}),
    ("C2.2", "E2", "Un copropriétaire multi-ACP voit ses montants séparés", "Should",
     {867: "L"}),
    ("C2.3", "E2", "Les fonds affectés sont une entité, pas une convention", "Could",
     {635: "L"}),

    ("C3.1", "E3", "Les modules communautaires sont atteignables", "Should",
     {779: "L", 781: "M"}),
    ("C3.2", "E3", "Le syndic a un rôle dans la communauté", "Could",
     {587: "M", 588: "M", 589: "S"}),
    ("C3.3", "E3", "Une ACP active les modules qu'elle veut", "Could",
     {585: "L", 586: "M", 590: "M", 591: "L"}),

    ("C4.1", "E4", "Toute route décide de l'identité qu'elle reçoit", "Must",
     {864: "L", 845: "L"}),
    ("C4.2", "E4", "Le périmètre est l'ACP, et il survit à la navigation", "Must",
     {694: "L", 798: "L", 841: "M", 868: "S"}),
    ("C4.3", "E4", "Les droits RGPD sont exerçables depuis l'interface", "Must",
     {842: "M"}),
    ("C4.4", "E4", "Un prestataire reçoit un seul lien", "Should",
     {835: "L"}),
    ("C4.5", "E4", "Les erreurs sont typées, pas classées par sous-chaînes", "Should",
     {762: "L", 555: "L"}),

    ("C5.1", "T1", "Le socle visuel : jetons, icônes, libellés traduits", "Should",
     {797: "L", 834: "L"}),
    ("C5.2", "T1", "Le contrat de tests tient la refonte", "Must",
     {802: "L", 803: "L"}),
    ("C5.3", "T1", "Les huit maquettes sont implémentées", "Could",
     {556: "L", 818: "L", 820: "L", 821: "M", 822: "M", 823: "L", 824: "L",
      825: "L", 826: "L", 827: "M"}),

    ("C6.1", "T2", "L'audit d'accessibilité voit les écrans authentifiés", "Should",
     {865: "M", 592: "M"}),
    ("C6.2", "T2", "Le produit est utilisable à une largeur de téléphone", "Should",
     {866: "M", 869: "M", 871: "S"}),

    ("C7.1", "T3", "La recette peut se connecter et s'exécuter", "Must",
     {872: "L", 870: "S", 832: "M", 696: "M"}),
    ("C7.2", "T3", "La taxonomie des tests est la gate de release", "Should",
     {427: "L"}),

    ("C8.1", "T4", "Les six parcours par persona sont filmés", "Could",
     {805: "M", 806: "L", 807: "L", 808: "M", 809: "M", 815: "M", 816: "M"}),
    ("C8.2", "T4", "Les quatre workflows transverses sont filmés", "Could",
     {810: "L", 811: "M", 812: "M", 817: "M"}),
    ("C8.3", "T4", "Les cent specs e2e racontent le produit par persona", "Could",
     {813: "L"}),
    ("C8.4", "T4", "La documentation est rangée et publiée", "Could",
     {854: "M", 595: "S"}),

    ("C9.1", "T5", "Le déploiement tient la charge et le partage du réseau", "Should",
     {731: "M", 718: "L", 515: "L", 453: "M"}),
    ("C9.2", "T5", "L'IaC est testée et relue", "Could",
     {355: "L", 354: "L", 466: "M"}),
    ("C9.3", "T5", "Les vulnérabilités connues sont fermées", "Should",
     {432: "M"}),
    ("C9.4", "T5", "Les garde-fous des agents sont audités", "Could",
     {425: "L", 429: "L"}),

    ("C10.1", "T6", "Le groupe « Communauté » du comptable est tranché", "Must",
     {856: "S"}),
]

JOURS = {"S": 0.5, "M": 0.75, "L": 1.0}
# Un tour = un aller-retour rouge/vert/bleu. Première passe, resserrée par le
# CSI story après story — la méthode le prévoit explicitement.
TOURS = {"S": 2, "M": 3, "L": 4}


def issues_ouvertes():
    brut = subprocess.run(
        ["gh", "issue", "list", "--state", "open", "--limit", "300",
         "--label", JALON, "--json", "number,title,labels"],
        capture_output=True, text=True, cwd=DEPOT, check=True).stdout
    return {i["number"]: i for i in json.loads(brut)}


def verifier(vivantes):
    """Le classement doit être exhaustif et exclusif. Sinon on s'arrête."""
    classees = {}
    doublons = []
    for cid, _, _, _, issues in CAPACITES:
        for n in issues:
            if n in classees:
                doublons.append((n, classees[n], cid))
            classees[n] = cid

    manquantes = sorted(set(vivantes) - set(classees))
    fantomes = sorted(set(classees) - set(vivantes))

    erreurs = []
    if doublons:
        erreurs.append("Issues classées deux fois :\n" + "\n".join(
            f"  #{n} : {a} et {b}" for n, a, b in doublons))
    if manquantes:
        erreurs.append(
            "Issues ouvertes que le classement ignore :\n" + "\n".join(
                f"  #{n}  {vivantes[n]['title'][:72]}" for n in manquantes)
            + "\n\n  Une issue non classée n'a pas d'épopée, donc pas de "
              "capacité, donc aucun palier à franchir. Rattachez-la.")
    if fantomes:
        erreurs.append(
            "Issues classées mais fermées ou hors périmètre :\n" + "\n".join(
                f"  #{n}" for n in fantomes)
            + "\n\n  Retirez-les : un backlog qui cite des tickets fermés "
              "cesse d'être lu.")
    return classees, erreurs


def corps_des_issues():
    """Le CORPS, pas les commentaires : c'est lui qu'un agent lit en reprenant
    un ticket, et un commentaire se fait enterrer par les quarante suivants."""
    brut = subprocess.run(
        ["gh", "issue", "list", "--state", "open", "--limit", "300",
         "--label", JALON, "--json", "number,body"],
        capture_output=True, text=True, cwd=DEPOT, check=True).stdout
    return {i["number"]: i["body"] or "" for i in json.loads(brut)}


def livrable(vivantes, classees):
    corps = corps_des_issues()
    par_epopee = {}
    for cid, eid, titre, moscow, issues in CAPACITES:
        par_epopee.setdefault(eid, []).append((cid, titre, moscow, issues))

    out = []
    w = out.append

    total_j = sum(JOURS[t] for _, _, _, _, iss in CAPACITES for t in iss.values())
    total_t = sum(TOURS[t] for _, _, _, _, iss in CAPACITES for t in iss.values())

    # BMAD : « Chaque livrable doit inclure un frontmatter de signature
    # humaine — trace obligatoire pour le répondre-de. » Il reste VIDE tant
    # que le superviseur n'a pas relu : déclarer signé un livrable qui ne
    # l'est pas serait précisément la faute que la signature doit empêcher.
    w("---")
    w("livrable: Epics & User Stories (BMAD phase E — TOGAF Solutions)")
    w("projet: KoproGo")
    w("jalon: v0.1.0")
    w("genere_par: scripts/backlog-structure.py")
    w("signature_humaine:")
    w("  date: null")
    w("  nom: null")
    w("  role: null")
    w("  etat: NON SIGNÉ — en attente de validation du superviseur")
    w("---")
    w("")
    w("# Backlog structuré — v0.1.0")
    w("")
    w("*Épopées et capacités. Généré par `scripts/backlog-structure.py` : ne pas")
    w("éditer à la main, la prochaine génération écraserait la correction.*")
    w("")
    w(f"**{len(vivantes)} issues ouvertes**, "
      f"{len(EPOPEES)} épopées, {len(CAPACITES)} capacités. "
      f"Classement exhaustif et exclusif : chaque issue appartient à exactement")
    w("une capacité, et une issue non classée fait échouer la génération.")
    w("")
    w("## Comment lire ce document")
    w("")
    w("Le WBS classe par **provenance** — d'où vient le ticket. Celui-ci classe")
    w("par **capacité** — ce que le produit saura faire quand elle sera tenue.")
    w("Les deux servent : le WBS dit le périmètre, celui-ci dit l'ordre et le")
    w("coût.")
    w("")
    w("Les épopées de domaine ne sont pas inventées ici : ce sont les **quatre**")
    w("**contextes bornés** que `backend/tests/architecture.rs` déclare et dont")
    w("il interdit les dépendances croisées. Le backlog dit donc ce que le code")
    w("dit déjà.")
    w("")
    w("**Must / Should / Could ordonne, et ne retire rien.** La décision du")
    w("2026-09-06 a mis les 84 issues au périmètre du tag ; seul le superviseur")
    w("peut la défaire. Un `Could` ici veut dire « en dernier », pas « hors")
    w("release ».")
    w("")
    w("Une capacité est **tenue** quand ses issues sont fermées *et* qu'un test")
    w("la traverse de bout en bout. Une fonctionnalité codée n'est pas une")
    w("capacité disponible.")
    w("")
    # ── Préparation ────────────────────────────────────────────────────
    pretes = [n for n in vivantes if not _pret.manquants(corps.get(n, ""))]
    w("## Préparation des stories")
    w("")
    w("Une capacité **structurée** n'est pas une capacité **prête**. La Méthode")
    w("Foyer pose huit éléments sans lesquels une story n'entre pas en")
    w("fabrication ; le Scrum Master de conception l'écrit sans détour :")
    w("« aucune story n'est prête sans elles ».")
    w("")
    w("| Élément | Ce qu'il pré-engage |")
    w("|---|---|")
    w("| Récit *En tant que… je veux… afin de…* | à qui ça sert, donc ce qu'on "
      "peut retirer |")
    w("| Critères Gherkin | le critère, **avant** la génération |")
    w("| `@happy` | le chemin nominal |")
    w("| `@negative` | les entrées invalides et les échecs attendus |")
    w("| `@edge` | les bornes : vide, max, concurrence |")
    w("| `@security` | abus, injection, autorisation |")
    w("| Couche(s) | où le code atterrit, donc quelles gardes s'appliquent |")
    w("| Taille + tours | le coût, sur les deux axes |")
    w("")
    w(f"**{len(pretes)} issues sur {len(vivantes)}** portent les huit. Le relevé")
    w("d'origine, avant ce travail, donnait **zéro**.")
    w("")
    w("Le chiffre n'est pas écrit à la main : `scripts/backlog-pret.py` le relève")
    w("à chaque exécution, et cette page est générée. Un taux de préparation")
    w("recopié vieillit en trois jours sans que personne s'en aperçoive — c'est")
    w("arrivé à la section « Ordre d'exécution » du WBS, dont les quatre")
    w("premières étapes désignent des issues toutes fermées.")
    w("")
    w("Le script cherche des marqueurs de **forme**, pas du sens : une issue qui")
    w("écrit `@security` au-dessus d'un critère creux est comptée. C'est une")
    w("**borne haute**, jamais un verdict.")
    w("")
    w("### Le neuvième élément, propre à ce dépôt : le témoin")
    w("")
    w("Un test écrit après coup peut ne rien garder. Une assertion du banc")
    w("mobile comparait `document.scrollWidth` à `window.innerWidth`, deux")
    w("valeurs qui grandissent ensemble : elle **ne pouvait pas échouer**, et")
    w("cachait un vrai débordement. Toute story livrée ici porte donc : **le")
    w("défaut est remis, et le test doit échouer.**")
    w("")
    w("### La règle de préparation")
    w("")
    w("Une story est mise au gabarit **quand elle devient la prochaine étape**,")
    w("pas trois semaines avant. Préparer les 84 d'un coup serait une correction")
    w("de masse — ce que la méthode range parmi les gestes qui retirent à")
    w("l'humain les moyens d'assumer — et produirait 84 stories creuses.")
    w("")
    w("### Sprint 0 : la story habilitante est fermée")
    w("")
    w("BMAD pose que pour un archétype *full-stack*, le Sprint 0 inclut")
    w("**obligatoirement** le harnais de contrat API, et que sur un projet")
    w("existant qui en manque, c'est une story de correction structurelle qui")
    w("**bloque le reste du backlog**.")
    w("")
    w("Vérifié : #765 est fermée. Les dix-sept routes `/expenses` et `/invoices`")
    w("hors contrat OpenAPI — celles qui avaient laissé `line_items` se perdre en")
    w("silence — y sont. Le backlog n'est pas bloqué à ce titre.")
    w("")

    for eid, slug, titre, propos in EPOPEES:
        caps = par_epopee.get(eid, [])
        jours = sum(JOURS[t] for _, _, _, iss in caps for t in iss.values())
        tours = sum(TOURS[t] for _, _, _, iss in caps for t in iss.values())
        n = sum(len(iss) for _, _, _, iss in caps)
        w(f"## {eid} — {titre}")
        w("")
        w(f"`epic:{slug}` · {n} issues · {jours:.2f} j · {tours} tours")
        w("")
        w(propos)
        w("")
        for cid, ctitre, moscow, issues in caps:
            cj = sum(JOURS[t] for t in issues.values())
            ct = sum(TOURS[t] for t in issues.values())
            w(f"### {cid} — {ctitre}")
            w("")
            pretes = sum(1 for n in issues
                         if not _pret.manquants(corps.get(n, "")))
            w(f"**{moscow}** · `cap:{cid}` · {len(issues)} issues · "
              f"{cj:.2f} j · {ct} tours · **{pretes}/{len(issues)} prêtes**")
            w("")
            w("| Issue | Titre | Taille | Prête |")
            w("|---|---|---|---|")
            for num in sorted(issues):
                t = vivantes[num]["title"].replace("|", "\\|")
                if len(t) > 80:
                    t = t[:77] + "…"
                trous = _pret.manquants(corps.get(num, ""))
                marque = "oui" if not trous else f"manque {len(trous)}/8"
                w(f"| [#{num}](https://github.com/gilmry/koprogo/issues/{num}) "
                  f"| {t} | {issues[num]} | {marque} |")
            w("")

    w("## Estimation")
    w("")
    w("| Épopée | Issues | Jours | Tours |")
    w("|---|---:|---:|---:|")
    for eid, slug, titre, _ in EPOPEES:
        caps = par_epopee.get(eid, [])
        n = sum(len(iss) for _, _, _, iss in caps)
        j = sum(JOURS[t] for _, _, _, iss in caps for t in iss.values())
        t = sum(TOURS[t] for _, _, _, iss in caps for t in iss.values())
        w(f"| {eid} — {titre.split('—')[0].strip()} | {n} | {j:.2f} | {t} |")
    w(f"| **Total** | **{len(vivantes)}** | **{total_j:.2f}** | **{total_t}** |")
    w("")
    w("`S` = 0,5 j · `M` = 0,75 j · `L` = 1 j — wall-clock du superviseur, pas")
    w("temps machine. Les tours mesurent l'autre axe, le coût en tokens.")
    w("")
    w("**Ce sont des bornes hautes de première passe.** La méthode prévoit")
    w("qu'elles soient resserrées story après story par le CSI, à partir du réel")
    w("observé. Les publier non resserrées est le seul moyen d'avoir un point de")
    w("départ falsifiable ; les publier comme un engagement serait une faute.")
    w("")

    w("## Ordre")
    w("")
    w("| Rang | Capacités | Pourquoi ce rang |")
    w("|---|---|---|")
    w("| 1 | C7.1 | Sans harnais de recette qui s'exécute, aucune autre capacité "
      "ne peut être déclarée tenue. |")
    w("| 2 | C4.1, C4.2, C4.3 | Ce qui expose des données ou empêche d'exercer "
      "un droit. |")
    w("| 3 | C10.1 | Un arbitrage qui borne C5.2 : le trancher tôt coûte une "
      "conversation, le trancher tard coûte un revirement. |")
    w("| 4 | C5.2, C5.1 | Le contrat de tests AVANT de déplacer un écran, puis "
      "le socle visuel. |")
    w("| 5 | C1.1, C1.3 | Le noyau légal : une AG qui aboutit, un registre qui "
      "atteste. |")
    w("| 6 | C1.5, C2.1, C2.2, C3.1, C4.4, C4.5, C6.1, C6.2, C9.1, C9.3 | Les "
      "`Should`, largement parallélisables. |")
    w("| 7 | les `Could` | C5.3 après C5.1 ; tout T4 après que les parcours "
      "fonctionnent. |")
    w("| 8 | G1 puis G2 | Revue humaine signée, puis le tag. Hors périmètre "
      "agent. |")
    w("")
    w("---")
    w("")
    w("*Dérivé du Manifeste Maury (CC BY-SA 4.0). Gabarit BMAD phase E :")
    w("`foyer/bmad/livrables/epics-stories.template.md`.*")
    return "\n".join(out) + "\n"


def appliquer(vivantes, classees):
    """Pose `epic:<slug>` et `cap:<id>` sur chaque issue."""
    epopee_de = {cid: eid for cid, eid, _, _, _ in CAPACITES}
    slug_de = {eid: slug for eid, slug, _, _ in EPOPEES}

    existantes = {l["name"] for l in json.loads(subprocess.run(
        ["gh", "label", "list", "--limit", "300", "--json", "name"],
        capture_output=True, text=True, cwd=DEPOT, check=True).stdout)}

    voulues = {f"epic:{s}" for s in slug_de.values()}
    voulues |= {f"cap:{c}" for c in classees.values()}
    for nom in sorted(voulues - existantes):
        subprocess.run(["gh", "label", "create", nom, "--force",
                        "--color", "0E8A16" if nom.startswith("epic") else "C2E0C6",
                        "--description", "Backlog structuré v0.1.0"],
                       cwd=DEPOT, check=True, capture_output=True)
        print(f"  étiquette créée : {nom}")

    for num, cid in sorted(classees.items()):
        actuelles = {l["name"] for l in vivantes[num]["labels"]}
        cibles = {f"epic:{slug_de[epopee_de[cid]]}", f"cap:{cid}"}
        a_poser = cibles - actuelles
        a_retirer = {l for l in actuelles
                     if (l.startswith("epic:") or l.startswith("cap:"))
                     and l not in cibles}
        if not a_poser and not a_retirer:
            continue
        cmd = ["gh", "issue", "edit", str(num)]
        for l in sorted(a_poser):
            cmd += ["--add-label", l]
        for l in sorted(a_retirer):
            cmd += ["--remove-label", l]
        subprocess.run(cmd, cwd=DEPOT, check=True, capture_output=True)
        print(f"  #{num} → {' '.join(sorted(cibles))}")


def main():
    vivantes = issues_ouvertes()
    classees, erreurs = verifier(vivantes)
    if erreurs:
        print("\n\n".join(erreurs), file=sys.stderr)
        return 1
    if "--verifier" in sys.argv:
        print(f"Classement exhaustif : {len(classees)} issues, "
              f"{len(CAPACITES)} capacités, aucun trou.")
        return 0
    if "--appliquer" in sys.argv:
        appliquer(vivantes, classees)
        return 0
    print(livrable(vivantes, classees), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
