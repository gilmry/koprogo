# OpenAPI Handlers Report

Total handlers found: 423

## By Tag:

### Account (8 endpoints)
- POST /accounts → create_account
- GET /accounts → list_accounts
- GET /accounts/{id} → get_account
- GET /accounts/code/{code} → get_account_by_code
- PUT /accounts/{id} → update_account
- DELETE /accounts/{id} → delete_account
- POST /accounts/seed/belgian-pcmn → seed_belgian_pcmn
- GET /accounts/count → count_accounts

### Admin_Gdpr (3 endpoints)
- GET /admin/gdpr/audit-logs → list_audit_logs
- GET /admin/gdpr/users/{user_id}/export → admin_export_user_data
- DELETE /admin/gdpr/users/{user_id}/erase → admin_erase_user_data

### Auth (5 endpoints)
- POST /auth/login → login
- POST /auth/register → register
- GET /auth/me → get_current_user
- POST /auth/refresh → refresh_token
- POST /auth/switch-role → switch_role

### BoardMembers (9 endpoints)
- POST /board-members → elect_board_member
- GET /board-members/{id} → get_board_member
- GET /buildings/{building_id}/board-members/active → list_active_board_members
- GET /buildings/{building_id}/board-members → list_all_board_members
- PUT /board-members/{id}/renew → renew_mandate
- DELETE /board-members/{id} → remove_board_member
- GET /buildings/{building_id}/board-members/stats → get_board_stats
- GET /board-members/dashboard → get_board_dashboard
- GET /board-members/my-mandates → get_my_mandates

### Board_Decision (9 endpoints)
- POST /board-decisions → create_decision
- GET /board-decisions/{id} → get_decision
- GET /buildings/{building_id}/board-decisions → list_decisions_by_building
- GET /buildings/{building_id}/board-decisions/status/{status} → list_decisions_by_status
- GET /buildings/{building_id}/board-decisions/overdue → list_overdue_decisions
- PUT /board-decisions/{id} → update_decision_status
- POST /board-decisions/{id}/notes → add_notes
- PUT /board-decisions/{id}/complete → complete_decision
- GET /buildings/{building_id}/board-decisions/stats → get_decision_stats

### Budgets (16 endpoints)
- POST /budgets → create_budget
- GET /budgets/{id} → get_budget
- GET /buildings/{building_id}/budgets/fiscal-year/{fiscal_year} → get_budget_by_building_and_fiscal_year
- GET /buildings/{building_id}/budgets/active → get_active_budget
- GET /buildings/{building_id}/budgets → list_budgets_by_building
- GET /budgets/fiscal-year/{fiscal_year} → list_budgets_by_fiscal_year
- GET /budgets/status/{status} → list_budgets_by_status
- GET /budgets → list_budgets
- PUT /budgets/{id} → update_budget
- PUT /budgets/{id}/submit → submit_budget
- PUT /budgets/{id}/approve → approve_budget
- PUT /budgets/{id}/reject → reject_budget
- PUT /budgets/{id}/archive → archive_budget
- GET /budgets/stats → get_budget_stats
- GET /budgets/{id}/variance → get_budget_variance
- DELETE /budgets/{id} → delete_budget

### Buildings (5 endpoints)
- POST /buildings → create_building
- GET /buildings → list_buildings
- GET /buildings/{id} → get_building
- PUT /buildings/{id} → update_building
- DELETE /buildings/{id} → delete_building

### Charge_Distribution (4 endpoints)
- POST /invoices/{expense_id}/calculate-distribution → calculate_and_save_distribution
- GET /invoices/{expense_id}/distribution → get_distribution_by_expense
- GET /owners/{owner_id}/distributions → get_distributions_by_owner
- GET /owners/{owner_id}/total-due → get_total_due_by_owner

### Convocations (15 endpoints)
- POST /convocations → create_convocation
- GET /convocations/{id} → get_convocation
- GET /meetings/{meeting_id}/convocation → get_convocation_by_meeting
- GET /buildings/{building_id}/convocations → list_building_convocations
- GET /organizations/{organization_id}/convocations → list_organization_convocations
- DELETE /convocations/{id} → delete_convocation
- PUT /convocations/{id}/schedule → schedule_convocation
- POST /convocations/{id}/send → send_convocation
- PUT /convocations/{id}/cancel → cancel_convocation
- GET /convocations/{id}/recipients → list_convocation_recipients
- GET /convocations/{id}/tracking-summary → get_convocation_tracking_summary
- PUT /convocation-recipients/{id}/email-opened → mark_recipient_email_opened
- PUT /convocation-recipients/{id}/attendance → update_recipient_attendance
- PUT /convocation-recipients/{id}/proxy → set_recipient_proxy
- POST /convocations/{id}/send-reminders → send_convocation_reminders

### Dashboard (2 endpoints)
- GET /dashboard/accountant/stats → get_accountant_stats
- GET /dashboard/accountant/transactions → get_recent_transactions

### Documents (10 endpoints)
- POST /documents → upload_document
- GET /documents/{id} → get_document
- GET /documents → list_documents
- GET /documents/{id}/download → download_document
- GET /buildings/{building_id}/documents → list_documents_by_building
- GET /meetings/{meeting_id}/documents → list_documents_by_meeting
- GET /expenses/{expense_id}/documents → list_documents_by_expense
- PUT /documents/{id}/link-meeting → link_document_to_meeting
- PUT /documents/{id}/link-expense → link_document_to_expense
- DELETE /documents/{id} → delete_document

### EtatsDates (15 endpoints)
- POST /etats-dates → create_etat_date
- GET /etats-dates/{id} → get_etat_date
- GET /etats-dates/reference/{reference_number} → get_by_reference_number
- GET /etats-dates → list_etats_dates
- GET /units/{unit_id}/etats-dates → list_etats_dates_by_unit
- GET /buildings/{building_id}/etats-dates → list_etats_dates_by_building
- PUT /etats-dates/{id}/mark-in-progress → mark_in_progress
- PUT /etats-dates/{id}/mark-generated → mark_generated
- PUT /etats-dates/{id}/mark-delivered → mark_delivered
- PUT /etats-dates/{id}/financial → update_financial_data
- PUT /etats-dates/{id}/additional-data → update_additional_data
- GET /etats-dates/overdue → list_overdue
- GET /etats-dates/expired → list_expired
- GET /etats-dates/stats → get_stats
- DELETE /etats-dates/{id} → delete_etat_date

### Expenses (16 endpoints)
- POST /expenses → create_expense
- GET /expenses/{id} → get_expense
- GET /expenses → list_expenses
- GET /buildings/{building_id}/expenses → list_expenses_by_building
- PUT /expenses/{id}/mark-paid → mark_expense_paid
- POST /expenses/{id}/mark-overdue → mark_expense_overdue
- POST /expenses/{id}/cancel → cancel_expense
- POST /expenses/{id}/reactivate → reactivate_expense
- POST /expenses/{id}/unpay → unpay_expense
- POST /invoices/draft → create_invoice_draft
- PUT /invoices/{id} → update_invoice_draft
- PUT /invoices/{id}/submit → submit_invoice_for_approval
- PUT /invoices/{id}/approve → approve_invoice
- PUT /invoices/{id}/reject → reject_invoice
- GET /invoices/pending → get_pending_invoices
- GET /invoices/{id} → get_invoice

### Financial_Report (2 endpoints)
- GET /reports/balance-sheet → generate_balance_sheet
- GET /reports/income-statement → generate_income_statement

### GDPR (6 endpoints)
- GET /gdpr/export → export_user_data
- DELETE /gdpr/erase → erase_user_data
- GET /gdpr/can-erase → can_erase_user
- PUT /gdpr/rectify → rectify_user_data
- PUT /gdpr/restrict-processing → restrict_user_processing
- PUT /gdpr/marketing-preference → set_marketing_preference

### Gamification (27 endpoints)
- POST /achievements → create_achievement
- GET /achievements/{id} → get_achievement
- GET /organizations/{organization_id}/achievements → list_achievements
- GET /organizations/{organization_id}/achievements/category/{category} → list_achievements_by_category
- GET /organizations/{organization_id}/achievements/visible → list_visible_achievements
- PUT /achievements/{id} → update_achievement
- DELETE /achievements/{id} → delete_achievement
- POST /users/achievements → award_achievement
- GET /users/achievements → get_user_achievements
- GET /users/achievements/recent → get_recent_achievements
- POST /challenges → create_challenge
- GET /challenges/{id} → get_challenge
- GET /organizations/{organization_id}/challenges → list_challenges
- GET /organizations/{organization_id}/challenges/status/{status} → list_challenges_by_status
- GET /buildings/{building_id}/challenges → list_building_challenges
- GET /organizations/{organization_id}/challenges/active → list_active_challenges
- PUT /challenges/{id} → update_challenge
- PUT /challenges/{id}/activate → activate_challenge
- PUT /challenges/{id}/complete → complete_challenge
- PUT /challenges/{id}/cancel → cancel_challenge
- DELETE /challenges/{id} → delete_challenge
- GET /challenges/{challenge_id}/progress → get_challenge_progress
- GET /challenges/{challenge_id}/all-progress → list_challenge_progress
- GET /users/challenges/active → list_user_active_challenges
- POST /challenges/{challenge_id}/progress/increment → increment_progress
- GET /organizations/{organization_id}/gamification/stats → get_gamification_user_stats
- GET /organizations/{organization_id}/gamification/leaderboard → get_gamification_leaderboard

### Journal_Entry (4 endpoints)
- POST /journal-entries → create_journal_entry
- GET /journal-entries → list_journal_entries
- GET /journal-entries/{id} → get_journal_entry
- DELETE /journal-entries/{id} → delete_journal_entry

### LocalExchanges (17 endpoints)
- POST /exchanges → create_exchange
- GET /exchanges/{id} → get_exchange
- GET /buildings/{building_id}/exchanges → list_building_exchanges
- GET /buildings/{building_id}/exchanges/available → list_available_exchanges
- GET /owners/{owner_id}/exchanges → list_owner_exchanges
- GET /buildings/{building_id}/exchanges/type/{exchange_type} → list_exchanges_by_type
- POST /exchanges/{id}/request → request_exchange
- POST /exchanges/{id}/start → start_exchange
- POST /exchanges/{id}/complete → complete_exchange
- POST /exchanges/{id}/cancel → cancel_exchange
- PUT /exchanges/{id}/rate-provider → rate_provider
- PUT /exchanges/{id}/rate-requester → rate_requester
- DELETE /exchanges/{id} → delete_exchange
- GET /owners/{owner_id}/buildings/{building_id}/credit-balance → get_credit_balance
- GET /buildings/{building_id}/leaderboard → get_leaderboard
- GET /buildings/{building_id}/sel-statistics → get_sel_statistics
- GET /owners/{owner_id}/exchange-summary → get_owner_summary

### Meetings (11 endpoints)
- POST /meetings → create_meeting
- GET /meetings/{id} → get_meeting
- GET /meetings → list_meetings
- GET /buildings/{building_id}/meetings → list_meetings_by_building
- PUT /meetings/{id} → update_meeting
- POST /meetings/{id}/agenda → add_agenda_item
- POST /meetings/{id}/complete → complete_meeting
- POST /meetings/{id}/cancel → cancel_meeting
- POST /meetings/{id}/reschedule → reschedule_meeting
- DELETE /meetings/{id} → delete_meeting
- GET /meetings/{id}/export-minutes-pdf → export_meeting_minutes_pdf

### Notices (17 endpoints)
- POST /notices → create_notice
- GET /notices/{id} → get_notice
- GET /buildings/{building_id}/notices → list_building_notices
- GET /buildings/{building_id}/notices/published → list_published_notices
- GET /buildings/{building_id}/notices/pinned → list_pinned_notices
- GET /buildings/{building_id}/notices/type/{notice_type} → list_notices_by_type
- GET /buildings/{building_id}/notices/category/{category} → list_notices_by_category
- GET /buildings/{building_id}/notices/status/{status} → list_notices_by_status
- GET /owners/{author_id}/notices → list_author_notices
- PUT /notices/{id} → update_notice
- POST /notices/{id}/publish → publish_notice
- POST /notices/{id}/archive → archive_notice
- POST /notices/{id}/pin → pin_notice
- POST /notices/{id}/unpin → unpin_notice
- PUT /notices/{id}/expiration → set_expiration
- DELETE /notices/{id} → delete_notice
- GET /buildings/{building_id}/notices/statistics → get_notice_statistics

### Notifications (11 endpoints)
- POST /notifications → create_notification
- GET /notifications/{id} → get_notification
- GET /notifications/my-notifications → list_my_notifications
- GET /notifications/unread → list_unread_notifications
- PUT /notifications/{id}/mark-read → mark_notification_read
- PUT /notifications/mark-all-read → mark_all_notifications_read
- DELETE /notifications/{id} → delete_notification
- GET /notifications/stats → get_notification_stats
- GET /notification-preferences → get_user_preferences
- GET /notification-preferences/{notification_type} → get_preference
- PUT /notification-preferences/{notification_type} → update_preference

### Organization (6 endpoints)
- GET /organizations → list_organizations
- POST /organizations → create_organization
- PUT /organizations/{id} → update_organization
- PUT /organizations/{id}/activate → activate_organization
- PUT /organizations/{id}/suspend → suspend_organization
- DELETE /organizations/{id} → delete_organization

### Owners (5 endpoints)
- POST /owners → create_owner
- GET /owners → list_owners
- GET /owners/{id} → get_owner
- PUT /owners/{id} → update_owner
- PUT /owners/{id}/link-user → link_owner_to_user

### PaymentMethods (15 endpoints)
- POST /payment-methods → create_payment_method
- GET /payment-methods/{id} → get_payment_method
- GET /payment-methods/stripe/{stripe_payment_method_id} → get_payment_method_by_stripe_id
- GET /owners/{owner_id}/payment-methods → list_owner_payment_methods
- GET /owners/{owner_id}/payment-methods/active → list_active_owner_payment_methods
- GET /owners/{owner_id}/payment-methods/default → get_default_payment_method
- GET /organizations/{organization_id}/payment-methods → list_organization_payment_methods
- GET /owners/{owner_id}/payment-methods/type/{method_type} → list_payment_methods_by_type
- PUT /payment-methods/{id} → update_payment_method
- PUT /payment-methods/{id}/set-default → set_payment_method_as_default
- PUT /payment-methods/{id}/deactivate → deactivate_payment_method
- PUT /payment-methods/{id}/reactivate → reactivate_payment_method
- DELETE /payment-methods/{id} → delete_payment_method
- GET /owners/{owner_id}/payment-methods/count → count_active_payment_methods
- GET /owners/{owner_id}/payment-methods/has-active → has_active_payment_methods

### PaymentRecovery (16 endpoints)
- POST /payment-reminders → create_reminder
- GET /payment-reminders/{id} → get_reminder
- GET /expenses/{expense_id}/payment-reminders → list_by_expense
- GET /owners/{owner_id}/payment-reminders → list_by_owner
- GET /owners/{owner_id}/payment-reminders/active → list_active_by_owner
- GET /payment-reminders → list_by_organization
- PUT /payment-reminders/{id}/mark-sent → mark_as_sent
- PUT /payment-reminders/{id}/mark-opened → mark_as_opened
- PUT /payment-reminders/{id}/mark-paid → mark_as_paid
- PUT /payment-reminders/{id}/cancel → cancel_reminder
- POST /payment-reminders/{id}/escalate → escalate_reminder
- PUT /payment-reminders/{id}/tracking-number → add_tracking_number
- GET /payment-reminders/stats → get_recovery_stats
- GET /payment-reminders/overdue-without-reminders → find_overdue_without_reminders
- POST /payment-reminders/bulk-create → bulk_create_reminders
- DELETE /payment-reminders/{id} → delete_reminder

### Payments (22 endpoints)
- POST /payments → create_payment
- GET /payments/{id} → get_payment
- GET /payments/stripe/{stripe_payment_intent_id} → get_payment_by_stripe_intent
- GET /owners/{owner_id}/payments → list_owner_payments
- GET /buildings/{building_id}/payments → list_building_payments
- GET /expenses/{expense_id}/payments → list_expense_payments
- GET /organizations/{organization_id}/payments → list_organization_payments
- GET /payments/status/{status} → list_payments_by_status
- GET /payments/pending → list_pending_payments
- GET /payments/failed → list_failed_payments
- PUT /payments/{id}/processing → mark_payment_processing
- PUT /payments/{id}/requires-action → mark_payment_requires_action
- PUT /payments/{id}/succeeded → mark_payment_succeeded
- PUT /payments/{id}/failed → mark_payment_failed
- PUT /payments/{id}/cancelled → mark_payment_cancelled
- POST /payments/{id}/refund → refund_payment
- DELETE /payments/{id} → delete_payment
- GET /owners/{owner_id}/payments/stats → get_owner_payment_stats
- GET /buildings/{building_id}/payments/stats → get_building_payment_stats
- GET /expenses/{expense_id}/payments/total → get_expense_total_paid
- GET /owners/{owner_id}/payments/total → get_owner_total_paid
- GET /buildings/{building_id}/payments/total → get_building_total_paid

### Pcn (3 endpoints)
- POST /pcn/report/{building_id} → generate_pcn_report
- GET /pcn/export/pdf/{building_id} → export_pcn_pdf
- GET /pcn/export/excel/{building_id} → export_pcn_excel

### Public (1 endpoints)
- GET /public/buildings/{slug}/syndic → get_public_syndic_info

### Quotes (15 endpoints)
- POST /quotes → create_quote
- GET /quotes/{id} → get_quote
- GET /buildings/{building_id}/quotes → list_building_quotes
- GET /contractors/{contractor_id}/quotes → list_contractor_quotes
- GET /buildings/{building_id}/quotes/status/{status} → list_quotes_by_status
- POST /quotes/{id}/submit → submit_quote
- POST /quotes/{id}/review → start_review
- POST /quotes/{id}/accept → accept_quote
- POST /quotes/{id}/reject → reject_quote
- POST /quotes/{id}/withdraw → withdraw_quote
- POST /quotes/compare → compare_quotes
- PUT /quotes/{id}/contractor-rating → update_contractor_rating
- DELETE /quotes/{id} → delete_quote
- GET /buildings/{building_id}/quotes/count → count_building_quotes
- GET /buildings/{building_id}/quotes/status/{status}/count → count_quotes_by_status

### Resolutions (9 endpoints)
- POST /meetings/{meeting_id}/resolutions → create_resolution
- GET /resolutions/{id} → get_resolution
- GET /meetings/{meeting_id}/resolutions → list_meeting_resolutions
- DELETE /resolutions/{id} → delete_resolution
- POST /resolutions/{resolution_id}/vote → cast_vote
- GET /resolutions/{resolution_id}/votes → list_resolution_votes
- PUT /votes/{vote_id} → change_vote
- PUT /resolutions/{resolution_id}/close → close_voting
- GET /meetings/{meeting_id}/vote-summary → get_meeting_vote_summary

### ResourceBookings (19 endpoints)
- POST /resource-bookings → create_booking
- GET /resource-bookings/{id} → get_booking
- GET /buildings/{building_id}/resource-bookings → list_building_bookings
- GET /buildings/{building_id}/resource-bookings/type/{resource_type} → list_by_resource_type
- GET /buildings/{building_id}/resource-bookings/resource/{resource_type}/{resource_name} → list_by_resource
- GET /resource-bookings/my → list_my_bookings
- GET /resource-bookings/my/status/{status} → list_my_bookings_by_status
- GET /buildings/{building_id}/resource-bookings/status/{status} → list_building_bookings_by_status
- GET /buildings/{building_id}/resource-bookings/upcoming → list_upcoming_bookings
- GET /buildings/{building_id}/resource-bookings/active → list_active_bookings
- GET /buildings/{building_id}/resource-bookings/past → list_past_bookings
- PUT /resource-bookings/{id} → update_booking
- POST /resource-bookings/{id}/cancel → cancel_booking
- POST /resource-bookings/{id}/complete → complete_booking
- POST /resource-bookings/{id}/no-show → mark_no_show
- POST /resource-bookings/{id}/confirm → confirm_booking
- DELETE /resource-bookings/{id} → delete_booking
- GET /resource-bookings/check-conflicts → check_conflicts
- GET /buildings/{building_id}/resource-bookings/statistics → get_booking_statistics

### Seed (2 endpoints)
- POST /seed/demo → seed_demo_data
- POST /seed/clear → clear_demo_data

### SharedObjects (17 endpoints)
- POST /shared-objects → create_shared_object
- GET /shared-objects/{id} → get_shared_object
- GET /buildings/{building_id}/shared-objects → list_building_objects
- GET /buildings/{building_id}/shared-objects/available → list_available_objects
- GET /buildings/{building_id}/shared-objects/borrowed → list_borrowed_objects
- GET /buildings/{building_id}/shared-objects/overdue → list_overdue_objects
- GET /buildings/{building_id}/shared-objects/free → list_free_objects
- GET /buildings/{building_id}/shared-objects/category/{category} → list_objects_by_category
- GET /owners/{owner_id}/shared-objects → list_owner_objects
- GET /shared-objects/my-borrowed → list_my_borrowed_objects
- PUT /shared-objects/{id} → update_shared_object
- POST /shared-objects/{id}/mark-available → mark_object_available
- POST /shared-objects/{id}/mark-unavailable → mark_object_unavailable
- POST /shared-objects/{id}/borrow → borrow_object
- POST /shared-objects/{id}/return → return_object
- DELETE /shared-objects/{id} → delete_shared_object
- GET /buildings/{building_id}/shared-objects/statistics → get_object_statistics

### Skills (14 endpoints)
- POST /skills → create_skill
- GET /skills/{id} → get_skill
- GET /buildings/{building_id}/skills → list_building_skills
- GET /buildings/{building_id}/skills/available → list_available_skills
- GET /buildings/{building_id}/skills/free → list_free_skills
- GET /buildings/{building_id}/skills/professional → list_professional_skills
- GET /buildings/{building_id}/skills/category/{category} → list_skills_by_category
- GET /buildings/{building_id}/skills/expertise/{level} → list_skills_by_expertise
- GET /owners/{owner_id}/skills → list_owner_skills
- PUT /skills/{id} → update_skill
- POST /skills/{id}/mark-available → mark_skill_available
- POST /skills/{id}/mark-unavailable → mark_skill_unavailable
- DELETE /skills/{id} → delete_skill
- GET /buildings/{building_id}/skills/statistics → get_skill_statistics

### Stats (5 endpoints)
- GET /stats/dashboard → get_dashboard_stats
- GET /stats/owner → get_owner_stats
- GET /stats/syndic → get_syndic_stats
- GET /stats/syndic/urgent-tasks → get_syndic_urgent_tasks
- GET /stats/seed-data → get_seed_data_stats

### TechnicalInspections (13 endpoints)
- POST /technical-inspections → create_technical_inspection
- GET /technical-inspections/{id} → get_technical_inspection
- GET /buildings/{building_id}/technical-inspections → list_building_technical_inspections
- GET /organizations/{organization_id}/technical-inspections → list_organization_technical_inspections
- GET /technical-inspections → list_technical_inspections_paginated
- PUT /technical-inspections/{id} → update_technical_inspection
- DELETE /technical-inspections/{id} → delete_technical_inspection
- GET /buildings/{building_id}/technical-inspections/overdue → get_overdue_inspections
- GET /buildings/{building_id}/technical-inspections/upcoming → get_upcoming_inspections
- GET /buildings/{building_id}/technical-inspections/type/{inspection_type} → get_inspections_by_type
- POST /technical-inspections/{id}/reports → add_report
- POST /technical-inspections/{id}/photos → add_inspection_photo
- POST /technical-inspections/{id}/certificates → add_certificate

### Tickets (16 endpoints)
- POST /tickets → create_ticket
- GET /tickets/{id} → get_ticket
- GET /buildings/{building_id}/tickets → list_building_tickets
- GET /organizations/{organization_id}/tickets → list_organization_tickets
- GET /tickets/my-tickets → list_my_tickets
- GET /tickets/assigned-to-me → list_assigned_tickets
- GET /buildings/{building_id}/tickets/status/{status} → list_tickets_by_status
- DELETE /tickets/{id} → delete_ticket
- PUT /tickets/{id}/assign → assign_ticket
- PUT /tickets/{id}/start-work → start_work
- PUT /tickets/{id}/resolve → resolve_ticket
- PUT /tickets/{id}/close → close_ticket
- PUT /tickets/{id}/cancel → cancel_ticket
- PUT /tickets/{id}/reopen → reopen_ticket
- GET /buildings/{building_id}/tickets/statistics → get_ticket_statistics
- GET /buildings/{building_id}/tickets/overdue → get_overdue_tickets

### Unit_Owner (9 endpoints)
- POST /units/{unit_id}/owners → add_owner_to_unit
- DELETE /units/{unit_id}/owners/{owner_id} → remove_owner_from_unit
- PUT /unit-owners/{id} → update_unit_owner
- GET /units/{unit_id}/owners → get_unit_owners
- GET /owners/{owner_id}/units → get_owner_units
- GET /units/{unit_id}/owners/history → get_unit_ownership_history
- GET /owners/{owner_id}/units/history → get_owner_ownership_history
- POST /units/{unit_id}/owners/transfer → transfer_ownership
- GET /units/{unit_id}/owners/total-percentage → get_total_ownership_percentage

### Units (7 endpoints)
- POST /units → create_unit
- GET /units/{id} → get_unit
- GET /units → list_units
- GET /buildings/{building_id}/units → list_units_by_building
- PUT /units/{id} → update_unit
- DELETE /units/{id} → delete_unit
- PUT /units/{unit_id}/assign-owner/{owner_id} → assign_owner

### User (6 endpoints)
- GET /users → list_users
- POST /users → create_user
- PUT /users/{id} → update_user
- PUT /users/{id}/activate → activate_user
- PUT /users/{id}/deactivate → deactivate_user
- DELETE /users/{id} → delete_user

### WorkReports (11 endpoints)
- POST /work-reports → create_work_report
- GET /work-reports/{id} → get_work_report
- GET /buildings/{building_id}/work-reports → list_building_work_reports
- GET /organizations/{organization_id}/work-reports → list_organization_work_reports
- GET /work-reports → list_work_reports_paginated
- PUT /work-reports/{id} → update_work_report
- DELETE /work-reports/{id} → delete_work_report
- GET /buildings/{building_id}/work-reports/warranties/active → get_active_warranties
- GET /buildings/{building_id}/work-reports/warranties/expiring → get_expiring_warranties
- POST /work-reports/{id}/photos → add_photo
- POST /work-reports/{id}/documents → add_document
