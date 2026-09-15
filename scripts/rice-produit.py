#!/usr/bin/env python3
"""RICE — priorisation produit du backlog, à Reach et Impact SUBSTITUÉS.

── Pourquoi ce fichier ne s'appelle pas simplement « RICE » ────────────────

`skills/adoption.md` de la Méthode Foyer pose :

    RICE = Reach × Impact × Confidence ÷ Effort
      Confidence ← le CSI (mesure réelle, pas intuition)
      Effort     ← l'estimation en tours de la primitive

Et il place RICE en **phase 5 — pilotage continu**, dont le rôle est de
*rouvrir le point 0 de la capacité suivante*. C'est un instrument d'APRÈS
livraison, par construction : Reach et Impact se lisent sur l'usage.

KoproGo n'a aucun utilisateur. La v0.1.0 n'est jamais sortie, la bêta fermée
n'a pas commencé. Calculer un Reach aujourd'hui reviendrait à inventer un
nombre, le multiplier par un autre nombre inventé, et présenter le produit
comme une mesure — exactement ce que la méthode interdit quand elle écrit
« mesure réelle, pas intuition ».

Deux facteurs sont donc **substitués**, et le substitut est observable :

    Reach  ← chantiers débloqués (graphe de dépendances de gantt-passes.py)
    Impact ← palier légal (la règle d'entrée en 0.1.0 du WBS)

Les deux se recalculent depuis des artefacts qui existent. Aucun n'est une
intuition déguisée. Mais ce ne sont pas Reach et Impact au sens de RICE, et
ce fichier le répète partout où quelqu'un pourrait l'oublier.

── Quand les substitutions tombent ────────────────────────────────────────

À la bêta fermée (5 à 10 copropriétés), les signaux RACE existent. Reach
devient l'audience réellement touchée, Impact l'effet mesuré sur le parcours.
Ce script devra alors être RÉÉCRIT, pas ajusté : un substitut qu'on garde
« parce qu'il marchait bien » est la façon dont une mesure devient un rite.

── Ce que RICE ne fait PAS ici ────────────────────────────────────────────

L'ADR 0049 a tranché : **tout entre dans la 0.1.0**. RICE ne retire rien du
périmètre — il ORDONNE, au même titre que MoSCoW. Un score bas veut dire
« en dernier », jamais « hors release ».

Usage :
    python3 scripts/rice-produit.py              # le tableau, sur stdout
    python3 scripts/rice-produit.py --json       # les scores, en JSON
    python3 scripts/rice-produit.py --verifier   # les substitutions tiennent-elles

Dérivé du Manifeste Maury (CC BY-SA 4.0).
"""

import importlib.util
import json
import os
import re
import subprocess
import sys

DEPOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
REGISTRE_CSI = os.path.join(DEPOT, "docs", "telemetrie", "passes.jsonl")


# ── Impact — le palier légal, et il est écrit ailleurs que dans ma tête ────
#
# `docs/WBS_v0_1_0.md` pose la règle d'entrée en 0.1.0, opposable :
#
#   « Entre en 0.1.0 ce qui porte un invariant du registre RFC-0002 ou rend
#     le modèle irréversible. Reste en 0.2.0 ce qui est un mécanisme de mise
#     en œuvre. »
#
# Trois paliers, donc, et la détection est mécanique — donc auditable, et
# donc réfutable. Une issue qui devrait citer un article et n'en cite aucun
# est un défaut de story, pas un trou du barème : le `--verifier` le dit.
PALIERS = {
    3: "invariant légal — un article du Code civil",
    2: "irréversibilité du modèle de données",
    1: "mécanisme de mise en œuvre",
}

# Un article du Livre 3 : « Art. 3.87 § 2 », « Art. 3.89 § 5, 7° »…
ARTICLE = re.compile(r"\bArt(?:icle)?\.?\s*3\.\d{2}\b", re.I)

# Ce qui rend le modèle irréversible : le schéma, le périmètre, l'identité.
# Une fois qu'une vraie copropriété a encodé, ces choix ne se défont plus
# sans migration de données — c'est la définition retenue par le WBS.
IRREVERSIBLE = re.compile(
    r"\bmigration\b|\bschéma\b|\bmodèle de données\b|\bpérimètre\b"
    r"|\bscoping\b|\bACP\b|\bidentité\b|\bRGPD\b",
    re.I,
)


def charger_module(nom, fichier):
    """Charge un script voisin comme module, sans exécuter son `main`."""
    spec = importlib.util.spec_from_file_location(
        nom, os.path.join(DEPOT, "scripts", fichier))
    mod = importlib.util.module_from_spec(spec)
    argv = sys.argv
    sys.argv = [nom, "--noop"]
    try:
        spec.loader.exec_module(mod)
    except SystemExit:
        pass
    finally:
        sys.argv = argv
    return mod


def issues_github():
    """Titre et corps de chaque issue ouverte du jalon, pour le palier."""
    brut = subprocess.run(
        ["gh", "issue", "list", "--state", "open", "--limit", "300",
         "--label", "release:0.1.0", "--json", "number,title,body,labels"],
        capture_output=True, text=True, cwd=DEPOT)
    if brut.returncode != 0:
        return {}
    return {i["number"]: i for i in json.loads(brut.stdout)}


def palier(num, fiches):
    """Le palier légal d'une issue, et la raison qui l'a décidé."""
    f = fiches.get(num)
    if not f:
        return 1, "issue introuvable — palier plancher par défaut"

    texte = f"{f.get('title', '')}\n{f.get('body', '') or ''}"
    etiquettes = {e["name"] for e in f.get("labels", [])}

    m = ARTICLE.search(texte)
    if m:
        return 3, f"cite {m.group(0)}"
    if "legal-compliance" in etiquettes:
        return 3, "étiquette legal-compliance, sans article cité"
    m = IRREVERSIBLE.search(texte)
    if m:
        return 2, f"touche « {m.group(0).lower()} »"
    return 1, "aucun article, aucun marqueur d'irréversibilité"


def csi():
    """Ce que la télémétrie mesure vraiment, par taille estimée.

    Rend `(tours_par_taille, nombre_de_mesures_utilisables)`. Une mesure
    n'est utilisable que si `tours_mesures` n'est pas `null` : l'absence de
    mesure ne se moyenne pas avec des mesures.
    """
    par_taille, utilisables = {}, 0
    if not os.path.exists(REGISTRE_CSI):
        return {}, 0
    for ligne in open(REGISTRE_CSI, encoding="utf-8"):
        ligne = ligne.strip()
        if not ligne:
            continue
        try:
            d = json.loads(ligne)
        except ValueError:
            continue
        t, tours = d.get("taille_estimee"), d.get("tours_mesures")
        if t and isinstance(tours, int):
            par_taille.setdefault(t, []).append(tours)
            utilisables += 1
    return ({t: sum(v) / len(v) for t, v in par_taille.items()}, utilisables)


def confiance(taille, mesures, n_utilisables):
    """La confiance, bornée par ce qu'on a mesuré — pas par notre humeur.

    Le CSI de Foyer dit : « le point 0 est un prior à certitude basse ;
    chaque story close est un test qui le confirme ou le réfute ». Tant
    qu'aucune story n'est mesurée, la confiance reste celle du prior, et
    l'écrire plus haut serait mentir sur la seule chose que RICE apporte de
    rigoureux.
    """
    n = len(mesures.get(taille, [])) if isinstance(mesures.get(taille), list) \
        else (1 if taille in mesures else 0)
    if n_utilisables == 0:
        return 0.20, "aucune passe mesurée — confiance du prior"
    if n == 0:
        return 0.35, f"{n_utilisables} passe(s) mesurée(s), aucune en {taille}"
    if n < 3:
        return 0.60, f"{n} passe(s) mesurée(s) en {taille}"
    return 0.85, f"{n} passes mesurées en {taille}"


def calculer():
    bs = charger_module("bs", "backlog-structure.py")
    gp = charger_module("gp", "gantt-passes.py")

    _, meta, _ = gp.calculer(bs)
    leviers = gp.levier(meta)
    fiches = issues_github()
    tours_mesures, n_utilisables = csi()

    lignes = []
    for num, m in meta.items():
        taille = m["taille"]
        imp, raison_imp = palier(num, fiches)
        conf, raison_conf = confiance(taille, tours_mesures, n_utilisables)

        if taille in tours_mesures:
            effort, source_effort = tours_mesures[taille], "mesuré (CSI)"
        else:
            effort, source_effort = bs.TOURS[taille], "prior (abaque)"

        # Reach à 0 ne doit pas annuler le score : une issue qui ne débloque
        # rien reste à faire. Le +1 dit « elle vaut pour elle-même ».
        reach = leviers.get(num, 0)
        score = (reach + 1) * imp * conf / effort

        lignes.append({
            "issue": num,
            "capacite": m["cap"],
            "moscow": m["moscow"],
            "taille": taille,
            "reach_substitue": reach,
            "impact_substitue": imp,
            "impact_raison": raison_imp,
            "confiance": round(conf, 2),
            "confiance_raison": raison_conf,
            "effort_tours": round(effort, 2),
            "effort_source": source_effort,
            "score": round(score, 3),
        })

    lignes.sort(key=lambda x: (-x["score"], x["issue"]))
    return lignes, n_utilisables, fiches


def verifier(lignes, n_utilisables, fiches):
    """Les substitutions tiennent-elles encore ?

    Sort 1 dès qu'une hypothèse du barème ne tient plus. Un score de
    priorisation qui se calcule sur des fondations tombées est pire qu'un
    backlog non priorisé : il porte l'autorité d'un chiffre.
    """
    fautes = []

    if not fiches:
        fautes.append(
            "Aucune issue lue depuis GitHub. Le palier légal retombe alors "
            "sur son plancher pour TOUTES les issues, et l'Impact devient "
            "une constante — le score ne distingue plus rien.")

    plancher = [l for l in lignes if l["impact_substitue"] == 1]
    if lignes and len(plancher) == len(lignes):
        fautes.append(
            "Toutes les issues sont au palier 1. Soit la détection est "
            "cassée, soit le backlog ne porte plus aucun invariant légal — "
            "et dans les deux cas, l'Impact ne mesure plus rien.")

    if n_utilisables == 0:
        print(
            "⚠️  Aucune passe mesurée : Effort vient du prior et Confiance "
            "est plafonnée à 0,20.\n"
            "   Le classement vaut comme ORDRE, pas comme mesure. Il se "
            "resserrera passe après passe (#875).",
            file=sys.stderr)

    # Un facteur constant ne hiérarchise rien. Le dire, plutôt que de laisser
    # croire que quatre facteurs travaillent quand trois seulement le font.
    confs = {l["confiance"] for l in lignes}
    if len(confs) == 1:
        print(
            f"⚠️  Confiance identique ({confs.pop()}) pour les 88 issues : "
            "elle ne HIÉRARCHISE rien aujourd'hui.\n"
            "   Le score se réduit donc à (Reach*+1) × Impact* ÷ Effort. "
            "C'est trois facteurs sur quatre, et c'est normal tant que le "
            "registre CSI est vide.",
            file=sys.stderr)

    # La limite structurelle du barème, et elle a un exemple vivant.
    print(
        "ℹ️  Ce barème pèse le POIDS LÉGAL et le LEVIER DE DÉPENDANCE. Il ne "
        "voit pas « bloque tout le monde opérationnellement » :\n"
        "   #872 (la recette, rang 1 du registre) sort à un score modeste "
        "parce qu'aucun article ne la concerne.\n"
        "   RICE COMPLÈTE le rang du Gantt, il ne le remplace pas. Les lire "
        "l'un contre l'autre serait un contresens.",
        file=sys.stderr)

    # Le rappel qui empêche le substitut de devenir un rite.
    print(
        "ℹ️  Reach et Impact sont SUBSTITUÉS faute d'usage réel. À la bêta "
        "fermée, ce script se réécrit — il ne s'ajuste pas.",
        file=sys.stderr)

    for f in fautes:
        print(f"::error::{f}", file=sys.stderr)
    return 1 if fautes else 0


def tableau(lignes, n_utilisables):
    out = []
    w = out.append
    w("# RICE produit — v0.1.0, à Reach et Impact substitués")
    w("")
    w("*Généré par `scripts/rice-produit.py`. Ne pas éditer à la main.*")
    w("")
    w("> **Deux facteurs sur quatre sont des substituts.** KoproGo n'a aucun")
    w("> utilisateur : la v0.1.0 n'est pas sortie. `Reach` est donc le nombre")
    w("> de chantiers qu'une issue débloque, et `Impact` son palier légal.")
    w("> Ce ne sont pas Reach et Impact au sens de RICE, et les appeler ainsi")
    w("> sans le dire reviendrait à habiller de l'intuition en arithmétique.")
    w(">")
    w("> À la bêta fermée, les signaux RACE existeront et ce script devra")
    w("> être **réécrit**, pas ajusté.")
    w("")
    w(f"**Confiance** : {n_utilisables} passe(s) réellement mesurée(s) au ")
    w("registre CSI. Tant que ce nombre est bas, le classement vaut comme")
    w("**ordre**, pas comme mesure.")
    w("")
    w("**L'ADR 0049 tient** : tout reste dans la 0.1.0. Un score bas dit")
    w("« en dernier », jamais « hors release ».")
    w("")
    # ── Là où les deux ordres se contredisent ──────────────────────────
    #
    # C'est le seul endroit où ce tableau sert à quelque chose. Un
    # classement qui confirmerait MoSCoW ligne pour ligne n'apprendrait
    # rien ; ce sont les DÉSACCORDS qui demandent un arbitrage.
    rangs = {l["issue"]: i for i, l in enumerate(lignes)}
    hauts_could = [l for l in lignes[:15] if l["moscow"] == "Could"]
    bas_must = [l for l in lignes[len(lignes) // 2:] if l["moscow"] == "Must"]

    if hauts_could or bas_must:
        w("## Là où RICE contredit MoSCoW")
        w("")
        w("Ces lignes ne tranchent rien : elles demandent un arbitrage. Un")
        w("classement qui confirmerait MoSCoW partout n'apprendrait rien.")
        w("")
        if hauts_could:
            w("**Classées `Could`, et pourtant dans les quinze premières** —")
            w("elles débloquent beaucoup, ou portent un article :")
            w("")
            for l in hauts_could:
                w(f"- **#{l['issue']}** (rang {rangs[l['issue']] + 1}, score "
                  f"{l['score']}) — débloque {l['reach_substitue']} chantiers, "
                  f"palier {l['impact_substitue']} : {l['impact_raison']}")
            w("")
        if bas_must:
            w("**Classées `Must`, et dans la moitié basse** — à vérifier : un")
            w("`Must` qui ne débloque rien et ne porte aucun article est")
            w("peut-être un `Should` qui s'ignore :")
            w("")
            for l in bas_must[:8]:
                w(f"- **#{l['issue']}** (rang {rangs[l['issue']] + 1}, score "
                  f"{l['score']}) — {l['impact_raison']}")
            if len(bas_must) > 8:
                w(f"- … et {len(bas_must) - 8} autres")
            w("")

    w("## Le classement")
    w("")
    w("| # | Score | Cap. | MoSCoW | Reach* | Impact* | Conf. | Effort | Pourquoi ce palier |")
    w("|---|---:|---|---|---:|---:|---:|---:|---|")
    for l in lignes:
        w(f"| #{l['issue']} | **{l['score']}** | {l['capacite']} | "
          f"{l['moscow']} | {l['reach_substitue']} | {l['impact_substitue']} | "
          f"{l['confiance']} | {l['effort_tours']} | {l['impact_raison']} |")
    w("")
    w("`Reach*` = chantiers débloqués, transitivement. `Impact*` = palier :")
    for k in sorted(PALIERS, reverse=True):
        w(f"**{k}** {PALIERS[k]} · ")
    w("")
    w("---")
    w("")
    w("*Dérivé du Manifeste Maury (CC BY-SA 4.0). Skill `adoption` de la")
    w("Méthode Foyer, à Reach et Impact substitués faute d'usage réel.*")
    return "\n".join(out)


def main():
    lignes, n_utilisables, fiches = calculer()
    if "--verifier" in sys.argv:
        return verifier(lignes, n_utilisables, fiches)
    if "--json" in sys.argv:
        print(json.dumps(lignes, ensure_ascii=False, indent=2))
        return 0
    verifier(lignes, n_utilisables, fiches)
    print(tableau(lignes, n_utilisables))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
