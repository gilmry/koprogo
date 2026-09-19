---
persona: "Copropriétaire (owner)"
role_code: "owner"
statut: brouillon
population_recette: 5
eprouve: true
date: "2026-09-15"
version: "0.2"
superviseur: null
signature_humaine: null
issue_cadre: 805
issue_parcours: 807
videos: []
---

# Copropriétaire (owner)

Titulaire d'un droit réel : il vote, paie, consulte ses quotes-parts et
participe à la communauté. C'est le rôle le plus peuplé en base (5 comptes)
— et pourtant celui qu'aucune des cinq premières recettes navigateur n'a
éprouvé de bout en bout, faute d'un mot de passe qui fonctionne sur les
comptes de recette de ce rôle. Tant que ce mot de passe n'est pas rétabli
sur la pile de recette (geste d'exploitation, cf. « Comptes de recette »
ci-dessous), un tiers du produit reste hors recette manuelle — ce que #807
contourne en donnant à ce rôle son propre parcours automatisé, amorcé par
API avec des comptes jetables plutôt que par les comptes de recette figés.

## Parcours nominal

Dix étapes, de la connexion au paiement d'un appel de fonds — l'ordre posé
par #807. Chacune est rejouée par
`frontend/tests/e2e/journeys/coproprietaire.journey.ts` et filmée par
`coproprietaire.spec.ts` (`@happy`).

1. Se connecter → `owner-dashboard`.
2. Voir ses lots et ses quotes-parts, **agrégées sur plusieurs ACP** →
   `dettes-par-acp`, `dette-acp` (une ligne par ACP), `dettes-par-acp-total`
   (somme, seulement si plusieurs ACP), `dettes-par-acp-explication`. Voir
   « Le multi-ACP » ci-dessous.
3. Comprendre comment sa quote-part est calculée — voir l'encadré dédié
   ci-dessous. Signalé rouge : voir « Ce qui ne marche pas encore ».
4. Consulter les charges qui lui sont imputées → `owner-expenses` (lecture
   seule : pas de bouton `create-button`, réservé syndic/comptable).
5. Payer un appel de fonds → signalé rouge : voir « Ce qui ne marche pas
   encore ».
6. Recevoir une convocation et la lire → `convocations-list` (envoi
   vérifié en #784 ; la lecture, atteignable, l'est aussi : aucun garde de
   rôle ne restreint `/convocations`, cf. `frontend/src/lib/guards.ts`).
7. Voter, ou donner procuration → `vote-unit-select`, `vote-btn-pour`,
   `resolution-vote-submit-button` (vote direct) ou `vote-proxy-input`
   (procuration, non rejouée par ce parcours mais déjà couverte par
   `Resolutions.spec.ts`).
8. Lire le résultat d'une assemblée → `vote-progress-pour` (décompte
   plafonné correct, cf. `Resolutions.spec.ts`).
9. Consulter le procès-verbal → `meeting-documents-list`, sur le même écran
   que le vote (`/meeting-detail`) — accessible au rôle `owner`
   (`frontend/src/lib/guards.ts` : route partagée SYNDIC + OWNER).
10. Participer aux modules communautaires → offrir une compétence
    (`create-offer-button`, `skill-create-name-input`,
    `submit-skill-offer-button`) **fonctionne**. Prêter un objet ne
    fonctionne pas encore — signalé rouge : voir « Ce qui ne marche pas
    encore ».

### Le multi-ACP

Un copropriétaire peut détenir des lots dans plusieurs ACP à la fois — c'est
l'ordinaire d'un investisseur, et le modèle du produit le permet dès lors
que ces ACP relèvent du même cabinet syndic (`Owner.organization_id` est
unique ; `Acp.organization_id` peut porter plusieurs ACP). La règle produit,
déjà implémentée dans `DettesParAcp.svelte` : **agréger pour informer,
séparer pour agir**. Le total consolidé répond à « combien dois-je ce
mois-ci » ; le paiement, lui, reste ACP par ACP, un bouton chacune, parce
que chaque ACP est une personne morale distincte avec son propre compte
bancaire (Art. 3.86 § 1er et § 3) — un virement groupé paierait la mauvaise
personne morale pour une partie de la somme. Étape 2 du parcours le vérifie
avec un copropriétaire lié à deux ACP.

### Comment sa quote-part est calculée

La quote-part d'un lot n'est pas une part de propriété au sens courant :
c'est le rapport entre la **quotité** du lot (`units.quota`, fixée par
l'acte de base, Art. 3.85 § 1er al. 2) et le **total des tantièmes** de
l'immeuble (`buildings.total_tantiemes`, 1000 par convention). Une charge
commune se répartit ensuite au prorata de ce rapport : un lot à 120/1000
doit 12 % d'une charge de l'immeuble, pas 100 %.

C'est un argument produit, pas un détail d'affichage — la revue de design
en a fait un point de vente : un copropriétaire qui comprend pourquoi il
doit tel montant conteste moins, et un lot minoritaire ne doit jamais
apparaître comme devant la totalité d'une charge. **C'est pourtant
exactement ce qui se produit aujourd'hui** : voir l'étape rouge ci-dessous.

## Ce que ce rôle ne peut pas faire, et pourquoi

- Ne peut pas **saisir une écriture comptable** — la tenue des livres est
  réservée au comptable (cf. `accountant.md`), séparation des tâches
  élémentaire entre qui décide de la dépense et qui la comptabilise.
- Ne peut pas **convoquer ni présider une assemblée** — c'est une mission du
  syndic (Art. 3.89 § 5), pas du copropriétaire.
- Ne peut pas **voter au-delà du plafond de procuration** (3 mandats,
  exception à 10 % des quotités) — limite anti-concentration du pouvoir de
  vote (Art. 3.87 § 6).

### #781 tranchée par l'observation

#781 demandait si un copropriétaire peut faire ce qu'un syndic agissant
pour le compte de l'ACP ne peut pas : offrir une compétence, prêter un
objet, réserver une ressource — des engagements qui portent sur une
personne nommée, pas sur la copropriété. L'étape 10 de ce parcours répond
par l'usage plutôt que par le raisonnement : un copropriétaire authentifié
**offre bien une compétence avec succès** (`submit-skill-offer-button`
aboutit, le formulaire se ferme). Le refus déjà opposé au syndic sur ce
même module (cf. `docs/tests/BRIEF-COWORK-recette-5.md` §2) était donc
légitime — il ne bloquait pas la fonctionnalité, il bloquait le mauvais
acteur. #781 peut se clore sur cette base.

## Références légales

- Art. 3.85 § 1er al. 2 — quotités de l'acte de base, base du calcul de
  quote-part.
- Art. 3.86 § 1er et § 3 — personnalité juridique et compte bancaire propre
  de chaque ACP, fondement du paiement séparé en multi-ACP.
- Art. 3.87 § 6 — plafonnement des procurations.
- Art. 3.87 § 5 — droit à l'information avant l'assemblée.
- Art. 3.88 — majorités applicables au vote.
- Art. 3.90 — conseil de copropriété (quand le copropriétaire y siège).

Voir `backend/src/domain/copropriete/registre_legal.rs` et
`docs/legal/coproprietaire/`.

## Ce qui ne marche pas encore

- 🔴 **Étape 3 — la quote-part affichée n'est pas prorata** (#807) :
  `GET /stats/owner/dues-by-acp` (`stats_repository_impl.rs`,
  `get_owner_dues_by_acp`) additionne le montant TOTAL des charges en
  attente de chaque immeuble où le copropriétaire détient un lot, sans
  jamais le multiplier par sa quotité. Un lot à 120/1000 (12 %) affiche donc
  100 % de la charge de son immeuble, pas 12 %. Caractérisé par le test
  `@negative` de `coproprietaire.spec.ts` (#807) — cette assertion fixe le
  comportement observé pour qu'un correctif futur la fasse échouer
  délibérément, elle ne dit pas que c'est le comportement voulu.
- 🔴 **Étape 5 — payer un appel de fonds reste hors de portée** (#807) : le
  bouton « Payer » de l'étape 2 mène à `/owner/payments`, un écran de lecture
  seule (historique et statistiques, `PaymentList`/`PaymentStats` — aucun
  formulaire de paiement). Le seul formulaire connu du contrat gelé,
  `owner-contribution-payment-form`, vit sur `/owner-contributions`, une
  route réservée SYNDIC/ACCOUNTANT (`frontend/src/lib/guards.ts`) malgré son
  préfixe `owner-`. Un copropriétaire qui visite cette route est renvoyé
  vers `/owner`. Caractérisé par deux tests `@negative` de
  `coproprietaire.spec.ts` (#807).
- 🔴 **Étape 10 — prêter un objet appelle une route absente** (#779) :
  `frontend/src/lib/api/sharing.ts` (`createLoan`) poste sur `POST /loans`,
  que le backend ne sert pas — il expose `/shared-objects/{id}/borrow` et
  `/return`. Le même désaccord touche les réservations de ressource
  (`/bookable-resources` côté frontend, `/resource-bookings` côté backend).
  C'est le périmètre de #779, déjà ouverte ; caractérisé ici par un test
  `@negative` de `coproprietaire.spec.ts`.

## Comptes de recette

**Préalable bloquant, non résolu par ce parcours** : les trois comptes de
recette du rôle copropriétaire sur la pile de recette (organisation
« Érables », cf. `docs/tests/BRIEF-COWORK-recette-5.md` §3) ont chacun une
fiche de copropriétaire valide mais un mot de passe qui ne fonctionne plus.
Rétablir ce mot de passe est un geste
d'exploitation sur un environnement partagé (réinitialisation d'un compte
existant en base de recette) — **Tier 1, à faire main par un humain**
(`.claude/rules/CRITICAL.md` § 11) : aucune migration ni fichier de semis
de ce dépôt ne crée ni ne gère ces comptes, donc aucun changement de code
ne peut lever ce préalable. Le parcours et les tests de #807 le contournent
en amorçant leurs propres comptes jetables par API à chaque exécution
(`Carine Copropriétaire`, `Sophie Syndic` — comptes fictifs, jamais
recopiés ici par adresse), ce qui prouve que les écrans fonctionnent mais
ne remplace pas une recette humaine sur les comptes réels.

Pour une recette manuelle une fois le mot de passe rétabli, les dix
personas copropriétaires de `docs/specs/00-personas-et-seed.rst`
(« Résidence du Parc Royal ») restent la référence : Alice Dubois, Bob
Janssen, Charlie Martin, Diane Peeters, Emmanuel Claes, Nadia Benali,
Marguerite Lemaire, Jeanne Devos, Philippe Vandermeulen, Marcel Dupont —
chacun désigné par son nom, jamais par une adresse recopiée ici.
