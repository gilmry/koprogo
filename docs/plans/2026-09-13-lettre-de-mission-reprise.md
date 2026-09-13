<!--
  Lettre de mission — pour qu'une session neuve reprenne sans redécouvrir.

  Exportée le 2026-09-13, HEAD `775cef47` sur `feature/dev`. C'est l'état du
  chantier à cet instant, pas un document qui se met à jour tout seul.

  La SOURCE DE VÉRITÉ vivante est `RELEASE.md` à la racine. Cette lettre
  l'introduit ; elle ne le remplace pas. En cas de désaccord entre les deux,
  `RELEASE.md` gagne — il est mis à jour à chaque étape conclusive.
-->

# Lettre de mission — reprise du chantier v0.1.0

## Avant la première commande

**Travaille dans `/home/ubuntu/koprogo`.**

Il existe une seconde copie, `/home/ubuntu/jarvis/projects/koprogo`, qui est
un *submodule* et qui **retarde de plusieurs jours**. Rien n'y signale qu'elle
est périmée : `git log` y répond normalement, les fichiers sont cohérents
entre eux. Le 2026-09-12, elle ignorait le Gantt, le registre Foyer et cinq
ADR — une recherche honnête y conclut que le travail n'existe pas.

Puis, dans l'ordre :

```bash
cd /home/ubuntu/koprogo
cat RELEASE.md                      # le registre Foyer — à lire AVANT d'agir
cat docs/GANTT_PASSES_v0_1_0.md     # l'ordre d'exécution, en passes d'agent
cat docs/WBS_v0_1_0.md              # le périmètre et sa provenance
```

Un `git checkout` tourne **toutes les deux minutes** dans ce dépôt
(`/etc/cron.d/ecosolva-auto-deploy` déploie `feature/dev` sur le VPS).
Committe avant de t'interrompre : des modifications non commitées y sont
exposées.

## La mission, en une phrase

Dérouler `docs/GANTT_PASSES_v0_1_0.md` vague par vague, en t'arrêtant à chaque
point humain, jusqu'à ce que la v0.1.0 soit taggable.

Le périmètre est **intégral** — 88 issues, ADR 0049. `Must/Should/Could`
ordonne et ne retire rien. Un score RICE bas dit « en dernier », jamais « hors
release ».

## Où en est le chantier

Quarante commits le 2026-09-12/13. Ce qui compte :

| | État |
|---|---|
| `main` | **à jour** — les 408 commits promus (PR #764), plus de RCE astro |
| Pile de recette | **debout**, 8090/8091/15432/19000-19001, isolée de la démo |
| Gate `e2e` | 🟠 **mesuré** : 300 ✓ / 8 ✗ / 14 sautés, code 2 |
| Gate `doc-vivante` | 🟢 parcours complet, 10 chapitres, 81 s |
| Gate `integration` | 🔴 #877 — il était déclaré 🟢 à tort |
| Fan-out | plan prouvé, **une passe réelle faite**, mais sans aucun gate |
| Registre CSI | `docs/telemetrie/passes.jsonl` — **une ligne, à `null`** |

## Ce qui bloque, et qui n'est PAS de ton ressort

Ne tente pas de contourner ces points. Signale-les et travaille ailleurs.

1. **Le secret `FANOUT_GITHUB_TOKEN` n'est pas posé.** Sans lui le fan-out
   pousse avec le `GITHUB_TOKEN`, et **GitHub ne déclenche aucun workflow sur
   un événement issu de ce jeton**. Chaque branche d'agent sort alors sans
   gates et sans vitrine. Un agent ne crée pas de credential.
2. **La PR #879 attend sa revue humaine** (ARRÊT 2). C'est la première
   promotion, et l'exercice est de la **chronométrer**.
3. **#877 attend un arbitrage** : rafraîchir le tag MinIO, changer de
   registre, ou le mettre en miroir épinglé par digest.
4. **Les arbitrages RICE** : six `Could` dans les quinze premières (dont #805,
   première du classement), treize `Must` dans la moitié basse.

## Ce que tu peux faire sans attendre

Par ordre d'utilité :

1. **Instruire les 4 échecs e2e non expliqués** — `Dashboard`, `I18n`,
   `OwnerScreensJourney`, `story2-acp-organization`, `role-assignment`. Les
   quatre autres sont **#718** (502 sous rafale), reproduite hors production
   pour la première fois. C'est #832, et la pile de recette est debout pour
   ça : `make test-e2e` vise `http://localhost:8090`.
2. **Les stories du rang 2** — C4.1/C4.2/C4.3, identité, périmètre, RGPD.
   #864 (87 routes prennent une identité sans s'en servir) est critique et
   n'est bloquée par personne.
3. **Le Gantt se réévalue** à chaque jalon. Si une dépendance déclarée dans
   `scripts/gantt-passes.py` est contredite par une passe réelle, corrige-la —
   le graphe est une hypothèse falsifiable, c'est écrit dans le document.

## Les sept pièges qui ont coûté du temps

Ils sont tous dans le dépôt, mais les redécouvrir coûte des heures.

**1. Ne vise jamais `http://localhost` nu.** Le port 80 de cet hôte est tenu
par le Traefik de la démo, qui route vers `Host(api.koprogo.com)`. La suite
e2e écrit — 480 POST/PUT/DELETE — et `make seed-reset` POSTe sur
`/seed/scenario/world`. La recette est sur **8090** (ADR 0050). Deux variables
à poser, pas une : `PLAYWRIGHT_BASE_URL` dit où va le navigateur,
`PLAYWRIGHT_API_BASE` où la campagne amorce son monde.

**2. Lis le code de sortie, pas la sortie.** `cmd | tail` rend le statut de
`tail`. Un vérificateur qui « affiche une erreur » peut sortir 0, et
l'inverse. Mesure avec `${PIPESTATUS[0]}` ou sans tube.

**3. Un gate jamais lancé ne dit rien, et son silence se lit comme un
accord.** Trois documents affirmaient des mécanismes que personne n'avait
exécutés, et les trois étaient faux. Avant d'écrire qu'une chose marche,
lance-la.

**4. `.env` de ce répertoire porte les secrets de la DÉMO** — Compose le
charge automatiquement. N'écris jamais `${JWT_SECRET:-...}` dans la pile de
recette : elle hériterait du secret de signature de production, et un jeton
forgé sur la recette vaudrait sur la démo.

**5. Rust en conteneur, jamais sur l'hôte.** `~/bin/kcargo`, et `--lib` ne
suffit pas : les harnais e2e/BDD ne compilent qu'avec `--tests`.

**6. Les gardes avant de pousser.** Le barrage de `vps-feature-dev.yml` en
lance quatorze en plus de fmt/clippy/test. Un commit rouge ne produit pas
d'image et la démo reste sur sa dernière version saine.

**7. Format des commits** : `type(scope): titre ≤70` + `Refs: #issue`. Le
scope est souvent le numéro d'issue — `fix(#872): …`.

## Les conventions de fond

- **Une question va en RFC (`docs/governance/rfc/`) ou en issue.** Jamais dans
  un markdown créé pour l'occasion : sans numéro ni statut, elle n'a pas de
  cycle de vie.
- **Un arbitrage se pose en 🔴 dans `RELEASE.md` avec sa preuve AVANT d'être
  tranché**, même quand celui qui pose et celui qui tranche sont la même
  personne. S'en dispenser, c'est signer au lieu de valider.
- **Chaque lot part d'un test rouge**, et pour le Track J d'un test qui **cite
  son article** du Code civil.
- **L'absence de mesure s'écrit `null`, jamais `0`.** Un zéro se confond avec
  « mesuré à zéro », et c'est ce qu'un prior ne doit pas pouvoir imiter.
- **Ne dégrade jamais en silence.** Si un dispositif ne peut pas tenir sa
  promesse, il doit le DIRE — au journal, et dans l'artefact qu'il produit.

## Les trois instruments de pilotage

| Fichier | Ce qu'il dit | Régénéré par |
|---|---|---|
| `docs/BACKLOG_STRUCTURE_v0_1_0.md` | le classement par capacité, le chiffrage | `scripts/backlog-structure.py` |
| `docs/GANTT_PASSES_v0_1_0.md` | l'ordre, les dépendances, les créneaux | `scripts/gantt-passes.py` |
| `docs/RICE_PRODUIT_v0_1_0.md` | l'ordre **de valeur**, et ses désaccords avec MoSCoW | `scripts/rice-produit.py` |

Aucun ne s'édite à la main. Deux vérificateurs doivent rester à 0 :

```bash
python3 scripts/backlog-structure.py --verifier   # aucun trou de classement
python3 scripts/backlog-pret.py                   # 88/88 « Agent IA Ready »
python3 scripts/rice-produit.py --verifier        # le barème tient debout
```

**RICE est substitué et le dit** : faute d'utilisateurs, `Reach` est le nombre
de chantiers débloqués et `Impact` le palier légal. À la bêta fermée, le
script se **réécrit** — il ne s'ajuste pas. Et il pèse le poids légal et le
levier, pas « bloque tout le monde » : #872, rang 1 du registre, y sort à
0,55. RICE **complète** le rang du Gantt, il ne le remplace pas.

## Ce qui n'est jamais de ton ressort

- **La promotion d'une branche** vers `feature/dev` ou `main`.
- **G1** (revue humaine signée) et **G2** (le tag) — Tier 1 par
  `docs/governance/RESPONSABILITE.md`.
- **Créer un secret** ou un credential.
- **Lancer `docker compose up`** sans avoir vérifié le projet ciblé. La démo
  tourne sur le même hôte, et 31 conteneurs pour dix projets avec elle.

## Interdits explicites

Ne desserre aucun limiteur, n'allonge aucun délai pour faire passer un test,
ne supprime aucune assertion. Si un test est rouge, comprends la cause —
test faux, produit faux, ou spécification périmée — et dis laquelle.

---

*Dérivé du Manifeste Maury (CC BY-SA 4.0). Méthode Foyer, porte `release`.*
