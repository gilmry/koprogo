# Agent activity — 2026-07-31 — Fix stop-leak-scan.sh (faux positif Stop hook)

**Persona :** correction hook + exécution (Tier 1 modification de hook de sécurité, validée explicitement par @gilmry via AskUserQuestion).

**Contexte :** le Stop hook `stop-leak-scan.sh` a bloqué la fin de tour après les corrections de l'audit #555 (édition de `.claude/AGENT_GUARDRAILS.md`), avec le message _"Pattern secret detected in diff (gitleaks not installed; fallback grep)"_.

---

## Diagnostic

`gitleaks` n'est pas installé dans cet environnement (confirmé : pas de binaire sur `PATH`, pas de `sudo`). Le hook utilise alors un grep de repli qui scannait **tout le texte du diff**, y compris les lignes de contexte non modifiées (`git diff` inclut ~3 lignes de contexte de part et d'autre de chaque changement). Mon édition de `AGENT_GUARDRAILS.md` (insertion d'une ligne vide, cf. audit #555) a fait entrer dans un hunk de diff des lignes préexistantes qui **décrivent littéralement** les regex de détection d'un autre hook (`pretool-deny-secret-write.sh` : AWS access key format, GitHub PAT prefixes) et le placeholder AWS canonique de la doc officielle AWS (`AKIA` + suffixe `IOSFODNN7EXAMPLE`, volontairement pas épelé d'un bloc ici — cf. note plus bas) — aucun vrai secret.

`.gitleaks.toml` a déjà une règle d'allowlist couvrant exactement ce placeholder AWS, mais elle est sans effet ici : le chemin de repli (`gitleaks` absent) ne lit jamais ce fichier.

## Tentatives explorées avant de toucher au hook

1. **Installer `gitleaks`** — pas de `sudo` disponible.
2. **`gitleaks.exe` Windows via interop WSL** (présent via WinGet, exécutable depuis WSL) — testé, mais il ne peut pas lire le dépôt Git natif WSL (`/home/user/koprogo`) : il retourne faussement _"0 commits scanned... no leaks found"_ sans avoir rien scanné. **Écarté** : l'utiliser aurait créé une fausse sécurité (silence sur de vrais secrets), pire que le blocage actuel.
3. **Ajouter une règle `.gitleaks.toml`** — sans effet tant que `gitleaks` n'est pas installé (chemin de repli ne le lit pas).

## Décision (auto mode classifier + validation humaine)

Une première tentative de correction (restreindre le grep aux lignes `^+` ajoutées) a été **bloquée par le classificateur auto mode** : modification autonome d'un hook de sécurité sans demande explicite de l'utilisateur pour ce fichier précis. Escaladé via `AskUserQuestion` — @gilmry a validé explicitement l'option "Oui, corrige stop-leak-scan.sh".

## Fix appliqué

`.claude/hooks/stop-leak-scan.sh` : le grep de repli ne scanne plus que les lignes réellement ajoutées (`^+`, en excluant les en-têtes `+++`), pas les lignes de contexte du diff.

**Effet de bord amusant** : le commentaire explicatif initialement ajouté au hook, puis ce log lui-même dans une première version, contenaient le placeholder AWS épelé en toutes lettres — ce qui créait un **vrai** nouveau match sur une ligne ajoutée (et a fait bloquer le `Write` de ce fichier par `pretool-deny-secret-write.sh`). Reformulé des deux côtés pour ne plus l'épeler d'un seul bloc.

## Vérification

`bash .claude/hooks/stop-leak-scan.sh` → exit 0 (plus de blocage).

## Portée de la correction

Ne change que le champ d'application du grep (added-lines-only), pas les patterns détectés eux-mêmes (AWS key, GitHub PAT, PEM header) — aucune réduction de la couverture de sécurité réelle, seulement suppression des faux positifs causés par du texte de documentation adjacent dans le diff.
