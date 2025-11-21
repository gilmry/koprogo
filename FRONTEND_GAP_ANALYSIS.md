# KoproGo Frontend Gap Analysis

**Date**: 2025-11-17
**Branch**: `claude/review-remaining-issues-018z8PJuUPF4CXEuhBN9zV3y`
**Backend Status**: ✅ ~95% Complete (400+ endpoints)
**Frontend Status**: ⚠️ **~40% Complete** - Significant gaps for Phase 2 features

---

## Executive Summary

**Overall Status**: 🟡 **~40% Feature Parity** - Core features implemented, Phase 2 features missing

**Frontend Inventory**:
- ✅ **68 Svelte Components** (core CRUD features)
- ✅ **43 Astro Pages** (buildings, units, owners, expenses, meetings)
- ✅ **2 Stores** (auth.ts, toast.ts)
- ✅ **Generic API Client** (lib/api.ts with CRUD helpers)
- ⚠️ **Specialized API Clients**: Only 2 (callForFundsApi, ownerContributionsApi)
- ❌ **Missing 15+ Phase 2 Feature UIs** (Tickets, Notifications, Payments, etc.)

**Critical Gap**: Backend has ~400 endpoints across 22 feature domains, but frontend only implements ~10 domains.

---

## ✅ What's Implemented (Frontend Features)

### Core CRUD Features (Phase 1)
- ✅ **Authentication**: LoginForm.svelte, RegisterForm.svelte, auth.ts store
- ✅ **Buildings**: BuildingList.svelte, BuildingDetail.svelte, admin/BuildingForm.svelte
- ✅ **Units**: UnitList.svelte, UnitEditModal.svelte
- ✅ **Owners**: OwnerList.svelte, OwnerCreateModal.svelte, OwnerEditModal.svelte, OwnerUnits.svelte
- ✅ **Multi-Owner Support**: UnitOwners.svelte, UnitOwnerAddModal.svelte, UnitOwnerEditModal.svelte
- ✅ **Expenses**: ExpenseList.svelte, ExpenseDetail.svelte, ExpenseDocuments.svelte
- ✅ **Meetings**: MeetingList.svelte, MeetingDetail.svelte, MeetingDocuments.svelte
- ✅ **Documents**: DocumentList.svelte, DocumentUploadModal.svelte
- ✅ **Organizations**: OrganizationList.svelte, admin/OrganizationForm.svelte
- ✅ **GDPR**: GdprDataPanel.svelte, admin/AdminGdprPanel.svelte, pages/settings/gdpr.astro
- ✅ **Board of Directors**: BoardDashboard.svelte, BoardMemberList.svelte, admin/BoardManagement.svelte, DecisionTracker.svelte

### Legal Compliance Features (Partial)
- ✅ **Invoice Workflow**: InvoiceWorkflow.svelte, InvoiceForm.svelte, InvoiceList.svelte
- ✅ **Payment Reminders**: PaymentReminderList.svelte, PaymentReminderDetail.svelte (Issue #83)
- ✅ **Call for Funds**: callForFundsApi in lib/api.ts, pages/call-for-funds.astro
- ✅ **Owner Contributions**: ownerContributionsApi in lib/api.ts, pages/owner-contributions.astro
- ✅ **Journal Entries**: pages/journal-entries.astro
- ✅ **Financial Reports**: pages/reports.astro

### Admin & Infrastructure
- ✅ **Admin Dashboard**: pages/admin/index.astro
- ✅ **User Management**: UserListAdmin.svelte, admin/UserForm.svelte, admin/user-owner-links.astro
- ✅ **Seed Manager**: admin/SeedManager.svelte
- ✅ **Storage Metrics**: admin/StorageMetrics.svelte
- ✅ **Monitoring**: pages/admin/monitoring.astro
- ✅ **Session Management**: SessionManager.svelte, SyncStatus.svelte

### UI Components
- ✅ **UI Kit**: Button.svelte, Modal.svelte, Toast.svelte, ToastContainer.svelte, ConfirmDialog.svelte
- ✅ **Form Components**: FormInput.svelte, FormTextarea.svelte, FormSelect.svelte
- ✅ **Pagination**: Pagination.svelte
- ✅ **Language Selector**: LanguageSelector.svelte

### Role-Specific Pages
- ✅ **Accountant**: pages/accountant/index.astro
- ✅ **Owner Portal**: pages/owner/* (index, profile, documents, expenses, units, contact)
- ✅ **Syndic Portal**: pages/syndic/* (index, board-members)

---

## ❌ Frontend Gap Analysis: Missing Features

### 🔴 GAP 1: Tickets System (Issue #85 - Phase 2) - **NO FRONTEND**

**Backend Status**: ✅ Complete (17 REST endpoints, 18 use case methods)
**Frontend Status**: ❌ **NOT IMPLEMENTED**

**Missing Components**:
```
frontend/src/components/tickets/
  ├── TicketList.svelte                 # List all tickets with filters (status, priority, category)
  ├── TicketDetail.svelte               # Ticket details + workflow actions
  ├── TicketCreateModal.svelte          # Create maintenance request
  ├── TicketAssignModal.svelte          # Assign to contractor
  ├── TicketStatusBadge.svelte          # Status indicator (Open/Assigned/InProgress/Resolved)
  └── TicketStatistics.svelte           # Dashboard widget (count by status, avg resolution time)
```

**Missing Pages**:
```
frontend/src/pages/
  ├── tickets.astro                     # All tickets (Syndic view)
  ├── ticket-detail.astro               # Individual ticket
  ├── owner/tickets.astro               # My tickets (Owner view)
  └── contractor/tickets.astro          # Assigned tickets (Contractor view)
```

**Missing API Client**:
```typescript
// lib/api/tickets.ts
export const ticketsApi = {
  list(filters?: {buildingId?, status?, priority?, category?}): Promise<Ticket[]>,
  getById(id: string): Promise<Ticket>,
  create(data: CreateTicketDto): Promise<Ticket>,
  assign(id: string, contractorId: string): Promise<Ticket>,
  start(id: string): Promise<Ticket>,
  resolve(id: string): Promise<Ticket>,
  close(id: string): Promise<Ticket>,
  cancel(id: string): Promise<Ticket>,
  reopen(id: string): Promise<Ticket>,
  getStatistics(): Promise<TicketStatistics>,
  getOverdue(): Promise<Ticket[]>,
}
```

**Backend Endpoints (17 total)**:
- `GET /tickets`, `POST /tickets`, `GET /tickets/:id`, `DELETE /tickets/:id`
- `GET /buildings/:id/tickets`, `GET /organizations/:id/tickets`
- `GET /tickets/my`, `GET /tickets/assigned`, `GET /tickets/status/:status`
- `PUT /tickets/:id/assign`, `PUT /tickets/:id/start`, `PUT /tickets/:id/resolve`
- `PUT /tickets/:id/close`, `PUT /tickets/:id/cancel`, `PUT /tickets/:id/reopen`
- `GET /tickets/statistics`, `GET /tickets/overdue`

**Estimated Effort**: 4-5 days (5 components, 4 pages, 1 API client)

---

### 🔴 GAP 2: Notifications System (Issue #86 - Phase 2) - **NO FRONTEND**

**Backend Status**: ✅ Complete (11 REST endpoints, 13 use case methods)
**Frontend Status**: ❌ **NOT IMPLEMENTED**

**Missing Components**:
```
frontend/src/components/notifications/
  ├── NotificationBell.svelte           # Header bell icon with unread count badge
  ├── NotificationDropdown.svelte       # Dropdown list (recent 10 notifications)
  ├── NotificationList.svelte           # Full notifications page
  ├── NotificationItem.svelte           # Single notification card
  ├── NotificationPreferences.svelte    # User preferences (Email/SMS/Push/InApp per type)
  └── NotificationStats.svelte          # Statistics widget
```

**Missing Pages**:
```
frontend/src/pages/
  ├── notifications.astro               # All notifications page
  └── settings/notifications.astro      # Notification preferences
```

**Missing Store**:
```typescript
// stores/notifications.ts
export const notificationStore = writable({
  unreadCount: 0,
  recentNotifications: [],
});
```

**Missing API Client**:
```typescript
// lib/api/notifications.ts
export const notificationsApi = {
  list(): Promise<Notification[]>,
  getUnread(): Promise<Notification[]>,
  getById(id: string): Promise<Notification>,
  markAsRead(id: string): Promise<void>,
  markAllAsRead(): Promise<void>,
  delete(id: string): Promise<void>,
  getStats(): Promise<NotificationStats>,
  getPreferences(userId: string): Promise<NotificationPreference[]>,
  updatePreference(userId: string, notificationType: string, data: Partial<NotificationPreference>): Promise<void>,
}
```

**Backend Endpoints (11 total)**:
- `POST /notifications`, `GET /notifications/:id`, `GET /notifications/my`
- `GET /notifications/unread`, `PUT /notifications/:id/read`, `PUT /notifications/read-all`
- `DELETE /notifications/:id`, `GET /notifications/stats`
- `GET /notification-preferences/:user_id`, `GET /notification-preferences/:user_id/:notification_type`
- `PUT /notification-preferences/:user_id/:notification_type`

**Estimated Effort**: 4-5 days (6 components, 2 pages, 1 store, 1 API client)

---

### 🔴 GAP 3: Payments & Payment Methods (Issue #84 - Phase 2) - **NO FRONTEND**

**Backend Status**: ✅ Complete (38 REST endpoints: 22 payments + 16 payment methods)
**Frontend Status**: ❌ **NOT IMPLEMENTED**

**Missing Components**:
```
frontend/src/components/payments/
  ├── PaymentList.svelte                # List payments with filters (status, owner, building)
  ├── PaymentDetail.svelte              # Payment details + refund action
  ├── PaymentCreateModal.svelte         # Create payment (amount, method, expense)
  ├── PaymentMethodList.svelte          # Owner's stored payment methods
  ├── PaymentMethodCard.svelte          # Single payment method card (Card/SEPA/Bank)
  ├── PaymentMethodAddModal.svelte      # Add new payment method (Stripe integration)
  ├── PaymentStats.svelte               # Statistics widget (total paid, succeeded count)
  └── PaymentStatusBadge.svelte         # Status indicator (Pending/Processing/Succeeded/Failed)
```

**Missing Pages**:
```
frontend/src/pages/
  ├── payments.astro                    # All payments (Syndic/Accountant view)
  ├── payment-detail.astro              # Individual payment
  ├── owner/payments.astro              # My payments (Owner view)
  └── owner/payment-methods.astro       # Manage payment methods
```

**Missing API Client**:
```typescript
// lib/api/payments.ts
export const paymentsApi = {
  // Payments (22 endpoints)
  create(data: CreatePaymentDto): Promise<Payment>,
  getById(id: string): Promise<Payment>,
  getByStripeIntentId(stripeIntentId: string): Promise<Payment>,
  listByOwner(ownerId: string): Promise<Payment[]>,
  listByBuilding(buildingId: string): Promise<Payment[]>,
  listByExpense(expenseId: string): Promise<Payment[]>,
  listByStatus(status: PaymentStatus): Promise<Payment[]>,
  getPending(): Promise<Payment[]>,
  getFailed(): Promise<Payment[]>,
  markAsProcessing(id: string): Promise<Payment>,
  markAsRequiresAction(id: string): Promise<Payment>,
  markAsSucceeded(id: string): Promise<Payment>,
  markAsFailed(id: string): Promise<Payment>,
  markAsCancelled(id: string): Promise<Payment>,
  refund(id: string, amount?: number): Promise<Payment>,
  delete(id: string): Promise<void>,
  getOwnerStats(ownerId: string): Promise<PaymentStats>,
  getBuildingStats(buildingId: string): Promise<PaymentStats>,
}

export const paymentMethodsApi = {
  // Payment Methods (16 endpoints)
  create(data: CreatePaymentMethodDto): Promise<PaymentMethod>,
  getById(id: string): Promise<PaymentMethod>,
  getByStripeId(stripeId: string): Promise<PaymentMethod>,
  listByOwner(ownerId: string): Promise<PaymentMethod[]>,
  getActiveByOwner(ownerId: string): Promise<PaymentMethod[]>,
  getDefaultByOwner(ownerId: string): Promise<PaymentMethod>,
  listByType(ownerId: string, methodType: string): Promise<PaymentMethod[]>,
  update(id: string, data: Partial<PaymentMethod>): Promise<PaymentMethod>,
  setAsDefault(id: string): Promise<void>,
  deactivate(id: string): Promise<void>,
  reactivate(id: string): Promise<void>,
  delete(id: string): Promise<void>,
  getCount(ownerId: string): Promise<number>,
  hasActive(ownerId: string): Promise<boolean>,
}
```

**Backend Endpoints (38 total)**:
- **Payments**: 22 endpoints (create, list variants, status transitions, refund, stats)
- **Payment Methods**: 16 endpoints (CRUD, set default, activate/deactivate, count)

**Estimated Effort**: 6-7 days (8 components, 4 pages, 2 API clients, Stripe SDK integration)

---

### 🔴 GAP 4: Quotes Module (Issue #91 - Phase 2) - **NO FRONTEND**

**Backend Status**: ✅ Complete (15 REST endpoints, 20 use case methods)
**Frontend Status**: ❌ **NOT IMPLEMENTED**

**Missing Components**:
```
frontend/src/components/quotes/
  ├── QuoteList.svelte                  # List quotes with filters (status, building, contractor)
  ├── QuoteDetail.svelte                # Quote details + decision actions
  ├── QuoteRequestModal.svelte          # Syndic creates quote request
  ├── QuoteSubmitModal.svelte           # Contractor submits quote with pricing
  ├── QuoteComparisonTable.svelte       # Compare multiple quotes (Belgian 3-quote rule)
  ├── QuoteStatusBadge.svelte           # Status indicator (Requested/Received/Accepted/Rejected)
  └── QuoteScoreCard.svelte             # Automatic scoring display (Price 40%, Delay 30%, Warranty 20%, Reputation 10%)
```

**Missing Pages**:
```
frontend/src/pages/
  ├── quotes.astro                      # All quotes (Syndic view)
  ├── quote-detail.astro                # Individual quote
  ├── quotes/compare.astro              # Compare quotes side-by-side
  └── contractor/quotes.astro           # My quotes (Contractor view)
```

**Missing API Client**:
```typescript
// lib/api/quotes.ts
export const quotesApi = {
  create(data: CreateQuoteDto): Promise<Quote>,
  getById(id: string): Promise<Quote>,
  listByBuilding(buildingId: string): Promise<Quote[]>,
  listByContractor(contractorId: string): Promise<Quote[]>,
  listByStatus(buildingId: string, status: QuoteStatus): Promise<Quote[]>,
  submit(id: string, data: SubmitQuoteDto): Promise<Quote>,
  startReview(id: string): Promise<Quote>,
  accept(id: string, data: AcceptQuoteDto): Promise<Quote>,
  reject(id: string, data: RejectQuoteDto): Promise<Quote>,
  withdraw(id: string): Promise<Quote>,
  compare(quoteIds: string[]): Promise<QuoteComparison>,
  updateRating(id: string, rating: number): Promise<Quote>,
  delete(id: string): Promise<void>,
  getCountByBuilding(buildingId: string): Promise<number>,
  getCountByStatus(buildingId: string, status: QuoteStatus): Promise<number>,
}
```

**Backend Endpoints (15 total)**:
- `POST /quotes`, `GET /quotes/:id`, `GET /buildings/:id/quotes`, `GET /contractors/:id/quotes`
- `GET /buildings/:id/quotes/status/:status`, `POST /quotes/:id/submit`, `POST /quotes/:id/review`
- `POST /quotes/:id/accept`, `POST /quotes/:id/reject`, `POST /quotes/:id/withdraw`
- `POST /quotes/compare` (Belgian 3-quote comparison), `PUT /quotes/:id/contractor-rating`
- `DELETE /quotes/:id`, `GET /buildings/:id/quotes/count`, `GET /buildings/:id/quotes/status/:status/count`

**Estimated Effort**: 5-6 days (7 components, 4 pages, 1 API client)

---

### 🔴 GAP 5: Convocations (Issue #88 - Phase 2) - **NO FRONTEND**

**Backend Status**: ✅ Complete (14 REST endpoints, 21 use case methods)
**Frontend Status**: ❌ **NOT IMPLEMENTED**

**Missing Components**:
```
frontend/src/components/convocations/
  ├── ConvocationList.svelte            # List convocations
  ├── ConvocationDetail.svelte          # Convocation details + recipients
  ├── ConvocationCreateModal.svelte     # Create convocation (validates legal deadlines)
  ├── ConvocationRecipientList.svelte   # Recipient tracking (email opened, attendance)
  ├── ConvocationTrackingSummary.svelte # Aggregate stats (opening rate, attendance rate)
  └── ConvocationStatusBadge.svelte     # Status indicator (Draft/Scheduled/Sent/Cancelled)
```

**Missing Pages**:
```
frontend/src/pages/
  ├── convocations.astro                # All convocations (Syndic view)
  └── convocation-detail.astro          # Individual convocation with tracking
```

**Missing API Client**:
```typescript
// lib/api/convocations.ts
export const convocationsApi = {
  create(data: CreateConvocationDto): Promise<Convocation>,
  getById(id: string): Promise<Convocation>,
  getByMeetingId(meetingId: string): Promise<Convocation>,
  listByBuilding(buildingId: string): Promise<Convocation[]>,
  listByOrganization(organizationId: string): Promise<Convocation[]>,
  schedule(id: string, sendDate: string): Promise<Convocation>,
  send(id: string): Promise<Convocation>,
  cancel(id: string): Promise<Convocation>,
  delete(id: string): Promise<void>,
  getRecipients(id: string): Promise<ConvocationRecipient[]>,
  getTrackingSummary(id: string): Promise<TrackingSummary>,
  markEmailOpened(recipientId: string): Promise<void>,
  updateAttendance(recipientId: string, status: AttendanceStatus): Promise<void>,
  setProxy(recipientId: string, proxyOwnerId: string): Promise<void>,
  sendReminders(id: string): Promise<void>,
}
```

**Backend Endpoints (14 total)**:
- `POST /convocations`, `GET /convocations/:id`, `GET /convocations/meeting/:meeting_id`
- `GET /buildings/:id/convocations`, `GET /organizations/:id/convocations`, `DELETE /convocations/:id`
- `PUT /convocations/:id/schedule`, `POST /convocations/:id/send`, `PUT /convocations/:id/cancel`
- `GET /convocations/:id/recipients`, `GET /convocations/:id/tracking-summary`
- `PUT /convocation-recipients/:id/email-opened`, `PUT /convocation-recipients/:id/attendance`
- `PUT /convocation-recipients/:id/proxy`, `POST /convocations/:id/reminders`

**Estimated Effort**: 4-5 days (6 components, 2 pages, 1 API client)

---

### 🔴 GAP 6: Resolutions & Voting (Issue #46 - Phase 2) - **NO FRONTEND**

**Backend Status**: ✅ Complete (9 REST endpoints, 14 use case methods)
**Frontend Status**: ❌ **NOT IMPLEMENTED**

**Missing Components**:
```
frontend/src/components/resolutions/
  ├── ResolutionList.svelte             # List resolutions for meeting
  ├── ResolutionDetail.svelte           # Resolution details + vote counts
  ├── ResolutionCreateModal.svelte      # Create resolution (type, majority required)
  ├── VoteCard.svelte                   # Cast vote (Pour/Contre/Abstention)
  ├── VoteSummary.svelte                # Vote summary (result, majority reached)
  └── ProxyVoteModal.svelte             # Vote on behalf of proxy owner
```

**Missing Pages**:
```
frontend/src/pages/
  ├── meeting-detail.astro (ENHANCE)    # Add resolutions tab
  └── resolution-detail.astro           # Individual resolution with voting
```

**Missing API Client**:
```typescript
// lib/api/resolutions.ts
export const resolutionsApi = {
  create(meetingId: string, data: CreateResolutionDto): Promise<Resolution>,
  getById(id: string): Promise<Resolution>,
  listByMeeting(meetingId: string): Promise<Resolution[]>,
  delete(id: string): Promise<void>,
  castVote(resolutionId: string, data: CastVoteDto): Promise<Vote>,
  getVotes(resolutionId: string): Promise<Vote[]>,
  changeVote(voteId: string, newChoice: VoteChoice): Promise<Vote>,
  closeVoting(resolutionId: string): Promise<Resolution>,
  getVoteSummary(meetingId: string): Promise<VoteSummary>,
}
```

**Backend Endpoints (9 total)**:
- `POST /meetings/:id/resolutions`, `GET /resolutions/:id`, `GET /meetings/:id/resolutions`
- `DELETE /resolutions/:id`, `POST /resolutions/:id/vote`, `GET /resolutions/:id/votes`
- `PUT /votes/:id`, `PUT /resolutions/:id/close`, `GET /meetings/:id/vote-summary`

**Estimated Effort**: 3-4 days (6 components, 1 page enhancement, 1 API client)

---

### 🔴 GAP 7: Local Exchanges (SEL) (Issue #49 - Phase 1) - **NO FRONTEND**

**Backend Status**: ✅ Complete (17 REST endpoints, 20 use case methods)
**Frontend Status**: ❌ **NOT IMPLEMENTED**

**Missing Components**:
```
frontend/src/components/sel/
  ├── ExchangeMarketplace.svelte        # Browse available exchanges (Service/ObjectLoan/SharedPurchase)
  ├── ExchangeCard.svelte               # Single exchange card with provider info
  ├── ExchangeCreateModal.svelte        # Create exchange offer
  ├── ExchangeDetail.svelte             # Exchange details + workflow actions
  ├── ExchangeRatingModal.svelte        # Rate provider/requester (1-5 stars)
  ├── CreditBalanceWidget.svelte        # Owner's credit balance (time-based currency)
  ├── SelLeaderboard.svelte             # Top contributors (ordered by balance)
  ├── SelStatistics.svelte              # Building SEL statistics
  └── OwnerExchangeSummary.svelte       # Owner's exchange summary (offered/requested/completed)
```

**Missing Pages**:
```
frontend/src/pages/
  ├── sel/marketplace.astro             # Browse available exchanges
  ├── sel/my-exchanges.astro            # My exchanges (Owner view)
  ├── sel/leaderboard.astro             # Community leaderboard
  └── sel/statistics.astro              # Building SEL statistics
```

**Missing API Client**:
```typescript
// lib/api/sel.ts
export const selApi = {
  // Exchanges (17 endpoints)
  create(data: CreateExchangeDto): Promise<LocalExchange>,
  getById(id: string): Promise<LocalExchange>,
  listByBuilding(buildingId: string): Promise<LocalExchange[]>,
  getAvailable(buildingId: string): Promise<LocalExchange[]>,
  listByOwner(ownerId: string): Promise<LocalExchange[]>,
  listByType(buildingId: string, exchangeType: string): Promise<LocalExchange[]>,
  request(id: string): Promise<LocalExchange>,
  start(id: string): Promise<LocalExchange>,
  complete(id: string): Promise<LocalExchange>,
  cancel(id: string, reason: string): Promise<LocalExchange>,
  rateProvider(id: string, rating: number): Promise<LocalExchange>,
  rateRequester(id: string, rating: number): Promise<LocalExchange>,
  delete(id: string): Promise<void>,
  
  // Credit Balance & Analytics
  getCreditBalance(ownerId: string, buildingId: string): Promise<OwnerCreditBalance>,
  getLeaderboard(buildingId: string, limit?: number): Promise<OwnerCreditBalance[]>,
  getStatistics(buildingId: string): Promise<SelStatistics>,
  getOwnerSummary(ownerId: string): Promise<OwnerExchangeSummary>,
}
```

**Backend Endpoints (17 total)**:
- `POST /exchanges`, `GET /exchanges/:id`, `GET /buildings/:id/exchanges`
- `GET /buildings/:id/exchanges/available`, `GET /owners/:id/exchanges`, `GET /buildings/:id/exchanges/type/:type`
- `POST /exchanges/:id/request`, `POST /exchanges/:id/start`, `POST /exchanges/:id/complete`
- `POST /exchanges/:id/cancel`, `PUT /exchanges/:id/rate-provider`, `PUT /exchanges/:id/rate-requester`
- `DELETE /exchanges/:id`, `GET /owners/:id/buildings/:building_id/credit-balance`
- `GET /buildings/:id/leaderboard`, `GET /buildings/:id/sel-statistics`, `GET /owners/:id/exchange-summary`

**Estimated Effort**: 5-6 days (9 components, 4 pages, 1 API client)

---

### 🔴 GAP 8: Gamification & Achievements (Issue #49 - Phase 6) - **NO FRONTEND**

**Backend Status**: ✅ Complete (22 REST endpoints, 28 use case methods)
**Frontend Status**: ❌ **NOT IMPLEMENTED**

**Missing Components**:
```
frontend/src/components/gamification/
  ├── AchievementList.svelte            # List achievements (visible + earned)
  ├── AchievementCard.svelte            # Single achievement card (icon, tier, points)
  ├── AchievementBadge.svelte           # Achievement badge (Bronze/Silver/Gold/Platinum/Diamond)
  ├── UserAchievements.svelte           # User's earned achievements
  ├── RecentAchievements.svelte         # Recent achievements widget
  ├── ChallengeList.svelte              # Active challenges
  ├── ChallengeCard.svelte              # Single challenge card (progress bar)
  ├── ChallengeProgress.svelte          # Challenge progress tracker
  ├── Leaderboard.svelte                # Gamification leaderboard (top users by points)
  └── GamificationStats.svelte          # User stats (total points, achievements, challenges)
```

**Missing Pages**:
```
frontend/src/pages/
  ├── gamification/achievements.astro   # All achievements
  ├── gamification/challenges.astro     # Active challenges
  ├── gamification/leaderboard.astro    # Community leaderboard
  └── owner/achievements.astro          # My achievements (Owner view)
```

**Missing API Client**:
```typescript
// lib/api/gamification.ts
export const gamificationApi = {
  // Achievements (7 endpoints)
  createAchievement(data: CreateAchievementDto): Promise<Achievement>,
  getAchievement(id: string): Promise<Achievement>,
  listAchievements(organizationId: string): Promise<Achievement[]>,
  listByCategory(organizationId: string, category: string): Promise<Achievement[]>,
  getVisibleAchievements(organizationId: string): Promise<Achievement[]>,
  updateAchievement(id: string, data: UpdateAchievementDto): Promise<Achievement>,
  deleteAchievement(id: string): Promise<void>,
  
  // User Achievements (3 endpoints)
  awardAchievement(data: AwardAchievementDto): Promise<UserAchievement>,
  getUserAchievements(userId: string): Promise<UserAchievement[]>,
  getRecentAchievements(userId: string, limit?: number): Promise<UserAchievement[]>,
  
  // Challenges (9 endpoints)
  createChallenge(data: CreateChallengeDto): Promise<Challenge>,
  getChallenge(id: string): Promise<Challenge>,
  listChallenges(organizationId: string): Promise<Challenge[]>,
  listByStatus(organizationId: string, status: string): Promise<Challenge[]>,
  getActiveChallenges(organizationId: string): Promise<Challenge[]>,
  activateChallenge(id: string): Promise<Challenge>,
  completeChallenge(id: string): Promise<Challenge>,
  cancelChallenge(id: string): Promise<Challenge>,
  deleteChallenge(id: string): Promise<void>,
  
  // Challenge Progress (4 endpoints)
  getChallengeProgress(challengeId: string, userId: string): Promise<ChallengeProgress>,
  listChallengeProgress(challengeId: string): Promise<ChallengeProgress[]>,
  getUserActiveChallenges(userId: string): Promise<ChallengeProgress[]>,
  incrementProgress(challengeId: string, userId: string): Promise<ChallengeProgress>,
  
  // Statistics (2 endpoints)
  getUserStats(userId: string): Promise<GamificationUserStats>,
  getLeaderboard(organizationId: string, buildingId?: string, limit?: number): Promise<LeaderboardEntry[]>,
}
```

**Backend Endpoints (22 total)**:
- **Achievements**: 7 endpoints (CRUD, list variants, visibility logic)
- **User Achievements**: 3 endpoints (award, list, recent)
- **Challenges**: 9 endpoints (CRUD, state transitions, list variants)
- **Challenge Progress**: 4 endpoints (get, list, increment with auto-complete)
- **Statistics**: 2 endpoints (user stats, leaderboard)

**Estimated Effort**: 6-7 days (10 components, 4 pages, 1 API client)

---

### 🟡 GAP 9: Community Features (Issue #49 - Phases 2-5) - **NO FRONTEND**

**Backend Status**: ✅ Complete (4 modules: Notice Board, Skills Directory, Object Sharing, Resource Booking)
**Frontend Status**: ❌ **NOT IMPLEMENTED**

**Missing for Notice Board**:
- Components: NoticeList, NoticeDetail, NoticeCreateModal, NoticeStatusBadge
- Pages: notices.astro, notice-detail.astro
- API Client: noticesApi (8 endpoints)

**Missing for Skills Directory**:
- Components: SkillList, SkillCard, SkillOfferModal, SkillRequestModal
- Pages: skills/marketplace.astro, skills/my-skills.astro
- API Client: skillsApi (10+ endpoints)

**Missing for Object Sharing**:
- Components: SharedObjectList, SharedObjectCard, ObjectLoanModal, ObjectBookingCalendar
- Pages: sharing/marketplace.astro, sharing/my-objects.astro
- API Client: sharedObjectsApi (12+ endpoints)

**Missing for Resource Booking**:
- Components: ResourceList, ResourceCalendar, BookingModal, BookingList
- Pages: bookings/calendar.astro, bookings/my-bookings.astro
- API Client: bookingsApi (15+ endpoints)

**Estimated Effort**: 10-12 days (all 4 modules combined)

---

### 🟡 GAP 10: Budget & État Daté (Issues #81, #80) - **PARTIAL FRONTEND**

**Backend Status**: ✅ Complete (Budget: 10 endpoints, État Daté: 9 endpoints)
**Frontend Status**: ⚠️ **PARTIAL** - Pages exist but components may be missing

**Check Required**:
- Does `pages/reports.astro` include Budget variance reports?
- Are there dedicated Budget approval components?
- Are there État Daté generation/tracking components?

**Potentially Missing Components**:
```
frontend/src/components/budgets/
  ├── BudgetList.svelte                 # Annual budgets
  ├── BudgetDetail.svelte               # Budget details + approval workflow
  ├── BudgetCreateModal.svelte          # Create draft budget
  ├── BudgetApprovalModal.svelte        # Approve budget in AG
  └── BudgetVarianceReport.svelte       # Variance analysis (actual vs budget)

frontend/src/components/etats-dates/
  ├── EtatDateList.svelte               # List États Datés
  ├── EtatDateDetail.svelte             # 16 legal sections display
  ├── EtatDateCreateModal.svelte        # Request État Daté
  ├── EtatDateFormWizard.svelte         # Multi-step form (16 sections)
  └── EtatDateStatusTracker.svelte      # Workflow tracker (Requested → InProgress → Generated → Delivered)
```

**Estimated Effort**: 3-4 days (if components missing)

---

### 🟡 GAP 11: GDPR Complementary Articles (Issue #90) - **PARTIAL FRONTEND**

**Backend Status**: ✅ Complete (3 REST endpoints: rectify, restrict-processing, marketing-preference)
**Frontend Status**: ⚠️ **PARTIAL** - GdprDataPanel.svelte exists but may be missing new endpoints

**Check Required**:
- Does `GdprDataPanel.svelte` include Article 16 (Rectification) form?
- Does it include Article 18 (Restriction of Processing) toggle?
- Does it include Article 21 (Marketing Opt-Out) toggle?

**Potentially Missing UI Elements**:
```svelte
<!-- In GdprDataPanel.svelte or settings/gdpr.astro -->
<section>
  <h3>Article 16: Right to Rectification</h3>
  <form on:submit={handleRectify}>
    <input name="email" />
    <input name="first_name" />
    <input name="last_name" />
    <button>Correct My Data</button>
  </form>
</section>

<section>
  <h3>Article 18: Restriction of Processing</h3>
  <button on:click={restrictProcessing}>Restrict My Data Processing</button>
  <button on:click={unrestrictProcessing}>Unrestrict Processing</button>
</section>

<section>
  <h3>Article 21: Marketing Opt-Out</h3>
  <label>
    <input type="checkbox" bind:checked={marketingOptOut} on:change={updateMarketingPreference} />
    Opt-out of marketing communications
  </label>
</section>
```

**Estimated Effort**: 1-2 days (if missing)

---

### 🟡 GAP 12: Belgian Accounting (PCMN) (Issue #79) - **PARTIAL FRONTEND**

**Backend Status**: ✅ Complete (7 REST endpoints, ~90 pre-seeded accounts)
**Frontend Status**: ⚠️ **UNKNOWN** - Need to check if `pages/journal-entries.astro` or `pages/reports.astro` use PCMN accounts

**Potentially Missing**:
- Account selector dropdown (Belgian PCMN hierarchy: Class → Subclass → Group → Account)
- Financial reports using PCMN codes (Balance Sheet, Income Statement)
- Journal entry form with PCMN account selection

**Check Required**:
- Does journal entries page use PCMN account codes?
- Do financial reports use PCMN structure?

**Estimated Effort**: 2-3 days (if missing PCMN integration)

---

## 📊 Summary Matrix

| Feature Domain | Backend Endpoints | Backend Status | Frontend Components | Frontend Pages | Frontend Status | Gap |
|----------------|------------------|----------------|-------------------|----------------|-----------------|-----|
| **Auth** | 4 | ✅ Complete | 2 | 2 | ✅ Complete | None |
| **Buildings** | 8 | ✅ Complete | 4 | 2 | ✅ Complete | None |
| **Units** | 10 | ✅ Complete | 3 | 1 | ✅ Complete | None |
| **Owners** | 8 | ✅ Complete | 5 | 2 | ✅ Complete | None |
| **Multi-Owner** | 8 | ✅ Complete | 3 | 0 | ✅ Integrated | None |
| **Expenses** | 12 | ✅ Complete | 5 | 3 | ✅ Complete | None |
| **Meetings** | 8 | ✅ Complete | 3 | 2 | ✅ Complete | None |
| **Documents** | 6 | ✅ Complete | 2 | 1 | ✅ Complete | None |
| **Board of Directors** | 10 | ✅ Complete | 4 | 3 | ✅ Complete | None |
| **Payment Reminders** | 12 | ✅ Complete | 2 | 2 | ✅ Complete | None |
| **GDPR** | 8 | ✅ Complete | 2 | 2 | ⚠️ Partial | Article 16/18/21 UI? |
| **Tickets** | 17 | ✅ Complete | 0 | 0 | ❌ Missing | **HIGH** |
| **Notifications** | 11 | ✅ Complete | 0 | 0 | ❌ Missing | **HIGH** |
| **Payments** | 22 | ✅ Complete | 0 | 0 | ❌ Missing | **HIGH** |
| **Payment Methods** | 16 | ✅ Complete | 0 | 0 | ❌ Missing | **HIGH** |
| **Quotes** | 15 | ✅ Complete | 0 | 0 | ❌ Missing | **HIGH** |
| **Convocations** | 14 | ✅ Complete | 0 | 0 | ❌ Missing | **HIGH** |
| **Resolutions** | 9 | ✅ Complete | 0 | 0 | ❌ Missing | **MEDIUM** |
| **Local Exchanges (SEL)** | 17 | ✅ Complete | 0 | 0 | ❌ Missing | **MEDIUM** |
| **Gamification** | 22 | ✅ Complete | 0 | 0 | ❌ Missing | **MEDIUM** |
| **Notice Board** | 8 | ✅ Complete | 0 | 0 | ❌ Missing | **LOW** |
| **Skills Directory** | 10+ | ✅ Complete | 0 | 0 | ❌ Missing | **LOW** |
| **Object Sharing** | 12+ | ✅ Complete | 0 | 0 | ❌ Missing | **LOW** |
| **Resource Booking** | 15+ | ✅ Complete | 0 | 0 | ❌ Missing | **LOW** |
| **Budgets** | 10 | ✅ Complete | ? | 1 | ⚠️ Unknown | Check needed |
| **États Datés** | 9 | ✅ Complete | ? | 0 | ⚠️ Unknown | Check needed |
| **PCMN Accounting** | 7 | ✅ Complete | ? | 2 | ⚠️ Unknown | Check needed |

---

## 🎯 Prioritized Action Plan

### Priority 1: Critical Features (MUST HAVE for Phase 2)

1. **Tickets System** (4-5 days)
   - 5 components, 4 pages, 1 API client
   - Maintenance request workflow critical for property management

2. **Notifications** (4-5 days)
   - 6 components, 2 pages, 1 store, 1 API client
   - Essential for user engagement and communication

3. **Payments & Payment Methods** (6-7 days)
   - 8 components, 4 pages, 2 API clients, Stripe SDK
   - Blocker for monetization (cannot charge owners without UI)

4. **Quotes Module** (5-6 days)
   - 7 components, 4 pages, 1 API client
   - Belgian legal requirement (3-quote rule for >5000€ works)

5. **Convocations** (4-5 days)
   - 6 components, 2 pages, 1 API client
   - Belgian legal requirement for AG invitations

**Subtotal Priority 1**: 23-28 days (~5-6 weeks)

---

### Priority 2: Important Features (SHOULD HAVE)

6. **Resolutions & Voting** (3-4 days)
   - 6 components, 1 page enhancement, 1 API client
   - Belgian copropriété law compliance

7. **Local Exchanges (SEL)** (5-6 days)
   - 9 components, 4 pages, 1 API client
   - Differentiating community feature

8. **Gamification** (6-7 days)
   - 10 components, 4 pages, 1 API client
   - User engagement and retention

9. **Budgets & États Datés** (3-4 days)
   - Check existing, add missing components
   - Legal compliance features

**Subtotal Priority 2**: 17-21 days (~3.5-4 weeks)

---

### Priority 3: Nice-to-Have Features

10. **Community Features** (10-12 days)
    - Notice Board, Skills, Object Sharing, Resource Booking
    - Differentiating features but not critical path

11. **GDPR Complementary** (1-2 days)
    - Add Article 16/18/21 UI elements to existing panel

12. **PCMN Integration** (2-3 days)
    - Verify and enhance accounting features

**Subtotal Priority 3**: 13-17 days (~2.5-3.5 weeks)

---

## 🚀 Total Estimated Effort

**All Gaps**: 53-66 developer days (~11-13 weeks for 1 developer, ~5.5-6.5 weeks for 2 developers)

**Priority 1 Only** (Production-Ready for Phase 2): 23-28 days (~5-6 weeks)

---

## 🎓 Recommendations

### Option A: Full Feature Parity (~13 weeks)
- Implement all gaps
- Achieve 100% backend-frontend parity
- Deploy with complete feature set

### Option B: Priority 1 Only (~6 weeks)
- Focus on critical Phase 2 features
- Deploy production-ready system
- Defer community features to Phase 3

### Option C: Hybrid Approach (~8-9 weeks)
- Implement Priority 1 (tickets, notifications, payments, quotes, convocations)
- Add Priority 2 essentials (resolutions, SEL, budgets)
- Defer Priority 3 to post-launch

---

## 📝 Next Steps

1. **Confirm Priorities**: Which features are MVP-critical?
2. **Resource Allocation**: 1 or 2 frontend developers?
3. **Timeline**: Deploy by when?
4. **Incremental Delivery**: Feature flags for gradual rollout?

---

**Analysis completed by**: Claude (Anthropic)
**Date**: 2025-11-17
**Branch**: `claude/review-remaining-issues-018z8PJuUPF4CXEuhBN9zV3y`
**Backend**: ~95% complete (400+ endpoints)
**Frontend**: ~40% complete (significant Phase 2 gaps)
