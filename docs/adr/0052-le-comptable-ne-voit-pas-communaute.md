# ADR 0052 : le comptable ne voit pas « Communauté » — c'est la maquette qui est corrigée

- **Status**: Accepted (tranché par @gilmry le 2026-09-12)
- **Date**: 2026-09-12
- **Track**: Security / UX
- **Authors**: @gilmry (décision) + Claude Opus 5 (rédaction)
- **Related**: #856, #802, `frontend/src/lib/auth/permissions.ts`,
  `frontend/src/components/navigation/__tests__/Navigation.test.ts`

## Contexte

Une remise de design propose de montrer au comptable un groupe « Communauté »
réduit. Le code l'interdit, et pas incidemment : un test de la section
`describe("Navigation @security")` le vérifie nommément —

```
accountant n'a pas le menu Communaute (RBAC produit)
```

`permissions.ts` en porte la règle : *accountant → uniquement `compta`*.

Deux artefacts se contredisaient donc, et il fallait dire lequel faisait foi.
Tant que ce n'était pas tranché, la story #802 livrait tout sauf ce point.

## Décision

**La maquette est corrigée. `permissions.ts` et le test `@security` ne bougent
pas.** Le comptable conserve `compta` et rien d'autre.

## Pourquoi

**Le comptable est un prestataire, pas un copropriétaire.** « Communauté » porte
les échanges SEL et la vie collective de l'immeuble — des données de
copropriétaires, sans lien avec la tenue des comptes. `permissions.ts` applique
déjà ce raisonnement ailleurs et l'écrit : le conseiller garde « Communauté »
*parce qu'il est copropriétaire avant d'être conseiller*. Le comptable ne l'est
pas. Lui ouvrir ce menu ne raffinerait pas un rôle, cela le déplacerait.

**Un test `@security` n'est pas une préférence d'affichage.** Il encode une
décision d'autorisation. L'assouplir pour qu'il coïncide avec une maquette
inverse l'ordre des autorités : c'est le rendu qui doit refléter la règle, et
non l'inverse. Le jour où l'on procède ainsi, la suite `@security` cesse d'être
une garde et devient un procès-verbal de ce que le design a décidé.

**Le coût du contraire est asymétrique.** Corriger une maquette coûte une
passe de design. Élargir un périmètre d'accès coûte une revue, une migration de
droits, et un risque qui ne se voit pas au moment où on le prend.

## Conséquences

- La remise de design est corrigée sur ce point ; aucun code de permission ne
  change, aucun test n'est réécrit.
- **#802 est débloquée intégralement** — la borne qui la faisait livrer « tout
  sauf ce point » tombe.
- La règle *accountant → `compta`* est confirmée comme intentionnelle. Elle
  reste raffinable **vers le bas** par les sous-rôles `accountant.encodeur` et
  `accountant.emetteur` déjà déclarés, jamais élargie vers d'autres groupes.

### Ce qui rouvrirait la question

Un motif métier écrit : par exemple un rapprochement de flux financiers issus
d'échanges communautaires, si le SEL venait à produire des écritures. Il
faudrait alors l'inscrire dans le commentaire du test, avec sa référence
d'issue — et non simplement relâcher l'assertion.
