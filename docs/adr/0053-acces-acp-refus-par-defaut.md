# ADR 0053 : l'accès d'un collaborateur à une ACP est refusé par défaut

- **Status**: Accepted (tranché par @gilmry le 2026-09-12)
- **Date**: 2026-09-12
- **Track**: Security / Software
- **Authors**: @gilmry (décision) + Claude Opus 5 (rédaction)
- **Related**: #694, #864, [ADR 0046](0046-lacp-porte-le-perimetre-communautaire.md),
  [ADR 0049](0049-perimetre-v0-1-0-integral.md)

## Contexte

Le modèle actuel ne permet pas de restreindre un syndic ou un comptable à un
sous-ensemble des ACP de son cabinet. `users.organization_id` est le **seul**
rattachement d'un utilisateur ; aucune table n'associe une personne à une ACP.
Une fois une ACP rattachée à une organisation, **tous** ses collaborateurs y
accèdent en bloc.

Un cabinet gère typiquement plusieurs copropriétés, rarement avec le même
personnel sur chacune. L'étape « le syndic devient autonome sur son ACP »
s'arrête donc au niveau de l'organisation : impossible de dire *« ce
collaborateur s'occupe des ACP A et B, pas de C, D et E »*.

La story de #694 posait la question sans la trancher, parce que c'est une
**destination** et non une modalité : accès à l'organisation entière par défaut
avec restriction optionnelle — ou refus par défaut et accès explicite ?

## Décision

**Refus par défaut.** Un collaborateur n'accède à une ACP que si une ligne
d'association l'y autorise explicitement. Aucun accès n'est déduit du seul
rattachement à l'organisation.

## Pourquoi

**Parce que l'autre option se dégrade en silence, et pas celle-ci.** Un accès
par défaut avec restriction optionnelle produit un système où l'oubli d'une
restriction ouvre un périmètre. Le refus par défaut produit un système où
l'oubli d'une autorisation ferme un accès : quelqu'un s'en plaint dans l'heure.
Les deux erreurs ne coûtent pas la même chose — l'une se signale, l'autre se
découvre à l'audit.

**Parce que #864 montre que ce défaut existe déjà ailleurs.** Quatre-vingt-sept
routes prennent une identité sans s'en servir pour décider ; on peut supprimer
le budget d'une autre copropriété en connaissant son UUID. Ajouter un modèle
d'accès en mode « ouvert sauf mention contraire » reviendrait à institutionnaliser
le motif qu'on est en train de corriger.

**Parce que la donnée est nominative.** Une ACP porte des dettes de
copropriétaires nommés, des procès-verbaux, des états datés. Le périmètre par
défaut d'un prestataire sur ces données est « rien », pas « tout ce que son
employeur gère ».

### Ce que cette décision change au rang de #694

L'issue se déclarait elle-même « **non bloquant pour v0.1.0** », ce qui
contredisait son rang 2 en capacité `Must` — contradiction inscrite en 🔴 au
registre.

**Le refus par défaut tranche cette contradiction.** Sans table d'association,
un refus par défaut n'accorde aucun accès à personne : le produit ne fonctionne
pas. La table cesse donc d'être une commodité pour cabinets multi-ACP et
devient **structurellement requise**. #694 reste au rang 2, et la mention
« non bloquant » est périmée.

## Conséquences

### À construire

- Une table d'association `user_acp_access` — ou l'extension de
  `user_building_access`, présente au schéma depuis `20250102000000` et
  **référencée nulle part** dans le code. Le sort de ce vestige se coordonne
  avec #846, qui le liste parmi onze tables mortes.
- La révision des gestionnaires qui cloisonnent aujourd'hui par
  `organization_id` seul.
- Un sélecteur d'ACP **filtré par les accès** de l'utilisateur courant.

### La migration de l'existant, et c'est le point coûteux

Le refus par défaut appliqué à des données existantes **retire l'accès à tout le
monde** au moment où il entre en vigueur. La migration doit donc **peupler la
table** à partir de l'état actuel — chaque collaborateur reçoit l'accès aux ACP
de son organisation — puis le refus s'applique aux ajouts futurs.

Sans cette étape, la mise en service coupe l'accès de tous les cabinets d'un
coup. C'est la contrepartie assumée de l'option retenue, et elle doit être
écrite dans la story avant le code.

### Ce qui reste hors périmètre

Le point d'entrée `GET /organizations/{id}/users` conserve sa portée
organisation : il n'est pas concerné.
