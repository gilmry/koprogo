================================================================================
Issue #49: feat: Community features (SEL, neighbor exchange, social engagement)
================================================================================

:State: **OPEN**
:Milestone: Phase 2: K3s + Automation
:Labels: phase:k3s,track:software priority:medium
:Assignees: Unassigned
:Created: 2025-10-27
:Updated: 2025-11-01
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/49>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Context
   
   **Vision ASBL KoproGo:**
   Beyond property management, KoproGo can address **social isolation** and **community building** in urban co-owned buildings by fostering neighbor engagement and local solidarity economy.
   
   **Current implementation:**
   - ✅ Technical property management (expenses, meetings, documents)
   - ❌ No community-building features
   - ❌ No neighbor interaction mechanisms
   - ❌ No local economy support
   
   **Social phenomena addressed:**
   1. **Urban isolation** - Neighbors don't know each other
   2. **Lack of solidarity** - No mutual aid networks
   3. **Consumerism** - Over-reliance on external services
   4. **Resource waste** - Underutilized skills and possessions
   
   ## Objective
   
   Transform KoproGo from a **management tool** into a **community platform** that:
   - Creates social bonds between co-owners
   - Facilitates skill/service/object exchange
   - Reduces costs through mutualization
   - Supports sustainable living
   - Aligns with ASBL social mission
   
   ## Proposed Features
   
   ### 1. SEL - Système d'Échange Local (Local Exchange Trading System)
   
   **Concept:**
   - Time-based currency (e.g., 1 hour = 1 credit)
   - Co-owners exchange services without money
   - Builds reciprocity and solidarity
   
   **Examples:**
   - "I'll help you move furniture (2h) → You teach me cooking (2h)"
   - "I'll babysit your kids (3h) → You fix my bike (3h)"
   - "I'll water your plants (1h) → You lend me your drill (1h)"
   
   **Implementation:**
   
   **Domain entity:** `LocalExchange`
   
   ```rust
   pub struct LocalExchange {
       pub id: Uuid,
       pub building_id: Uuid,
       pub provider_id: Uuid,  // Owner offering service
       pub requester_id: Uuid, // Owner requesting service
       pub exchange_type: ExchangeType,
       pub title: String,       // "Babysitting", "Gardening help"
       pub description: String,
       pub credits: i32,        // Time in hours (or custom unit)
       pub status: ExchangeStatus,
       pub offered_at: DateTime<Utc>,
       pub completed_at: Option<DateTime<Utc>>,
   }
   
   pub enum ExchangeType {
       Service,      // Skills (plumbing, gardening, tutoring)
       ObjectLoan,   // Temporary loan (tools, books, equipment)
       SharedPurchase, // Co-buying (bulk food, equipment rental)
   }
   
   pub enum ExchangeStatus {
       Offered,      // Available for anyone
       Requested,    // Someone claimed it
       InProgress,   // Exchange happening
       Completed,    // Both parties confirmed
       Cancelled,
   }
   ```
   
   **Credit tracking:**
   
   ```rust
   pub struct OwnerCreditBalance {
       pub owner_id: Uuid,
       pub building_id: Uuid,
       pub credits_earned: i32,   // Services provided
       pub credits_spent: i32,    // Services received
       pub balance: i32,          // earned - spent
   }
   ```
   
   **API Endpoints:**
   - `POST /api/v1/buildings/:id/exchanges` - Create offer
   - `GET /api/v1/buildings/:id/exchanges` - List available exchanges
   - `POST /api/v1/exchanges/:id/request` - Request exchange
   - `PUT /api/v1/exchanges/:id/complete` - Mark completed (both parties confirm)
   - `GET /api/v1/owners/:id/credit-balance` - Get credit balance
   
   **UI Components:**
   
   `ExchangeMarketplace.svelte`:
   ```svelte
   <div class="exchange-grid">
     {#each exchanges as exchange}
       <div class="exchange-card">
         <h3>{exchange.title}</h3>
         <p>{exchange.description}</p>
         <span class="credits">{exchange.credits} crédits</span>
         <span class="provider">Proposé par {exchange.provider_name}</span>
         
         {#if exchange.status === 'Offered'}
           <button on:click={() => requestExchange(exchange.id)}>
             Demander
           </button>
         {/if}
       </div>
     {/each}
   </div>
   ```
   
   ---
   
   ### 2. Neighbor Skills Directory (Annuaire des Compétences)
   
   **Concept:**
   - Co-owners publish their skills/expertise
   - Others can find help within the building
   - Reduces external service costs
   
   **Skills categories:**
   - 🔧 Bricolage (plumbing, carpentry, electrical)
   - 🍳 Cooking & baking
   - 💻 Tech support (computers, smartphones)
   - 🎨 Creative (design, photography, music)
   - 📚 Education (tutoring, languages)
   - 🌱 Gardening
   - 🚗 Car repair
   - 🧵 Sewing & repair
   - 🐕 Pet sitting
   
   **Implementation:**
   
   ```rust
   pub struct OwnerSkill {
       pub owner_id: Uuid,
       pub skill_category: SkillCategory,
       pub skill_name: String,
       pub description: String,
       pub available_for_exchange: bool, // SEL eligible
       pub hourly_rate_credits: Option<i32>, // If SEL, how many credits/hour
       pub is_public: bool, // Visible to all building residents
   }
   ```
   
   **UI:** Profile section showing skills + search/filter
   
   ---
   
   ### 3. Object Sharing Library (Bibliothèque d'Objets)
   
   **Concept:**
   - Co-owners register items they're willing to lend
   - Reduces duplicate purchases
   - Promotes circular economy
   
   **Shareable items:**
   - 🛠️ Tools (drill, ladder, lawnmower)
   - 📚 Books & media
   - 🎮 Games & entertainment
   - 🏕️ Sports & outdoor equipment (tent, bike, kayak)
   - 🍳 Kitchen appliances (mixer, raclette machine)
   - 👶 Baby/kids items (stroller, toys)
   - 🚗 Car accessories (roof rack, snow chains)
   
   **Implementation:**
   
   ```rust
   pub struct SharedObject {
       pub id: Uuid,
       pub owner_id: Uuid,
       pub building_id: Uuid,
       pub name: String,
       pub category: ObjectCategory,
       pub description: String,
       pub condition: String,
       pub photo_url: Option<String>,
       pub is_available: bool,
       pub current_borrower_id: Option<Uuid>,
       pub borrowed_at: Option<DateTime<Utc>>,
       pub return_by: Option<DateTime<Utc>>,
   }
   
   pub struct BorrowingRequest {
       pub id: Uuid,
       pub object_id: Uuid,
       pub requester_id: Uuid,
       pub start_date: DateTime<Utc>,
       pub end_date: DateTime<Utc>,
       pub status: BorrowStatus, // Pending, Approved, Rejected, Returned
   }
   ```
   
   **API Endpoints:**
   - `POST /api/v1/buildings/:id/shared-objects` - Register object
   - `GET /api/v1/buildings/:id/shared-objects` - Browse available items
   - `POST /api/v1/shared-objects/:id/borrow` - Request to borrow
   - `PUT /api/v1/borrowings/:id/return` - Mark returned
   
   **UI:**
   
   `SharedObjectLibrary.svelte`:
   ```svelte
   <div class="object-grid">
     {#each objects as object}
       <div class="object-card">
         <img src={object.photo_url} alt={object.name} />
         <h4>{object.name}</h4>
         <p>Propriétaire: {object.owner_name}</p>
         
         {#if object.is_available}
           <span class="badge success">Disponible</span>
           <button on:click={() => requestBorrow(object.id)}>Emprunter</button>
         {:else}
           <span class="badge warning">Emprunté jusqu'au {object.return_by}</span>
         {/if}
       </div>
     {/each}
   </div>
   ```
   
   ---
   
   ### 4. Community Notice Board (Panneau d'Affichage Communautaire)
   
   **Concept:**
   - Digital bulletin board for neighbors
   - Announcements, events, lost & found, recommendations
   
   **Post types:**
   - 📢 Announcements (building events, maintenance notices)
   - 🎉 Events (potluck, movie night, garage sale)
   - 🔍 Lost & Found
   - 💡 Recommendations (local businesses, services)
   - ❓ Questions & Answers
   - 🚗 Carpool / rideshare
   - 📦 Group buying (bulk orders)
   
   **Implementation:**
   
   ```rust
   pub struct CommunityPost {
       pub id: Uuid,
       pub building_id: Uuid,
       pub author_id: Uuid,
       pub post_type: PostType,
       pub title: String,
       pub content: String,
       pub photo_url: Option<String>,
       pub expires_at: Option<DateTime<Utc>>, // Auto-hide after date
       pub created_at: DateTime<Utc>,
       pub is_pinned: bool, // Syndic can pin important posts
   }
   
   pub struct PostComment {
       pub id: Uuid,
       pub post_id: Uuid,
       pub author_id: Uuid,
       pub content: String,
       pub created_at: DateTime<Utc>,
   }
   ```
   
   **UI:**
   
   `CommunityBoard.svelte`:
   ```svelte
   <div class="notice-board">
     <button on:click={createPost}>+ Nouvelle annonce</button>
     
     {#each posts as post}
       <div class="post-card {post.is_pinned ? 'pinned' : ''}">
         <span class="post-type-badge">{post.post_type}</span>
         <h3>{post.title}</h3>
         <p>{post.content}</p>
         <span class="author">Par {post.author_name}</span>
         <span class="date">{formatDate(post.created_at)}</span>
         
         <div class="comments">
           {#each post.comments as comment}
             <div class="comment">{comment.content}</div>
           {/each}
         </div>
       </div>
     {/each}
   </div>
   ```
   
   ---
   
   ### 5. Shared Building Resources Calendar
   
   **Concept:**
   - Reserve shared spaces/resources
   - Avoid conflicts, maximize usage
   
   **Reservable resources:**
   - 🏠 Common room (parties, meetings)
   - 🚗 Guest parking spot
   - 🧺 Laundry machines
   - 🚲 Bike repair station
   - 🌳 Rooftop garden plot
   - 📦 Package room key
   
   **Implementation:**
   
   ```rust
   pub struct SharedResource {
       pub id: Uuid,
       pub building_id: Uuid,
       pub name: String,
       pub resource_type: ResourceType,
       pub capacity: i32, // Max simultaneous users
       pub booking_window_days: i32, // How far in advance can book
   }
   
   pub struct ResourceBooking {
       pub id: Uuid,
       pub resource_id: Uuid,
       pub owner_id: Uuid,
       pub start_time: DateTime<Utc>,
       pub end_time: DateTime<Utc>,
       pub purpose: String,
       pub status: BookingStatus, // Pending, Confirmed, Cancelled
   }
   ```
   
   **UI:**
   
   `ResourceCalendar.svelte`:
   ```svelte
   <FullCalendar
     events={bookings}
     on:dateClick={handleDateClick}
     on:eventClick={handleEventClick}
   />
   
   <ResourceBookingModal
     bind:this={bookingModal}
     resources={availableResources}
     on:book={createBooking}
   />
   ```
   
   ---
   
   ### 6. Gamification & Engagement
   
   **Concept:**
   - Reward community participation
   - Encourage sustainable behaviors
   
   **Achievements:**
   - 🌟 "Good Neighbor" - 10 exchanges completed
   - 🔧 "Handyman Hero" - Helped 5 neighbors
   - 📚 "Book Lover" - Shared 20 books
   - ♻️ "Eco Warrior" - Participated in 10 shared purchases
   - 🎉 "Event Organizer" - Organized 3 community events
   
   **Leaderboard:**
   - Top contributors (by exchange credits earned)
   - Most active posters
   - Most helpful neighbors (based on ratings)
   
   **Implementation:**
   
   ```rust
   pub struct OwnerAchievement {
       pub owner_id: Uuid,
       pub achievement_type: AchievementType,
       pub earned_at: DateTime<Utc>,
       pub level: i32, // Bronze, Silver, Gold
   }
   
   pub struct CommunityLeaderboard {
       pub building_id: Uuid,
       pub period: String, // "monthly", "yearly", "all-time"
       pub top_contributors: Vec<(Uuid, i32, String)>, // (owner_id, points, name)
   }
   ```
   
   ---
   
   ## Integration with ASBL Mission
   
   **Social impact metrics:**
   - Number of exchanges per building
   - Total credits exchanged (time saved)
   - Number of objects shared (waste reduction)
   - Community engagement rate (% active users)
   - Social events organized per month
   
   **Reports for ASBL:**
   - Quarterly community impact report
   - Annual social solidarity report (for funders/grants)
   - Sustainability metrics (CO2 saved, waste reduced)
   
   ---
   
   ## Phasing
   
   ### Phase 1 (K3s - Q3 2026) - MVP Community Features
   - SEL basic (service exchange)
   - Community notice board
   - Skills directory
   
   ### Phase 2 (K3s - Q4 2026) - Sharing Economy
   - Object sharing library
   - Resource booking calendar
   
   ### Phase 3 (K8s - Q2 2027+) - Gamification & Analytics
   - Achievements & leaderboard
   - Community impact dashboard
   - Social analytics for ASBL
   
   ---
   
   ## Technical Considerations
   
   **Moderation:**
   - Syndic can moderate/delete inappropriate posts
   - Flagging system for abuse
   - Community guidelines
   
   **Privacy:**
   - Opt-in for skill directory visibility
   - Contact info hidden (messaging via platform)
   - GDPR-compliant (users can delete data)
   
   **Notifications:**
   - Email/SMS for exchange requests
   - Reminders for object returns
   - Event announcements
   
   **Mobile-first:**
   - SEL works best on mobile (spontaneous exchanges)
   - Push notifications for real-time engagement
   
   ---
   
   ## Competitive Advantage
   
   **Unique positioning:**
   - Only copropriété platform with community features
   - Addresses social isolation (mental health benefit)
   - Reduces living costs (solidarity economy)
   - Aligns with ESG/sustainability goals
   - Perfect for ASBL funding/grants (social impact)
   
   **Similar platforms (inspiration):**
   - Peerby (object sharing, Netherlands)
   - ShareVoisins (neighbor exchange, France)
   - TimeBank (time-based currency, UK)
   - Nextdoor (neighborhood social network, US)
   
   **KoproGo advantage:** Integrated with property management (no separate app)
   
   ---
   
   ## Testing & Validation
   
   - [ ] SEL exchange flow (offer, request, complete)
   - [ ] Credit balance calculation correct
   - [ ] Object borrowing/return flow
   - [ ] Resource calendar booking (no double-booking)
   - [ ] Notification delivery
   - [ ] Mobile UX optimized
   - [ ] Moderation tools functional
   
   ## Acceptance Criteria
   
   - [ ] SEL entities + API endpoints functional
   - [ ] Skills directory complete
   - [ ] Object sharing library operational
   - [ ] Community notice board with comments
   - [ ] Resource booking calendar
   - [ ] Gamification achievements (basic)
   - [ ] Community impact metrics dashboard
   - [ ] Mobile-responsive UI
   - [ ] Moderation tools for Syndic
   - [ ] GDPR-compliant (opt-in, data deletion)
   
   ## Effort Estimate
   
   **Large** (10-15 days)
   - Week 1: SEL (domain, API, UI) - 5 days
   - Week 2: Skills + Objects + Notice board - 5 days
   - Week 3: Resource calendar + Gamification - 3-5 days
   
   ## Related
   
   - Aligns with: ASBL social mission
   - Enhances: User retention & engagement
   - Differentiates: From competitors (unique feature)
   - Supports: Sustainability goals (shared economy)
   
   ## Future Ideas (Post-MVP)
   
   - Carpool coordination (reduce traffic)
   - Community vegetable garden management
   - Bulk energy purchasing (solar panels group buy)
   - Neighbor-to-neighbor marketplace (garage sale)
   - Integration with local businesses (discounts for co-owners)
   - Inter-building exchanges (city-wide SEL network)
   
   ---
   
   ## Legal & Compliance
   
   **SEL legal status (Belgium):**
   - SELs are legal and recognized
   - No taxation if non-commercial (barter)
   - Must not replace professional services (insurance issues)
   - Clear T&Cs required (liability disclaimer)
   
   **Insurance:**
   - Object damage/loss liability (owner's responsibility)
   - Platform not liable for exchanges (disclaimer)
   
   **GDPR:**
   - Skills/objects are personal data (consent required)
   - Users can hide/delete their offerings anytime
   
   ---
   
   ## References
   
   - SEL Belgium: https://www.selsbelgium.be/
   - Peerby: https://www.peerby.com/
   - TimeBank: https://timebanking.org/
   - Circular economy: https://ellenmacarthurfoundation.org/
   - Community engagement metrics: https://www.socialpinpoint.com/

.. raw:: html

   </div>

