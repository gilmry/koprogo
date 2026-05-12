# Prompt Claude Code — Synchronisation GitHub ↔ docs/github-export & Création d'issues

> **Usage** : Copier ce prompt dans une session Claude Code dans le dépôt `gilmry/koprogo` avec `gh` authentifié.
> **Prérequis** : `gh auth status` OK, `jq` installé, accès internet.

---

## Contexte

Le dépôt KoproGo dispose d'un système d'export GitHub → RST dans `docs/github-export/`. Le script `scripts/export-github-to-rst.sh` (630 lignes) génère des fichiers `.rst` à partir des issues, milestones, projets et labels GitHub. Le dernier export date du 2025-11-18, il est donc obsolète par rapport à l'état actuel du projet (mars 2026).

Suite aux tests E2E manuels du 22/03/2026, un rapport (`rapport-tests-e2e-koprogo.md`) a identifié 8 bugs, 1 GAP architectural et 10 items de conformité manquants. Ces éléments doivent être créés comme issues GitHub puis l'export RST doit être resynchronisé.

**Fichiers clés :**
- `scripts/export-github-to-rst.sh` — Script d'export existant (ne pas modifier)
- `docs/github-export/` — Dossier de sortie RST (sera régénéré)
- `rapport-tests-e2e-koprogo.md` — Rapport de tests source
- `prompt-gh-update-issues.md` — Détail des 17 issues à créer
- `docs/WBS_RELEASE_0_1_0.md` — WBS release v0.1.0
- `docs/WBS_PROJET_COMPLET.rst` — WBS général du projet

---

## Instructions

Tu es un assistant Claude Code. Exécute les étapes suivantes dans l'ordre. Utilise `gh` CLI pour toutes les opérations GitHub.

---

### PHASE 1 : Audit de l'état actuel

```bash
# 1.1 Vérifier l'authentification
gh auth status

# 1.2 Compter les issues actuelles vs exportées
echo "=== Issues GitHub actuelles ==="
gh issue list -R gilmry/koprogo --state all --limit 500 --json number | jq length
echo "=== Issues dans docs/github-export ==="
ls docs/github-export/issues/issue-*.rst 2>/dev/null | wc -l

# 1.3 Lister les issues créées après le dernier export (2025-11-18)
gh issue list -R gilmry/koprogo --state all --limit 200 \
  --json number,title,state,createdAt,labels,milestone \
  --jq '.[] | select(.createdAt > "2025-11-18") | "\(.number)\t\(.state)\t\(.title)"'

# 1.4 Vérifier les milestones actuels
gh api repos/gilmry/koprogo/milestones --jq '.[] | "\(.number)\t\(.title)\t\(.open_issues)/\(.open_issues + .closed_issues)"'

# 1.5 Vérifier les labels existants (pour ne pas doublonner)
gh label list -R gilmry/koprogo --limit 100 --json name --jq '.[].name' | sort
```

**Après cet audit, note les écarts :**
- Nombre d'issues GitHub vs nombre de fichiers `.rst` dans `docs/github-export/issues/`
- Issues sans fichier RST correspondant
- Fichiers RST orphelins (issue supprimée sur GitHub)

---

### PHASE 2 : Créer les labels manquants

Vérifie d'abord si ces labels existent, puis crée-les seulement si absents :

```bash
# Labels de sévérité bugs
for label_info in "bug:critique:922B21:Bug critique bloquant" \
                  "bug:majeur:7D6608:Bug majeur fonctionnel" \
                  "bug:mineur:1E8449:Bug mineur" \
                  "conformité:5B2C6F:Conformité légale belge" \
                  "rgpd:1A5276:RGPD/GDPR compliance" \
                  "test:e2e:2E86C1:Tests E2E manuels" \
                  "architecture:6C3483:Architecture et Design"; do
  IFS=':' read -r name color desc <<< "$label_info"
  if ! gh label list -R gilmry/koprogo --json name --jq '.[].name' | grep -qx "$name"; then
    gh label create "$name" --color "$color" --description "$desc" -R gilmry/koprogo
    echo "✅ Label créé : $name"
  else
    echo "⏭️  Label existant : $name"
  fi
done
```

---

### PHASE 3 : Créer les 17 issues du rapport de tests E2E

**IMPORTANT** : Avant de créer chaque issue, vérifie qu'une issue similaire n'existe pas déjà :

```bash
# Rechercher les doublons potentiels
gh issue list -R gilmry/koprogo --search "multi-tenant isolation" --json number,title
gh issue list -R gilmry/koprogo --search "tantièmes validation" --json number,title
# etc.
```

#### 3.1 — Bugs (8 issues)

**Bug #B1 — Permissions rôles (MAJEUR)**
```bash
gh issue create -R gilmry/koprogo \
  --title "[Bug] Permissions rôles : boutons admin visibles pour le syndic" \
  --label "bug:majeur,test:e2e" \
  --milestone "Jalon 1: Sécurité & GDPR" \
  --body "$(cat <<'ISSUE_EOF'
## Description
Le syndic voit les boutons **Créer Organisation** et **Créer Utilisateur** dans le sidebar.
Ces actions sont réservées au rôle SuperAdmin.

## Impact
Confusion UX. Les endpoints backend retournent 403, mais l'UI ne devrait pas exposer ces actions.

## Reproduction
1. Se connecter en tant que syndic (jean.dupont@koprogo.be)
2. Observer le sidebar → boutons admin visibles

## Correction proposée
Filtrer les éléments de navigation par `active_role` dans `Navigation.svelte`.

## Origine
Tests E2E manuels — 22/03/2026 (rapport-tests-e2e-koprogo.md, Bug #1)
ISSUE_EOF
)"
```

**Bug #B2 — Isolation multi-tenant (CRITIQUE)**
```bash
gh issue create -R gilmry/koprogo \
  --title "[Bug] CRITIQUE : Isolation multi-tenant — données non filtrées par organization_id" \
  --label "bug:critique,test:e2e" \
  --milestone "Jalon 1: Sécurité & GDPR" \
  --body "$(cat <<'ISSUE_EOF'
## Description
Plusieurs endpoints retournent les données de TOUTES les organisations au lieu de filtrer par `organization_id`.

## Impact
**CRITIQUE** — Fuite de données inter-tenants. Violation RGPD potentielle.

## Endpoints concernés
- `GET /buildings` (retourne les immeubles de toutes les organisations)
- `GET /units` (idem — vérifié manuellement)
- Potentiellement d'autres à auditer (511 endpoints au total)

## Reproduction
1. Se connecter en syndic org A
2. GET /api/v1/units → affiche les lots de l'org B

## Correction proposée
Ajouter `WHERE organization_id = $user.organization_id` à TOUS les handlers de listing.
Audit systématique des 511 endpoints.

## Origine
Tests E2E — 22/03/2026 (rapport-tests-e2e-koprogo.md, Bug #2)
ISSUE_EOF
)"
```

**Bug #B3 — Calcul tantièmes (MAJEUR)**
```bash
gh issue create -R gilmry/koprogo \
  --title "[Bug] Calcul tantièmes : total des lots ≠ total immeuble (1000 millièmes)" \
  --label "bug:majeur,conformité,test:e2e" \
  --milestone "Jalon 2: Conformité Légale Belge" \
  --body "$(cat <<'ISSUE_EOF'
## Description
Aucune validation frontend/backend ne vérifie que la somme des tantièmes des lots = 1000 millièmes.

## Impact
Non-conformité Art. 577-2 §4 Code Civil belge. Calculs de quorum et charges erronés.

## Correction proposée
1. Ajouter champ `total_shares` au Building (défaut: 1000)
2. Valider à la création/modification d'un lot que sum(unit.shares) <= building.total_shares
3. Afficher warning si sum < total

## Origine
Tests E2E — 22/03/2026 (rapport-tests-e2e-koprogo.md, Bug #3)
ISSUE_EOF
)"
```

**Bug #B4 — Pages en anglais (COSMÉTIQUE)**
```bash
gh issue create -R gilmry/koprogo \
  --title "[Bug] Pages non traduites : Tickets, Announcements, Bookings en anglais" \
  --label "bug:mineur,test:e2e" \
  --body "$(cat <<'ISSUE_EOF'
## Description
Pages Tickets, Annonces et Réservations restent en anglais ('Create New Ticket', 'Community Notice Board', 'Resource Booking Calendar').

## Correction
Traduire tous les textes statiques. Idéalement implémenter i18n (FR/NL/DE/EN).

## Origine
Tests E2E — 22/03/2026 (rapport-tests-e2e-koprogo.md, Bug #4)
ISSUE_EOF
)"
```

**Bug #B5 — Bouton créer ticket silencieux (MAJEUR)**
```bash
gh issue create -R gilmry/koprogo \
  --title "[Bug] Création ticket : bouton silencieux si building_id manquant" \
  --label "bug:majeur,test:e2e" \
  --body "$(cat <<'ISSUE_EOF'
## Description
Le bouton 'Create New Ticket' sur /tickets ne fait rien quand `building_id` n'est pas en query param.
`showPageToast()` est appelé mais ne s'affiche pas.

## Reproduction
1. Aller sur /tickets (sans ?building_id=xxx)
2. Cliquer 'Create New Ticket' → rien ne se passe

## Correction
1. Ajouter sélecteur d'immeuble (dropdown) comme sur /devis, /travaux
2. Corriger `showPageToast`
3. Désactiver le bouton si aucun immeuble sélectionné

## Fichier concerné
`frontend/src/pages/tickets.astro` lignes 93-121

## Origine
Tests E2E — 22/03/2026 (rapport-tests-e2e-koprogo.md, Bug #5)
ISSUE_EOF
)"
```

**Bug #B6 — Validation tantièmes >100% (CRITIQUE)**
```bash
gh issue create -R gilmry/koprogo \
  --title "[Bug] CRITIQUE : Validation tantièmes — dépassement 100% possible" \
  --label "bug:critique,conformité,test:e2e" \
  --milestone "Jalon 2: Conformité Légale Belge" \
  --body "$(cat <<'ISSUE_EOF'
## Description
Le trigger PostgreSQL `validate_unit_ownership_total` avertit si total < 100% mais ne bloque que > 100%.
En ajoutant des propriétaires séquentiellement, on peut arriver à un état incohérent.

## Impact
**CRITIQUE** — Calculs de charges et votes erronés. Non-conformité Art. 577-2 §4 CC.

## Correction
1. Validation frontend en temps réel
2. Renforcer le trigger PostgreSQL avec validation transactionnelle
3. Utiliser l'endpoint GET /units/:id/owners/total-percentage

## Origine
Tests E2E — 22/03/2026 (rapport-tests-e2e-koprogo.md, Bug #6)
ISSUE_EOF
)"
```

**Bug #B7 — Chargement immeubles (MAJEUR)**
```bash
gh issue create -R gilmry/koprogo \
  --title "[Bug] Sondages/Annonces/Réservations : immeubles non chargés" \
  --label "bug:majeur,test:e2e" \
  --body "$(cat <<'ISSUE_EOF'
## Description
Pages /polls, /announcements, /bookings affichent « Erreur lors du chargement des immeubles ».

## Correction
Corriger l'appel API pour charger les immeubles filtrés par organization_id.

## Origine
Tests E2E — 22/03/2026 (rapport-tests-e2e-koprogo.md, Bug #7)
ISSUE_EOF
)"
```

**Bug #B8 — Label sondages EN (COSMÉTIQUE)**
```bash
gh issue create -R gilmry/koprogo \
  --title "[Bug] Sondages : label 'Building' au lieu de 'Immeuble'" \
  --label "bug:mineur,test:e2e" \
  --body "$(cat <<'ISSUE_EOF'
## Description
Formulaire création sondage : label 'Building' en anglais au lieu de 'Immeuble'.

## Correction
Remplacer par 'Immeuble' pour cohérence FR.

## Origine
Tests E2E — 22/03/2026 (rapport-tests-e2e-koprogo.md, Bug #8)
ISSUE_EOF
)"
```

#### 3.2 — GAP Architecture (1 issue)

```bash
gh issue create -R gilmry/koprogo \
  --title "[Architecture] Connecter la chaîne d'approbation des dépenses (Ticket → Rapport → Validation → Dépense)" \
  --label "architecture,conformité" \
  --milestone "Jalon 3: Features Différenciantes" \
  --body "$(cat <<'ISSUE_EOF'
## Description
Les modules Ticket, ContractorReport, BoardDecision et Expense existent individuellement mais ne sont PAS connectés dans un workflow métier.

## Workflow attendu
1. **Ticket créé** → Ordre de service généré
2. **Ordre de service accepté** → Magic link PWA envoyé au prestataire
3. **Prestataire soumet rapport** via PWA (/contractor/?token=XXX)
4. **Rapport validé** par CdC (si >20 lots) ou Syndic (si ≤20 lots)
5. **Dépense approuvée** → Écriture comptable automatique + paiement déclenché

## Tâches
- [ ] Créer l'entité WorkOrder (lien Ticket → ContractorReport)
- [ ] Ajouter FK contractor_report_id à Expense
- [ ] Rendre ContractorReport obligatoire avant Expense.approve()
- [ ] Ajouter FK board_decision_id à Expense si immeuble >20 lots
- [ ] Auto-trigger magic link PWA à l'acceptation de l'ordre de service
- [ ] Créer BuildingContractor (registre fournisseurs officiels ACP)

## Réf.
rapport-tests-e2e-koprogo.md — Section 3.3
ISSUE_EOF
)"
```

#### 3.3 — Conformité AG (5 issues)

```bash
gh issue create -R gilmry/koprogo \
  --title "[Conformité] AG : Lien agenda-résolutions — bloquer votes hors ODJ" \
  --label "conformité" \
  --milestone "Jalon 2: Conformité Légale Belge" \
  --body "$(cat <<'ISSUE_EOF'
## Exigence légale
Art. 3.87 CC : seuls les points inscrits à l'ODJ peuvent faire l'objet d'un vote.
Vote hors ODJ = nullité.

## Implémentation
- Ajouter FK resolution.agenda_item_id → meeting_agenda_items
- Bloquer création de résolutions sans point d'agenda
- BDD scenario dans features/meetings.feature

## Statut matrice
MANQUANT (docs/legal/matrice_conformite.rst)
ISSUE_EOF
)"

gh issue create -R gilmry/koprogo \
  --title "[Conformité] AG : Quorum 50%+50% et 2ème convocation automatique" \
  --label "conformité" \
  --milestone "Jalon 2: Conformité Légale Belge" \
  --body "$(cat <<'ISSUE_EOF'
## Exigence légale
Art. 3.87 §5 CC : Si le quorum (>50% des quotes-parts) n'est pas atteint, 2ème convocation obligatoire (15j minimum).
À la 2ème convocation, aucun quorum requis.

## Implémentation
- Ajouter champ meeting.is_second_convocation (boolean)
- Valider quorum seulement si is_second_convocation = false
- Auto-créer Meeting + Convocation de 2ème appel si quorum KO

## Statut matrice
MANQUANT (docs/legal/matrice_conformite.rst)
ISSUE_EOF
)"

gh issue create -R gilmry/koprogo \
  --title "[Conformité] AG : Procurations — max 3 mandats + exception 10%" \
  --label "conformité" \
  --milestone "Jalon 2: Conformité Légale Belge" \
  --body "$(cat <<'ISSUE_EOF'
## Exigence légale
Art. 3.87 §7 CC : Max 3 procurations par mandataire, sauf s'il ne dépasse pas 10% des voix totales.

## Implémentation
- Compter procurations par proxy_owner_id lors du vote
- Bloquer si > 3 ET > 10% des voix
- Afficher procurations restantes dans l'UI

## Statut matrice
MANQUANT (docs/legal/matrice_conformite.rst)
ISSUE_EOF
)"

gh issue create -R gilmry/koprogo \
  --title "[Conformité] AG : Distribution PV dans les 30 jours + génération automatique" \
  --label "conformité" \
  --milestone "Jalon 2: Conformité Légale Belge" \
  --body "$(cat <<'ISSUE_EOF'
## Exigence légale
PV distribué à tous les copropriétaires dans les 30 jours suivant l'AG.

## Implémentation
- Génération auto du PV à la clôture (résolutions, votes, quorum, présences)
- Tracking date d'envoi du PV
- Alerte si PV non envoyé après 30 jours
- Todo list auto pour le CdC basée sur les décisions votées

## Statut matrice
MANQUANT (docs/legal/matrice_conformite.rst)
ISSUE_EOF
)"

gh issue create -R gilmry/koprogo \
  --title "[Conformité] Syndic : Mandat max 3 ans avec validation" \
  --label "conformité" \
  --milestone "Jalon 2: Conformité Légale Belge" \
  --body "$(cat <<'ISSUE_EOF'
## Exigence légale
Art. 3.89 CC : Mandat du syndic max 3 ans, renouvelable.

## Implémentation
- Validation domain : mandate_end - mandate_start <= 3 ans
- Alertes avant expiration (3 mois, 1 mois, 15 jours)
- Point AGO obligatoire pour renouvellement

## Statut matrice
MANQUANT (docs/legal/matrice_conformite.rst)
ISSUE_EOF
)"
```

#### 3.4 — RGPD (3 issues)

```bash
gh issue create -R gilmry/koprogo \
  --title "[RGPD] Art. 13-14 : Publier politique de confidentialité" \
  --label "rgpd" \
  --milestone "Jalon 1: Sécurité & GDPR" \
  --body "$(cat <<'ISSUE_EOF'
## Exigence
RGPD Art. 13-14 : Information obligatoire des personnes concernées.

## Actions
- Rédiger la politique de confidentialité (FR/NL)
- Créer page publique /privacy-policy
- Lien dans le footer
- Formulaire de consentement au premier login

## Risque
Sanctions APD jusqu'à 20M€ ou 4% CA (avg. 18 000€ PME belge).

## Statut matrice
MANQUANT (docs/legal/rgpd_conformite.rst)
ISSUE_EOF
)"

gh issue create -R gilmry/koprogo \
  --title "[RGPD] Art. 28 : DPA avec sous-traitants (Stripe, AWS S3, email)" \
  --label "rgpd" \
  --milestone "Jalon 1: Sécurité & GDPR" \
  --body "$(cat <<'ISSUE_EOF'
## Exigence
RGPD Art. 28 : Contrat de sous-traitance obligatoire avec tout processeur de données.

## Sous-traitants identifiés
- Stripe (paiements)
- AWS S3 (stockage backups/documents)
- Fournisseur email (notifications)
- Fournisseur SMS (si activé)

## Actions
- Signer les DPA avec chaque sous-traitant
- Documenter dans le registre de traitement (Art. 30)
- Vérifier clauses transfert hors UE

## Statut matrice
MANQUANT (docs/legal/rgpd_conformite.rst)
ISSUE_EOF
)"

gh issue create -R gilmry/koprogo \
  --title "[RGPD] Art. 33 : Procédure notification violation de données (72h)" \
  --label "rgpd" \
  --milestone "Jalon 1: Sécurité & GDPR" \
  --body "$(cat <<'ISSUE_EOF'
## Exigence
RGPD Art. 33 : Notification de l'APD dans les 72h en cas de violation.

## Actions
- Documenter la procédure d'incident response
- Créer template de notification APD
- Implémenter endpoint interne /admin/security-incident
- Exercice de simulation annuel

## Statut matrice
MANQUANT (docs/legal/rgpd_conformite.rst)
ISSUE_EOF
)"
```

---

### PHASE 4 : Mettre à jour les issues existantes avec commentaire E2E

```bash
# Trouver les issues existantes liées aux findings et ajouter un commentaire
# Adapter les numéros selon ce que gh issue list retourne

# Exemple : si une issue #78 existe déjà pour la sécurité/GDPR
RELATED_ISSUES=$(gh issue list -R gilmry/koprogo --search "GDPR" --state open --json number --jq '.[].number')
for issue_num in $RELATED_ISSUES; do
  gh issue comment "$issue_num" -R gilmry/koprogo --body "## 🧪 Résultat tests E2E (22/03/2026)

Ce point a été vérifié lors de la campagne de tests E2E manuels du 22/03/2026.
Voir le rapport complet : \`rapport-tests-e2e-koprogo.md\`

**Score matrice conformité RGPD** : 60% (6/10 articles implémentés)
**Items MANQUANTS** : Art. 13-14, Art. 28, Art. 32 (partiel), Art. 33"
done

# Idem pour les issues liées à la conformité AG
RELATED_AG=$(gh issue list -R gilmry/koprogo --search "quorum OR assemblée OR AG" --state open --json number --jq '.[].number')
for issue_num in $RELATED_AG; do
  gh issue comment "$issue_num" -R gilmry/koprogo --body "## 🧪 Résultat tests E2E (22/03/2026)

Point vérifié lors des tests E2E du 22/03/2026.
**Score conformité AG** : 72% (18/25 exigences légales)
**7 items MANQUANTS** : lien agenda-résolutions, quorum 50%+50%, 2ème convocation, procurations max 3, PV 30j, mandat syndic 3 ans, quorum 3/4.
Voir \`rapport-tests-e2e-koprogo.md\` section 4.3 et \`docs/legal/matrice_conformite.rst\`"
done
```

---

### PHASE 5 : Resynchroniser l'export docs/github-export

```bash
# 5.1 Lancer l'export complet (régénère TOUT le dossier)
./scripts/export-github-to-rst.sh

# 5.2 Vérifier le résultat
echo "=== Nouvelles stats ==="
cat docs/github-export/stats.rst
echo ""
echo "=== Nombre de fichiers issues ==="
ls docs/github-export/issues/issue-*.rst | wc -l
echo ""
echo "=== Dernières issues exportées ==="
ls -lt docs/github-export/issues/ | head -20

# 5.3 Vérifier la cohérence
GITHUB_COUNT=$(gh issue list -R gilmry/koprogo --state all --limit 500 --json number | jq length)
RST_COUNT=$(ls docs/github-export/issues/issue-*.rst 2>/dev/null | wc -l)
echo "GitHub: $GITHUB_COUNT issues | RST: $RST_COUNT fichiers"
if [ "$GITHUB_COUNT" -ne "$RST_COUNT" ]; then
  echo "⚠️  ÉCART DÉTECTÉ — vérifier le script d'export"
else
  echo "✅ Synchronisation OK"
fi
```

---

### PHASE 6 : Mettre à jour le WBS

#### 6.1 WBS Release v0.1.0

Ouvrir `docs/WBS_RELEASE_0_1_0.md` et mettre à jour :

```bash
# Lire les métriques actuelles
head -100 docs/WBS_RELEASE_0_1_0.md

# Ajouter les nouvelles issues aux sections pertinentes :
# - Section "Jalon 1: Sécurité & GDPR" → ajouter Bug #B1, #B2, issues RGPD
# - Section "Jalon 2: Conformité Légale Belge" → ajouter Bug #B3, #B6, issues conformité AG
# - Section "Jalon 3: Features Différenciantes" → ajouter issue chaîne dépenses
#
# Mettre à jour les métriques de couverture :
# - Matrice conformité : 67% → objectif 90% pour v0.1.0
# - RGPD : 60% → objectif 80% pour v0.1.0
# - Tests E2E manuels : 20+ pages testées / ~30 pages totales
```

#### 6.2 WBS Projet Complet

```bash
# Vérifier la structure du WBS global
head -80 docs/WBS_PROJET_COMPLET.rst

# Ajouter une section "Tests E2E — Mars 2026" dans le jalon correspondant
# avec les résultats de la matrice de conformité (67%, 10 items MANQUANTS)
```

---

### PHASE 7 : Commit & vérification finale

```bash
# 7.1 Vérifier tous les changements
git status
git diff --stat

# 7.2 Commit les docs régénérées
git add docs/github-export/
git add rapport-tests-e2e-koprogo.md
git add prompt-gh-update-issues.md
git add prompt-sync-github-docs.md

git commit -m "$(cat <<'EOF'
docs: resynchroniser github-export + ajouter rapport tests E2E

- Régénéré docs/github-export/ avec les 17 nouvelles issues
- Ajouté rapport-tests-e2e-koprogo.md (8 bugs, 1 GAP architecture, 10 items conformité)
- Score matrice conformité : 67% (25/37)
- Score RGPD : 60% (6/10 articles)
- 17 issues créées : 8 bugs, 1 architecture, 5 conformité AG, 3 RGPD
EOF
)"

# 7.3 Vérification finale
echo "=== Résumé ==="
echo "Issues GitHub total : $(gh issue list -R gilmry/koprogo --state all --limit 500 --json number | jq length)"
echo "Issues ouvertes : $(gh issue list -R gilmry/koprogo --state open --limit 500 --json number | jq length)"
echo "Fichiers RST : $(ls docs/github-export/issues/issue-*.rst | wc -l)"
echo "Milestones : $(gh api repos/gilmry/koprogo/milestones | jq length)"
echo "Labels : $(gh label list -R gilmry/koprogo --limit 100 --json name | jq length)"
```

---

## Résumé des 17 issues à créer

| # | Type | Titre | Sévérité | Milestone |
|---|---|---|---|---|
| B1 | Bug | Permissions rôles | MAJEUR | Jalon 1 |
| B2 | Bug | Isolation multi-tenant | CRITIQUE | Jalon 1 |
| B3 | Bug | Calcul tantièmes ≠ 1000 | MAJEUR | Jalon 2 |
| B4 | Bug | Pages en anglais | COSMÉTIQUE | — |
| B5 | Bug | Bouton ticket silencieux | MAJEUR | — |
| B6 | Bug | Validation tantièmes >100% | CRITIQUE | Jalon 2 |
| B7 | Bug | Immeubles non chargés | MAJEUR | — |
| B8 | Bug | Label sondages EN | COSMÉTIQUE | — |
| A1 | Archi | Chaîne approbation dépenses | GAP CRITIQUE | Jalon 3 |
| C1 | Conf | Lien agenda-résolutions | CONFORMITÉ | Jalon 2 |
| C2 | Conf | Quorum + 2ème convocation | CONFORMITÉ | Jalon 2 |
| C3 | Conf | Procurations max 3 | CONFORMITÉ | Jalon 2 |
| C4 | Conf | Distribution PV 30j | CONFORMITÉ | Jalon 2 |
| C5 | Conf | Mandat syndic max 3 ans | CONFORMITÉ | Jalon 2 |
| R1 | RGPD | Politique confidentialité | RGPD | Jalon 1 |
| R2 | RGPD | DPA sous-traitants | RGPD | Jalon 1 |
| R3 | RGPD | Notification violation 72h | RGPD | Jalon 1 |

**Total : 17 issues → 3 milestones (Jalon 1, 2, 3) + 4 sans milestone**
