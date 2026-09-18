#!/usr/bin/env bash
#
# Rend à la recette une base VIERGE, avant une campagne.
#
# ── Pourquoi ──────────────────────────────────────────────────────────────
#
# La base de recette grossissait d'environ 640 organisations par campagne, et
# rien ne les retirait (#954). Mesuré le 2026-09-18 :
#
#     2026-09-12   236 organisations créées
#     2026-09-13   637
#     2026-09-16   644
#     2026-09-17  2026   (trois campagnes)
#
#     état : 3 606 organisations · 4 120 utilisateurs · 24 232 lots
#
# Conséquence : **chaque campagne est plus lente que la précédente, sur le
# même code**. Le 2026-09-18, deux specs ont dépassé les 30 s sur du code
# inchangé de la veille. C'est le mode de défaillance le plus coûteux qu'un
# gate puisse avoir — il accuse le dernier changement, quel qu'il soit, et on
# instruit un diff innocent pendant des heures.
#
# ── Pourquoi on ne touche AUCUN volume ────────────────────────────────────
#
# La cible `reset-db` du Makefile nommait `koprogo_postgres_data`. C'est le
# volume de la DÉMO : celui de la recette s'appelle `koprogo-dev_postgres_data`.
# Elle était interactive, donc inoffensive en pratique — mais c'est une
# faute d'un caractère entre « vider la recette » et « détruire la
# production ».
#
# Ce script ne nomme donc jamais un volume. Il agit DANS le conteneur, sur la
# base, par son nom. Le pire qu'une erreur de cible puisse produire est un
# refus.
#
# ── Ce qu'il fait ────────────────────────────────────────────────────────
#
#   1. prouve qu'il vise bien la recette, et s'arrête sinon ;
#   2. relève les conteneurs de la démo AVANT ;
#   3. arrête le backend — sans quoi son pool empêche le DROP ;
#   4. recrée la base ;
#   5. redémarre le backend, qui rejoue les migrations et resème le
#      superadministrateur (`main.rs:118` et `:126`) ;
#   6. revérifie que la démo n'a pas bougé d'un identifiant.
#
# Le monde de scénario, lui, est semé par le `globalSetup` de Playwright
# (#955) : une base neuve a besoin de quelqu'un pour la peupler, et c'est
# exactement ce que ce fichier orphelin attendait depuis toujours.

set -uo pipefail

# Paramétrables pour que le garde-fou soit TESTABLE.
#
# Un garde-fou qu'on ne peut pas éprouver n'est qu'un commentaire rassurant :
# `recette-base-neuve.test.sh` pointe ce script vers des cibles interdites et
# vérifie qu'il refuse. Sans ces variables, il faudrait viser la démo pour
# tester qu'on ne la vise pas.
PROJET="${KOPROGO_PROJET_RECETTE:-koprogo-dev}"
CONTENEUR_PG="${KOPROGO_PG_RECETTE:-koprogo-dev-postgres}"
CONTENEUR_BACKEND="${KOPROGO_BACKEND_RECETTE:-koprogo-dev-backend}"
BASE="${KOPROGO_BASE_RECETTE:-koprogo_db}"
UTILISATEUR="${KOPROGO_PG_USER:-koprogo}"

# La démo, qu'on ne doit jamais toucher. Nommée ici pour que le refus soit
# explicite plutôt que déduit.
INTERDITS="koprogo-postgres koprogo-backend koprogo-frontend koprogo-minio"

rouge() { printf '\033[0;31m%s\033[0m\n' "$*"; }
vert()  { printf '\033[0;32m%s\033[0m\n' "$*"; }

docker_() { sudo docker "$@"; }

# ── 1. Prouver la cible ───────────────────────────────────────────────────

for interdit in $INTERDITS; do
  if [ "$CONTENEUR_PG" = "$interdit" ] || [ "$CONTENEUR_BACKEND" = "$interdit" ]; then
    rouge "✗ REFUS : ce script viserait « $interdit », qui est la DÉMO."
    exit 2
  fi
done

# Le projet compte autant que le nom du conteneur : un conteneur peut être
# renommé, une étiquette de projet beaucoup moins facilement.
if [ "$PROJET" = "koprogo" ]; then
  rouge "✗ REFUS : « koprogo » est le projet de la DÉMO (docker-compose.prod.yml)."
  rouge "  La recette est « koprogo-dev »."
  exit 2
fi

if ! docker_ ps --format '{{.Names}}' | grep -qx "$CONTENEUR_PG"; then
  rouge "✗ « $CONTENEUR_PG » ne tourne pas. Lancez la pile de recette d'abord :"
  rouge "    sudo docker compose -f docker-compose.yml up -d"
  exit 1
fi

etiquette="$(docker_ inspect "$CONTENEUR_PG" \
  --format '{{index .Config.Labels "com.docker.compose.project"}}' 2>/dev/null)"
if [ "$etiquette" != "$PROJET" ]; then
  rouge "✗ REFUS : « $CONTENEUR_PG » appartient au projet « $etiquette »,"
  rouge "  et non « $PROJET ». Refus de toucher à une base dont je ne suis pas sûr."
  exit 2
fi

# ── 2. Relever la démo AVANT ──────────────────────────────────────────────

avant="$(docker_ ps --filter 'name=koprogo' --format '{{.ID}} {{.Names}}' \
  | grep -v 'koprogo-dev' | sort)"

# ── 3 à 5. La base neuve ──────────────────────────────────────────────────

echo "→ arrêt du backend de recette (son pool empêcherait le DROP)"
docker_ stop "$CONTENEUR_BACKEND" >/dev/null 2>&1 || true

echo "→ base « $BASE » recréée"
docker_ exec "$CONTENEUR_PG" psql -U "$UTILISATEUR" -d postgres -v ON_ERROR_STOP=1 \
  -c "DROP DATABASE IF EXISTS $BASE WITH (FORCE);" \
  -c "CREATE DATABASE $BASE OWNER $UTILISATEUR;" >/dev/null
code=$?
if [ "$code" -ne 0 ]; then
  rouge "✗ la recréation de la base a échoué (code $code)"
  docker_ start "$CONTENEUR_BACKEND" >/dev/null 2>&1 || true
  exit 1
fi

echo "→ redémarrage du backend : migrations et superadministrateur"
docker_ start "$CONTENEUR_BACKEND" >/dev/null

# ── 6. Revérifier la démo ─────────────────────────────────────────────────

apres="$(docker_ ps --filter 'name=koprogo' --format '{{.ID}} {{.Names}}' \
  | grep -v 'koprogo-dev' | sort)"

if [ "$avant" != "$apres" ]; then
  rouge "✗ ALERTE : les conteneurs de la démo ont changé pendant l'opération."
  rouge "AVANT :"; echo "$avant"
  rouge "APRÈS :"; echo "$apres"
  exit 3
fi

vert "✅ base de recette neuve — la démo n'a pas bougé d'un identifiant"
echo "   Le monde de scénario sera semé par le globalSetup de Playwright."
