#!/usr/bin/env bash
#
# Gate — couverture OpenAPI des routes Actix (issue #732).
#
# ── Le défaut que ce gate ferme ────────────────────────────────────────────
#
# Le job CI « Contract Types Check (end-to-end anti-drift) » vérifie deux
# choses : que `docs/api/openapi.json` correspond à la source Rust, puis que
# `frontend/src/types/api.d.ts` correspond à ce fichier. Les deux comparaisons
# sont bonnes, mais elles partagent un angle mort :
#
#     un endpoint ABSENT de la spec est absent des DEUX côtés,
#     donc parfaitement cohérent, donc invisible au gate.
#
# C'est ainsi que les 16 endpoints `payment-methods` ont vécu hors contrat : les
# routes existent, la struct porte bien `#[derive(utoipa::ToSchema)]`, mais
# aucune n'a d'annotation `#[utoipa::path]` et la struct n'est pas enregistrée
# dans `infrastructure/openapi.rs`. Sans type généré, le frontend a écrit son
# DTO à la main — en oubliant deux champs requis. Résultat : `400` à chaque
# ajout de moyen de paiement, avec une CI verte de bout en bout.
#
# « Aucun drift » ne veut pas dire « aucun désaccord ». Ce gate mesure la
# seconde chose : l'exhaustivité de la déclaration.
#
# ── Pourquoi un cliquet et non un seuil ───────────────────────────────────
#
# Exiger 100 % casserait toutes les PR sur un héritage que personne n'a choisi.
# Le gate applique donc la règle déjà retenue pour le projet Playwright `smoke`
# (b1b6648e) : « plancher chiffré puis bascule ». Il échoue UNIQUEMENT si la
# dette augmente.
#
# ── Attention à l'historique de ce chiffre ────────────────────────────────
#
# Le cliquet a valu 558 du 2026-08-29 au 2026-09-01, sur la foi d'un comptage
# FAUX : la détection exigeait moins de 12 lignes entre `#[utoipa::path]` et la
# macro de route, ce qui excluait toute route dont l'annotation décrit
# plusieurs réponses. 118 routes pourtant annotées étaient comptées comme
# manquantes. La dette réelle au 2026-08-29 n'était pas de 558 mais de ~463.
#
# Ne pas lire une baisse brutale de ce chiffre comme un progrès : vérifier
# d'abord que la mesure n'a pas changé.
#
# Faire baisser BASELINE au fil des annotations ajoutées est le sens de marche
# attendu ; le gate y invite explicitement quand l'écart se creuse.
#
# ── Le second angle mort, fermé le 2026-09-03 (#734) ──────────────────────
#
# Compter les `#[utoipa::path]` ne suffisait pas. Une route peut être annotée
# et n'atteindre malgré tout NI la spec NI les types du frontend, si sa
# fonction n'est pas listée dans le `paths(...)` de
# `backend/src/infrastructure/openapi.rs`. utoipa ne découvre rien tout seul :
# l'annotation décrit, l'enregistrement publie.
#
# C'est ainsi que les onze routes `portfolio`, deux routes `ticket` et la
# déconnexion vivaient hors contrat tout en étant proprement documentées —
# invisibles au gate précédent, qui les comptait comme couvertes.
#
# Le gate mesure donc deux choses distinctes, et échoue sur l'une ou l'autre :
# l'exhaustivité de l'ANNOTATION (cliquet), et l'exhaustivité de
# l'ENREGISTREMENT (tolérance zéro — une route annotée sans être enregistrée
# est un travail à moitié fait, pas une dette héritée).
#
# Usage : backend/scripts/check-openapi-coverage.sh [chemin/handlers]

set -euo pipefail

ROOT="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/backend/src/infrastructure/web/handlers}"

# Garde-fou : une substitution de processus ne propage PAS son code de sortie.
# Sans ce contrôle, un chemin erroné ferait échouer `awk` en silence, le gate
# compterait 0 route et passerait au vert — exactement le genre de succès
# trompeur qu'il est censé empêcher.
if [[ ! -d "$ROOT" ]]; then
  echo "check-openapi-coverage: répertoire introuvable : $ROOT" >&2
  exit 2
fi

# Nombre de routes SANS `#[utoipa::path]` toléré. Ne doit que décroître.
# Mesuré le 2026-09-01 après annotation des modules journal-entries, units,
# owner-contributions et call-for-funds (rapport de test workflows financiers).
BASELINE=440

read -r total annotated < <(
  awk '
    # Une annotation `#[utoipa::path(...)]` couvre la PROCHAINE macro de route,
    # quelle que soit sa longueur.
    #
    # La version precedente exigeait moins de 12 lignes entre les deux. Le
    # seuil punissait la documentation soignee : une route decrivant cinq
    # reponses, ses parametres et une description multiligne depasse 12 lignes
    # et etait comptee NON ANNOTEE. Au 2026-09-01, 23 routes fraichement
    # annotees n\x27en faisaient remonter que 9 — le gate sous-estimait donc le
    # travail accompli, et aurait pu laisser reperdre un gain reel.
    #
    # Le drapeau est remis a zero a chaque fichier ET a chaque route consommee :
    # une annotation ne peut couvrir qu une seule route.
    FNR == 1 { pending = 0 }
    /#\[utoipa::path/ { pending = 1 }
    /^[[:space:]]*#\[(get|post|put|patch|delete)\("/ {
      total++
      if (pending) { annotated++; pending = 0 }
    }
    END { print total+0, annotated+0 }
  ' "$ROOT"/*.rs
)

if [[ "${total:-0}" -eq 0 ]]; then
  echo "check-openapi-coverage: aucune route trouvée sous $ROOT — analyse invalide" >&2
  exit 2
fi

undocumented=$(( total - annotated ))
pct=$(( total > 0 ? annotated * 100 / total : 0 ))

echo "Couverture OpenAPI des routes Actix"
echo "  routes totales        : ${total}"
echo "  avec #[utoipa::path]  : ${annotated} (${pct} %)"
echo "  sans annotation       : ${undocumented}  (cliquet : ${BASELINE})"

if [[ "$undocumented" -gt "$BASELINE" ]]; then
  cat >&2 <<MSG

──────────────────────────────────────────────────────────────────
La dette de couverture OpenAPI AUGMENTE : ${undocumented} > ${BASELINE}.

$(( undocumented - BASELINE )) route(s) ont été ajoutées sans
\`#[utoipa::path]\`. Elles n'entreront ni dans \`docs/api/openapi.json\`
ni dans \`frontend/src/types/api.d.ts\` — et le gate anti-drift ne
les verra pas, puisqu'il compare deux fichiers qui les ignorent
tous les deux.

Le frontend devra donc écrire son DTO à la main, sans filet.
C'est exactement ainsi que POST /payment-methods a fini par
renvoyer 400 en silence (issue #732).

À faire :
  1. annoter la nouvelle route avec \`#[utoipa::path(...)]\` ;
  2. enregistrer ses schémas dans \`infrastructure/openapi.rs\`
     (le \`#[derive(ToSchema)]\` seul ne suffit PAS) ;
  3. \`make openapi-export\` puis committer la spec.
──────────────────────────────────────────────────────────────────
MSG
  exit 1
fi

if [[ "$undocumented" -lt "$BASELINE" ]]; then
  echo
  echo "✅ La dette a baissé de $(( BASELINE - undocumented )) route(s)."
  echo "   Abaissez BASELINE à ${undocumented} dans $(basename "${BASH_SOURCE[0]}")"
  echo "   pour verrouiller le gain — sans quoi il peut être reperdu sans bruit."
  exit 0
fi

echo
echo "✅ Aucune nouvelle route hors contrat."

# ── Second contrôle : annoté ET enregistré ────────────────────────────────
#
# Tolérance zéro, contrairement au cliquet ci-dessus : annoter une route sans
# l'enregistrer n'est pas une dette héritée mais un travail interrompu à
# mi-chemin, et le corriger ne coûte qu'une ligne.

OPENAPI_RS="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/backend/src/infrastructure/openapi.rs"

if [[ ! -f "$OPENAPI_RS" ]]; then
  echo "check-openapi-coverage: openapi.rs introuvable : $OPENAPI_RS" >&2
  exit 2
fi

non_enregistrees=$(
  python3 - "$ROOT" "$OPENAPI_RS" <<'PY'
import os, re, sys

racine, openapi = sys.argv[1], sys.argv[2]
enregistrees = set(
    re.findall(r"handlers::(?:\w+::)*(\w+)\s*,", open(openapi, encoding="utf-8").read())
)

manquantes = []
for fichier in sorted(os.listdir(racine)):
    if not fichier.endswith(".rs"):
        continue
    annotee = False
    for ligne in open(os.path.join(racine, fichier), encoding="utf-8"):
        if "#[utoipa::path" in ligne:
            annotee = True
            continue
        m = re.match(r"\s*pub async fn (\w+)", ligne)
        if m and annotee:
            if m.group(1) not in enregistrees:
                manquantes.append(f"{fichier[:-3]}::{m.group(1)}")
            annotee = False

print("\n".join(manquantes))
PY
)

if [[ -n "$non_enregistrees" ]]; then
  nb=$(printf '%s\n' "$non_enregistrees" | wc -l | tr -d ' ')
  cat >&2 <<MSG

──────────────────────────────────────────────────────────────────
${nb} route(s) ANNOTÉE(S) mais NON ENREGISTRÉE(S) dans openapi.rs :

$(printf '%s\n' "$non_enregistrees" | sed 's/^/  /')

utoipa ne découvre rien tout seul. L'annotation \`#[utoipa::path]\` décrit la
route ; c'est le \`paths(...)\` de \`backend/src/infrastructure/openapi.rs\`
qui la publie. Sans les deux, la route n'entre ni dans
\`docs/api/openapi.json\` ni dans \`frontend/src/types/api.d.ts\` — et le
frontend écrira son DTO à la main, en oubliant un champ (#732).

Ajouter la ligne correspondante dans \`paths(...)\`.
──────────────────────────────────────────────────────────────────
MSG
  exit 1
fi

echo "  enregistrement        : toutes les routes annotées sont dans paths()"
