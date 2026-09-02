# Recentrer le modèle sur l'ACP

Analyse d'écart établie le 2026-09-02, après démonstration en conditions
réelles d'une passation de syndic.

> **Cadrage.** KoproGo prépare sa première release. Ce document décrit un
> écart de modélisation à résorber avant la mise en service, pas une panne
> en cours.

---

## 1. Le modèle métier

```
ACP  — entité juridique, numéro BCE, acte de base
 │
 ├── acte de base : total des tantièmes
 │     └── validé par le SuperAdmin SaaS AVANT confiage à un syndic
 ├── immeubles (au moins un)
 │     └── lots
 │           └── détentions : indivisions, mutations
 ├── comptabilité : grand livre, budgets, appels de fonds, dépenses
 └── MANDAT DE GESTION ──► Syndic (organisation)
       accordé et révoqué par l'assemblée générale
                            ├── comptable, interne ou externe
                            └── personnel, selon la taille du cabinet
```

Le point qui décide de tout : **le syndic est un mandataire, pas un
propriétaire**. L'ACP existe avant lui, lui survit, et son mandat change
au gré des AG. Sa comptabilité appartient à l'ACP.

---

## 2. Ce que dit le schéma aujourd'hui

| clé de rattachement | tables |
|---|---:|
| `organization_id` — le **syndic** | **44** |
| `acp_id` — l'**entité** | **3** |

Les trois qui portent `acp_id` : `buildings`, `units`, `technical_specs`.
Tout le reste — comptabilité, budgets, appels de fonds, états datés,
convocations, copropriétaires — est indexé sur le mandataire.

---

## 3. La démonstration

Une ACP avec un historique complet, confiée au **cabinet sortant** :
une charge approuvée (donc une écriture au grand livre), un budget voté,
un appel de fonds envoyé, un lot, une copropriétaire.

Puis l'AG nomme un nouveau syndic : `PUT /acps/{id}` avec le nouveau
`organization_id` → **HTTP 200**.

| | charges | écritures | budgets | quotes-parts | lots | proprios |
|---|---:|---:|---:|---:|---:|---:|
| avant, sortant | 1 | 1 | 1 | 1 | 1 | 1 |
| **après, ENTRANT** | 1 | **0** | **0** | **0** | 1 | **0** |
| **après, SORTANT** | 403 | **1** | **1** | **1** | 403 | **1** |

### 3.1 — Le syndic entrant hérite d'une copropriété amputée

Pas de grand livre. Pas de budget. Pas d'appels de fonds. **Pas même la
liste des copropriétaires.** Il ne peut ni tenir la comptabilité, ni
appeler les charges, ni convoquer l'assemblée.

Et l'article 3.94 § 1er lui impose de transmettre, sur simple demande
d'un notaire, « les procès-verbaux des assemblées générales ordinaires et
extraordinaires des **trois dernières années**, ainsi que les décomptes
périodiques des charges des **deux dernières années** » et « une copie du
dernier bilan approuvé ». Il ne les a pas.

### 3.2 — Le syndic sortant garde l'accès

Son mandat est révoqué. Il ne voit plus ni les charges, ni les lots
(403 — le contrôle par ACP fonctionne là où il est appliqué). Mais il
voit encore le **grand livre**, les **budgets**, les **appels de fonds**
et la **liste nominative des copropriétaires**.

C'est le point le plus grave, et il n'est pas seulement fonctionnel :
noms, adresses, adresses e-mail et **arriérés de paiement** de personnes
physiques restent accessibles à un cabinet qui n'a plus aucune base
légale pour les traiter. Un traitement sans base légale au sens du RGPD.

### 3.3 — La cause, unique

Les points de lecture filtrent sur le `organization_id` du jeton, et les
enregistrements portent celui du syndic qui les a créés. Le mandat
change ; l'estampille, non.

---

## 4. Chemin de résorption

L'ordre proposé va du plus urgent au plus structurant. Les trois
premières tranches n'exigent aucune migration de données.

### Tranche 1 — Couper l'accès du syndic sortant *(urgent, RGPD)*

Les points de lecture qui filtrent sur `organization_id` doivent filtrer
sur **les ACP actuellement gérées**. Pour les tables portant un
`building_id`, cela ne demande qu'une jointure :

```sql
WHERE building_id IN (
    SELECT b.id FROM buildings b
    JOIN acps a ON b.acp_id = a.id
    WHERE a.organization_id = $1
)
```

Concernées et déjà outillées : `expenses`, `budgets`, `call_for_funds`,
`journal_entries`, `etats_dates`, `payments`, `meetings`, `convocations`.

Cette seule tranche résout les deux moitiés du problème : l'entrant voit,
le sortant ne voit plus.

### Tranche 2 — Les tables sans `building_id`

`owners`, `payment_reminders` et `owner_contributions` n'ont pas de
chemin vers l'ACP. `owner_contributions` porte un `unit_id` *nullable* ;
`owners` et `payment_reminders`, rien.

Un copropriétaire appartient à une ACP — il détient des lots dans un
immeuble de cette ACP. `owners.acp_id` est le rattachement juste. Cela
demande une migration avec reprise depuis `unit_owners → units →
buildings → acps`, et une décision sur les propriétaires sans lot.

### Tranche 3 — Terminer le cloisonnement en écriture

24 routes acceptent encore un `building_id` de corps sans contrôle de
portée (5 sont fermées). La garde `verify_building_org_access` existe et
s'applique en une ligne par route.

### Tranche 4 — Le mandat comme relation datée

Aujourd'hui `acps.organization_id` est un champ mutable : changer de
syndic **efface** le précédent. Il n'existe aucune trace de qui gérait
l'ACP à quelle période.

Or un état daté porte sur une **date de référence** et engage le syndic
en fonction à cette date. Une table `acp_mandates (acp_id,
organization_id, date_debut, date_fin, decision_ag_id)` restituerait
cette dimension, et rendrait vérifiable la question « qui était le
mandataire le 12 mars ? ».

C'est la tranche la plus structurante et la moins urgente.

---

## 5. Ce qui est déjà fait

`verify_building_org_access` (2026-09-02) résout **immeuble → ACP →
organisation** et pose donc déjà la bonne question : *ce syndic a-t-il
la gestion de cette ACP ?* Elle reste juste après le recentrage, et sert
de modèle aux tranches 1 et 3.
