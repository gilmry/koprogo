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
