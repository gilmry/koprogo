mod common;

use actix_web::http::header;
use actix_web::{test, App};
use koprogo_api::infrastructure::web::configure_routes;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

#[actix_web::test]
#[serial]
async fn protected_route_requires_jwt() {
    let (app_state, _container, _org_id) = common::setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    // Without Authorization → 401
    let req = test::TestRequest::get()
        .uri("/api/v1/buildings")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
#[serial]
async fn protected_route_with_valid_jwt_succeeds() {
    let (app_state, _container, org_id) = common::setup_test_db().await;

    let token = common::register_and_login(&app_state, org_id).await;

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;
    let req = test::TestRequest::get()
        .uri("/api/v1/buildings")
        .insert_header((
            actix_web::http::header::AUTHORIZATION,
            format!("Bearer {}", token),
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
#[serial]
async fn post_building_injects_org_from_jwt() {
    use actix_web::http::header;
    use serde::Deserialize;

    let (app_state, _container, org_id) = common::setup_test_db().await;
    let pool = app_state.pool.clone();

    let token = common::register_and_login(&app_state, org_id).await;

    // SuperAdmin can specify organization_id in request body
    // Test that SuperAdmin can create building with valid organization_id
    #[derive(Deserialize)]
    struct BuildingResp {
        id: String,
    }

    // Use the valid org_id that was created in setup
    let payload = serde_json::json!({
        "organization_id": org_id.to_string(),
        "name": "JWT Building",
        "address": "1 JWT St",
        "city": "Brussels",
        "postal_code": "1000",
        "country": "Belgium",
        "total_units": 5,
        "construction_year": 2000
    });

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;
    let req = test::TestRequest::post()
        .uri("/api/v1/buildings")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .set_json(&payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let body: BuildingResp = test::read_body_json(resp).await;
    let building_id = Uuid::parse_str(&body.id).expect("uuid");

    // Verify in DB that organization_id is org_id (as specified by SuperAdmin)
    let fetched_org_id: Uuid =
        sqlx::query_scalar("SELECT organization_id FROM buildings WHERE id = $1")
            .bind(building_id)
            .fetch_one(&pool)
            .await
            .expect("select org id");
    assert_eq!(
        fetched_org_id, org_id,
        "SuperAdmin should be able to create building with specified organization_id"
    );
}

#[actix_web::test]
#[serial]
async fn login_returns_all_roles_and_switch_active_role_updates_claims() {
    use koprogo_api::application::dto::{LoginRequest, RegisterRequest};

    let (app_state, _container, _org_id) = common::setup_test_db().await;
    let pool = app_state.pool.clone();

    // Create two organizations for multi-role scenario
    let org_a = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO organizations (id, name, slug, contact_email, subscription_plan, max_buildings, max_users, is_active, created_at, updated_at)
           VALUES ($1, 'Org Multi A', 'org-multi-a', 'a@multi.org', 'starter', 5, 10, true, NOW(), NOW())"#,
    )
    .bind(org_a)
    .execute(&pool)
    .await
    .expect("insert org A");

    let org_b = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO organizations (id, name, slug, contact_email, subscription_plan, max_buildings, max_users, is_active, created_at, updated_at)
           VALUES ($1, 'Org Multi B', 'org-multi-b', 'b@multi.org', 'starter', 5, 10, true, NOW(), NOW())"#,
    )
    .bind(org_b)
    .execute(&pool)
    .await
    .expect("insert org B");

    // Register the user (first role automatically created)
    let email = format!("multi+{}@test.com", Uuid::new_v4());
    let register_response = app_state
        .auth_use_cases
        .register(RegisterRequest {
            email: email.clone(),
            password: "Passw0rd!".to_string(),
            first_name: "Multi".to_string(),
            last_name: "Role".to_string(),
            role: "syndic".to_string(),
            organization_id: Some(org_a),
        })
        .await
        .expect("register multi-role user");

    let user_id = register_response.user.id;
    let primary_role_id = register_response
        .user
        .active_role
        .as_ref()
        .map(|role| role.id)
        .expect("active role exists");

    // Manually attach a second role for the same user
    let secondary_role_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO user_roles (id, user_id, role, organization_id, is_primary, created_at, updated_at)
           VALUES ($1, $2, $3, $4, false, NOW(), NOW())"#,
    )
    .bind(secondary_role_id)
    .bind(user_id)
    .bind("accountant")
    .bind(org_b)
    .execute(&pool)
    .await
    .expect("insert secondary role");

    // Login should expose all role assignments
    let login_response = app_state
        .auth_use_cases
        .login(LoginRequest {
            email: email.clone(),
            password: "Passw0rd!".to_string(),
        })
        .await
        .expect("login with multi roles");

    assert_eq!(
        login_response.user.roles.len(),
        2,
        "Login response should list both role assignments"
    );
    assert!(
        login_response
            .user
            .roles
            .iter()
            .any(|role| role.role == "syndic" && role.organization_id == Some(org_a)),
        "Roles should include the primary syndic role"
    );
    assert!(
        login_response
            .user
            .roles
            .iter()
            .any(|role| role.role == "accountant" && role.organization_id == Some(org_b)),
        "Roles should include the secondary accountant role"
    );
    assert_eq!(
        login_response.user.active_role.as_ref().map(|role| role.id),
        Some(primary_role_id),
        "Active role defaults to the primary assignment"
    );

    // Switch active role to the accountant assignment
    let switch_response = app_state
        .auth_use_cases
        .switch_active_role(user_id, secondary_role_id)
        .await
        .expect("switch active role");

    let active_role = switch_response
        .user
        .active_role
        .expect("active role should be set after switch");
    assert_eq!(active_role.id, secondary_role_id);
    assert_eq!(active_role.role, "accountant");
    assert_eq!(active_role.organization_id, Some(org_b));

    // Token claims must reflect the switched role and carry role_id
    let claims = app_state
        .auth_use_cases
        .verify_token(&switch_response.token)
        .expect("verify switched token");
    assert_eq!(claims.role, "accountant");
    assert_eq!(claims.organization_id, Some(org_b));
    assert_eq!(
        claims.role_id.expect("claims include role_id"),
        secondary_role_id
    );
}

#[actix_web::test]
#[serial]
async fn admin_can_manage_user_roles_via_http() {
    use koprogo_api::application::dto::{LoginRequest, RegisterRequest};

    let (app_state, _container, _org_id) = common::setup_test_db().await;
    let pool = app_state.pool.clone();

    let org_id = Uuid::new_v4();
    let org_slug = format!("org-multi-{}", org_id);
    sqlx::query(
        r#"
        INSERT INTO organizations (
            id, name, slug, contact_email, contact_phone,
            subscription_plan, max_buildings, max_users, is_active, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
        "#,
    )
    .bind(org_id)
    .bind("Org Multi Admin")
    .bind(&org_slug)
    .bind("multi-admin@org.com")
    .bind(Option::<String>::None)
    .bind("starter")
    .bind(50)
    .bind(50)
    .bind(true)
    .execute(&pool)
    .await
    .expect("insert organization");

    let admin_email = format!("admin+{}@test.com", Uuid::new_v4());
    let admin_password = "Passw0rd!";
    app_state
        .auth_use_cases
        .register(RegisterRequest {
            email: admin_email.clone(),
            password: admin_password.to_string(),
            first_name: "Super".to_string(),
            last_name: "Admin".to_string(),
            role: "superadmin".to_string(),
            organization_id: None,
        })
        .await
        .expect("register superadmin");

    let admin_login = app_state
        .auth_use_cases
        .login(LoginRequest {
            email: admin_email.clone(),
            password: admin_password.to_string(),
        })
        .await
        .expect("login superadmin");

    let admin_token = admin_login.token.clone();

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .configure(configure_routes),
    )
    .await;

    let staff_email = format!("staff+{}@test.com", Uuid::new_v4());
    let create_payload = json!({
        "email": staff_email,
        "password": "Passw0rd!",
        "first_name": "Multi",
        "last_name": "Role",
        "roles": [
            { "role": "syndic", "organization_id": org_id.to_string(), "is_primary": true },
            { "role": "accountant", "organization_id": org_id.to_string() }
        ]
    });

    let create_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/users")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", admin_token)))
            .set_json(&create_payload)
            .to_request(),
    )
    .await;
    assert_eq!(create_resp.status(), 201, "create user response status");

    let created_user: serde_json::Value = test::read_body_json(create_resp).await;
    let created_user_id =
        Uuid::parse_str(created_user["id"].as_str().expect("user id")).expect("uuid");

    let roles_json = created_user["roles"]
        .as_array()
        .expect("roles array after creation");
    assert_eq!(roles_json.len(), 2);
    assert_eq!(
        created_user["active_role"]["role"]
            .as_str()
            .expect("active role"),
        "syndic"
    );

    let update_payload = json!({
        "email": staff_email,
        "first_name": "Multi",
        "last_name": "Role",
        "roles": [
            { "role": "accountant", "organization_id": org_id.to_string(), "is_primary": true },
            { "role": "syndic", "organization_id": org_id.to_string(), "is_primary": false }
        ]
    });

    let update_resp = test::call_service(
        &app,
        test::TestRequest::put()
            .uri(&format!("/api/v1/users/{}", created_user_id))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", admin_token)))
            .set_json(&update_payload)
            .to_request(),
    )
    .await;
    assert_eq!(update_resp.status(), 200, "update user response status");

    let updated_user: serde_json::Value = test::read_body_json(update_resp).await;
    assert_eq!(
        updated_user["active_role"]["role"]
            .as_str()
            .expect("updated active role"),
        "accountant"
    );
    assert_eq!(
        updated_user["roles"]
            .as_array()
            .expect("roles array on update")
            .len(),
        2
    );

    let list_resp = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/users?per_page=100")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", admin_token)))
            .to_request(),
    )
    .await;
    assert_eq!(list_resp.status(), 200, "list users response status");

    let list_body: serde_json::Value = test::read_body_json(list_resp).await;
    let users_array = list_body["data"].as_array().expect("users array");
    let created_entry = users_array
        .iter()
        .find(|u| u["id"].as_str() == Some(&created_user_id.to_string()))
        .expect("created user in list");
    assert_eq!(
        created_entry["roles"]
            .as_array()
            .expect("roles array in list")
            .len(),
        2
    );
    assert_eq!(
        created_entry["active_role"]["role"]
            .as_str()
            .expect("active role in list"),
        "accountant"
    );
}
