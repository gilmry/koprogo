#!/usr/bin/env bash
# Tests 4-catégories du garde-fou de cible de `recette-base-neuve.sh` (#954).
#
# Ce qu'ils éprouvent n'est PAS la destruction — c'est le REFUS. Un script qui
# recrée une base à côté d'une démo vivante doit prouver qu'il vise la bonne,
# et ce sont ces refus qu'il faut pouvoir exercer sans rien casser.
#
# D'où les variables d'environnement du script : sans elles, tester qu'il ne
# vise pas la démo obligerait à le pointer vers la démo.

set -uo pipefail

SCRIPT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/recette-base-neuve.sh"
ECHECS=0
LANCES=0

assert_code() {
  local libelle="$1" attendu="$2" obtenu="$3"
  LANCES=$((LANCES + 1))
  if [ "$attendu" != "$obtenu" ]; then
    echo "FAIL — ${libelle}: attendu exit ${attendu}, obtenu ${obtenu}"
    ECHECS=$((ECHECS + 1))
  else
    echo "ok   — ${libelle}"
  fi
}

assert_contient() {
  local libelle="$1" aiguille="$2" meule="$3"
  LANCES=$((LANCES + 1))
  if [[ "${meule}" != *"${aiguille}"* ]]; then
    echo "FAIL — ${libelle}: la sortie ne contient pas « ${aiguille} »"
    ECHECS=$((ECHECS + 1))
  else
    echo "ok   — ${libelle}"
  fi
}

# --- @security : viser la démo doit être REFUSÉ -------------------------------
test_security_refuse_le_postgres_de_la_demo() {
  local sortie rc
  sortie="$(KOPROGO_PG_RECETTE=koprogo-postgres bash "$SCRIPT" 2>&1)"; rc=$?
  assert_code "@security postgres de la démo → refus (exit 2)" 2 "$rc"
  assert_contient "@security le refus NOMME la démo" "DÉMO" "$sortie"
}

test_security_refuse_le_backend_de_la_demo() {
  local rc
  KOPROGO_BACKEND_RECETTE=koprogo-backend bash "$SCRIPT" >/dev/null 2>&1; rc=$?
  assert_code "@security backend de la démo → refus (exit 2)" 2 "$rc"
}

test_security_refuse_le_projet_de_la_demo() {
  # Le nom du conteneur peut être changé ; l'étiquette de projet beaucoup
  # moins facilement. Les deux doivent barrer la route.
  local sortie rc
  sortie="$(KOPROGO_PROJET_RECETTE=koprogo bash "$SCRIPT" 2>&1)"; rc=$?
  assert_code "@security projet « koprogo » → refus (exit 2)" 2 "$rc"
  assert_contient "@security le refus nomme docker-compose.prod.yml" "prod" "$sortie"
}

# --- @negative : une cible absente ne doit pas être traitée comme un succès ---
test_negative_conteneur_absent() {
  local sortie rc
  sortie="$(KOPROGO_PG_RECETTE=koprogo-dev-postgres-qui-nexiste-pas bash "$SCRIPT" 2>&1)"; rc=$?
  assert_code "@negative conteneur absent → exit 1, pas 0" 1 "$rc"
  assert_contient "@negative dit COMMENT relancer la pile" "docker compose" "$sortie"
}

# --- @edge : un conteneur d'un AUTRE projet est refusé ------------------------
test_edge_conteneur_dun_autre_projet() {
  # `elevia-postgres` tourne sur cet hôte et appartient à un projet voisin.
  # Si le script l'acceptait, une erreur de nom détruirait la base d'un
  # autre produit.
  local rc sortie
  if ! sudo docker ps --format '{{.Names}}' | grep -qx "elevia-postgres"; then
    echo "ok   — @edge (ignoré : elevia-postgres ne tourne pas sur cet hôte)"
    LANCES=$((LANCES + 1))
    return
  fi
  sortie="$(KOPROGO_PG_RECETTE=elevia-postgres bash "$SCRIPT" 2>&1)"; rc=$?
  assert_code "@edge conteneur d'un projet voisin → refus (exit 2)" 2 "$rc"
  assert_contient "@edge le refus nomme le projet trouvé" "projet" "$sortie"
}

# --- @happy : les valeurs par défaut désignent bien la recette ----------------
test_happy_les_defauts_visent_la_recette() {
  # On ne lance PAS le script : on vérifie que ses défauts sont les bons.
  # L'exécuter ici détruirait la base de recette au milieu d'une suite de
  # tests, ce qui est précisément le genre d'effet de bord qu'un test ne doit
  # pas avoir.
  local contenu
  contenu="$(cat "$SCRIPT")"
  assert_contient "@happy défaut du projet = koprogo-dev" 'KOPROGO_PROJET_RECETTE:-koprogo-dev' "$contenu"
  assert_contient "@happy défaut du postgres = koprogo-dev-postgres" 'KOPROGO_PG_RECETTE:-koprogo-dev-postgres' "$contenu"
  # Le script ne doit manipuler AUCUN volume : c'est par là que `reset-db`
  # visait `koprogo_postgres_data`, celui de la démo, à un caractère près du
  # `koprogo-dev_postgres_data` de la recette.
  #
  # On cherche la COMMANDE, pas le mot : le script nomme `postgres_data` dans
  # un commentaire, précisément pour expliquer pourquoi il n'y touche pas.
  LANCES=$((LANCES + 1))
  if grep -qE 'docker[_ ]+volume' "$SCRIPT"; then
    echo "FAIL — @happy le script manipule un volume : c'est la porte de l'accident"
    ECHECS=$((ECHECS + 1))
  else
    echo "ok   — @happy aucune manipulation de volume"
  fi
}

test_security_refuse_le_postgres_de_la_demo
test_security_refuse_le_backend_de_la_demo
test_security_refuse_le_projet_de_la_demo
test_negative_conteneur_absent
test_edge_conteneur_dun_autre_projet
test_happy_les_defauts_visent_la_recette

echo ""
echo "------------------------------------------------------------"
echo "${LANCES} assertions, ${ECHECS} échec(s)"
[ "${ECHECS}" -gt 0 ] && exit 1
exit 0
