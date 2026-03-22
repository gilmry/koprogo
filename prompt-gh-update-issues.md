# Prompt Claude Code — Mise à jour GitHub Issues & WBS via `gh`

> **Usage** : Copier ce prompt dans une session Claude Code ayant accès au dépôt `gilmry/koprogo` avec `gh` authentifié.

---

## Contexte

Suite aux tests E2E manuels réalisés le 22/03/2026, un rapport complet (`rapport-tests-e2e-koprogo.docx`) a identifié :
- **8 bugs** (2 critiques, 4 majeurs, 2 cosmétiques)
- **10 items MANQUANTS** dans la matrice de conformité légale (67% conforme)
- **4 lacunes RGPD** critiques (Art. 13-14, 28, 32, 33)
- **1 GAP architectural majeur** : chaîne d'approbation des dépenses non connectée
- **Points de vérification** couvrant comptabilité, AG, droits résidents, devis/marchés, énergie

## Instructions

Tu es un assistant Claude Code. Utilise exclusivement la CLI `gh` pour mettre à jour le projet GitHub `gilmry/koprogo`. Ne crée PAS de branches ni de commits — uniquement des opérations GitHub (issues, milestones, projects).

### Étape 1 : Vérifier l'état actuel

```bash
# Lister les milestones
gh api repos/gilmry/koprogo/milestones --jq '.[] | "\(.number) \(.title) \(.open_issues)/\(.open_issues + .closed_issues)"'

# Lister les issues ouvertes par milestone
gh issue list -R gilmry/koprogo --state open --limit 100

# Lister les labels existants
gh label list -R gilmry/koprogo --limit 100

# Vérifier les projets
gh project list --owner gilmry
```

### Étape 2 : Créer les labels manquants (si absents)

```bash
gh label create "bug:critique" --color "922B21" --description "Bug critique - bloquant" -R gilmry/koprogo 2>/dev/null || true
gh label create "bug:majeur" --color "7D6608" --description "Bug majeur - fonctionnel" -R gilmry/koprogo 2>/dev/null || true
gh label create "bug:mineur" --color "1E8449" --description "Bug mineur" -R gilmry/koprogo 2>/dev/null || true
gh label create "conformité" --color "5B2C6F" --description "Conformité légale belge" -R gilmry/koprogo 2>/dev/null || true
gh label create "rgpd" --color "1A5276" --description "RGPD/GDPR compliance" -R gilmry/koprogo 2>/dev/null || true
gh label create "test:e2e" --color "2E86C1" --description "Tests E2E manuels" -R gilmry/koprogo 2>/dev/null || true
gh label create "architecture" --color "6C3483" --description "Architecture/Design" -R gilmry/koprogo 2>/dev/null || true
```

### Étape 3 : Créer les issues pour les 8 bugs identifiés

**Bug #1 — Permissions rôles (MAJEUR)**
```bash
gh issue create -R gilmry/koprogo \
  --title "[Bug] Permissions rôles : boutons admin visibles pour le syndic" \
  --label "bug:majeur,test:e2e" \
  --milestone "Jalon 1: Sécurité & GDPR" \
  --body "## Description
Le syndic voit les boutons **Créer Organisation** et **Créer Utilisateur** dans le sidebar.
Ces actions sont réservées au rôle SuperAdmin.

## Impact
Risque de confusion UX. Les endpoints backend retournent 403, mais l'UI ne devrait pas exposer ces actions.

## Correction proposée
Filtrer les éléments de navigation par \`active_role\` dans le frontend (Navigation.svelte).

## Origine
Tests E2E manuels — 22/03/2026"
```

**Bug #2 — Isolation multi-tenant (CRITIQUE)**
```bash
gh issue create -R gilmry/koprogo \
  --title "[Bug] CRITIQUE : Isolation multi-tenant — données non filtrées par organization_id" \
  --label "bug:critique,test:e2e" \
  --milestone "Jalon 1: Sécurité & GDPR" \
  --body "## Description
Plusieurs endpoints retournent les données de TOUTES les organisations au lieu de filtrer par \`organization_id\` de l'utilisateur authentifié.

## Impact
**CRITIQUE** — Fuite de données inter-tenants. Violation RGPD potentielle.

## Endpoints concernés
- GET /buildings (retourne les immeubles de toutes les organisations)
- GET /owners (idem)
- Potentiellement d'autres endpoints à auditer

## Correction proposée
Ajouter un filtre \`WHERE organization_id = $user.organization_id\` à TOUS les handlers de listing.
Audit systématique des 511 endpoints.

## Origine
Tests E2E manuels — 22/03/2026"
```

**Bug #3 — Calcul tantièmes (MAJEUR)**
```bash
gh issue create -R gilmry/koprogo \
  --title "[Bug] Calcul tantièmes : total des lots ≠ total immeuble (1000 millièmes)" \
  --label "bug:majeur,conformité,test:e2e" \
  --milestone "Jalon 2: Conformité Légale Belge" \
  --body "## Description
Aucune validation frontend/backend ne vérifie que la somme des tantièmes des lots = 1000 millièmes.
Le trigger PostgreSQL \`validate_unit_ownership_total\` valide les quotes-parts des propriétaires mais pas la somme des tantièmes structurels.

## Impact
Non-conformité Art. 577-2 §4 Code Civil belge. Les calculs de quorum et charges seront erronés.

## Correction proposée
1. Ajouter un champ \`total_shares\` au Building (défaut: 1000)
2. Valider à la création/modification d'un lot que sum(unit.shares) <= building.total_shares
3. Afficher un warning si sum < total (lots manquants)

## Origine
Tests E2E manuels — 22/03/2026"
```

**Bug #4 — Pages en anglais (COSMÉTIQUE)**
```bash
gh issue create -R gilmry/koprogo \
  --title "[Bug] Pages non traduites en français : Tickets, Announcements, Bookings" \
  --label "bug:mineur,test:e2e" \
  --body "## Description
Plusieurs pages du dashboard syndic restent en anglais alors que l'interface est en français :
- /tickets : 'Create New Ticket', 'Loading tickets...', 'No tickets found'
- /announcements : 'Community Notice Board', 'Post an Announcement'
- /bookings : 'Resource Booking Calendar'

## Correction proposée
Traduire tous les textes statiques en français. Idéalement implémenter i18n (FR/NL/DE/EN).

## Origine
Tests E2E manuels — 22/03/2026"
```

**Bug #5 — Bouton créer ticket silencieux (MAJEUR)**
```bash
gh issue create -R gilmry/koprogo \
  --title "[Bug] Création ticket : bouton silencieux si building_id manquant" \
  --label "bug:majeur,test:e2e" \
  --body "## Description
Le bouton 'Create New Ticket' sur /tickets ne fait rien de visible quand \`building_id\` n'est pas dans les query params.
Le handler appelle \`showPageToast('Veuillez d'abord sélectionner un immeuble.', 'warning')\` mais le toast ne s'affiche pas.

## Impact
UX bloquante — l'utilisateur ne comprend pas pourquoi rien ne se passe.

## Correction proposée
1. Ajouter un sélecteur d'immeuble dans la page tickets (dropdown)
2. Corriger \`showPageToast\` pour qu'il affiche réellement le toast
3. Désactiver le bouton si aucun immeuble n'est sélectionné

## Origine
Tests E2E manuels — 22/03/2026 (tickets.astro lignes 93-121)"
```

**Bug #6 — Validation tantièmes 100% (CRITIQUE)**
```bash
gh issue create -R gilmry/koprogo \
  --title "[Bug] CRITIQUE : Validation tantièmes — possible de dépasser 100% par ajouts séquentiels" \
  --label "bug:critique,conformité,test:e2e" \
  --milestone "Jalon 2: Conformité Légale Belge" \
  --body "## Description
Le trigger PostgreSQL \`validate_unit_ownership_total\` avertit si total < 100% mais bloque seulement > 100%.
En ajoutant des propriétaires séquentiellement, on peut arriver à un état incohérent.

## Impact
**CRITIQUE** — Les calculs de charges et votes seront erronés. Non-conformité Art. 577-2 §4 CC.

## Correction proposée
1. Bloquer au niveau frontend avec calcul en temps réel du total restant
2. Renforcer le trigger PostgreSQL avec validation transactionnelle
3. Ajouter un endpoint de validation : GET /units/:id/owners/total-percentage

## Origine
Tests E2E manuels — 22/03/2026"
```

**Bug #7 — Chargement immeubles (MAJEUR)**
```bash
gh issue create -R gilmry/koprogo \
  --title "[Bug] Sondages/Annonces/Réservations : immeubles non chargés" \
  --label "bug:majeur,test:e2e" \
  --body "## Description
Les pages Sondages (/polls), Annonces (/announcements) et Réservations (/bookings) ne chargent pas la liste des immeubles, rendant impossible la création de contenu.

## Impact
Pages communautaires inutilisables.

## Correction proposée
Corriger l'appel API pour charger les immeubles de l'organisation du syndic (filtré par organization_id).

## Origine
Tests E2E manuels — 22/03/2026"
```

**Bug #8 — Label sondages (COSMÉTIQUE)**
```bash
gh issue create -R gilmry/koprogo \
  --title "[Bug] Sondages : label 'Building' au lieu de 'Immeuble'" \
  --label "bug:mineur,test:e2e" \
  --body "## Description
Dans le formulaire de création de sondage, le label du sélecteur d'immeuble affiche 'Building' en anglais.

## Correction proposée
Remplacer par 'Immeuble' (cohérence FR).

## Origine
Tests E2E manuels — 22/03/2026"
```

### Étape 4 : Créer les issues pour les GAPs architecturaux

**GAP — Chaîne d'approbation des dépenses**
```bash
gh issue create -R gilmry/koprogo \
  --title "[Architecture] Connecter la chaîne d'approbation des dépenses (Ticket → Rapport → Validation → Dépense)" \
  --label "architecture,conformité" \
  --milestone "Jalon 3: Features Différenciantes" \
  --body "## Description
Les modules Ticket, ContractorReport, BoardDecision et Expense existent individuellement mais ne sont PAS connectés dans un workflow métier.

## Workflow attendu
1. **Ticket créé** → Ordre de service généré
2. **Ordre de service accepté** → Magic link PWA envoyé au prestataire
3. **Prestataire soumet rapport** via PWA (photos, dictée vocale FR-BE, pièces)
4. **Rapport validé** par CdC (si >20 lots) ou Syndic (si ≤20 lots)
5. **Dépense approuvée** → Écriture comptable automatique + paiement déclenché

## Actions requises
- [ ] Créer l'entité WorkOrder (lien Ticket → ContractorReport)
- [ ] Ajouter FK contractor_report_id à Expense
- [ ] Rendre ContractorReport obligatoire avant Expense.approve()
- [ ] Ajouter FK board_decision_id à Expense si immeuble >20 lots
- [ ] Auto-trigger magic link PWA à l'acceptation de l'ordre de service
- [ ] Créer BuildingContractor (registre fournisseurs officiels ACP)

## Réf. rapport
rapport-tests-e2e-koprogo.docx — Section 3.3"
```

### Étape 5 : Créer les issues pour les items MANQUANTS de la matrice de conformité

**Conformité AG**
```bash
gh issue create -R gilmry/koprogo \
  --title "[Conformité] AG : Lien agenda-résolutions — bloquer votes hors ODJ" \
  --label "conformité" \
  --milestone "Jalon 2: Conformité Légale Belge" \
  --body "## Exigence légale
Art. 3.87 Code Civil belge : seuls les points inscrits à l'ordre du jour peuvent faire l'objet d'un vote.
Un vote hors ODJ est frappé de nullité.

## Implémentation requise
- Ajouter FK resolution.agenda_item_id → meeting_agenda_items
- Bloquer la création de résolutions sans point d'agenda correspondant
- BDD scenario dans features/meetings.feature

## Statut matrice
MANQUANT (matrice_conformite.rst)"
```

```bash
gh issue create -R gilmry/koprogo \
  --title "[Conformité] AG : Quorum 50%+50% et 2ème convocation automatique" \
  --label "conformité" \
  --milestone "Jalon 2: Conformité Légale Belge" \
  --body "## Exigence légale
Art. 3.87 §5 CC : Si le quorum (>50% des quotes-parts) n'est pas atteint à la 1ère convocation, une 2ème convocation est obligatoire (15 jours minimum).
À la 2ème convocation, aucun quorum n'est requis.

## Implémentation requise
- Ajouter champ meeting.is_second_convocation (boolean)
- Valider quorum seulement si is_second_convocation = false
- Auto-créer Meeting + Convocation de 2ème appel si quorum KO
- BDD scenarios

## Statut matrice
MANQUANT (matrice_conformite.rst)"
```

```bash
gh issue create -R gilmry/koprogo \
  --title "[Conformité] AG : Procurations — max 3 mandats + exception 10%" \
  --label "conformité" \
  --milestone "Jalon 2: Conformité Légale Belge" \
  --body "## Exigence légale
Art. 3.87 §7 CC : Un mandataire ne peut détenir plus de 3 procurations, sauf s'il ne dépasse pas 10% du total des voix.

## Implémentation requise
- Compter les procurations par proxy_owner_id lors du vote
- Bloquer si > 3 procurations ET > 10% des voix totales
- Afficher le nombre de procurations restantes dans l'UI
- BDD scenarios

## Statut matrice
MANQUANT (matrice_conformite.rst)"
```

```bash
gh issue create -R gilmry/koprogo \
  --title "[Conformité] AG : Distribution PV dans les 30 jours + génération automatique" \
  --label "conformité" \
  --milestone "Jalon 2: Conformité Légale Belge" \
  --body "## Exigence légale
Le procès-verbal doit être distribué à tous les copropriétaires dans les 30 jours suivant l'AG.

## Implémentation requise
- Génération automatique du PV à la clôture de l'AG (résolutions, votes, quorum, présences)
- Tracking de la date d'envoi du PV
- Alerte si PV non envoyé après 30 jours
- Todo list automatique pour le CdC basée sur les décisions votées

## Statut matrice
MANQUANT (matrice_conformite.rst)"
```

```bash
gh issue create -R gilmry/koprogo \
  --title "[Conformité] Syndic : Mandat max 3 ans avec validation" \
  --label "conformité" \
  --milestone "Jalon 2: Conformité Légale Belge" \
  --body "## Exigence légale
Art. 3.89 CC : Le mandat du syndic ne peut excéder 3 ans, renouvelable.

## Implémentation requise
- Ajouter validation domain : mandate_end - mandate_start <= 3 ans
- Alerte avant expiration du mandat (3 mois, 1 mois, 15 jours)
- Point AGO obligatoire pour renouvellement

## Statut matrice
MANQUANT (matrice_conformite.rst)"
```

### Étape 6 : Créer les issues RGPD manquantes

```bash
gh issue create -R gilmry/koprogo \
  --title "[RGPD] Art. 13-14 : Publier politique de confidentialité" \
  --label "rgpd" \
  --milestone "Jalon 1: Sécurité & GDPR" \
  --body "## Exigence
RGPD Art. 13-14 : Information obligatoire des personnes concernées sur le traitement de leurs données.

## Actions
- Rédiger la politique de confidentialité (FR/NL)
- Créer une page publique /privacy-policy
- Lien dans le footer de toutes les pages
- Formulaire de consentement au premier login

## Risque
Sanctions APD jusqu'à 20M€ ou 4% du CA global (avg. 18 000€ pour PME belge)."
```

```bash
gh issue create -R gilmry/koprogo \
  --title "[RGPD] Art. 28 : DPA avec sous-traitants (Stripe, AWS S3, email)" \
  --label "rgpd" \
  --milestone "Jalon 1: Sécurité & GDPR" \
  --body "## Exigence
RGPD Art. 28 : Contrat de sous-traitance obligatoire avec tout processeur de données.

## Sous-traitants identifiés
- Stripe (paiements)
- AWS S3 (stockage backups/documents)
- Fournisseur email (notifications)
- Fournisseur SMS (si activé)

## Actions
- Signer les DPA avec chaque sous-traitant
- Documenter dans le registre de traitement (Art. 30)
- Vérifier les clauses de transfert hors UE"
```

```bash
gh issue create -R gilmry/koprogo \
  --title "[RGPD] Art. 33 : Procédure notification violation de données (72h)" \
  --label "rgpd" \
  --milestone "Jalon 1: Sécurité & GDPR" \
  --body "## Exigence
RGPD Art. 33 : Notification de l'APD dans les 72h en cas de violation de données personnelles.

## Actions
- Documenter la procédure d'incident response
- Créer un template de notification APD
- Implémenter un endpoint interne /admin/security-incident
- Former l'équipe à la procédure
- Tester avec un exercice de simulation"
```

### Étape 7 : Mettre à jour les issues existantes avec commentaires

```bash
# Commenter les issues existantes liées aux findings
# D'abord trouver les issues pertinentes
gh issue list -R gilmry/koprogo --search "multi-tenant" --state open --json number,title
gh issue list -R gilmry/koprogo --search "quorum" --state open --json number,title
gh issue list -R gilmry/koprogo --search "GDPR" --state open --json number,title
gh issue list -R gilmry/koprogo --search "expense workflow" --state open --json number,title

# Pour chaque issue trouvée, ajouter un commentaire avec les findings
# Exemple :
# gh issue comment <NUMBER> -R gilmry/koprogo --body "## Résultat tests E2E (22/03/2026)
# Ce point a été identifié comme MANQUANT lors des tests E2E manuels.
# Voir rapport complet : rapport-tests-e2e-koprogo.docx
# Priorité : [CRITIQUE/MAJEUR/MINEUR]"
```

### Étape 8 : Mettre à jour le WBS Release v0.1.0

```bash
# Vérifier l'existence du fichier WBS
cat docs/WBS_RELEASE_0_1_0.md | head -50

# Les issues créées ci-dessus doivent être ajoutées au WBS dans la section appropriée.
# Mettre à jour les métriques de couverture :
# - Matrice conformité : 67% (25/37 conforme) → objectif 90% pour v0.1.0
# - RGPD : 60% (6/10 articles) → objectif 80% pour v0.1.0
# - Tests E2E frontend : identifier les pages testées vs non testées
```

### Étape 9 : Ajouter au projet GitHub (board)

```bash
# Récupérer l'ID du projet "KoproGo Software Roadmap"
PROJECT_ID=$(gh project list --owner gilmry --format json | jq -r '.projects[] | select(.title | contains("Software")) | .number')

# Pour chaque issue créée, l'ajouter au projet
# gh project item-add $PROJECT_ID --owner gilmry --url <ISSUE_URL>
```

### Résumé des issues à créer

| # | Titre | Sévérité | Milestone |
|---|---|---|---|
| 1 | Permissions rôles | MAJEUR | Jalon 1 |
| 2 | Isolation multi-tenant | CRITIQUE | Jalon 1 |
| 3 | Calcul tantièmes ≠ 1000 | MAJEUR | Jalon 2 |
| 4 | Pages en anglais | COSMÉTIQUE | — |
| 5 | Bouton ticket silencieux | MAJEUR | — |
| 6 | Validation tantièmes >100% | CRITIQUE | Jalon 2 |
| 7 | Immeubles non chargés | MAJEUR | — |
| 8 | Label sondages EN | COSMÉTIQUE | — |
| 9 | Chaîne approbation dépenses | ARCHITECTURE | Jalon 3 |
| 10 | Lien agenda-résolutions | CONFORMITÉ | Jalon 2 |
| 11 | Quorum + 2ème convocation | CONFORMITÉ | Jalon 2 |
| 12 | Procurations max 3 | CONFORMITÉ | Jalon 2 |
| 13 | Distribution PV 30j | CONFORMITÉ | Jalon 2 |
| 14 | Mandat syndic max 3 ans | CONFORMITÉ | Jalon 2 |
| 15 | Politique confidentialité | RGPD | Jalon 1 |
| 16 | DPA sous-traitants | RGPD | Jalon 1 |
| 17 | Notification violation 72h | RGPD | Jalon 1 |

**Total : 17 issues à créer, couvrant 3 milestones.**
