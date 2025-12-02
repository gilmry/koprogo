# Sample Handler Annotations


## 1. create_booking (POST /resource-bookings)
```rust
#[utoipa::path(
    post,
    path = "/resource-bookings",
    tag = "ResourceBookings",
    summary = "Create Booking",
    request_body = CreateResourceBookingDto,
    responses(
        (status = 201, description = "Resource created successfully"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(
        ("bearer_auth" = []),
    ),
)]
#[post("/resource-bookings")]
pub async fn create_booking(...) -> impl Responder {
    // handler body
}
```


## 2. get_booking (GET /resource-bookings/{id})
```rust
#[utoipa::path(
    get,
    path = "/resource-bookings/{id}",
    tag = "ResourceBookings",
    summary = "Get Booking",
    params(
        ("id" = String, Path, description = "Id"),
    ),
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(
        ("bearer_auth" = []),
    ),
)]
#[get("/resource-bookings/{id}")]
pub async fn get_booking(...) -> impl Responder {
    // handler body
}
```


## 3. list_building_bookings (GET /buildings/{building_id}/resource-bookings)
```rust
#[utoipa::path(
    get,
    path = "/buildings/{building_id}/resource-bookings",
    tag = "ResourceBookings",
    summary = "List Building Bookings",
    params(
        ("building_id" = String, Path, description = "Building Id"),
    ),
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(
        ("bearer_auth" = []),
    ),
)]
#[get("/buildings/{building_id}/resource-bookings")]
pub async fn list_building_bookings(...) -> impl Responder {
    // handler body
}
```


## 4. list_by_resource_type (GET /buildings/{building_id}/resource-bookings/type/{resource_type})
```rust
#[utoipa::path(
    get,
    path = "/buildings/{building_id}/resource-bookings/type/{resource_type}",
    tag = "ResourceBookings",
    summary = "List By Resource Type",
    params(
        ("building_id" = String, Path, description = "Building Id"),
        ("resource_type" = String, Path, description = "Resource Type"),
    ),
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(
        ("bearer_auth" = []),
    ),
)]
#[get("/buildings/{building_id}/resource-bookings/type/{resource_type}")]
pub async fn list_by_resource_type(...) -> impl Responder {
    // handler body
}
```


## 5. list_by_resource (GET /buildings/{building_id}/resource-bookings/resource/{resource_type}/{resource_name})
```rust
#[utoipa::path(
    get,
    path = "/buildings/{building_id}/resource-bookings/resource/{resource_type}/{resource_name}",
    tag = "ResourceBookings",
    summary = "List By Resource",
    params(
        ("building_id" = String, Path, description = "Building Id"),
        ("resource_type" = String, Path, description = "Resource Type"),
        ("resource_name" = String, Path, description = "Resource Name"),
    ),
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(
        ("bearer_auth" = []),
    ),
)]
#[get("/buildings/{building_id}/resource-bookings/resource/{resource_type}/{resource_name}")]
pub async fn list_by_resource(...) -> impl Responder {
    // handler body
}
```


## 6. list_my_bookings (GET /resource-bookings/my)
```rust
#[utoipa::path(
    get,
    path = "/resource-bookings/my",
    tag = "ResourceBookings",
    summary = "List My Bookings",
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(
        ("bearer_auth" = []),
    ),
)]
#[get("/resource-bookings/my")]
pub async fn list_my_bookings(...) -> impl Responder {
    // handler body
}
```


## 7. list_my_bookings_by_status (GET /resource-bookings/my/status/{status})
```rust
#[utoipa::path(
    get,
    path = "/resource-bookings/my/status/{status}",
    tag = "ResourceBookings",
    summary = "List My Bookings By Status",
    params(
        ("status" = String, Path, description = "Status"),
    ),
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(
        ("bearer_auth" = []),
    ),
)]
#[get("/resource-bookings/my/status/{status}")]
pub async fn list_my_bookings_by_status(...) -> impl Responder {
    // handler body
}
```


## 8. list_building_bookings_by_status (GET /buildings/{building_id}/resource-bookings/status/{status})
```rust
#[utoipa::path(
    get,
    path = "/buildings/{building_id}/resource-bookings/status/{status}",
    tag = "ResourceBookings",
    summary = "List Building Bookings By Status",
    params(
        ("building_id" = String, Path, description = "Building Id"),
        ("status" = String, Path, description = "Status"),
    ),
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(
        ("bearer_auth" = []),
    ),
)]
#[get("/buildings/{building_id}/resource-bookings/status/{status}")]
pub async fn list_building_bookings_by_status(...) -> impl Responder {
    // handler body
}
```


## 9. list_upcoming_bookings (GET /buildings/{building_id}/resource-bookings/upcoming)
```rust
#[utoipa::path(
    get,
    path = "/buildings/{building_id}/resource-bookings/upcoming",
    tag = "ResourceBookings",
    summary = "List Upcoming Bookings",
    params(
        ("building_id" = String, Path, description = "Building Id"),
    ),
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(
        ("bearer_auth" = []),
    ),
)]
#[get("/buildings/{building_id}/resource-bookings/upcoming")]
pub async fn list_upcoming_bookings(...) -> impl Responder {
    // handler body
}
```


## 10. list_active_bookings (GET /buildings/{building_id}/resource-bookings/active)
```rust
#[utoipa::path(
    get,
    path = "/buildings/{building_id}/resource-bookings/active",
    tag = "ResourceBookings",
    summary = "List Active Bookings",
    params(
        ("building_id" = String, Path, description = "Building Id"),
    ),
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(
        ("bearer_auth" = []),
    ),
)]
#[get("/buildings/{building_id}/resource-bookings/active")]
pub async fn list_active_bookings(...) -> impl Responder {
    // handler body
}
```
