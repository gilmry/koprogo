---
persona: "Comptable (accountant)"
role_code: "accountant"
statut: brouillon
population_recette: 2
eprouve: true
date: "2026-09-16"
version: "0.2"
superviseur: null
signature_humaine: null
issue_cadre: 805
issue_parcours: 808
videos: []
---

# Comptable (accountant)

Prestataire qui tient les comptes de l'ACP, sans en être membre. Écritures,
PCMN, rapports. Ne préside pas.

C'est le rôle le mieux cadré du produit : son périmètre vient d'être
confirmé par l'ADR-0052 — `compta`, et rien d'autre (voir plus bas, section
« Le groupe Communauté »). C'est aussi le seul rôle conçu pour le bureau
d'abord, avec un mobile en consultation seule — on ne saisit pas une
écriture comptable au doigt.

Le rôle se raffine en deux sous-rôles côté code
(`accountant.encodeur` pour la facture et le devis, `accountant.emetteur`
pour les charges et les appels de fonds — cf.
`frontend/src/lib/auth/permissions.ts`), pas encore documentés
séparément : ce document couvre le rôle `accountant` de base, dont les deux
sous-rôles héritent.

## Parcours nominal

Le marqueur 🔒 signale une étape que le serveur refuse si l'immeuble n'est
pas conforme à son acte de base — voir « Cas dégradé » ci-dessous, c'est la
question de support la plus fréquente du produit.

1. Se connecter → `login-email`, `login-password`, `login-submit`. Arrive
   sur son périmètre, jamais sur un tableau de bord générique →
   `accountant-dashboard`.
2. Consulter les tuiles de son périmètre (immeubles, dépenses, factures,
   rapports) → `accountant-buildings-tile`, `accountant-expenses-tile`,
   `accountant-invoices-tile`, `accountant-reports-tile`. Le plan comptable
   PCMN lui-même (40+ comptes, conforme à l'AR du 12/07/2012 — voir
   Références légales) n'a pas d'écran de consultation dédié au 2026-09-16 :
   il se lit au fil des écritures (étape 5, champs de code compte) et du
   rappel pédagogique de l'écran Rapports (`reports`, encart « À propos du
   PCMN Belge »). Ce n'est pas une étape rouge — rien n'y échoue — mais ce
   n'est pas non plus une bibliothèque qu'on parcourt.
3. 🔒 Saisir une dépense → `/expenses` : `create-button`,
   `building-select`, `description-input`, `amount-input`,
   `submit-button`.
4. Suivre le workflow d'une facture, brouillon → soumission → **[le syndic
   approuve]** → paiement → `/invoice-workflow` : `invoice-card`,
   `submit-approval-button`, `mark-paid-button`.

   **L'approbation n'est PAS de son ressort.** Le comptable saisit la
   dépense et la soumet ; c'est le syndic (ou un superadmin) qui l'approuve ;
   le comptable reprend la main pour le paiement. Qui saisit une dépense ne
   l'approuve pas.

   Ce document affirmait l'inverse jusqu'au 2026-09-17, et le produit le
   démentait des deux côtés : `check_syndic_role` rend 403 côté API, et
   `InvoiceWorkflow.svelte:220` ne rend `approve-button` que pour syndic ou
   superadmin. C'est la doc qui avait tort — arbitrage rendu par le PO
   (#942). `approve-button` est retiré de la liste des ancres de ce parcours :
   le comptable ne le verra jamais.

   Le parcours e2e vérifie désormais cette absence comme une PROPRIÉTÉ, pas
   comme une gêne à contourner.
5. Passer une écriture au journal (ACH/VEN/FIN/ODS), en partie double →
   `/journal-entries` : `journal-entry-panel`,
   `journal-entry-description-input`, une ligne par code de compte PCMN
   (débit/crédit), `submit-journal-entry-button`.

   **Deux corrections du 2026-09-18**, trouvées en filmant le parcours
   (#808) :

   - L'ancre annoncée ici était `journal-entry-form`. Elle n'existe pas :
     `JournalEntryForm.svelte:181` expose **`journal-entry-panel`**,
     renommée parce que l'ancienne résolvait DEUX éléments. Ce document
     n'avait pas suivi — et rien ne l'y obligeait, aucune garde ne confronte
     les ancres des personas au code.
   - L'écran ne s'ouvre pas sur le formulaire : il s'ouvre sur « choisissez
     un immeuble » (`journal-entries-no-building`), parce que le périmètre
     est nul au chargement de chacun des douze écrans qui le lisent
     (**#841**). Le parcours filmé le montre tel quel.
6. Établir un budget (brouillon), le soumettre à l'assemblée, puis
   l'approuver → `/budgets` : `create-budget-button`,
   `budget-building-select`, `budget-ordinary-amount`,
   `budget-submit-button` (crée le brouillon) ; sur la fiche budget,
   `submit-budget-button` (brouillon → soumis) puis `approve-budget-button`,
   `budget-approve-meeting-id-input`, `budget-approve-confirm-button`
   (soumis → approuvé).
7. 🔒 Répartir une dépense par tantièmes → sur la fiche dépense
   (`/expense-detail`) : `distributions-section`,
   `calculate-distribution-button`, `distribution-row`,
   `distribution-total`.
8. 🔒 Émettre un appel de fonds collectif → `/call-for-funds` :
   `call-for-funds-create-button`, `call-for-funds-form`,
   `call-for-funds-building-select`, `call-for-funds-amount-input`,
   `call-for-funds-submit-button`, `call-for-funds-send-button`.
9. 🔒 Produire un état daté (Art. 3.87 § 3 — voir Références légales) →
   `/etats-dates` : `etat-date-create-form`, `building`, `unit`,
   `reference-date`, `etat-date-generate-button` ; puis sur la fiche,
   `etat-date-pdf-submit`, `etat-date-pdf-link`.
10. Sortir le bilan et vérifier l'équilibre actif/passif → `/reports` :
    `financial-reports-generate`, `building-report-balance-tab-button`,
    `building-report-income-tab-button`.

### Cas dégradé — immeuble non conforme

Les étapes 🔒 (3, 7, 8, 9) partagent un même verrou serveur : avant de
créer une dépense, une répartition, un appel de fonds ou un état daté, le
serveur recalcule Σ(quotités des lots) et la compare au total déclaré par
l'acte de base de l'ACP (`total_tantiemes`). Si les deux divergent, la
requête est refusée en HTTP 422 (`BUILDING_NOT_CONFORMANT`, ADR-0010) et un
toast narratif s'affiche :

> **Calcul bloqué — Immeuble non conforme**
> `{n}` lot(s) manquant(s), Σ quotas = `{somme}` / `{base}`

**Correction du 2026-09-16** : contrairement à ce qu'un premier passage en
recette laissait croire, un simple écart sur le **nombre** de lots déclarés
(`total_units`) ne bloque plus rien depuis #770 — c'est le chemin nominal
d'un acte de base encodé progressivement. Seul un écart sur les
**quotités** (les millièmes, qui doivent sommer exactement au total de
l'acte) ferme la comptabilité. Le message affiche encore le nombre de lots
manquants à titre indicatif ; ce n'est pas lui qui bloque.

**Le recours du comptable** : il ne peut pas corriger lui-même — modifier
l'acte de base (quotités des lots, ou total déclaré) est un geste de
syndic, hors du périmètre `compta`. Il doit remonter au syndic : « les
quotités des lots de cet immeuble ne totalisent pas le total déclaré par
l'acte, corrigez l'un ou l'autre ». #783 a débloqué la création d'un
immeuble volontairement incohérent en recette, ce qui permet enfin de filmer
ce cas plutôt que de le décrire de mémoire.

## Ce que ce rôle ne peut pas faire, et pourquoi

- Ne peut pas **présider ni clôturer une assemblée générale** — les
  écritures qu'elle décide exigent le rôle comptable, mais la clôture de
  l'assemblée elle-même exige le rôle syndic (cf. `syndic.md`). Cette
  séparation est volontaire : un cabinet où la même personne porterait les
  deux casquettes mélangerait la tenue des comptes et la décision politique
  qu'ils éclairent.
- Ne peut pas **convoquer** ni **décider** des travaux — il rapporte des
  chiffres, il ne prend pas les décisions que ces chiffres éclairent.
- Ne peut pas **corriger l'acte de base** d'un immeuble non conforme (voir
  « Cas dégradé » ci-dessus) — modifier les quotités ou le total déclaré
  est un geste de syndic.

### Le groupe Communauté

Le comptable **n'a jamais** le menu « Communauté » (échanges SEL,
sondages, vie collective). `Navigation.test.ts` l'affirme en `@security`
(« accountant n'a pas le menu Communaute (RBAC produit) »), et
`frontend/src/lib/auth/permissions.ts` l'implémente : `accountant` et ses
deux sous-rôles ne voient que `compta`.

Une revue de design avait proposé d'y donner un accès réduit ; **l'ADR-0052
tranche en faveur du test**, explicitement : la maquette est corrigée,
`permissions.ts` et le test ne bougent pas. Raison retenue — le comptable
est un prestataire, pas un copropriétaire ; « Communauté » porte des
données de copropriétaires (échanges SEL, vie collective) sans lien avec la
tenue des comptes, et un test `@security` encode une décision
d'autorisation, pas une préférence d'affichage. Ce qui rouvrirait la
question : un motif métier écrit (ex. rapprochement de flux financiers si
le SEL en vient à produire des écritures), à inscrire alors dans le
commentaire du test avec sa référence d'issue — pas une préférence
d'ergonomie.

## Références légales

Le mandat du comptable externe n'est pas un mandat statutaire du Code
civil (à la différence du syndic ou du commissaire aux comptes, qui est
lui un copropriétaire désigné, Art. 3.91) : sa base est le plan comptable
normalisé, pas le chapitre copropriété.

- `docs/legal/pcmn_ar_12_07_2012.rst` — plan comptable minimum normalisé
  applicable aux ACP (Arrêté Royal du 12/07/2012).
- Art. 3.85 § 1er al. 2 — quotités de l'acte de base, base du calcul de
  conformité (voir « Cas dégradé »).
- Art. 3.87 § 3 — délai et contenu de l'état daté (étape 9). Délai lu
  depuis le registre exécutable, pas recopié ici — voir
  `backend/src/domain/copropriete/registre_legal.rs`.
- Art. 3.91 — accès du commissaire aux comptes aux pièces comptables (rôle
  distinct, tenu par un copropriétaire — cf. `owner.md` — pas par ce
  comptable externe).
- `docs/adr/0052-le-comptable-ne-voit-pas-communaute.md` — périmètre
  `compta` du rôle.

## Ce qui ne marche pas encore

Aucune étape rouge confirmée au 2026-09-16.

Point en suspens, volontairement **non** marqué comme étape rouge faute de
preuve : la story #808 rapporte un bouton d'approbation de budget resté
inerte lors d'une recette (référence interne « R2-8 »), pour l'étape 6.
Cette référence est introuvable dans le suivi GitHub du dépôt au
2026-09-16. La lecture de
`frontend/src/components/budgets/BudgetDetail.svelte` montre
`submit-budget-button` et `approve-budget-button` câblés à des appels API
réels, mais le premier ouvre une **modale de confirmation**
(`actionEnAttente`, motif #844 : soixante `confirm()` natifs remplacés par
de vraies modales, qu'un navigateur piloté ne supprime plus) avant
d'exécuter quoi que ce soit — un clic qui s'arrête là, sans confirmer,
paraît exactement inerte. C'est l'hypothèse la mieux étayée, pas une
certitude : ce dépôt sandbox ne permet pas d'exécuter la suite Playwright
pour trancher en direct. La spécification `AccountantParcoursJourney.spec.ts`
(étape 6) confirme la modale via `confirmerSiDemande` ; si elle échoue
quand même en CI, remplacer ce paragraphe par une étape marquée comme
rouge et nommant l'issue réelle, avant de publier ce document (passer
`statut` à `publie`).

## Comptes de recette

- Gisele Vandenberghe, comptable externe, cf.
  `docs/specs/00-personas-et-seed.rst` (« Personas : Professionnels »).
