=============================================
Community Features (Issue #49 - 6 Phases)
=============================================

Overview
========

KoproGo includes a comprehensive community engagement platform with
6 interconnected modules designed to strengthen social bonds within
Belgian co-ownership buildings. All 6 phases are complete.

Phase Summary
=============

+-------+----------------------------+--------+
| Phase | Feature                    | LOC    |
+=======+============================+========+
| 1     | SEL (Local Exchange)       | ~3,000 |
+-------+----------------------------+--------+
| 2     | Community Notice Board     | ~2,940 |
+-------+----------------------------+--------+
| 3     | Skills Directory           | ~2,650 |
+-------+----------------------------+--------+
| 4     | Object Sharing Library     | ~2,905 |
+-------+----------------------------+--------+
| 5     | Resource Booking Calendar  | ~3,105 |
+-------+----------------------------+--------+
| 6     | Gamification & Achievements| ~6,500 |
+-------+----------------------------+--------+
| Total |                            |~21,100 |
+-------+----------------------------+--------+

Phase 1: SEL - Système d'Échange Local
=======================================

Time-based local exchange system (1 hour = 1 credit).

**Belgian Legal**: SELs are legal in Belgium, non-taxable if non-commercial.

**Exchange Types**: Service, ObjectLoan, SharedPurchase

**Workflow**: Offered → Requested → InProgress → Completed (or Cancelled)

**Features**:

- Automatic credit balance management
- Mutual rating system (1-5 stars)
- Participation levels (New → Beginner → Active → Veteran → Expert)
- Leaderboard and statistics

**Endpoints**: 17 REST endpoints

Phase 2: Community Notice Board
================================

Building-level notice board for announcements and community messages.

**Notice Types**: Announcement, Event, Information, Alert

**Features**: Pin/unpin, expiration dates, category filtering

**Endpoints**: 12 REST endpoints

Phase 3: Skills Directory
=========================

Directory of co-owner skills and professional competencies.

**Features**: Skill categories, search, availability status

**Endpoints**: 10 REST endpoints

Phase 4: Object Sharing Library
================================

Library for sharing and lending objects between co-owners.

**Object Categories**: Tools, Kitchen, Garden, Sports, Electronics, Books

**Features**: Availability tracking, condition reporting, lending history

**Endpoints**: 14 REST endpoints

Phase 5: Resource Booking Calendar
===================================

Calendar-based booking for shared building resources.

**Resource Types**: MeetingRoom, LaundryRoom, Parking, Garden, BBQ, Gym

**Features**: Time slot management, conflict detection, recurring bookings

**Endpoints**: 15 REST endpoints

Phase 6: Gamification & Achievements
======================================

Community engagement through achievements, challenges, and leaderboards.

**Achievement System**:

- 8 categories: Community, SEL, Booking, Sharing, Skills, Notice, Governance, Milestone
- 5 tiers: Bronze, Silver, Gold, Platinum, Diamond
- Secret achievements (hidden until earned)
- Repeatable achievements with ``times_earned`` counter

**Challenge System**:

- 3 types: Individual, Team, Building
- Time-bound with start/end dates
- Target metrics with auto-completion
- Reward points (0-10,000)

**Leaderboard**:

- Multi-source point aggregation (achievements + challenges)
- Building filter for localized competition

**Endpoints**: 22 REST endpoints (7 achievements, 3 user achievements, 9 challenges, 4 progress, 2 stats)

BDD Tests
=========

All community features are tested in ``bdd_community.rs`` (3,833 lines):

- notices.feature: 18 scenarios
- skills.feature: 14 scenarios
- shared_objects.feature: 17 scenarios
- resource_bookings.feature: 18 scenarios
- gamification.feature: 13 scenarios

**Total**: 80 BDD scenarios
