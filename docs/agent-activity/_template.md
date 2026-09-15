# Agent activity — YYYY-MM-DD — <titre court de la session>

**Persona :** <nom-agent> (Tier 2 — <lecture|diagnostic|proposal|reporting>)

**Contexte :** <pourquoi cette session a eu lieu — qui l'a demandée (humain ou
issue), quel ticket/slice/story elle sert>

## Actions Tier 2 effectuées

- ...

## Ce qui n'a PAS été exécuté (hors délégation / hors portée session)

<!-- Toute action Tier 1 (mutation prod, fermeture d'issue, envoi externe, etc.)
     nécessite une trace explicite de l'autorisation humaine reçue — sinon elle
     n'a pas eu lieu ici. -->

- ...

## Vérification

<!-- Commandes lancées, résultats observés (tests verts, CI, etc.) -->

- ...

<!--
Rappels avant de committer ce fichier (cf. docs/agent-activity/README.md) :
- Nommage : YYYY-MM-DD-<persona>.md, ou YYYY-MM-DD-<persona>-slice-N.md pour une
  story/slice Maury. Une entrée = un jour. Une story étalée sur plusieurs
  semaines = un fichier par semaine (dater au premier jour Tier 2 de la semaine).
- Sécurité : ne jamais coller la sortie brute d'une commande ayant manipulé un
  secret (.env, kubeconfig, clé API, mot de passe). Relire avant de committer —
  ne pas compter uniquement sur le hook Stop (gitleaks).
-->
