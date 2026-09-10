# ADR 0048 : le notaire a une identité et un tableau de bord ; trois routes seulement sont publiques

- **Status**: Accepted (tranché par @gilmry le 2026-09-10)
- **Date**: 2026-09-10
- **Track**: Software / Security / Legal-compliance
- **Authors**: @gilmry (décision) + Claude Opus 5 (rédaction)
- **Related**: issues #845, #772, [ADR 0045](0045-comptabilite-appartient-a-lacp.md)

## Contexte

Trente routes ne vérifiaient aucune identité. Onze ont été corrigées. Sur les
dix-neuf restantes, trois familles pouvaient être **publiques à dessein**, et
il ne fallait pas le supposer :

| Route | Hypothèse |
|---|---|
| `POST /energy-campaigns/{id}/join-as-individual` et les trois routes de consentement | une campagne énergétique s'ouvre-t-elle hors copropriété ? |
| `GET /etats-dates/reference/{reference_number}` | un notaire consulte-t-il un état daté par sa référence, sans compte ? |
| `PUT /convocation-recipients/{id}/email-opened` | pixel de suivi appelé par un client de messagerie sans jeton |

## Décision

**Les campagnes énergétiques sont publiques.** « L'achat groupé d'énergie sera
ouvert à tout le monde. » Les quatre routes rejoignent la liste `PUBLIQUES` de
la garde, **avec leur raison écrite** — une liste sans justification se remplit
toute seule.

**Le pixel de suivi est public.** Un `PUT` non authentifié, appelé par un
client de messagerie qui ne porte aucun jeton. Il rejoint la liste avec sa
raison.

**L'état daté par référence ne l'est pas.** Le notaire reçoit une **identité**
et un **tableau de bord**, où il retire les états datés des ACP qui le
concernent.

**Les treize autres routes restent gardées.**

## Pourquoi l'état daté fait exception

Ma recommandation proposait de laisser cette route publique : le notaire
connaît la référence, elle fait office de secret. La décision va dans l'autre
sens, et elle a raison.

**Un état daté porte les dettes d'un copropriétaire nommé.** Le rendre lisible
à quiconque connaît une référence, c'est publier une situation financière
individuelle derrière un identifiant devinable, transmissible, et jamais
révocable. Une référence n'est pas un secret : elle circule dans des courriels,
des dossiers de vente, des greffes.

La bonne réponse n'était donc pas d'accorder une exemption, mais de **créer le
rôle qui manquait**. Le notaire est un tiers légitime et récurrent du processus
de vente (Art. 3.94, Art. 3.95) ; il mérite une identité, pas une porte
dérobée.

## Conséquences

### Ce qui est à construire

- **Un rôle notaire**, avec le rattachement aux ACP pour lesquelles il agit.
  Portée à définir : par ACP, ou par lot en cours de vente.
- **Un tableau de bord notaire** : ses états datés, ses relevés de dettes
  (Art. 3.89 § 5, 5°), leurs échéances.
- La route `GET /etats-dates/reference/{reference_number}` passe sous garde
  d'identité.

### Ce que cela recoupe

Le module `releve_notaire.rs` existe déjà et suit les demandes de relevé avec
leur échéance de trente jours. Il n'a **aucune interface** : c'est un domaine
sans écran. Le tableau de bord notaire est son débouché naturel.

### La liste PUBLIQUES et sa discipline

Toute route qui y entre porte sa raison en commentaire, dans la garde. Sans
cette règle, la liste devient l'endroit où l'on range ce qu'on ne veut pas
corriger.
