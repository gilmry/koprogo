# 📦 KoproGo Frontend - Inventaire Complet & Câblage

**Date**: 6 Décembre 2025
**Version**: 1.0
**Frontend**: Astro 4.x + Svelte 5.x
**Total Pages**: 67 Astro pages
**Total Components**: 130+ Svelte components

---

## 🎯 Architecture Frontend

```
frontend/
├── src/
│   ├── components/          (130+ Svelte components)
│   │   ├── admin/           (7 components - SUPERADMIN only)
│   │   ├── dashboards/      (4 components - Role-specific)
│   │   ├── bookings/        (4 components - Community)
│   │   ├── convocations/    (1 component - Syndic)
│   │   ├── energy-campaigns/(7 components - Community)
│   │   ├── local-exchanges/ (7 components - SEL System)
│   │   ├── notices/         (5 components - Community)
│   │   ├── notifications/   (5 components - All roles)
│   │   ├── payments/        (6 components - Owner/Syndic)
│   │   ├── polls/           (6 components - Community)
│   │   ├── quotes/          (2 components - Syndic)
│   │   ├── sharing/         (6 components - Community)
│   │   ├── skills/          (6 components - Community)
│   │   ├── tickets/         (7 components - Owner/Syndic)
│   │   ├── ui/              (10 components - Shared UI)
│   │   └── [35+ core components]
│   ├── layouts/             (1 layout - Layout.astro)
│   ├── pages/               (67 Astro pages)
│   │   ├── admin/           (9 pages - SUPERADMIN)
│   │   ├── syndic/          (2 pages - SYNDIC)
│   │   ├── accountant/      (1 page - ACCOUNTANT)
│   │   ├── owner/           (9 pages - OWNER)
│   │   ├── settings/        (2 pages - All roles)
│   │   └── [44+ shared pages]
│   ├── stores/              (3 stores - auth, notifications, toast)
│   ├── lib/
│   │   ├── api/             (20+ API clients)
│   │   ├── types.ts         (Type definitions)
│   │   └── config.ts        (API config)
│   └── locales/             (i18n - FR only, NL/DE/EN TODO)
```

---

## 🔐 Système d'Authentification & Rôles

### **UserRole Enum** (`src/lib/types.ts`)

```typescript
export enum UserRole {
  SUPERADMIN = "superadmin",  // Admin plateforme
  SYNDIC = "syndic",           // Syndic
  ACCOUNTANT = "accountant",   // Comptable
  OWNER = "owner",             // Copropriétaire
}
```

### **Auth Store** (`src/stores/auth.ts`)

**Fonctionnalités**:
- ✅ Login/Logout
- ✅ Multi-role support (user.roles[])
- ✅ Role switching (authStore.switchRole())
- ✅ Token refresh automatique (10 min)
- ✅ Session validation
- ✅ IndexedDB local cache
- ✅ Sync service integration

**API Endpoints**:
- `POST /api/v1/auth/login` → token + refresh_token + user
- `POST /api/v1/auth/refresh` → new tokens
- `POST /api/v1/auth/switch-role` → switch active role
- `GET /api/v1/auth/me` → current user profile

---

## 🧭 Navigation Component

**Fichier**: [src/components/Navigation.svelte](../frontend/src/components/Navigation.svelte)

### **Navigation par Rôle**

#### **SUPERADMIN** (8 items)
```typescript
[
  { href: '/admin', label: 'Admin', icon: '⚙️' },
  { href: '/admin/monitoring', label: 'Monitoring', icon: '📈' },
  { href: '/buildings', label: 'Bâtiments', icon: '🏢' },
  { href: '/admin/organizations', label: 'Organisations', icon: '🏛️' },
  { href: '/admin/users', label: 'Utilisateurs', icon: '👥' },
  { href: '/admin/board-members', label: 'Conseil', icon: '👑' },
  { href: '/admin/gdpr', label: 'RGPD', icon: '🔒' },
]
```

#### **SYNDIC** (11 items)
```typescript
[
  { href: '/syndic', label: 'Dashboard', icon: '📊' },
  { href: '/buildings', label: 'Bâtiments', icon: '🏢' },
  { href: '/owners', label: 'Propriétaires', icon: '👤' },
  { href: '/units', label: 'Lots', icon: '🚪' },
  { href: '/expenses', label: 'Dépenses', icon: '💰' },
  { href: '/invoice-workflow', label: 'Workflow factures', icon: '✅' },
  { href: '/call-for-funds', label: 'Appels de fonds', icon: '📢' },
  { href: '/owner-contributions', label: 'Contributions', icon: '💶' },
  { href: '/payment-reminders', label: 'Relances', icon: '📧' },
  { href: '/meetings', label: 'Réunions', icon: '📅' },
  { href: '/syndic/board-members', label: 'Conseil', icon: '👑' },
  { href: '/documents', label: 'Documents', icon: '📄' },
]
```

#### **ACCOUNTANT** (8 items)
```typescript
[
  { href: '/accountant', label: 'Dashboard', icon: '📊' },
  { href: '/buildings', label: 'Bâtiments', icon: '🏢' },
  { href: '/expenses', label: 'Dépenses', icon: '💰' },
  { href: '/invoice-workflow', label: 'Workflow factures', icon: '✅' },
  { href: '/call-for-funds', label: 'Appels de fonds', icon: '📢' },
  { href: '/owner-contributions', label: 'Contributions', icon: '💶' },
  { href: '/payment-reminders', label: 'Relances', icon: '📧' },
  { href: '/journal-entries', label: 'Écritures comptables', icon: '📒' },
  { href: '/reports', label: 'Rapports PCMN', icon: '📈' },
]
```

#### **OWNER** (4 items)
```typescript
[
  { href: '/owner', label: 'Dashboard', icon: '🏠' },
  { href: '/owner/units', label: 'Mes lots', icon: '🚪' },
  { href: '/owner/expenses', label: 'Mes charges', icon: '💰' },
  { href: '/owner/documents', label: 'Documents', icon: '📄' },
]
```

### **Menu Utilisateur** (Tous rôles)
```typescript
[
  { href: '/profile', label: 'Profil', icon: '👤' },
  { href: '/settings', label: 'Paramètres', icon: '⚙️' },
  { href: '/settings/gdpr', label: 'Mes données RGPD', icon: '🔒' },
  { action: 'logout', label: 'Déconnexion', icon: '🚪' },
]
```

---

## 📄 Pages Astro (67 pages)

### **Pages Publiques** (3 pages - No Auth)
| Page | Route | Description |
|------|-------|-------------|
| `index.astro` | `/` | Landing page (redirect si auth) |
| `login.astro` | `/login` | Formulaire connexion |
| `register.astro` | `/register` | Inscription nouveau utilisateur |
| `mentions-legales.astro` | `/mentions-legales` | Mentions légales |

---

### **Admin Pages** (9 pages - SUPERADMIN only) 🔴

| Page | Route | Component(s) | Description | Menu |
|------|-------|-------------|-------------|------|
| `admin/index.astro` | `/admin` | `AdminDashboard.svelte` | Dashboard admin plateforme | ✅ Nav |
| `admin/monitoring.astro` | `/admin/monitoring` | - | Grafana/Prometheus metrics | ✅ Nav |
| `admin/organizations.astro` | `/admin/organizations` | `OrganizationList.svelte`, `OrganizationForm.svelte` | Gestion organisations | ✅ Nav |
| `admin/users.astro` | `/admin/users` | `UserListAdmin.svelte`, `UserForm.svelte` | Gestion utilisateurs | ✅ Nav |
| `admin/board-members.astro` | `/admin/board-members` | `BoardManagement.svelte` | Gestion conseil copropriété | ✅ Nav |
| `admin/gdpr.astro` | `/admin/gdpr` | `AdminGdprPanel.svelte` | Export GDPR masse | ✅ Nav |
| `admin/seed.astro` | `/admin/seed` | `SeedManager.svelte` | Seed data test | ❌ Hidden |
| `admin/subscriptions.astro` | `/admin/subscriptions` | - | Gestion abonnements | ❌ Hidden |
| `admin/user-owner-links.astro` | `/admin/user-owner-links` | `UserOwnerLinkManager.svelte` | Link users ↔ owners | ❌ Hidden |

---

### **Syndic Pages** (2 pages - SYNDIC only) 🟡

| Page | Route | Component(s) | Description | Menu |
|------|-------|-------------|-------------|------|
| `syndic/index.astro` | `/syndic` | `SyndicDashboard.svelte` | Dashboard syndic | ✅ Nav |
| `syndic/board-members.astro` | `/syndic/board-members` | `BoardMemberList.svelte` | Conseil copropriété | ✅ Nav |

---

### **Accountant Pages** (1 page - ACCOUNTANT only) 🟢

| Page | Route | Component(s) | Description | Menu |
|------|-------|-------------|-------------|------|
| `accountant/index.astro` | `/accountant` | `AccountantDashboard.svelte` | Dashboard comptable | ✅ Nav |

---

### **Owner Pages** (9 pages - OWNER only) 🔵

| Page | Route | Component(s) | Description | Menu |
|------|-------|-------------|-------------|------|
| `owner/index.astro` | `/owner` | `OwnerDashboard.svelte` | Dashboard copropriétaire | ✅ Nav |
| `owner/units.astro` | `/owner/units` | `UnitList.svelte` | Mes lots | ✅ Nav |
| `owner/expenses.astro` | `/owner/expenses` | `ExpenseList.svelte` | Mes charges | ✅ Nav |
| `owner/documents.astro` | `/owner/documents` | `DocumentList.svelte` | Mes documents | ✅ Nav |
| `owner/payments.astro` | `/owner/payments` | `PaymentList.svelte`, `PaymentStats.svelte` | Mes paiements | ❌ TODO |
| `owner/payment-methods.astro` | `/owner/payment-methods` | `PaymentMethodList.svelte` | Mes moyens paiement | ❌ TODO |
| `owner/tickets.astro` | `/owner/tickets` | `TicketList.svelte` | Mes tickets | ❌ TODO |
| `owner/profile.astro` | `/owner/profile` | - | Mon profil | ❌ TODO |
| `owner/contact.astro` | `/owner/contact` | - | Contact syndic | ❌ TODO |

---

### **Shared Pages** (44 pages - Multi-role) 🟣

#### **Core Management** (SYNDIC, ACCOUNTANT)

| Page | Route | Component(s) | Description | Menu |
|------|-------|-------------|-------------|------|
| `buildings/index.astro` | `/buildings` | `BuildingList.svelte` | Liste bâtiments | ✅ All roles |
| `building-detail.astro` | `/building-detail` | `BuildingDetail.svelte`, `BuildingFinancialReports.svelte` | Détail bâtiment | ❌ Link |
| `owners.astro` | `/owners` | `OwnerList.svelte` | Liste propriétaires | ✅ Syndic |
| `units.astro` | `/units` | `UnitList.svelte` | Liste lots | ✅ Syndic |

#### **Financial** (SYNDIC, ACCOUNTANT)

| Page | Route | Component(s) | Description | Menu |
|------|-------|-------------|-------------|------|
| `expenses.astro` | `/expenses` | `ExpenseList.svelte` | Liste dépenses | ✅ Syndic, Acct |
| `expense-detail.astro` | `/expense-detail` | `ExpenseDetail.svelte`, `ExpenseDocuments.svelte` | Détail dépense | ❌ Link |
| `invoice-workflow.astro` | `/invoice-workflow` | `InvoiceWorkflow.svelte`, `InvoiceForm.svelte`, `InvoiceLineItems.svelte` | Workflow factures | ✅ Syndic, Acct |
| `call-for-funds.astro` | `/call-for-funds` | `CallForFundsList.svelte`, `CallForFundsForm.svelte` | Appels de fonds | ✅ Syndic, Acct |
| `owner-contributions.astro` | `/owner-contributions` | `OwnerContributionList.svelte`, `OwnerContributionForm.svelte` | Contributions | ✅ Syndic, Acct |
| `payment-reminders.astro` | `/payment-reminders` | `PaymentReminderList.svelte` | Relances paiement | ✅ Syndic, Acct |
| `payment-reminder-detail.astro` | `/payment-reminder-detail` | `PaymentReminderDetail.svelte` | Détail relance | ❌ Link |
| `journal-entries.astro` | `/journal-entries` | `JournalEntryForm.svelte` | Écritures comptables | ✅ Acct only |
| `reports.astro` | `/reports` | `FinancialReports.svelte` | Rapports PCMN | ✅ Acct only |

#### **Meetings & Governance** (SYNDIC, OWNER)

| Page | Route | Component(s) | Description | Menu |
|------|-------|-------------|-------------|------|
| `meetings.astro` | `/meetings` | `MeetingList.svelte` | Liste réunions AG | ✅ Syndic |
| `meeting-detail.astro` | `/meeting-detail` | `MeetingDetail.svelte`, `MeetingDocuments.svelte` | Détail AG | ❌ Link |
| `board-dashboard.astro` | `/board-dashboard` | `BoardDashboard.svelte`, `DecisionTracker.svelte` | Conseil copropriété | ❌ TODO |

#### **Documents** (SYNDIC, OWNER)

| Page | Route | Component(s) | Description | Menu |
|------|-------|-------------|-------------|------|
| `documents.astro` | `/documents` | `DocumentList.svelte`, `DocumentUploadModal.svelte` | Gestion documents | ✅ Syndic |

#### **Tickets** (SYNDIC, OWNER)

| Page | Route | Component(s) | Description | Menu |
|------|-------|-------------|-------------|------|
| `tickets.astro` | `/tickets` | `TicketList.svelte`, `TicketCreateModal.svelte` | Liste tickets | ❌ TODO Nav |
| `ticket-detail.astro` | `/ticket-detail` | `TicketDetail.svelte`, `TicketAssignModal.svelte`, `TicketStatistics.svelte` | Détail ticket | ❌ Link |

#### **Community - SEL** (ALL ROLES)

| Page | Route | Component(s) | Description | Menu |
|------|-------|-------------|-------------|------|
| `exchanges.astro` | `/exchanges` | `ExchangeList.svelte`, `CreditBalance.svelte`, `Leaderboard.svelte`, `SelStatistics.svelte` | SEL échanges | ❌ TODO Nav |
| `exchanges/new.astro` | `/exchanges/new` | `CreateExchangeForm.svelte` | Créer échange | ❌ Link |

#### **Community - Polls** (ALL ROLES)

| Page | Route | Component(s) | Description | Menu |
|------|-------|-------------|-------------|------|
| `polls.astro` | `/polls` | `PollList.svelte` | Liste sondages | ❌ TODO Nav |
| `polls/[id].astro` | `/polls/[id]` | `PollDetail.svelte`, `PollResults.svelte` | Détail sondage | ❌ Link |
| `polls/new.astro` | `/polls/new` | `CreatePollForm.svelte` | Créer sondage | ❌ Link |

#### **Community - Notices** (ALL ROLES)

| Page | Route | Component(s) | Description | Menu |
|------|-------|-------------|-------------|------|
| `notices.astro` | `/notices` | `NoticeList.svelte`, `NoticeCreateModal.svelte` | Tableau affichage | ❌ TODO Nav |
| `notice-detail.astro` | `/notice-detail` | `NoticeDetail.svelte` | Détail annonce | ❌ Link |

#### **Community - Bookings** (ALL ROLES)

| Page | Route | Component(s) | Description | Menu |
|------|-------|-------------|-------------|------|
| `bookings.astro` | `/bookings` | `ResourceList.svelte` | Réservations salles | ❌ TODO Nav |
| `booking-detail.astro` | `/booking-detail` | `ResourceCard.svelte` | Détail réservation | ❌ Link |

#### **Community - Sharing** (ALL ROLES)

| Page | Route | Component(s) | Description | Menu |
|------|-------|-------------|-------------|------|
| `sharing.astro` | `/sharing` | `SharedObjectList.svelte` | Partage objets | ❌ TODO Nav |
| `sharing-detail.astro` | `/sharing-detail` | `SharedObjectCard.svelte` | Détail objet | ❌ Link |

#### **Community - Skills** (ALL ROLES)

| Page | Route | Component(s) | Description | Menu |
|------|-------|-------------|-------------|------|
| `skills.astro` | `/skills` | `SkillOfferList.svelte`, `SkillOfferCreateModal.svelte` | Annuaire compétences | ❌ TODO Nav |
| `skill-detail.astro` | `/skill-detail` | `SkillOfferDetail.svelte`, `SkillOfferCard.svelte` | Détail compétence | ❌ Link |

#### **Community - Energy Campaigns** (ALL ROLES)

| Page | Route | Component(s) | Description | Menu |
|------|-------|-------------|-------------|------|
| `energy-campaigns.astro` | `/energy-campaigns` | `EnergyCampaignList.svelte` | Groupements achat énergie | ❌ TODO Nav |
| `energy-campaigns/[id].astro` | `/energy-campaigns/[id]` | `CampaignDetail.svelte`, `ProviderOffersList.svelte`, `EnergyBillUpload.svelte` | Détail campagne | ❌ Link |
| `energy-campaigns/new.astro` | `/energy-campaigns/new` | `CreateCampaignForm.svelte`, `CreateProviderOfferForm.svelte` | Créer campagne | ❌ Link |

#### **Quotes** (SYNDIC only)

| Page | Route | Component(s) | Description | Menu |
|------|-------|-------------|-------------|------|
| `quotes/compare.astro` | `/quotes/compare` | `QuoteComparisonTable.svelte` | Comparaison devis | ❌ TODO Nav |

#### **Settings** (ALL ROLES)

| Page | Route | Component(s) | Description | Menu |
|------|-------|-------------|-------------|------|
| `settings.astro` | `/settings` | - | Paramètres utilisateur | ✅ User menu |
| `settings/gdpr.astro` | `/settings/gdpr` | `GdprDataPanel.svelte` | Mes données GDPR | ✅ User menu |
| `settings/notifications.astro` | `/settings/notifications` | `NotificationPreferences.svelte` | Préférences notifs | ❌ TODO |

#### **Other**

| Page | Route | Component(s) | Description | Menu |
|------|-------|-------------|-------------|------|
| `profile.astro` | `/profile` | - | Profil utilisateur | ✅ User menu |
| `notifications.astro` | `/notifications` | `NotificationList.svelte` | Notifications | ❌ Bell icon |
| `mcp-chat.astro` | `/mcp-chat` | `McpChatbot.svelte` | MCP chatbot | ❌ Hidden |

---

## 🧩 Composants Svelte (130+)

### **Dashboards** (4 components - Role-specific)

| Component | Utilisé par | Description |
|-----------|-------------|-------------|
| `AdminDashboard.svelte` | `/admin` | Dashboard admin (metrics, users, orgs) |
| `SyndicDashboard.svelte` | `/syndic` | Dashboard syndic (buildings, expenses, meetings) |
| `AccountantDashboard.svelte` | `/accountant` | Dashboard comptable (PCMN, journal, balance) |
| `OwnerDashboard.svelte` | `/owner` | Dashboard copropriétaire (units, charges, docs) |

### **Admin Components** (7 components - SUPERADMIN)

| Component | Utilisé par | Description |
|-----------|-------------|-------------|
| `AdminGdprPanel.svelte` | `/admin/gdpr` | Export GDPR masse |
| `BoardManagement.svelte` | `/admin/board-members` | Gestion conseil |
| `BuildingForm.svelte` | `/admin` | Formulaire bâtiment |
| `OrganizationForm.svelte` | `/admin/organizations` | Formulaire organisation |
| `SeedManager.svelte` | `/admin/seed` | Seed data test |
| `StorageMetrics.svelte` | `/admin/monitoring` | Métriques stockage |
| `UserForm.svelte` | `/admin/users` | Formulaire utilisateur |

### **Core Components** (35+ components)

| Component | Utilisé par | Description |
|-----------|-------------|-------------|
| `BuildingList.svelte` | `/buildings` | Liste bâtiments |
| `BuildingDetail.svelte` | `/building-detail` | Détail bâtiment |
| `BuildingFinancialReports.svelte` | `/building-detail` | Rapports financiers |
| `BuildingListExample.svelte` | Docs | Exemple liste |
| `OwnerList.svelte` | `/owners` | Liste propriétaires |
| `OwnerCreateModal.svelte` | `/owners` | Créer propriétaire |
| `OwnerEditModal.svelte` | `/owners` | Éditer propriétaire |
| `OwnerUnits.svelte` | `/owners` | Lots propriétaire |
| `UnitList.svelte` | `/units` | Liste lots |
| `UnitCreateModal.svelte` | `/units` | Créer lot |
| `UnitEditModal.svelte` | `/units` | Éditer lot |
| `UnitOwners.svelte` | `/units` | Propriétaires lot |
| `UnitOwnerAddModal.svelte` | `/units` | Ajouter propriétaire |
| `UnitOwnerEditModal.svelte` | `/units` | Éditer propriétaire |
| `ExpenseList.svelte` | `/expenses` | Liste dépenses |
| `ExpenseDetail.svelte` | `/expense-detail` | Détail dépense |
| `ExpenseDocuments.svelte` | `/expense-detail` | Documents dépense |
| `InvoiceWorkflow.svelte` | `/invoice-workflow` | Workflow factures |
| `InvoiceForm.svelte` | `/invoice-workflow` | Formulaire facture |
| `InvoiceLineItems.svelte` | `/invoice-workflow` | Lignes facture |
| `InvoiceList.svelte` | - | Liste factures |
| `CallForFundsList.svelte` | `/call-for-funds` | Liste appels fonds |
| `CallForFundsForm.svelte` | `/call-for-funds` | Formulaire appel |
| `OwnerContributionList.svelte` | `/owner-contributions` | Liste contributions |
| `OwnerContributionForm.svelte` | `/owner-contributions` | Formulaire contribution |
| `PaymentReminderList.svelte` | `/payment-reminders` | Liste relances |
| `PaymentReminderDetail.svelte` | `/payment-reminder-detail` | Détail relance |
| `MeetingList.svelte` | `/meetings` | Liste réunions |
| `MeetingDetail.svelte` | `/meeting-detail` | Détail réunion |
| `MeetingDocuments.svelte` | `/meeting-detail` | Documents réunion |
| `DocumentList.svelte` | `/documents` | Liste documents |
| `DocumentUploadModal.svelte` | `/documents` | Upload document |
| `JournalEntryForm.svelte` | `/journal-entries` | Formulaire écriture |
| `FinancialReports.svelte` | `/reports` | Rapports PCMN |

### **Tickets** (7 components)

| Component | Utilisé par | Description |
|-----------|-------------|-------------|
| `TicketList.svelte` | `/tickets`, `/owner/tickets` | Liste tickets |
| `TicketDetail.svelte` | `/ticket-detail` | Détail ticket |
| `TicketCreateModal.svelte` | `/tickets` | Créer ticket |
| `TicketAssignModal.svelte` | `/ticket-detail` | Assigner ticket |
| `TicketStatistics.svelte` | `/ticket-detail` | Stats tickets |
| `TicketStatusBadge.svelte` | Multiple | Badge statut |
| `TicketPriorityBadge.svelte` | Multiple | Badge priorité |

### **Notifications** (5 components)

| Component | Utilisé par | Description |
|-----------|-------------|-------------|
| `NotificationBell.svelte` | `Navigation.svelte` | Cloche notifications (TODO) |
| `NotificationDropdown.svelte` | `Navigation.svelte` | Dropdown notifs (TODO) |
| `NotificationItem.svelte` | Multiple | Item notification |
| `NotificationList.svelte` | `/notifications` | Liste notifications |
| `NotificationPreferences.svelte` | `/settings/notifications` | Préférences |

### **Payments** (6 components)

| Component | Utilisé par | Description |
|-----------|-------------|-------------|
| `PaymentList.svelte` | `/owner/payments` | Liste paiements |
| `PaymentStats.svelte` | `/owner/payments` | Stats paiements |
| `PaymentMethodList.svelte` | `/owner/payment-methods` | Liste moyens |
| `PaymentMethodCard.svelte` | Multiple | Carte moyen paiement |
| `PaymentMethodAddModal.svelte` | `/owner/payment-methods` | Ajouter moyen |
| `PaymentStatusBadge.svelte` | Multiple | Badge statut |

### **Local Exchanges (SEL)** (7 components)

| Component | Utilisé par | Description |
|-----------|-------------|-------------|
| `ExchangeList.svelte` | `/exchanges` | Liste échanges |
| `CreateExchangeForm.svelte` | `/exchanges/new` | Créer échange |
| `CreditBalance.svelte` | `/exchanges` | Solde crédits |
| `Leaderboard.svelte` | `/exchanges` | Classement |
| `SelStatistics.svelte` | `/exchanges` | Stats SEL |
| `ExchangeStatusBadge.svelte` | Multiple | Badge statut |
| `ExchangeTypeBadge.svelte` | Multiple | Badge type |

### **Polls** (6 components)

| Component | Utilisé par | Description |
|-----------|-------------|-------------|
| `PollList.svelte` | `/polls` | Liste sondages |
| `PollDetail.svelte` | `/polls/[id]` | Détail sondage |
| `CreatePollForm.svelte` | `/polls/new` | Créer sondage |
| `PollResults.svelte` | `/polls/[id]` | Résultats |
| `PollStatusBadge.svelte` | Multiple | Badge statut |
| `PollTypeBadge.svelte` | Multiple | Badge type |

### **Notices** (5 components)

| Component | Utilisé par | Description |
|-----------|-------------|-------------|
| `NoticeList.svelte` | `/notices` | Liste annonces |
| `NoticeDetail.svelte` | `/notice-detail` | Détail annonce |
| `NoticeCreateModal.svelte` | `/notices` | Créer annonce |
| `NoticeStatusBadge.svelte` | Multiple | Badge statut |
| `NoticeTypeBadge.svelte` | Multiple | Badge type |

### **Bookings** (4 components)

| Component | Utilisé par | Description |
|-----------|-------------|-------------|
| `ResourceList.svelte` | `/bookings` | Liste ressources |
| `ResourceCard.svelte` | `/booking-detail` | Carte ressource |
| `ResourceTypeBadge.svelte` | Multiple | Badge type |
| `BookingStatusBadge.svelte` | Multiple | Badge statut |

### **Sharing** (6 components)

| Component | Utilisé par | Description |
|-----------|-------------|-------------|
| `SharedObjectList.svelte` | `/sharing` | Liste objets |
| `SharedObjectCard.svelte` | `/sharing-detail` | Carte objet |
| `ObjectCategoryBadge.svelte` | Multiple | Badge catégorie |
| `ObjectConditionBadge.svelte` | Multiple | Badge état |
| `AvailabilityStatusBadge.svelte` | Multiple | Badge dispo |
| `LoanStatusBadge.svelte` | Multiple | Badge prêt |

### **Skills** (6 components)

| Component | Utilisé par | Description |
|-----------|-------------|-------------|
| `SkillOfferList.svelte` | `/skills` | Liste compétences |
| `SkillOfferDetail.svelte` | `/skill-detail` | Détail compétence |
| `SkillOfferCard.svelte` | Multiple | Carte compétence |
| `SkillOfferCreateModal.svelte` | `/skills` | Créer offre |
| `SkillCategoryBadge.svelte` | Multiple | Badge catégorie |
| `ProficiencyBadge.svelte` | Multiple | Badge niveau |

### **Energy Campaigns** (7 components)

| Component | Utilisé par | Description |
|-----------|-------------|-------------|
| `EnergyCampaignList.svelte` | `/energy-campaigns` | Liste campagnes |
| `CampaignDetail.svelte` | `/energy-campaigns/[id]` | Détail campagne |
| `CreateCampaignForm.svelte` | `/energy-campaigns/new` | Créer campagne |
| `CreateProviderOfferForm.svelte` | `/energy-campaigns/new` | Créer offre |
| `ProviderOffersList.svelte` | `/energy-campaigns/[id]` | Liste offres |
| `EnergyBillUpload.svelte` | `/energy-campaigns/[id]` | Upload facture |
| `CampaignStatusBadge.svelte` | Multiple | Badge statut |

### **Quotes** (2 components)

| Component | Utilisé par | Description |
|-----------|-------------|-------------|
| `QuoteComparisonTable.svelte` | `/quotes/compare` | Comparaison devis |
| `QuoteStatusBadge.svelte` | Multiple | Badge statut |

### **Convocations** (1 component)

| Component | Utilisé par | Description |
|-----------|-------------|-------------|
| `ConvocationTrackingSummary.svelte` | - | Suivi convocations (TODO) |

### **Board** (3 components)

| Component | Utilisé par | Description |
|-----------|-------------|-------------|
| `BoardDashboard.svelte` | `/board-dashboard` | Dashboard conseil |
| `BoardMemberList.svelte` | `/syndic/board-members` | Liste membres |
| `DecisionTracker.svelte` | `/board-dashboard` | Suivi décisions |

### **UI Components** (10 components - Shared)

| Component | Description |
|-----------|-------------|
| `Button.svelte` | Bouton standard |
| `AccessibleButton.svelte` | Bouton accessible WCAG |
| `Modal.svelte` | Modal standard |
| `AccessibleModal.svelte` | Modal accessible WCAG |
| `FormInput.svelte` | Input formulaire |
| `FormSelect.svelte` | Select formulaire |
| `FormTextarea.svelte` | Textarea formulaire |
| `ConfirmDialog.svelte` | Dialog confirmation |
| `Toast.svelte` | Toast notification |
| `ToastContainer.svelte` | Container toasts |

### **Other Components** (10 components)

| Component | Description |
|-----------|-------------|
| `Navigation.svelte` | Navigation principale |
| `LoginForm.svelte` | Formulaire login |
| `RegisterForm.svelte` | Formulaire inscription |
| `LanguageSelector.svelte` | Sélecteur langue (FR/NL/DE/EN) |
| `Pagination.svelte` | Pagination |
| `GdprDataPanel.svelte` | Panel GDPR (export/delete/rectify) |
| `UserListAdmin.svelte` | Liste users admin |
| `UserOwnerLinkManager.svelte` | Link users ↔ owners |
| `OrganizationList.svelte` | Liste organisations |
| `SessionManager.svelte` | Gestion session |
| `SyncStatus.svelte` | Statut sync offline |
| `PWAInstallPrompt.svelte` | Prompt install PWA (TODO) |
| `McpChatbot.svelte` | MCP chatbot |

---

## 📡 API Clients (`src/lib/api/`)

### **Core API Clients** (20+ clients)

| Client | Endpoints | Description |
|--------|-----------|-------------|
| `buildings.ts` | GET/POST/PUT/DELETE `/buildings` | CRUD bâtiments |
| `units.ts` | GET/POST/PUT/DELETE `/units` | CRUD lots |
| `owners.ts` | GET/POST/PUT/DELETE `/owners` | CRUD propriétaires |
| `expenses.ts` | GET/POST/PUT/DELETE `/expenses` | CRUD dépenses |
| `meetings.ts` | GET/POST/PUT/DELETE `/meetings` | CRUD réunions |
| `documents.ts` | GET/POST/DELETE `/documents` | CRUD documents |
| `tickets.ts` | GET/POST/PUT/DELETE `/tickets` | CRUD tickets |
| `notifications.ts` | GET/PUT/DELETE `/notifications` | Notifications |
| `payments.ts` | GET/POST/PUT `/payments` | Paiements |
| `polls.ts` | GET/POST/PUT/DELETE `/polls` | Sondages |
| `exchanges.ts` | GET/POST/PUT `/exchanges` | SEL échanges |
| `notices.ts` | GET/POST/PUT/DELETE `/notices` | Annonces |
| `bookings.ts` | GET/POST/PUT/DELETE `/bookings` | Réservations |
| `sharing.ts` | GET/POST/PUT/DELETE `/sharing` | Partage objets |
| `skills.ts` | GET/POST/PUT/DELETE `/skills` | Compétences |
| `energy-campaigns.ts` | GET/POST/PUT/DELETE `/energy-campaigns` | Campagnes énergie |
| `quotes.ts` | GET/POST/PUT/DELETE `/quotes` | Devis |
| `gdpr.ts` | GET/POST/PUT/DELETE `/gdpr` | GDPR actions |
| `accounts.ts` | GET/POST/PUT/DELETE `/accounts` | PCMN comptes |
| `journal-entries.ts` | GET/POST `/journal-entries` | Écritures |

---

## 🚀 Câblage TODO - Pages Manquantes au Menu

### **Navigation Principal**

#### **Community Features** (Tous rôles) - Ajouter au menu

```typescript
// Navigation.svelte - getNavItems()
const communityItems = [
  { href: '/exchanges', label: 'SEL', icon: '🔄' },
  { href: '/polls', label: 'Sondages', icon: '📊' },
  { href: '/notices', label: 'Tableau affichage', icon: '📌' },
  { href: '/bookings', label: 'Réservations', icon: '📅' },
  { href: '/sharing', label: 'Partage objets', icon: '🎁' },
  { href: '/skills', label: 'Compétences', icon: '🎓' },
  { href: '/energy-campaigns', label: 'Énergie', icon: '⚡' },
];
```

#### **Owner Submenu** - Compléter

```typescript
case UserRole.OWNER:
  return [
    { href: '/owner', label: 'Dashboard', icon: '🏠' },
    { href: '/owner/units', label: 'Mes lots', icon: '🚪' },
    { href: '/owner/expenses', label: 'Mes charges', icon: '💰' },
    { href: '/owner/payments', label: 'Paiements', icon: '💳' },    // TODO
    { href: '/owner/payment-methods', label: 'Moyens paiement', icon: '🏦' }, // TODO
    { href: '/owner/tickets', label: 'Mes tickets', icon: '🎫' },   // TODO
    { href: '/owner/documents', label: 'Documents', icon: '📄' },
    { href: '/owner/profile', label: 'Profil', icon: '👤' },        // TODO
  ];
```

#### **Syndic Menu** - Ajouter Tickets & Quotes

```typescript
case UserRole.SYNDIC:
  return [
    // ... existing items
    { href: '/tickets', label: 'Tickets', icon: '🎫' },       // TODO
    { href: '/quotes/compare', label: 'Devis', icon: '📋' }, // TODO
  ];
```

---

## 🔧 Actions Requises

### **1. Ajouter Community Features au Menu** (1 jour)

**Fichier**: `src/components/Navigation.svelte`

**Changement**:
```typescript
const getNavItems = (role: UserRole | undefined, t: any) => {
  if (!role) return [];

  const communityItems = [
    { href: '/exchanges', label: 'SEL', icon: '🔄' },
    { href: '/polls', label: 'Sondages', icon: '📊' },
    { href: '/notices', label: 'Annonces', icon: '📌' },
    { href: '/bookings', label: 'Réservations', icon: '📅' },
    { href: '/sharing', label: 'Partage', icon: '🎁' },
    { href: '/skills', label: 'Compétences', icon: '🎓' },
    { href: '/energy-campaigns', label: 'Énergie', icon: '⚡' },
  ];

  switch (role) {
    case UserRole.SUPERADMIN:
      return [
        // ... existing admin items
        ...communityItems,
      ];

    case UserRole.SYNDIC:
      return [
        // ... existing syndic items
        { href: '/tickets', label: 'Tickets', icon: '🎫' },
        { href: '/quotes/compare', label: 'Devis', icon: '📋' },
        ...communityItems,
      ];

    case UserRole.ACCOUNTANT:
      return [
        // ... existing accountant items
        ...communityItems,
      ];

    case UserRole.OWNER:
      return [
        { href: '/owner', label: 'Dashboard', icon: '🏠' },
        { href: '/owner/units', label: 'Mes lots', icon: '🚪' },
        { href: '/owner/expenses', label: 'Mes charges', icon: '💰' },
        { href: '/owner/payments', label: 'Paiements', icon: '💳' },
        { href: '/owner/payment-methods', label: 'Moyens paiement', icon: '🏦' },
        { href: '/owner/tickets', label: 'Mes tickets', icon: '🎫' },
        { href: '/owner/documents', label: 'Documents', icon: '📄' },
        { href: '/owner/profile', label: 'Profil', icon: '👤' },
        ...communityItems,
      ];
  }
};
```

---

### **2. Ajouter Notification Bell au Header** (2 heures)

**Fichier**: `src/components/Navigation.svelte`

**Changement** (après ligne 179):
```svelte
<!-- Right side: Notifications + User Menu -->
<div class="flex items-center gap-4">
  <!-- Notification Bell -->
  <NotificationBell />

  <!-- User Menu -->
  <div class="relative" data-testid="user-menu-container">
    <!-- ... existing user menu code -->
  </div>
</div>
```

**Créer**: `src/components/notifications/NotificationBell.svelte`

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { notificationStore } from '../../stores/notifications';

  $: unreadCount = $notificationStore.unreadCount;

  onMount(() => {
    notificationStore.fetchUnread();
  });
</script>

<a href="/notifications" class="relative p-2 rounded-lg hover:bg-gray-100">
  <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"/>
  </svg>

  {#if unreadCount > 0}
    <span class="absolute -top-1 -right-1 bg-red-500 text-white text-xs rounded-full w-5 h-5 flex items-center justify-center font-bold">
      {unreadCount > 99 ? '99+' : unreadCount}
    </span>
  {/if}
</a>
```

---

### **3. Protéger Routes par Rôle** (2 jours)

**Créer**: `src/lib/guards.ts`

```typescript
import type { UserRole } from './types';

export const roleGuards = {
  '/admin': [UserRole.SUPERADMIN],
  '/admin/*': [UserRole.SUPERADMIN],
  '/syndic': [UserRole.SYNDIC],
  '/syndic/*': [UserRole.SYNDIC],
  '/accountant': [UserRole.ACCOUNTANT],
  '/owner': [UserRole.OWNER],
  '/owner/*': [UserRole.OWNER],
  '/journal-entries': [UserRole.ACCOUNTANT],
  '/reports': [UserRole.ACCOUNTANT],
  '/quotes/*': [UserRole.SYNDIC],
};

export function canAccessRoute(route: string, userRole: UserRole): boolean {
  for (const [pattern, allowedRoles] of Object.entries(roleGuards)) {
    const regex = new RegExp(`^${pattern.replace('*', '.*')}$`);
    if (regex.test(route)) {
      return allowedRoles.includes(userRole);
    }
  }
  return true; // Allow if no guard defined
}
```

**Utiliser dans**: `src/layouts/Layout.astro`

```astro
---
import { authStore } from '../stores/auth';
import { canAccessRoute } from '../lib/guards';

const route = Astro.url.pathname;
const user = authStore.get().user;

if (user && !canAccessRoute(route, user.role)) {
  return Astro.redirect(`/${user.role}`);
}
---
```

---

### **4. Ajouter i18n Dutch (NL)** (3 jours)

**Installation**:
```bash
npm install astro-i18next i18next
```

**Config**: `astro.config.mjs`
```javascript
import { defineConfig } from 'astro/config';
import astroI18next from 'astro-i18next';

export default defineConfig({
  integrations: [
    astroI18next({
      locales: ['fr', 'nl', 'de', 'en'],
      defaultLocale: 'fr',
      routes: {
        nl: {
          buildings: 'gebouwen',
          owners: 'eigenaars',
          expenses: 'uitgaven',
        },
      },
    }),
  ],
});
```

**Traductions**: `public/locales/nl/common.json`
```json
{
  "navigation": {
    "buildings": "Gebouwen",
    "owners": "Eigenaars",
    "units": "Eenheden",
    "expenses": "Uitgaven",
    "meetings": "Vergaderingen",
    "documents": "Documenten",
    "dashboard": "Dashboard",
    "profile": "Profiel",
    "logout": "Uitloggen"
  }
}
```

---

## ✅ Résumé Actions

| # | Action | Effort | Priorité | Impact |
|---|--------|--------|----------|--------|
| 1 | Ajouter Community Features au menu | 1j | 🔴 Haute | Expose SEL, Polls, Notices, etc. |
| 2 | Ajouter Notification Bell au header | 2h | 🟡 Moyenne | UX notifications |
| 3 | Protéger routes par rôle (guards) | 2j | 🔴 Haute | Sécurité frontend |
| 4 | Ajouter i18n Dutch (NL) | 3j | 🟠 Haute | Flandre (60% Belgium) |
| 5 | Compléter Owner submenu | 1j | 🟡 Moyenne | Feature parity Owner |
| 6 | PWA Service Workers | 5j | 🟡 Moyenne | Offline support |
| 7 | WCAG 2.1 AA Accessibility | 8j | 🟠 Haute | Legal EU 2025 |

**Total effort**: ~20 jours pour frontend 100% production-ready

---

**Version**: 1.0
**Date**: 6 Décembre 2025
**Status**: ✅ Inventaire complet - 67 pages, 130+ components
