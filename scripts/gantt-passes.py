#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Diagramme de Gantt de la v0.1.0, en **passes d'agent**.

── Pourquoi ce script existe ──────────────────────────────────────────────

La persona chef de projet de la Méthode Foyer tient « un diagramme de Gantt —
axe relatif = **passes d'agent** ; il révèle parallélismes, points de concours
et de divergence ». L'axe n'est pas un calendrier : un agent ne travaille pas
en jours ouvrés, et dater un plan qu'on ne tiendra pas produit un document que
plus personne ne lit au troisième glissement.

Une **passe** est un tour de boucle supervisé : l'agent produit, le superviseur
relit, le tour est clos. Ce qui peut être mené de front dans une même passe est
borné par le `ratio_supervision` — le plafond du *répondre-de*, pas une variable
budgétaire.

── Ce qu'il ne prétend pas faire ─────────────────────────────────────────

Il ne prédit pas une date. Il ordonne. Les dépendances sont **déclarées ici** à
partir de ce que les stories disent d'elles-mêmes ; elles ne sont pas déduites
du code. Une dépendance manquante ici est un parallélisme optimiste dans le
résultat — le graphe est donc une **hypothèse falsifiable**, à corriger dès
qu'une passe réelle la contredit.

Usage :
    python3 scripts/gantt-passes.py            # le livrable sur stdout
    python3 scripts/gantt-passes.py --ecrire   # ... ou dans docs/
        # NE PAS faire `... > docs/GANTT...md` : le shell tronque le fichier
        # AVANT que le script le lise, donc la signature serait perdue.
    python3 scripts/gantt-passes.py --verifier # cohérence du graphe seulement
    python3 scripts/gantt-passes.py --creneaux-json [vague] [creneau]
        # le plan en JSON, pour que la CI lise CETTE source de vérité
        # plutôt que d'en tenir une seconde qui dérivera.
"""
import importlib.util
import json
import os
import sys

DEPOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SORTIE = os.path.join(DEPOT, "docs", "GANTT_PASSES_v0_1_0.md")

# Plafond du *répondre-de* : combien de chantiers un binôme tient de front sans
# cesser de pouvoir répondre de ce qui est produit. `[caler]` — prior à
# remplacer par du mesuré dès qu'on a tourné quelques passes réelles.
RATIO_SUPERVISION = 3

# ── Le graphe de dépendances ──────────────────────────────────────────────
#
# Chaque entrée vient de ce que la story de l'issue DIT, pas d'une intuition.
# La colonne de droite cite la raison ; sans raison écrite, pas de dépendance.
DEPS = {
    # Rang 0 — LA STORY HABILITANTE (Sprint 0). Elle livre « la capacité de
    # boucler ». La Méthode Foyer : « sans elle, aucune autre story ne peut
    # boucler » ; sur un projet existant, elle « bloque le reste du backlog
    # tant qu'elle n'est pas fermée ». D'où la BARRIÈRE ci-dessous.
    873: ([], "la vitrine devient un artefact de branche"),
    876: ([873], "un artefact ne suffit pas : sans narration ni chapitres, "
                 "la preuve n'est pas lisible"),
    874: ([876], "prouver le fan-out suppose une preuve ARBITRABLE, "
                 "pas seulement téléchargeable"),

    # Rang 1 — le harnais. Rien ne se DÉCLARE tenu avant lui.
    872: ([], "pile de recette jetable — ADR 0050"),
    870: ([], "mot de passe de recette découplé"),
    832: ([872], "rejouer les 15 specs exige une pile rejouable"),
    696: ([872], "idem — instabilité smoke"),

    # Rang 2 — identité et périmètre.
    855: ([], "identité notaire — ADR 0051"),
    845: ([855], "la dernière route nue est celle du notaire"),
    864: ([], "le cliquet AVANT la correction"),
    694: ([], "accès ACP — ADR 0053, refus par défaut"),
    798: ([], "#772 fermée : la précondition est levée"),
    841: ([], "réhydratation du périmètre"),
    868: ([], "un ancrage, un composant"),
    842: ([], "effacement RGPD"),

    # Rang 3 — arbitrage tranché.
    856: ([], "ADR 0052 — la maquette cède au test"),

    # Rang 4 — contrat de tests et socle visuel.
    803: ([], "ancrages : condition pratique de tout le frontend"),
    802: ([], "adapter les tests sans supprimer leurs règles"),
    797: ([], "jetons de design et jeu d'icônes"),
    834: ([], "cliquet i18n"),

    # Rang 5 — le noyau légal.
    780: ([872], "le cycle AG ne se vérifie que sur une pile rejouable"),
    848: ([], "levée de la suspension Art. 3.87 §1er"),
    850: ([], "le vote accepte l'identité depuis la requête"),
    576: ([], "quorum agrégé Decimal"),
    577: ([850], "attester fortement une identité déclarée ne protège rien"),
    581: ([576], "dépend de Story 4.1"),
    847: ([], "le registre atteste ce que ses tests ne vérifient pas"),
    846: ([694], "le sort de user_building_access se coordonne avec #694"),

    # Rang 6 — les Should.
    852: ([], "contrat : unit_id optionnel et refusé"),
    867: ([], "charges ventilées par ACP"),
    781: ([], "message de refus des modules communautaires"),
    779: ([798], "le porteur de périmètre est ce que #798 déplace"),
    835: ([], "un seul système de lien magique"),
    762: ([], "erreurs typées plutôt que sous-chaînes"),
    555: ([762], "l'épopée suit sa moitié bornée"),
    865: ([803], "auditer un écran authentifié suppose de l'atteindre"),
    592: ([803, 865], "le gate automatise ce que #865 définit"),
    869: ([], "un projet Playwright à largeur de téléphone"),
    866: ([869], "son @negative ne peut pas s'exécuter sans #869"),
    871: ([], "quatre défauts d'affichage"),
    731: ([], "collision d'alias DNS"),
    718: ([872], "ne pas conclure avant d'avoir rejoué au propre"),
    515: ([], "déploiement GitOps sur cluster vierge"),
    453: ([], "TLS non-prod par DNS-01"),
    466: ([515], "vraisemblablement l'un des cinq écarts de #515"),
    432: ([], "14 vulnérabilités Dependabot"),
    578: ([579, 780], "le PV suppose l'adaptateur ET une AG qui aboutit"),
    579: ([], "port de signature électronique"),
    427: ([872], "mesurer la taxonomie sur le code exige de pouvoir l'exécuter"),

    # Rang 7 — les Could.
    582: ([780], "l'élection du conseil suppose une AG qui aboutit"),
    583: ([579], "dépend de l'adaptateur de signature"),
    635: ([], "fonds affectés — ADR à écrire d'abord"),
    585: ([], "registre de modules + middleware"),
    586: ([585], "la garde d'interface suit la garde serveur"),
    587: ([585], "RBAC Communauté sur le registre de modules"),
    588: ([587], "l'exception syndic suit la règle"),
    589: ([587], "idem"),
    590: ([585], "audit et archivage du cycle"),
    591: ([585, 586], "l'assistant suppose registre et garde"),
    556: ([], "suivi d'épopée"),
    854: ([], "ranger docs/ selon le RFC 0003"),
    595: ([], "journal d'activité Tier 2"),
    354: ([], "tests IaC"),
    355: ([354], "restructurer sans filet, c'est déplacer à l'aveugle"),
    425: ([], "méta — garde-fous"),
    429: ([], "méta — ops runtime"),
    805: ([], "le cadre des parcours"),
    813: ([806, 807, 808, 809, 815, 816, 810, 811, 812, 817],
          "ranger selon une structure qui n'existe pas encore serait vain"),
}
# Les neuf maquettes : même dépendance, écrite une fois.
for _m in (818, 820, 821, 822, 823, 824, 825, 826, 827):
    DEPS[_m] = ([797, 802, 803],
                "refondre un écran sans jetons, ancrages ni tests adaptés")
# Les six parcours par persona et les quatre workflows.
for _p in (807, 808, 809, 816, 811, 812):
    DEPS[_p] = ([805], "suit le cadre commun")
DEPS[806] = ([805, 780], "le parcours du syndic passe par le cycle AG")
DEPS[810] = ([805, 780], "le workflow AG ne va pas à son terme aujourd'hui")
DEPS[815] = ([805, 835], "le prestataire reçoit deux liens pour un chantier")
DEPS[817] = ([805, 835], "idem — le ticket va jusqu'au prestataire")

RANGS = {
    0: ["C7.3"],
    1: ["C7.1"], 2: ["C4.1", "C4.2", "C4.3"], 3: ["C10.1"],
    4: ["C5.2", "C5.1"], 5: ["C1.1", "C1.3"],
    6: ["C1.2", "C1.5", "C2.1", "C2.2", "C3.1", "C4.4", "C4.5",
        "C6.1", "C6.2", "C7.2", "C9.1", "C9.3"],
    7: ["C1.4", "C2.3", "C3.2", "C3.3", "C5.3", "C8.1", "C8.2",
        "C8.3", "C8.4", "C9.2", "C9.4"],
}


NON_SIGNE = [
    "signature_humaine:",
    "  date: null",
    "  nom: null",
    "  role: null",
    "  etat: NON SIGNÉ — en attente de validation du superviseur",
]


def signature_existante():
    """Relit le bloc `signature_humaine` du livrable déjà écrit.

    Même raison que pour `backlog-structure.py` : la signature est le seul
    élément que le script n'a pas produit. L'écraser à chaque génération
    obligerait le superviseur à re-valider un plan qu'il a déjà validé.
    """
    try:
        with open(SORTIE, encoding="utf-8") as f:
            lignes = f.read().split("\n")
    except OSError:
        return NON_SIGNE
    if not lignes or lignes[0].strip() != "---":
        return NON_SIGNE
    bloc, dedans = [], False
    for ligne in lignes[1:]:
        if ligne.strip() == "---":
            break
        if ligne.startswith("signature_humaine:"):
            dedans = True
            bloc.append(ligne)
        elif dedans and ligne.startswith("  "):
            bloc.append(ligne)
        elif dedans:
            break
    return bloc if bloc else NON_SIGNE


# ── Domaines d'écriture — la vraie contrainte du parallélisme ─────────────
#
# Deux agents qui écrivent dans le même domaine entrent en conflit de fusion.
# Le parallélisme utile n'est donc pas « combien d'agents » mais « combien de
# domaines disjoints ». Dérivé de l'épopée, avec les exceptions qui comptent.
DOMAINES_EPOPEE = {
    "E1": "back/copropriete", "E2": "back/comptabilite",
    "E3": "back/communaute", "E4": "back/plateforme",
    "T1": "front/composants", "T2": "front/mobile-a11y",
    "T3": "harnais", "T4": "docs-vivante", "T5": "iac", "T6": "front/composants",
}
# Exceptions : ce que l'épopée ne dit pas bien.
DOMAINES_ISSUE = {
    841: "front/composants", 868: "front/composants", 842: "front/composants",
    798: "front/composants", 867: "front/composants", 871: "front/mobile-a11y",
    781: "back/communaute", 427: "harnais", 432: "iac",
    873: "harnais", 874: "harnais", 876: "harnais",
    854: "docs-vivante", 595: "docs-vivante", 425: "meta", 429: "meta",
    556: "meta",
}

# ── L'hôte, mesuré le 2026-09-12 sur `ecosolva` ──────────────────────────
#
# `[mesuré]`, pas supposé. À refaire sur toute machine qui orchestrerait.
HOTE = {
    "nom": "ecosolva",
    "cpu": 4,
    "charge_1min": 1.97,          # /proc/loadavg — déjà consommée
    "ram_go": 14,
    "ram_dispo_go": 10,
    "disque_libre_go": 16,
    "cout_worktree_mo": 600,      # mesuré sur le worktree kg-661
    "conteneurs": 31,
    "projets_heberges": 10,
    "target_rust": "volume docker PARTAGÉ (rustbuild-target-koprogo)",
}

# Claude Code plafonne les agents concurrents à min(16, CPU - 2).
CONCURRENCE_HOTE = min(16, HOTE["cpu"] - 2)


def domaine(num, meta):
    return DOMAINES_ISSUE.get(num, DOMAINES_EPOPEE.get(meta[num]["epopee"],
                                                       "divers"))


def levier(meta):
    """Combien de chantiers chaque issue débloque, transitivement.

    En régime séquentiel, l'ordre suit le rang : c'est la priorité métier.
    En régime parallèle, ce qui minimise le temps total est de sortir d'abord
    ce qui **tient une file** — sinon dix agents attendent qu'un seul finisse.
    Le levier ordonne donc À L'INTÉRIEUR d'un rang, jamais contre lui.
    """
    aval = {}
    for n, (ds, _) in DEPS.items():
        for d in ds:
            aval.setdefault(d, set()).add(n)

    memo = {}

    def compte(n, vus):
        if n in memo:
            return memo[n]
        if n in vus:
            return set()
        vus = vus | {n}
        acc = set()
        for x in aval.get(n, ()):
            acc.add(x)
            acc |= compte(x, vus)
        memo[n] = acc
        return acc

    return {n: len(compte(n, set())) for n in meta}


def orchestrer_multiagent(passe_logique, meta):
    """Le plan d'orchestration multiagent.

    Le *répondre-de* ne se joue plus sur le nombre d'agents tenus de front,
    mais sur la **revue de promotion de branche**, instruite par les gates et
    la vitrine. Le plafond de supervision cesse donc de brider l'éventail.

    Ce qui le bride encore, et qui est physique :

    1. **Les dépendances** — une vague ne s'ouvre qu'une fois l'amont fusionné.
    2. **Les conflits d'écriture** — deux agents dans le même domaine se
       marchent dessus. Un agent par domaine et par créneau.
    3. **La concurrence de l'hôte** — `min(16, CPU-2)`, mesurée.

    Rend une liste de vagues ; chaque vague est une liste de créneaux ; chaque
    créneau est une liste d'issues sans conflit entre elles.
    """
    vagues = {}
    for num, p in passe_logique.items():
        vagues.setdefault(p, []).append(num)

    rang_de = {c: r for r, cs in RANGS.items() for c in cs}
    poids_mos = {"Must": 0, "Should": 1, "Could": 2}
    lev = levier(meta)
    plan = []
    for v in sorted(vagues):
        restant = sorted(vagues[v],
                         key=lambda n: (rang_de.get(meta[n]["cap"], 9),
                                        -lev[n],
                                        poids_mos.get(meta[n]["moscow"], 3),
                                        -meta[n]["jours"], n))
        creneaux = []
        while restant:
            pris, vus = [], set()
            for n in list(restant):
                d = domaine(n, meta)
                if d in vus:
                    continue
                vus.add(d)
                pris.append(n)
                restant.remove(n)
            creneaux.append(pris)
        plan.append((v, creneaux))
    return plan


def charger_backlog():
    spec = importlib.util.spec_from_file_location(
        "bs", os.path.join(DEPOT, "scripts", "backlog-structure.py"))
    mod = importlib.util.module_from_spec(spec)
    argv = sys.argv
    sys.argv = ["backlog-structure", "--noop"]
    try:
        spec.loader.exec_module(mod)
    except SystemExit:
        pass
    finally:
        sys.argv = argv
    return mod


def calculer(mod):
    """Rend (passe_par_issue, meta_par_issue, alertes)."""
    meta, alertes = {}, []
    for cid, eid, titre, moscow, issues in mod.CAPACITES:
        for num, taille in issues.items():
            meta[num] = dict(cap=cid, epopee=eid, moscow=moscow,
                             taille=taille, jours=mod.JOURS[taille],
                             tours=mod.TOURS[taille])

    inconnues = sorted(set(DEPS) - set(meta))
    if inconnues:
        alertes.append("Issues déclarées au graphe et absentes du backlog : "
                       + ", ".join(f"#{n}" for n in inconnues))
    sans_deps = sorted(set(meta) - set(DEPS))
    if sans_deps:
        alertes.append("Issues du backlog absentes du graphe (traitées sans "
                       "dépendance) : " + ", ".join(f"#{n}" for n in sans_deps))

    # Couches topologiques : la passe au plus tôt.
    #
    # AVEC UNE BARRIÈRE. Les issues de C7.3 sont la story habilitante : elles
    # ne sont pas « une dépendance de plus », elles précèdent TOUT. L'encoder
    # par 84 arêtes vers #874 serait exact et illisible ; on le pose comme une
    # barrière explicite, qui est ce que la méthode décrit.
    habilitantes = {n for n in meta if meta[n]["cap"] in RANGS.get(0, [])}

    def couches(sous_ensemble, depart):
        res, restant = {}, {n: list(DEPS.get(n, ([], ""))[0])
                            for n in sous_ensemble}
        p = depart - 1
        while restant:
            p += 1
            pret = [n for n, d in restant.items()
                    if all(x in res or x not in restant for x in d)]
            if not pret:
                alertes.append("CYCLE dans le graphe : " +
                               ", ".join(f"#{n}" for n in sorted(restant)))
                break
            for n in pret:
                res[n] = p
                del restant[n]
        return res

    passe = couches(habilitantes, 1)
    barriere = max(passe.values(), default=0)
    passe.update(couches(set(meta) - habilitantes, barriere + 1))
    return passe, meta, alertes


def ordonnancer(passe_logique, meta):
    """Le plan RÉEL : dépendances **et** plafond de supervision.

    La couche topologique dit ce qui est *possible* ; elle place 39 chantiers
    en passe 1, ce qu'aucun binôme ne tient. Cette fonction dit ce qui est
    *supervisable* : à chaque passe on prend au plus `RATIO_SUPERVISION`
    chantiers dont les dépendances sont closes, par ordre de rang puis de
    MoSCoW. C'est ce plan-là qui se pilote.
    """
    rang_de = {c: r for r, cs in RANGS.items() for c in cs}
    poids_mos = {"Must": 0, "Should": 1, "Could": 2}
    restant = set(meta)
    faites, plan, p = set(), [], 0
    while restant:
        p += 1
        pret = sorted(
            (n for n in restant
             if all(d in faites or d not in meta
                    for d in DEPS.get(n, ([], ""))[0])),
            key=lambda n: (rang_de.get(meta[n]["cap"], 9),
                           poids_mos.get(meta[n]["moscow"], 3),
                           -meta[n]["jours"], n))
        if not pret:
            break
        lot = pret[:RATIO_SUPERVISION]
        plan.append((p, lot))
        faites.update(lot)
        restant.difference_update(lot)
    return plan


def barre(debut, fin, largeur):
    return "".join("█" if debut <= i <= fin else "·"
                   for i in range(1, largeur + 1))


def livrable(mod):
    passe, meta, alertes = calculer(mod)
    largeur = max(passe.values())

    par_cap = {}
    for num, p in passe.items():
        par_cap.setdefault(meta[num]["cap"], []).append((p, num))

    rang_de = {c: r for r, cs in RANGS.items() for c in cs}
    caps = {c[0]: (c[2], c[3]) for c in mod.CAPACITES}

    o = []
    w = o.append

    w("---")
    w("livrable: Gantt en passes d'agent (pilotage — persona chef de projet)")
    w("projet: KoproGo")
    w("jalon: v0.1.0")
    w("genere_par: scripts/gantt-passes.py")
    for ligne in signature_existante():
        w(ligne)
    w("---")
    w("")
    w("# Gantt de la v0.1.0 — en passes d'agent")
    w("")
    w("*Généré par `scripts/gantt-passes.py` : ne pas éditer à la main, la")
    w("prochaine génération écraserait la correction.*")
    w("")
    w("> **L'axe n'est pas un calendrier.** Une *passe d'agent* est un tour de")
    w("> boucle supervisé : l'agent produit, le superviseur relit, le tour est")
    w("> clos. Dater un plan qu'on ne tiendra pas produit un document que plus")
    w("> personne ne lit au troisième glissement ; ordonner sans dater reste")
    w("> vrai plus longtemps.")
    w("")

    # ── Méthode ──
    w("## Ce que le diagramme dit, et ce qu'il ne dit pas")
    w("")
    w("Il **ordonne**, il ne **prédit** pas. Les dépendances sont déclarées")
    w("dans `scripts/gantt-passes.py`, chacune avec sa raison écrite, à partir")
    w("de ce que la story de l'issue dit d'elle-même — jamais déduites du")
    w("code. Une dépendance oubliée ici devient un parallélisme optimiste dans")
    w("le résultat : le graphe est une **hypothèse falsifiable**, à corriger")
    w("dès qu'une passe réelle la contredit.")
    w("")
    w(f"Le plafond de parallélisme retenu est **{RATIO_SUPERVISION} chantiers")
    w("de front** par binôme — le `ratio_supervision` de l'abaque. Ce n'est pas")
    w("une variable budgétaire mais le plafond du *répondre-de* : au-delà, on")
    w("ne peut plus répondre de ce qui est produit. C'est un **prior**")
    w("`[caler]`, à remplacer par du mesuré dès qu'on aura tourné quelques")
    w("passes réelles.")
    w("")

    # ── Le Gantt logique ──
    w("## Le diagramme — ce qui est *possible*")
    w("")
    w(f"Colonnes : couches 1 à {largeur}. `█` = la capacité est ouvrable,")
    w("`·` = elle attend.")
    w("")
    w("```text")
    entete = "".join(str(i % 10) for i in range(1, largeur + 1))
    w(f"{'capacité':9} {'rang':>4} {'moscow':7} {entete}")
    w(f"{'-'*9} {'-'*4} {'-'*7} {'-'*largeur}")
    for cid in sorted(par_cap, key=lambda c: (rang_de.get(c, 9),
                                              min(p for p, _ in par_cap[c]),
                                              c)):
        ps = [p for p, _ in par_cap[cid]]
        w(f"{cid:9} {rang_de.get(cid,'—'):>4} "
          f"{caps[cid][1]:7} {barre(min(ps), max(ps), largeur)}")
    w("```")
    w("")

    # ── Points de concours et de divergence ──
    w("## Points de concours et de divergence")
    w("")
    bloquants = {}
    for n, (ds, _) in DEPS.items():
        for d in ds:
            bloquants.setdefault(d, []).append(n)
    majeurs = sorted(bloquants.items(), key=lambda kv: -len(kv[1]))[:6]
    w("**Divergence** — ce qui débloque le plus de chantiers en aval. Ce sont")
    w("les issues à ne pas laisser traîner : chacune tient une file.")
    w("")
    w("| Issue | Capacité | Couche | Chantiers débloqués |")
    w("|---|---|---:|---:|")
    for num, avals in majeurs:
        if num not in meta:
            continue
        w(f"| `#{num}` | {meta[num]['cap']} | {passe.get(num,'?')} "
          f"| **{len(avals)}** |")
    w("")
    concours = {n: ds for n, (ds, _) in DEPS.items() if len(ds) > 1}
    w("**Concours** — ce qui attend plusieurs chemins. Ce sont les points où")
    w("un retard sur *n'importe laquelle* des amont décale l'aval.")
    w("")
    w("| Issue | Attend |")
    w("|---|---|")
    for n, ds in sorted(concours.items(), key=lambda kv: -len(kv[1]))[:6]:
        w(f"| `#{n}` | " + ", ".join(f"`#{d}`" for d in ds) + " |")
    w("")

    # ── Ce que la largeur révèle ──
    larges = [(p, len([n for n, q in passe.items() if q == p]))
              for p in range(1, largeur + 1)]
    saturees = [(p, l) for p, l in larges if l > RATIO_SUPERVISION]
    w("## Ce que la largeur révèle")
    w("")
    if saturees:
        w(f"**{len(saturees)} des {largeur} couches dépassent le plafond de "
          f"{RATIO_SUPERVISION}** : "
          + ", ".join(f"L{p} ({l} chantiers)" for p, l in saturees) + ".")
        w("")
        w("C'est le résultat le plus utile du diagramme, et il est")
        w("contre-intuitif : **les dépendances ne sont pas le goulot.** Les")
        w(f"chaînes sont courtes — {largeur} couches seulement — et l'essentiel")
        w("du travail est parallélisable. Ce qui borne la release n'est donc")
        w("pas l'ordre des choses, c'est la capacité à *répondre de* ce qui est")
        w("produit.")
        w("")
        w("Deux issues, et une seule est bonne :")
        w("")
        w("- **Étaler** — le plan sous contrainte ci-dessous. C'est le défaut,")
        w("  et il est honnête.")
        w("- **Ajouter un pair** — coût en marche d'escalier, pas en pente.")
        w("  C'est ce que l'abaque appelle un investissement wall-clock dans la")
        w("  transmissibilité, pas un surcoût à raboter.")
        w("")
        w("Ce qu'il ne faut **pas** faire est réduire le binôme à une personne")
        w("seule pour tenir la largeur : on gagne du wall-clock et on rachète")
        w("du *bus factor* 1.")
    else:
        w(f"Aucune couche ne dépasse le plafond de {RATIO_SUPERVISION}.")
    w("")

    # ── Le plan multiagent ──
    plan_seq = ordonnancer(passe, meta)
    orch = orchestrer_multiagent(passe, meta)
    n_creneaux = sum(len(cs) for _, cs in orch)
    pic_dom = max((len(c) for _, cs in orch for c in cs), default=0)

    w("## Le plan d'orchestration — multiagent, parallélisme maximal")
    w("")
    w("**Amendement du 2026-09-12.** Le `ratio_supervision` ne bride plus")
    w("l'éventail : le *répondre-de* est porté par la **revue de promotion de")
    w("branche**, instruite par les gates et la vitrine. On ne supervise plus")
    w("des agents en direct, on relit une preuve attachée à une branche.")
    w("")
    w("Ce qui bride encore, et qui est **physique** :")
    w("")
    w("0. **La story habilitante** — `C7.3` est une **barrière**, pas une")
    w("   dépendance parmi d'autres. Tant qu'elle n'est pas close, le fan-out")
    w("   n'a ni preuve à produire ni mécanisme prouvé. La Méthode Foyer :")
    w("   « elle bloque le reste du backlog tant qu'elle n'est pas fermée ».")
    w("1. **Les dépendances** — une vague ne s'ouvre qu'une fois l'amont")
    w("   fusionné.")
    w("2. **Les conflits d'écriture** — deux agents dans le même domaine se")
    w("   marchent dessus. Un agent par domaine et par créneau, chacun dans")
    w("   son *worktree*.")
    w(f"3. **La concurrence de l'hôte** — `min(16, CPU-2)` = "
      f"**{CONCURRENCE_HOTE}** sur cette machine. Mesurée, pas supposée.")
    w("")
    w(f"Résultat : **{len(orch)} vagues**, **{n_creneaux} créneaux**, largeur")
    w(f"maximale **{pic_dom} agents simultanés** — contre {len(plan_seq)}")
    w("passes en séquentiel supervisé.")
    w("")
    w("> ⚠️ **Le goulot n'est plus le plan, c'est l'hôte.** La largeur")
    w(f"> demandée est {pic_dom} ; la machine en tient {CONCURRENCE_HOTE}. Un")
    w("> créneau large s'exécutera donc en plusieurs vagues réelles, ou")
    w("> ailleurs — agents distants, ou hôte plus gros. C'est le premier")
    w("> chiffre à caler avant de lancer l'expérimentation.")
    w("")
    hab_caps = RANGS.get(0, [])
    for v, creneaux in orch:
        dedans = [n for c in creneaux for n in c]
        est_hab = all(meta[n]["cap"] in hab_caps for n in dedans)
        titre = (f"### Vague {v} — **habilitation**" if est_hab
                 else f"### Vague {v}")
        w(titre)
        w("")
        if est_hab:
            w("> **Exécutée en session, pas par le fan-out.** Les habilitantes")
            w("> sont toutes dans le domaine `harnais` : le fan-out les")
            w("> sérialiserait sans gain. Et c'est un œuf et une poule — la")
            w("> valeur du fan-out est que les gates et la vitrine instruisent")
            w("> la revue, et ce sont précisément eux qu'on construit ici.")
            w("")
        w("| Créneau | Agents | Domaines |")
        w("|---|---|---|")
        for k, lot in enumerate(creneaux, 1):
            doms = ", ".join(f"`{domaine(n, meta)}`" for n in lot)
            ags = ", ".join(f"#{n}" for n in lot)
            w(f"| V{v}.{k} | {len(lot)} — {ags} | {doms} |")
        w("")

    # ── Parallélisme maximal par rapport à l'hôte ──
    coeurs_libres = HOTE["cpu"] - HOTE["charge_1min"]
    par_disque = int(HOTE["disque_libre_go"] * 1024 / HOTE["cout_worktree_mo"])
    cpu_pour_pic = pic_dom + 2
    creneaux_reels = -(-len(meta) // max(CONCURRENCE_HOTE, 1))

    w("## Parallélisme maximal — par rapport à l'hôte")
    w("")
    w(f"Mesuré le 2026-09-12 sur `{HOTE['nom']}`. Ce ne sont pas des ordres de")
    w("grandeur : ce sont les chiffres de la machine qui orchestrerait.")
    w("")
    w("| Ressource | Mesure | Agents qu'elle permet |")
    w("|---|---|---:|")
    w(f"| CPU | {HOTE['cpu']} cœurs, charge {HOTE['charge_1min']} "
      f"(~{coeurs_libres:.1f} libres) | **{CONCURRENCE_HOTE}** |")
    w(f"| RAM | {HOTE['ram_dispo_go']} Go disponibles sur "
      f"{HOTE['ram_go']} | confortable |")
    w(f"| Disque | {HOTE['disque_libre_go']} Go libres, "
      f"{HOTE['cout_worktree_mo']} Mo par worktree | ~{par_disque} |")
    w(f"| Build Rust | {HOTE['target_rust']} | **1 à la fois** |")
    w("")
    w("**Le plafond est le CPU, et il vaut "
      f"{CONCURRENCE_HOTE}** : Claude Code borne les agents concurrents à")
    w(f"`min(16, CPU − 2)`, soit `min(16, {HOTE['cpu']} − 2)`. Le disque en")
    w(f"permettrait ~{par_disque}, la RAM aussi — ils ne servent à rien.")
    w("")
    w("Deux aggravations que la formule ne voit pas :")
    w("")
    w(f"- **L'hôte n'est pas dédié.** Il porte {HOTE['conteneurs']} conteneurs")
    w(f"  pour {HOTE['projets_heberges']} projets, dont la démo KoproGo. La")
    w(f"  charge est déjà à {HOTE['charge_1min']} sur {HOTE['cpu']} cœurs :")
    w("  la moitié de la machine est prise avant qu'un seul agent démarre.")
    w("- **Le `target` Rust est un volume Docker partagé.** Deux agents qui")
    w("  compilent en même temps se bloquent sur le verrou de `cargo`, quel")
    w("  que soit le nombre de worktrees. Le parallélisme backend est donc")
    w("  **de 1** tant que chaque agent n'a pas son propre `target`.")
    w("")
    w("### Le verdict, et il est inconfortable")
    w("")
    w(f"Le plan demande une largeur de **{pic_dom}**. L'hôte en tient")
    w(f"**{CONCURRENCE_HOTE}**. L'expérimentation s'exécuterait donc à **un")
    w(f"cinquième** de la largeur pour laquelle elle est conçue :")
    w(f"~{creneaux_reels} créneaux réels au lieu de {n_creneaux}.")
    w("")
    w(f"À {CONCURRENCE_HOTE} de front, le parallélisme n'apporte presque rien :")
    w(f"le gain vient alors de la **suppression de l'attente humaine entre")
    w("passes**, pas du parallélisme lui-même. C'est un vrai gain — mais ce")
    w("n'est pas l'expérience qu'on voulait mener.")
    w("")
    w("**Pour tenir la largeur demandée**, trois voies, par coût croissant :")
    w("")
    w("| Voie | Ce qu'il faut | Ce que ça coûte |")
    w("|---|---|---|")
    w(f"| Hôte plus gros | ≥ {cpu_pour_pic} cœurs (`min(16, n−2) ≥ "
      f"{pic_dom}`) | une machine |")
    w("| Agents distants | orchestration en nuage | facturation à l'usage |")
    w("| Fan-out en CI | un job par story | temps de CI, pas de worktree |")
    w("")
    w("La troisième est **retenue et implémentée** :")
    w("`.github/workflows/fanout-stories.yml`. Elle ne demande pas de machine,")
    w("isole naturellement les `target` Rust, et produit déjà les artefacts —")
    w("gates et vitrine — que la revue de promotion attend. Le parallélisme y")
    w("est borné par les *runners*, pas par cet hôte.")
    w("")
    w("### Ce que le fan-out coûte, et c'est un choix")
    w("")
    w("Deux voies d'authentification, et **aucune ne donne du parallélisme")
    w("gratuit** :")
    w("")
    w("| Voie | Facturation | Ce qui borne |")
    w("|---|---|---|")
    w("| `CLAUDE_CODE_OAUTH_TOKEN` *(défaut)* | l'abonnement | les limites de débit, **partagées avec les sessions interactives** |")
    w("| `ANTHROPIC_API_KEY` | à l'usage | le budget |")
    w("")
    w("Ordre de grandeur pour les 84 stories, reprises comprises — hypothèses")
    w("visibles : ~5 M tokens d'entrée par story dont ~90 % en lecture de")
    w("cache, ~80 k en sortie. **À caler sur la première vague réelle.**")
    w("")
    w("| Modèle | par story | 84 stories | avec reprises (×1,5) |")
    w("|---|---:|---:|---:|")
    w("| Haiku 4.5 | ~1,4 $ | ~115 $ | **~170 $** |")
    w("| Sonnet 5 | ~2,7 $ | ~227 $ | **~340 $** |")
    w("| Opus 5 | ~6,8 $ | ~567 $ | **~850 $** |")
    w("")
    w("Le jeton d'abonnement évite la facture mais pas la contrainte : un")
    w("fan-out large consomme les limites de débit et **ralentit le travail")
    w("humain en cours**. Le défaut de `max_parallel` est donc **2**, à monter")
    w("une fois la première vague mesurée — pas avant.")
    w("")

    # ── Le protocole de promotion ──
    w("## Le protocole — une branche, une preuve, une revue")
    w("")
    w("Chaque agent travaille dans un **worktree isolé** et livre une branche")
    w("`story/<issue>`. La promotion vers `feature/dev` est le gate, et elle")
    w("exige **trois preuves attachées à la branche** :")
    w("")
    w("| Preuve | Gate | Bloquant |")
    w("|---|---|---|")
    w("| Correctness | `e2e` sur la pile de recette | oui |")
    w("| Les quatre classes | `unit` + `integration` + `bdd` | oui |")
    w("| Valeur | **vitrine** — parcours filmé | non bloquant, **non facultatif** |")
    w("")
    w("Le parcours Foyer est explicite : la doc vivante est « une preuve, pas")
    w("un verrou » — mais une story full-stack sans sa preuve de valeur **n'est")
    w("pas terminée**. C'est elle qui rend la revue de promotion possible sans")
    w("relire le diff ligne à ligne : le relecteur regarde le film et les")
    w("gates, pas le code.")
    w("")

    # ── La précondition ──
    w("## ⚠️ La précondition — le filet avant le saut")
    w("")
    w("**Le mécanisme choisi pour porter le répondre-de n'est pas")
    w("opérationnel.** Au 2026-09-12 :")
    w("")
    w("| Gate | État | Cause |")
    w("|---|---|---|")
    w("| `e2e` | 🔴 | vise la démo via Traefik — #872 |")
    w("| `doc-vivante` (vitrine) | 🔴 | `make docs-with-videos` — #872 |")
    w("")
    w("Lancer 84 chantiers en parallèle avant que la preuve existe reviendrait")
    w("à produire 84 branches que **rien ne permet de relire**. Le parallélisme")
    w("n'est pas risqué en soi : il l'est quand son filet n'est pas tendu.")
    w("")
    w("**V1 n'est donc pas une formalité, c'est ce qui rend le reste**")
    w("**légitime.** L'ADR 0050 — décaler les ports, pile de recette jetable —")
    w("est la condition d'existence de l'expérimentation, pas sa première")
    w("étape parmi d'autres.")
    w("")

    # ── Coût ──
    tot_j = sum(meta[n]["jours"] for n in meta)
    tot_t = sum(meta[n]["tours"] for n in meta)
    w("## Coût")
    w("")
    w("| Axe | Valeur | Ce que ça mesure |")
    w("|---|---:|---|")
    w(f"| Issues | {len(meta)} | le périmètre, intégral (ADR 0049) |")
    w(f"| Passes séquentielles | {len(plan_seq)} | régime supervisé, 3 de front |")
    w(f"| Créneaux multiagent | {n_creneaux} | régime parallèle, revue à la promotion |")
    w(f"| Jours | {tot_j:.2f} | wall-clock **superviseur** |")
    w(f"| Tours | {tot_t} | coût **tokens** |")
    w("")
    w("L'abaque est formelle sur la lecture de ces deux dernières lignes : le")
    w("**poste dominant est le superviseur, pas le modèle**. Optimiser les")
    w("tokens ne déplace presque rien. Ce sont donc les jours qu'il faut")
    w("regarder, et le seul levier qui les réduise sans rien racheter est de")
    w("retirer du périmètre — ce que l'ADR 0049 a explicitement refusé de")
    w("faire.")
    w("")
    w("**Ces chiffres sont des bornes hautes de première passe.** Le CSI doit")
    w("les resserrer story après story sur le réel observé. Les publier non")
    w("resserrés est le seul moyen d'avoir un point de départ falsifiable ;")
    w("les publier comme un engagement serait une faute.")
    w("")

    # ── Cadre de delivery ──
    pic = max(l for _, l in larges)
    w("## Cadre de delivery")
    w("")
    w(f"Largeur maximale *possible* : **{pic} chantiers simultanés**. Largeur")
    w(f"*retenue* : **{RATIO_SUPERVISION}**.")
    w("")
    if RATIO_SUPERVISION <= 9:
        w("Cela tient dans **une seule équipe** : Scrum suffit, et Nexus serait")
        w("une cérémonie sans objet. Le passage à Nexus se justifierait à")
        w("partir de trois équipes sur le produit.")
        w("")
        w("Le rescaling du barreau (Scrum → Nexus → SAFe) est un **point")
        w("irréversible** que la persona chef de projet porte en ADR, validé")
        w("par l'humain. Rien ne l'appelle aujourd'hui : ce qui manque n'est")
        w("pas de la coordination inter-équipes, c'est de la capacité de")
        w("supervision dans une seule.")
    else:
        w("La largeur retenue appelle un examen du barreau (Scrum → Nexus).")
        w("C'est un point irréversible : ADR, validation humaine.")
    w("")

    # ── Ce qui n'est pas dans le Gantt ──
    w("## Ce que le Gantt ne couvre pas")
    w("")
    w("- **La qualité des stories.** 84/84 « Agent IA Ready » est un contrôle")
    w("  de forme. Le plan ordonne ce qui est écrit, pas ce qui est bon.")
    w("- **Les deux gates humains** du rang 8 — revue signée puis tag — qui")
    w("  sont hors périmètre agent par construction.")
    w("- **Le réordonnancement.** Le Gantt se réévalue à chaque jalon, à mesure")
    w("  que le parallélisme réel se découvre. Celui-ci est la première passe")
    w("  du plan, pas le plan définitif.")
    w("")

    if alertes:
        w("## Alertes du générateur")
        w("")
        for a in alertes:
            w(f"- {a}")
        w("")

    w("---")
    w("")
    w("*Dérivé du Manifeste Maury (CC BY-SA 4.0). Persona `chef-de-projet` de")
    w("la Méthode Foyer. Chiffrage : `skills/abaque-cout-capacite.md`.*")
    w("")
    return "\n".join(o)


def main():
    mod = charger_backlog()
    if "--creneaux-json" in sys.argv:
        passe, meta, _ = calculer(mod)
        orch = orchestrer_multiagent(passe, meta)
        reste = [a for a in sys.argv[sys.argv.index("--creneaux-json") + 1:]
                 if not a.startswith("-")]
        f_vague = int(reste[0]) if len(reste) > 0 else None
        f_cren = int(reste[1]) if len(reste) > 1 else None
        sortie = []
        for v, creneaux in orch:
            if f_vague is not None and v != f_vague:
                continue
            for k, lot in enumerate(creneaux, 1):
                if f_cren is not None and k != f_cren:
                    continue
                for num in lot:
                    sortie.append({
                        "issue": num,
                        "vague": v,
                        "creneau": k,
                        "cle": f"V{v}.{k}",
                        "capacite": meta[num]["cap"],
                        "domaine": domaine(num, meta),
                        "taille": meta[num]["taille"],
                        "branche": f"story/{num}",
                    })
        print(json.dumps(sortie, ensure_ascii=False))
        return 0
    if "--verifier" in sys.argv:
        passe, meta, alertes = calculer(mod)
        for a in alertes:
            print(a, file=sys.stderr)
        print(f"Graphe cohérent : {len(passe)} issues, "
              f"{max(passe.values())} passes.")
        return 1 if any("CYCLE" in a for a in alertes) else 0
    texte = livrable(mod)          # lit SORTIE (donc la signature) AVANT
    if "--ecrire" in sys.argv:     # d'écrire quoi que ce soit
        with open(SORTIE, "w", encoding="utf-8") as f:
            f.write(texte)
        print(f"écrit : {SORTIE}", file=sys.stderr)
        return 0
    print(texte, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
