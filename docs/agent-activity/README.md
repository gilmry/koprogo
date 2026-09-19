# `docs/agent-activity/` — journal Tier 2

## Pourquoi ce dossier

Le modèle d'autorisation Tier 1 / Tier 2 (`.claude/rules/CRITICAL.md` §11,
[`.claude/AGENT_GUARDRAILS.md`](../../.claude/AGENT_GUARDRAILS.md)) autorise un
agent à agir sans validation humaine préalable pour tout ce qui relève de la
lecture, du diagnostic, de la proposition ou du reporting (Tier 2) — **à
condition que ce soit tracé**. Ce dossier est cette trace : sans lui,
« autorisé mais logué » n'est qu'une intention déclarative, pas une garantie
vérifiable par le superviseur humain.

Story d'origine : [Tx.3](https://github.com/gilmry/koprogo/issues/595)
(`docs/maury/refonte-ux-multi-role-acp/stories.md` §8).

## Convention de nommage

- **Cas général** : `YYYY-MM-DD-<persona>.md` — une entrée par persona et par
  jour d'activité Tier 2. Une même entrée peut regrouper plusieurs actions du
  même jour, jamais de plusieurs jours.
- **Cas story/slice Maury** : `YYYY-MM-DD-<persona>-slice-N.md`, où `N` est le
  numéro de la slice concernée.

## Règle des semaines (@edge)

Si une même story/slice s'étale sur plusieurs semaines calendaires, on ouvre
**un nouveau fichier par semaine** plutôt que d'accumuler indéfiniment dans un
seul fichier — un journal qui grossit sans borne cesse d'être relisible.
Exemple pour une slice 3 étalée sur trois semaines :

```
2026-05-04-bob-slice-3.md   # semaine du 4 mai
2026-05-11-bob-slice-3.md   # semaine du 11 mai
2026-05-18-bob-slice-3.md   # semaine du 18 mai
```

Le fichier est daté au premier jour d'activité Tier 2 de cette semaine-là.

## Sécurité (@security)

Ces journaux retranscrivent des commandes exécutées : c'est l'endroit où un
jeton ou un secret se recopie le plus facilement par inadvertance. Ne jamais
coller la sortie brute d'une commande ayant manipulé un secret (`.env`,
`kubeconfig`, clé API, mot de passe). Le hook `Stop` (`stop-leak-scan.sh`,
gitleaks) scanne tout le dépôt en fin de session, y compris ce dossier — mais
il ne remplace pas une relecture avant de committer.

## Revue (@negative)

Toute PR portant du travail Tier 2 (diagnostic, proposition, reporting) sans
son journal daté dans `docs/agent-activity/` doit être renvoyée par le
relecteur pour mise à jour — cf. la case correspondante dans
[`.github/pull_request_template.md`](../../.github/pull_request_template.md).
La règle vit dans la revue, pas seulement dans ce document.

## Démarrer un journal

Copier [`_template.md`](_template.md).
