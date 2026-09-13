#!/bin/sh
# Fabrique des fichiers vides pour toutes les cibles de test et de bench
# déclarées dans Cargo.toml.
#
# Pourquoi : le Dockerfile de production ne copie pas `tests/`, afin qu'une
# modification de test n'invalide pas la couche des dépendances (voir
# Dockerfile.production). Mais cargo refuse de parser un manifeste dont les
# fichiers cibles manquent, et cargo-chef parse le manifeste. D'où ces
# souches.
#
# Les chemins sont dérivés du manifeste lui-même : aucune liste à tenir à
# jour en double, et l'ajout d'un test ne casse pas le build.
#
# Subtilité qui a coûté un build : certaines sections déclarent un `path`
# explicite (tests/integration/...). Créer la souche en `tests/<nom>.rs`
# ferait découvrir automatiquement une seconde cible du même nom, et cargo
# refuse les doublons. On respecte donc le `path` quand il est présent.
set -e

MANIFEST="${1:-Cargo.toml}"

awk '
  /^\[\[(test|bench)\]\]/ { flush(); kind = ($0 ~ /test/) ? "tests" : "benches"; inblk = 1; next }
  /^\[\[/                 { flush(); inblk = 0 }
  /^\[[^[]/               { flush(); inblk = 0 }
  inblk && /^name *=/     { line = $0; sub(/^name *= *"/, "", line); sub(/".*$/, "", line); name = line }
  inblk && /^path *=/     { line = $0; sub(/^path *= *"/, "", line); sub(/".*$/, "", line); path = line }
  END                     { flush() }

  function flush() {
    if (inblk && name != "") print (path != "" ? path : kind "/" name ".rs")
    name = ""; path = ""
  }
' "$MANIFEST" | while read -r target; do
  mkdir -p "$(dirname "$target")"
  [ -f "$target" ] || echo 'fn main() {}' > "$target"
done
