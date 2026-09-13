# ADR 0046 : l'ACP porte le périmètre communautaire, et le community manager n'est pas forcément le syndic

- **Status**: Accepted (tranché par @gilmry le 2026-09-10)
- **Date**: 2026-09-10
- **Track**: Software / Product / Multi-tenancy
- **Authors**: @gilmry (décision) + Claude Opus 5 (rédaction)
- **Related**: [ADR 0045](0045-comptabilite-appartient-a-lacp.md), issues #779, #781, #587, #588

## Contexte

### La question, restée ouverte trois recettes durant

Les six modules communautaires — SEL, annonces, compétences, partage d'objets,
réservations, gamification — sont servis par le backend tantôt par immeuble,
tantôt par organisation. Le frontend, lui, les appelle par ACP : il a été écrit
dans le monde d'après l'ADR-0045, le backend sert encore celui d'avant.

Ce n'est pas une faute de frappe, c'est une divergence de modèle : **111 points
d'entrée**, dont 75 absents du contrat OpenAPI. Les décrire avant de trancher
aurait été du travail à refaire.

### Ma recommandation, et pourquoi elle était mauvaise

J'avais proposé de répartir module par module : l'ACP pour les annonces et la
gamification, l'immeuble pour le SEL, les compétences, le partage et les
réservations. Le raisonnement était que « l'entraide se joue entre voisins de
palier, pas entre bâtiments d'un même cabinet ».

Il reposait sur une intuition de proximité physique, et il ratait le but du
produit.

## Décision

**Les six modules sont portés par l'ACP.** Sans exception, sans répartition.

**Le rôle de community manager peut être tenu par le syndic OU par un membre
de l'ACP.** Ce n'est pas une prérogative du mandataire.

### La raison, qui est une vision et non une préférence technique

> « Tous les membres de KoproGo peuvent faire de l'économie circulaire
> **ensemble**. Le plus petit porteur est l'**ACP et ses membres**. »

L'objet du produit n'est pas de reproduire le voisinage de palier : c'est de
faire exister une communauté à l'échelle où elle a une personnalité juridique,
un patrimoine et une assemblée. L'ACP est cette échelle. C'est déjà elle qui
porte la comptabilité (ADR-0045), l'acte de base (ADR-0010) et les fonds
(ADR-0012) ; il serait incohérent qu'elle ne porte pas l'entraide.

### La conséquence que la question laissait ouverte

Une ACP peut compter plusieurs immeubles. **Oui, une perceuse se prête entre
immeubles d'une même copropriété.** C'était la vraie question sous-jacente, et
elle est tranchée dans le sens du partage.

### La conséquence que je n'avais pas envisagée

`#587` attribuait la modération communautaire au syndic seul
(`community.moderator`). C'est trop étroit : **un copropriétaire peut être
community manager**. Le rôle se distribue par désignation, pas par mandat.

Cela a du sens au-delà du produit : un syndic gère plusieurs ACP et n'a aucune
raison d'animer la vie de chacune. Un copropriétaire motivé, si.

## Conséquences

### Ce qui devient vrai

- Les 111 points d'entrée communautaires prennent l'ACP pour porteur. Le
  frontend n'a pas à changer : il appelait déjà par ACP.
- Le contrat OpenAPI peut enfin être écrit pour ces routes : la question qui
  bloquait leur description est répondue.
- `community.moderator` cesse d'être un attribut du syndic pour devenir un
  rôle attribuable à un membre.

### Ce qui reste à faire

- Migrer les routes servies par `building_id` ou `organization_id` vers
  `acp_id`. C'est #779.
- Ouvrir l'attribution du rôle de community manager à un copropriétaire, avec
  la trace de qui l'a désigné. C'est #587 étendu.
- Décrire les 75 points d'entrée absents du contrat.

### Ce que cela ne dit pas

Rien sur qui **paie** ou qui **répond** d'un échange communautaire. Une
perceuse prêtée entre immeubles d'une ACP soulève des questions d'assurance et
de responsabilité que cette décision n'aborde pas. Elle fixe le périmètre
technique et social, pas le régime juridique de l'échange.
