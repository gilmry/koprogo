// E2E tests for notification system HTTP endpoints (Issue #86)
// Tests focus on HTTP layer: endpoints, auth, JSON serialization
// Covers multi-channel notification system (Email, InApp, Push) for Belgian copropriete

mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::application::dto::*;
use koprogo_api::infrastructure::web::configure_routes;
use koprogo_api::infrastructure::web::AppState;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

/// Helper: Register a user and return (token, user_id)
async fn register_and_login_with_user_id(
    app_state: &actix_web::web::Data<AppState>,
    org_id: Uuid,
) -> (String, Uuid) {
    let email = format!("notification-test-{}@example.com", Uuid::new_v4());
    let reg = RegisterRequest {
        email: email.clone(),
        password: "SecurePass123!".to_string(),
        first_name: "Notification".to_string(),
        last_name: "Tester".to_string(),
        role: "superadmin".to_string(),
        organization_id: Some(org_id),
    };
    let login_resp = app_state
        .auth_use_cases
        .register(reg)
        .await
        .expect("Failed to register user");

    let user_id = login_resp.user.id;
    let token = login_resp.token;

    (token, user_id)
}

// ==================== Notification CRUD Tests ====================

#[actix_web::test]
#[serial]
async fn test_create_notification_success() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, user_id) = register_and_login_with_user_id(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/notifications")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "user_id": user_id.to_string(),
            "notification_type": "ExpenseCreated",
            "channel": "Email",
            "priority": "Medium",
            "title": "New expense created",
            "message": "A new maintenance expense has been added to your account."
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        201,
        "Should create notification successfully"
    );

    let notification: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(notification["title"], "New expense created");
    assert_eq!(notification["notification_type"], "ExpenseCreated");
    assert_eq!(notification["channel"], "Email");
    assert_eq!(notification["priority"], "Medium");
    assert_eq!(notification["status"], "Pending");
}

#[actix_web::test]
#[serial]
async fn test_create_notification_without_auth_fails() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (_token, user_id) = register_and_login_with_user_id(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/notifications")
        .set_json(json!({
            "user_id": user_id.to_string(),
            "notification_type": "System",
            "channel": "InApp",
            "priority": "Low",
            "title": "Unauthorized",
            "message": "This should fail"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should require authentication");
}

#[actix_web::test]
#[serial]
async fn test_create_notification_all_types() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, user_id) = register_and_login_with_user_id(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let notification_types = vec![
        "ExpenseCreated",
        "MeetingConvocation",
        "PaymentReceived",
        "TicketResolved",
        "DocumentAdded",
        "BoardMessage",
        "PaymentReminder",
        "BudgetApproved",
        "ResolutionVote",
        "System",
    ];

    for notif_type in notification_types {
        let req = test::TestRequest::post()
            .uri("/api/v1/notifications")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "user_id": user_id.to_string(),
                "notification_type": notif_type,
                "channel": "InApp",
                "priority": "Low",
                "title": format!("Test {} notification", notif_type),
                "message": format!("Testing {} type", notif_type)
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            201,
            "Should create notification for type {}",
            notif_type
        );
    }
}

#[actix_web::test]
#[serial]
async fn test_create_notification_all_channels() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, user_id) = register_and_login_with_user_id(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let channels = vec!["Email", "InApp", "Push"];

    for channel in channels {
        let req = test::TestRequest::post()
            .uri("/api/v1/notifications")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "user_id": user_id.to_string(),
                "notification_type": "System",
                "channel": channel,
                "priority": "Low",
                "title": format!("Test {} channel", channel),
                "message": format!("Testing {} channel", channel)
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            201,
            "Should create notification for channel {}",
            channel
        );
    }
}

#[actix_web::test]
#[serial]
async fn test_create_notification_all_priorities() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, user_id) = register_and_login_with_user_id(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let priorities = vec!["Low", "Medium", "High", "Critical"];

    for priority in priorities {
        let req = test::TestRequest::post()
            .uri("/api/v1/notifications")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "user_id": user_id.to_string(),
                "notification_type": "System",
                "channel": "InApp",
                "priority": priority,
                "title": format!("Test {} priority", priority),
                "message": format!("Testing {} priority", priority)
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            201,
            "Should create notification with priority {}",
            priority
        );
    }
}

#[actix_web::test]
#[serial]
async fn test_get_notification_by_id() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, user_id) = register_and_login_with_user_id(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create notification
    let create_req = test::TestRequest::post()
        .uri("/api/v1/notifications")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "user_id": user_id.to_string(),
            "notification_type": "PaymentReceived",
            "channel": "Email",
            "priority": "High",
            "title": "Payment received",
            "message": "Your payment of 500 EUR has been received."
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let notification: serde_json::Value = test::read_body_json(create_resp).await;
    let notification_id = notification["id"].as_str().unwrap();

    // Get notification
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/notifications/{}", notification_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let fetched: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(fetched["id"], notification_id);
    assert_eq!(fetched["title"], "Payment received");
}

#[actix_web::test]
#[serial]
async fn test_get_notification_not_found() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = register_and_login_with_user_id(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let fake_id = Uuid::new_v4();
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/notifications/{}", fake_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
#[serial]
async fn test_list_my_notifications() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, user_id) = register_and_login_with_user_id(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create 3 notifications for the user
    for i in 1..=3 {
        let req = test::TestRequest::post()
            .uri("/api/v1/notifications")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "user_id": user_id.to_string(),
                "notification_type": "System",
                "channel": "InApp",
                "priority": "Low",
                "title": format!("Notification #{}", i),
                "message": format!("Message {}", i)
            }))
            .to_request();

        test::call_service(&app, req).await;
    }

    // List all notifications for the user
    let req = test::TestRequest::get()
        .uri("/api/v1/notifications/my")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let notifications: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(notifications.as_array().unwrap().len(), 3);
}

#[actix_web::test]
#[serial]
async fn test_list_unread_notifications() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, user_id) = register_and_login_with_user_id(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create unread notification
    let req = test::TestRequest::post()
        .uri("/api/v1/notifications")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "user_id": user_id.to_string(),
            "notification_type": "MeetingConvocation",
            "channel": "InApp",
            "priority": "High",
            "title": "AG Convocation",
            "message": "You are invited to the general assembly on Dec 15"
        }))
        .to_request();

    test::call_service(&app, req).await;

    // List unread notifications
    let list_req = test::TestRequest::get()
        .uri("/api/v1/notifications/unread")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, list_req).await;
    assert_eq!(resp.status(), 200);

    let notifications: serde_json::Value = test::read_body_json(resp).await;
    assert!(!notifications.as_array().unwrap().is_empty());
}

#[actix_web::test]
#[serial]
async fn test_mark_notification_read() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, user_id) = register_and_login_with_user_id(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create notification
    let create_req = test::TestRequest::post()
        .uri("/api/v1/notifications")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "user_id": user_id.to_string(),
            "notification_type": "DocumentAdded",
            "channel": "InApp",
            "priority": "Low",
            "title": "New document",
            "message": "A new document has been uploaded"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let notification: serde_json::Value = test::read_body_json(create_resp).await;
    let notification_id = notification["id"].as_str().unwrap();
    assert!(notification["read_at"].is_null());

    // Mark as read
    let mark_read_req = test::TestRequest::put()
        .uri(&format!("/api/v1/notifications/{}/read", notification_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({}))
        .to_request();

    let mark_read_resp = test::call_service(&app, mark_read_req).await;
    assert_eq!(mark_read_resp.status(), 200);

    let marked: serde_json::Value = test::read_body_json(mark_read_resp).await;
    assert!(
        marked["read_at"].is_string(),
        "Should have read_at timestamp"
    );
}

#[actix_web::test]
#[serial]
async fn test_mark_all_notifications_read() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, user_id) = register_and_login_with_user_id(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create 3 unread notifications
    for i in 1..=3 {
        let req = test::TestRequest::post()
            .uri("/api/v1/notifications")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "user_id": user_id.to_string(),
                "notification_type": "System",
                "channel": "InApp",
                "priority": "Low",
                "title": format!("Unread notification {}", i),
                "message": format!("Message {}", i)
            }))
            .to_request();

        test::call_service(&app, req).await;
    }

    // Mark all as read
    let mark_all_req = test::TestRequest::put()
        .uri("/api/v1/notifications/read-all")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let mark_all_resp = test::call_service(&app, mark_all_req).await;
    assert_eq!(mark_all_resp.status(), 200);

    // Verify all notifications are now read
    let list_unread_req = test::TestRequest::get()
        .uri("/api/v1/notifications/unread")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, list_unread_req).await;
    let unread: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        unread.as_array().unwrap().len(),
        0,
        "Should have no unread notifications"
    );
}

#[actix_web::test]
#[serial]
async fn test_delete_notification() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, user_id) = register_and_login_with_user_id(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create notification
    let create_req = test::TestRequest::post()
        .uri("/api/v1/notifications")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "user_id": user_id.to_string(),
            "notification_type": "System",
            "channel": "InApp",
            "priority": "Low",
            "title": "To delete",
            "message": "This will be deleted"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let notification: serde_json::Value = test::read_body_json(create_resp).await;
    let notification_id = notification["id"].as_str().unwrap();

    // Delete notification
    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/notifications/{}", notification_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let delete_resp = test::call_service(&app, delete_req).await;
    assert_eq!(delete_resp.status(), 204);

    // Verify deletion
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/notifications/{}", notification_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 404);
}

#[actix_web::test]
#[serial]
async fn test_get_notification_stats() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, user_id) = register_and_login_with_user_id(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Create notifications with different statuses
    for i in 1..=3 {
        let req = test::TestRequest::post()
            .uri("/api/v1/notifications")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(json!({
                "user_id": user_id.to_string(),
                "notification_type": "System",
                "channel": "InApp",
                "priority": "Low",
                "title": format!("Stats notification {}", i),
                "message": format!("For statistics test {}", i)
            }))
            .to_request();

        test::call_service(&app, req).await;
    }

    // Get statistics
    let stats_req = test::TestRequest::get()
        .uri("/api/v1/notifications/stats")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let stats_resp = test::call_service(&app, stats_req).await;
    assert_eq!(stats_resp.status(), 200);

    let stats: serde_json::Value = test::read_body_json(stats_resp).await;
    assert!(stats["total"].as_i64().unwrap() >= 3);
}

// ==================== Notification Preference Tests ====================

#[actix_web::test]
#[serial]
async fn test_get_user_preferences() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = register_and_login_with_user_id(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/notification-preferences")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let preferences: serde_json::Value = test::read_body_json(resp).await;
    assert!(preferences.is_array());
}

#[actix_web::test]
#[serial]
async fn test_update_preference() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = register_and_login_with_user_id(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Update preference for ExpenseCreated notifications
    let update_req = test::TestRequest::put()
        .uri("/api/v1/notification-preferences/ExpenseCreated")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "email_enabled": false,
            "in_app_enabled": true,
            "push_enabled": false
        }))
        .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert_eq!(update_resp.status(), 200);

    let preference: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(preference["email_enabled"], false);
    assert_eq!(preference["in_app_enabled"], true);
    assert_eq!(preference["push_enabled"], false);
}

#[actix_web::test]
#[serial]
async fn test_get_specific_preference() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, _user_id) = register_and_login_with_user_id(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // First update the preference
    let update_req = test::TestRequest::put()
        .uri("/api/v1/notification-preferences/PaymentReceived")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "email_enabled": true,
            "in_app_enabled": true,
            "push_enabled": true
        }))
        .to_request();

    test::call_service(&app, update_req).await;

    // Get specific preference
    let get_req = test::TestRequest::get()
        .uri("/api/v1/notification-preferences/PaymentReceived")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 200);

    let preference: serde_json::Value = test::read_body_json(get_resp).await;
    assert_eq!(preference["notification_type"], "PaymentReceived");
    assert_eq!(preference["email_enabled"], true);
}

#[actix_web::test]
#[serial]
async fn test_complete_notification_lifecycle() {
    let (app_state, _container, org_id) = common::setup_test_db().await;
    let (token, user_id) = register_and_login_with_user_id(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // 1. Create notification
    let create_req = test::TestRequest::post()
        .uri("/api/v1/notifications")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "user_id": user_id.to_string(),
            "notification_type": "TicketResolved",
            "channel": "InApp",
            "priority": "High",
            "title": "Ticket #123 resolved",
            "message": "Your plumbing ticket has been resolved by the contractor.",
            "link_url": "/tickets/123",
            "metadata": json!({"ticket_id": "123", "category": "Plumbing"}).to_string()
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let notification: serde_json::Value = test::read_body_json(create_resp).await;
    let notification_id = notification["id"].as_str().unwrap();
    // InApp notifications are auto-marked as Sent on creation
    assert_eq!(notification["status"], "Sent");
    assert!(notification["read_at"].is_null());

    // 2. List unread notifications
    let unread_req = test::TestRequest::get()
        .uri("/api/v1/notifications/unread")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let unread_resp = test::call_service(&app, unread_req).await;
    let unread: serde_json::Value = test::read_body_json(unread_resp).await;
    assert!(!unread.as_array().unwrap().is_empty());

    // 3. Mark notification as read
    let mark_read_req = test::TestRequest::put()
        .uri(&format!("/api/v1/notifications/{}/read", notification_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({}))
        .to_request();

    let mark_read_resp = test::call_service(&app, mark_read_req).await;
    let marked: serde_json::Value = test::read_body_json(mark_read_resp).await;
    assert!(marked["read_at"].is_string());

    // 4. Verify in my notifications list
    let my_notif_req = test::TestRequest::get()
        .uri("/api/v1/notifications/my")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let my_notif_resp = test::call_service(&app, my_notif_req).await;
    let my_notifications: serde_json::Value = test::read_body_json(my_notif_resp).await;
    assert!(!my_notifications.as_array().unwrap().is_empty());

    // 5. Get notification statistics
    let stats_req = test::TestRequest::get()
        .uri("/api/v1/notifications/stats")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let stats_resp = test::call_service(&app, stats_req).await;
    let stats: serde_json::Value = test::read_body_json(stats_resp).await;
    assert!(stats["total"].as_i64().unwrap() >= 1);

    // 6. Update notification preferences for this type
    let pref_req = test::TestRequest::put()
        .uri("/api/v1/notification-preferences/TicketResolved")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(json!({
            "email_enabled": true,
            "in_app_enabled": true,
            "push_enabled": false
        }))
        .to_request();

    let pref_resp = test::call_service(&app, pref_req).await;
    assert_eq!(pref_resp.status(), 200);
}
