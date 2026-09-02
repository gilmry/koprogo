# Qui répond de quoi

> Human in the loop, AI first execution, human answerable.

Ce document dit, pour KoproGo, ce que cette phrase implique concrètement : quels
actes un agent pose seul, lesquels exigent une décision humaine, et lesquels ne
peuvent être posés que par un humain.

Il décline la [Méthode Foyer](https://github.com/gilmry/foyer) au projet. La méthode
oriente ; ce document contraint.

## Les trois temps

**Human in the loop.** L'humain intervient *pendant*, aux points de bascule, pas en
bout de chaîne sur un résultat figé. Une validation qui ne peut plus rien changer
n'est pas une validation, c'est une signature.

**AI first execution.** L'exécution part de l'agent. Il produit, rend son
raisonnement visible, montre ce que ça donne. Il s'arrête sur les choix engageants,
pas sur le travail.

**Human answerable.** C'est Gilles qui répond de ce logiciel devant un syndic, un
copropriétaire, un juge. Cela ne se délègue pas. Conséquence pratique : **tout ce
que l'agent produit doit rester vérifiable par celui qui en répondra.** Un
raisonnement opaque, une correction de masse par expression régulière, une décision
d'architecture prise en silence dans le code lui retirent les moyens d'assumer ce
qu'il assume de toute façon.

## Tier 1 — l'humain seul

Actes qu'un agent ne pose jamais, même s'il en a techniquement les moyens :

| Acte | Pourquoi |
|---|---|
| Tag de release (`v0.1.0` et suivants) | engage le produit auprès de vrais copropriétaires |
| Revue GO / NO-GO avant mise en ligne | suppose d'avoir utilisé le produit, pas seulement de l'avoir testé |
| Acceptation d'un ADR | un choix d'architecture engage au-delà de la session qui le propose |
| Arbitrage de périmètre de release | c'est un choix de ce qu'on livre, pas de comment on le code |
| Mise en production d'infrastructure, DNS, certificats | irréversible et visible de l'extérieur |
| Publication ou communication externe | engage le nom du projet |
| Traitement de données personnelles réelles | RGPD : le responsable de traitement est une personne |

## Tier 2 — l'agent exécute, la trace est obligatoire

Actes que l'agent pose seul, à condition de laisser une trace que l'humain puisse
relire *après coup et comprendre sans avoir été là* :

- écrire du code et des tests, dans la boucle rouge-vert ;
- rédiger un ADR ou une RFC — les **rédiger**, pas les accepter ;
- créer, étiqueter, ordonner et fermer des issues, la fermeture exigeant une preuve
  vérifiable et pas une présomption ;
- déployer sur l'environnement de démonstration ;
- refactorer, à condition que le refactoring soit couvert par un test qui échouait
  avant.

La trace n'est pas un rapport d'activité. C'est ce qui permet de **refaire le
chemin** : le motif du choix, l'alternative écartée, et la commande qui prouve.

## Ce qui ne compte pas comme une trace

Trois formes ont été observées sur ce projet et n'en sont pas :

**Une correction de masse.** Une substitution appliquée à cent soixante sites ne se
relit pas. Elle a produit, le 2026-09-02, un état non compilable et une migration SQL
qui exigeait une colonne que le code n'écrivait pas. Le remède n'est pas de mieux
relire, c'est de ne pas produire ce genre de diff.

**Un test écrit après le code.** Il valide ce qui existe au lieu de dire ce qui
devrait exister. Deux tests écrits ainsi n'avaient jamais été exécutés une seule
fois, dont un qui ne pouvait pas passer.

**Une décision prise dans le code.** Recentrer le modèle sur l'ACP est un choix
d'architecture. Le faire par édition d'entités, sans ADR, c'est le soustraire à
celui qui en répond. D'où l'ADR-0045, écrit après coup — et c'est précisément ce
qu'il ne fallait pas.

## L'ordre, parce qu'il pré-engage le critère

1. **Le quoi avant le comment** — choix structurant → ADR ; question ouverte → RFC.
2. **Le test d'abord** — il est la spécification, le garde-fou et la documentation.
3. **Le domaine ensuite** — les responsabilités visibles dans le code, pas devinées.
4. **La vérification système** — E2E et charge sur ce qui coûterait cher à défaire.
5. **La documentation** — écrite pour celui qui reprendra sans avoir connu l'auteur.

## Par capacité, pas par date

On ne franchit un palier que si le précédent tient. Une fonctionnalité codée n'est
pas une capacité disponible : une capacité englobe les tests, la conformité, la
documentation et l'usage réel.

Corollaire de vocabulaire, qui a valeur de consigne ici : tant que rien n'est livré à
de vrais copropriétaires, un défaut ne se date pas. On dit « la capacité existe, le
câblage manque », pas « cassé depuis dix mois ». La sévérité technique reste la
sévérité technique ; c'est la mise en récit qui change.

## Le danger surveillé

> *Assez fiable pour qu'on cesse de vérifier.*

C'est le point de bascule de ce document. Plus l'agent devient fiable, plus il est
tentant de sauter le premier temps et d'oublier le troisième. Les gates Tier 1
existent pour que la vérification reste un réflexe et non une politesse.

---

*Dérivé du Manifeste Maury (CC BY-SA 4.0). Voir `docs/adr/` et
`docs/governance/rfc/`.*
