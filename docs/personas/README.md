---
persona: "Cadre — pas un rôle"
role_code: null
statut: publie
population_recette: null
eprouve: null
date: "2026-09-13"
version: "0.1"
superviseur: null
signature_humaine: null
issue_cadre: 805
issue_parcours: null
videos: []
---

# Documentation vivante multi-persona — convention

Ce dossier porte le cadre commun de #805 : un document par rôle réel du
produit, sur le modèle de `gilmry/klaar`. Ce fichier fixe la convention ;
`GABARIT.md` est le squelette à copier ; les six fichiers listés plus bas
sont les documents de rôle eux-mêmes.

**Ce que #805 livre** : le gabarit, la convention, la règle des étapes
rouges, la discipline des données de recette, et les six documents
rattachés au cadre. **Ce que #805 ne livre pas** : le contenu détaillé et
filmé de chaque parcours — c'est le périmètre des six stories sœurs
(#806-#809, #815, #816). L'association précise entre chaque rôle et sa
story reste à confirmer par le PO ; ne pas l'inventer ici.

## Les six rôles

| Rôle | Fichier | En base au 2026-09-13 | Éprouvé |
|---|---|---|---|
| Syndic | [`syndic.md`](syndic.md) | 5 | oui |
| Copropriétaire (owner) | [`owner.md`](owner.md) | 5 | oui |
| Comptable (accountant) | [`accountant.md`](accountant.md) | 2 | oui |
| Superadmin | [`superadmin.md`](superadmin.md) | 1 | oui |
| Admin | [`admin.md`](admin.md) | 0 | **non** |
| Modérateur communauté | [`community-moderator.md`](community-moderator.md) | 0 | **non** |

Source : la story #805 et `frontend/src/lib/auth/permissions.ts`. Les deux
derniers rôles ont un code (et pour `admin`, une interface : `admin-dashboard`
figure au contrat `data-testid` gelé) mais aucun compte réel — leurs
documents le disent plutôt que d'inventer un parcours théorique par-dessus
(@edge, cf. story C8.1).

## Gabarit d'en-tête (trait n°1 de klaar)

Chaque document commence par le frontmatter YAML de `GABARIT.md` :
`persona`, `role_code`, `statut`, `population_recette`, `eprouve`, `date`,
`version`, `superviseur`, `signature_humaine`, `issue_cadre`,
`issue_parcours`, `videos`.

**Règle Tier 1** : un agent renseigne tous les champs sauf `superviseur` et
`signature_humaine`, qui restent `null` tant qu'un humain n'a pas validé le
contenu métier (cf. `.claude/rules/CRITICAL.md` §11). Un document dont ces
deux champs sont remplis par un agent est un document à rejeter en revue.

## Convention de numérotation des parcours (trait n°2 de klaar)

Le parcours nominal de chaque rôle est une liste numérotée simple
(`1.`, `2.`, …) dans la section `## Parcours nominal` — pas une liste de
fonctionnalités, un chemin, de la première connexion à l'acte le plus
courant du rôle.

Chaque étape **nomme au moins une ancre `data-testid`** issue du contrat
gelé `frontend/src/lib/__tests__/data-testid.contrat.json` (#802/#803). Une
étape sans ancre est un parcours qu'on ne peut pas prouver atteignable — la
vérification d'atteignabilité effective (le test qui rejoue chaque étape)
est le périmètre des stories de parcours, pas de #805.

## Règle des étapes rouges

Une étape ou une capacité qui ne fonctionne pas encore se marque, dans la
section `## Ce qui ne marche pas encore` :

```
🔴 **Étape rouge** — <description precise> (#<issue>)
```

Le marqueur `🔴` **doit toujours** être suivi, sur la même ligne, d'une
référence `#<numéro>` vers l'issue qui la ferme. `garde_documentation_personas.rs`
(`toute_etape_rouge_nomme_son_issue`) rejette toute étape rouge orpheline :
une affirmation qu'on ne peut pas suivre est moins honnête qu'une absence
de mention (cf. le précédent de la matrice de conformité légale périmée en
silence pendant six mois, #837).

## Références légales : lues, pas recopiées

Une étape qui s'appuie sur une obligation légale cite l'article
(ex. « Art. 3.89 § 5, 5° ») et renvoie au registre exécutable
`backend/src/domain/copropriete/registre_legal.rs` et à `docs/legal/`. Elle
ne recopie ni le texte ni un délai en jours : un délai recopié se périme
sans que rien ne le signale, exactement le risque que le registre existe
pour éliminer (cf. commentaire de `InvariantLegal::delai_jours`). C'est la
règle posée en #800, après trois références légales périmées et une
affirmation sans base trouvées en recette.

## Discipline des données de recette

Aucune donnée d'apparence réelle dans ces documents ni dans les vidéos qui
les accompagneront — une galerie publiée est un document public, et les
données d'une copropriété sont nominatives.

- Les personas cités sont uniquement ceux de
  `docs/specs/00-personas-et-seed.rst` (immeuble fictif « Résidence du Parc
  Royal »), désignés **par leur nom** (ex. « Alice Dubois »), jamais par une
  adresse email recopiée.
- `garde_documentation_personas.rs`
  (`aucune_adresse_email_litterale_dans_les_documents_persona`) rejette
  toute adresse email littérale trouvée dans `docs/personas/*.md`.
- Les vidéos éventuelles (champ `videos` de l'en-tête) ne montrent que des
  comptes de recette sur la pile de recette (ADR-0050), jamais de données
  de production — rappel : v0.1.0 n'est pas en production
  (`.claude/rules/CRITICAL.md` §10), mais la règle vaut pour toutes les
  versions à venir.

## Le cliquet

`backend/tests/garde_documentation_personas.rs` vérifie, à chaque build :

1. les six rôles ont chacun leur document (`chaque_role_reel_a_son_document_persona`) ;
2. le total de documents ne baisse pas (`le_nombre_de_documents_persona_ne_baisse_pas`) ;
3. chaque document a son en-tête complet (`chaque_document_persona_porte_son_en_tete_complet`) ;
4. chaque document a ses cinq sections (`chaque_document_persona_a_ses_cinq_sections`) ;
5. les rôles non peuplés en base se disent non éprouvés (`les_roles_non_peuples_en_base_se_disent_non_eprouves`) ;
6. toute étape rouge nomme son issue (`toute_etape_rouge_nomme_son_issue`) ;
7. aucune adresse email littérale ne s'y glisse (`aucune_adresse_email_litterale_dans_les_documents_persona`).

Il ne vérifie pas la qualité du contenu métier ni l'atteignabilité réelle
des parcours (ancrage `data-testid` effectivement rejouable) — ce
deuxième cliquet, annoncé par #805, est du ressort des stories de parcours
qui écrivent le contenu détaillé.

## Workflows transverses

Les parcours qui traversent les rôles (cycle de vie d'une assemblée, circuit
d'une facture, entrée d'un nouveau copropriétaire) ne vivent pas dans un
document de rôle unique. #805 ouvre le dossier ; leur emplacement précis
(probablement `docs/personas/workflows/`) est à trancher avec le PO au
moment où l'une des stories sœurs en a besoin — pas anticipé ici sans
matière.
