use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
    Json,
};
use bcrypt::{hash, verify as bcrypt_verify, DEFAULT_COST};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use common::errors::AppError;
use common::jwt::generate_jwt;

use crate::{app_state::AppState, proxy};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMeRequest {
    pub full_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RecordVisitRequest {
    pub page_key: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub slug: Option<String>,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub image_url: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub slug: Option<String>,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub image_url: Option<String>,
    pub category: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: i64,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub id: i64,
    pub email: String,
    pub role: String,
    #[serde(rename = "fullName")]
    pub full_name: Option<String>,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
    pub auth_provider: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ProductDto {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub image_url: Option<String>,
    pub category: Option<String>,
    pub is_active: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AdminUserDto {
    pub id: i64,
    pub email: String,
    pub role: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub auth_provider: String,
    pub created_at: Option<String>,
    pub last_seen_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AdminStatsResponse {
    #[serde(rename = "totalUsers")]
    pub total_users: i64,
    #[serde(rename = "totalProducts")]
    pub total_products: i64,
    #[serde(rename = "totalOrders")]
    pub total_orders: i64,
    #[serde(rename = "totalVisits")]
    pub total_visits: i64,
    #[serde(rename = "onlineUsers")]
    pub online_users: i64,
}

#[derive(Debug, Deserialize)]
struct OrderSummaryResponse {
    total_orders: i64,
}

#[derive(sqlx::FromRow, Debug)]
struct UserRecord {
    id: i64,
    email: String,
    password_hash: String,
    role: String,
    full_name: Option<String>,
    avatar_url: Option<String>,
    auth_provider: String,
}

#[derive(Debug, Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct GoogleUserInfo {
    email: String,
    name: Option<String>,
    picture: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JwtClaims {
    sub: i64,
    exp: usize,
}

fn build_frontend_redirect(token: &str) -> String {
    format!("http://localhost:5173/login?token={}", token)
}

fn current_user_id_from_headers(headers: &HeaderMap) -> Result<i64, AppError> {
    headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i64>().ok())
        .ok_or(AppError::Unauthorized)
}

fn current_auth_header(headers: &HeaderMap) -> Result<&str, AppError> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)
}

fn sanitize_optional_text(value: &Option<String>) -> Option<String> {
    value
        .as_ref()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn slugify(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut last_dash = false;

    for ch in input.trim().to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }

    out.trim_matches('-').to_string()
}

async fn ensure_admin(state: &AppState, user_id: i64) -> Result<(), AppError> {
    let role = sqlx::query_scalar::<_, String>("SELECT role FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::Unauthorized)?;

    if role.to_lowercase() != "admin" {
        return Err(AppError::Unauthorized);
    }

    Ok(())
}

pub async fn google_login_handler(State(state): State<AppState>) -> impl IntoResponse {
    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?response_type=code&client_id={}&redirect_uri={}&scope=openid%20email%20profile&access_type=offline&prompt=consent",
        state.google_client_id,
        urlencoding::encode(&state.google_redirect_url)
    );

    Redirect::to(&auth_url)
}

pub async fn google_callback_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, AppError> {
    let code = params.get("code").ok_or(AppError::Unauthorized)?;

    let params_to_send = [
        ("code", code.as_str()),
        ("client_id", state.google_client_id.as_str()),
        ("client_secret", state.google_client_secret.as_str()),
        ("redirect_uri", state.google_redirect_url.as_str()),
        ("grant_type", "authorization_code"),
    ];

    let token_res = state
        .client
        .post("https://oauth2.googleapis.com/token")
        .form(&params_to_send)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Google token exchange failed: {}", e)))?;

    if !token_res.status().is_success() {
        let body = token_res
            .text()
            .await
            .unwrap_or_else(|_| "unknown google token error".to_string());

        return Err(AppError::Internal(format!(
            "Google token exchange failed: {}",
            body
        )));
    }

    let token_data = token_res
        .json::<GoogleTokenResponse>()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse Google token: {}", e)))?;

    let user_info_res = state
        .client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(token_data.access_token)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get user info: {}", e)))?;

    if !user_info_res.status().is_success() {
        let body = user_info_res
            .text()
            .await
            .unwrap_or_else(|_| "unknown google user info error".to_string());

        return Err(AppError::Internal(format!(
            "Failed to get Google user info: {}",
            body
        )));
    }

    let user_info = user_info_res
        .json::<GoogleUserInfo>()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse user info: {}", e)))?;

    let existing_user = sqlx::query_as::<_, UserRecord>(
        r#"
        SELECT id, email, password_hash, role, full_name, avatar_url, auth_provider
        FROM users
        WHERE email = $1
        "#,
    )
    .bind(&user_info.email)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let user = if let Some(existing) = existing_user {
        sqlx::query_as::<_, UserRecord>(
            r#"
            UPDATE users
            SET
                full_name = COALESCE($2, full_name),
                avatar_url = COALESCE($3, avatar_url),
                auth_provider = 'google',
                last_seen_at = NOW()
            WHERE email = $1
            RETURNING id, email, password_hash, role, full_name, avatar_url, auth_provider
            "#,
        )
        .bind(&existing.email)
        .bind(&user_info.name)
        .bind(&user_info.picture)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    } else {
        sqlx::query_as::<_, UserRecord>(
            r#"
            INSERT INTO users (
                email,
                password_hash,
                role,
                full_name,
                avatar_url,
                auth_provider
            )
            VALUES ($1, 'google_authenticated', 'user', $2, $3, 'google')
            RETURNING id, email, password_hash, role, full_name, avatar_url, auth_provider
            "#,
        )
        .bind(&user_info.email)
        .bind(&user_info.name)
        .bind(&user_info.picture)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    };

    let token =
        generate_jwt(user.id, &state.jwt_secret).map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Redirect::to(&build_frontend_redirect(&token)))
}

pub async fn me_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<MeResponse>, AppError> {
    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?;

    let token_data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized)?;

    let user = sqlx::query_as::<_, UserRecord>(
        r#"
        SELECT id, email, password_hash, role, full_name, avatar_url, auth_provider
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(token_data.claims.sub)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .ok_or(AppError::Unauthorized)?;

    sqlx::query(
        r#"
        UPDATE users
        SET last_seen_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(user.id)
    .execute(&state.db_pool)
    .await
    .ok();

    Ok(Json(MeResponse {
        id: user.id,
        email: user.email,
        role: user.role,
        full_name: user.full_name,
        avatar_url: user.avatar_url,
        auth_provider: user.auth_provider,
    }))
}

pub async fn update_me_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateMeRequest>,
) -> Result<Json<MeResponse>, AppError> {
    let user_id = current_user_id_from_headers(&headers)?;
    let full_name = payload.full_name.trim();

    if full_name.is_empty() {
        return Err(AppError::BadRequest("Full name is required".into()));
    }

    let avatar_url = sanitize_optional_text(&payload.avatar_url);

    let user = sqlx::query_as::<_, UserRecord>(
        r#"
        UPDATE users
        SET
            full_name = $2,
            avatar_url = $3,
            last_seen_at = NOW()
        WHERE id = $1
        RETURNING id, email, password_hash, role, full_name, avatar_url, auth_provider
        "#,
    )
    .bind(user_id)
    .bind(full_name)
    .bind(avatar_url)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(MeResponse {
        id: user.id,
        email: user.email,
        role: user.role,
        full_name: user.full_name,
        avatar_url: user.avatar_url,
        auth_provider: user.auth_provider,
    }))
}

pub async fn record_visit_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<RecordVisitRequest>,
) -> Result<StatusCode, AppError> {
    let page_key = payload.page_key.trim();

    if page_key.is_empty() {
        return Err(AppError::BadRequest("page_key is required".into()));
    }

    let user_id = headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i64>().ok());

    sqlx::query(
        r#"
        INSERT INTO page_visits (page_key, user_id, session_id)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(page_key)
    .bind(user_id)
    .bind(sanitize_optional_text(&payload.session_id))
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if let Some(user_id) = user_id {
        sqlx::query("UPDATE users SET last_seen_at = NOW() WHERE id = $1")
            .bind(user_id)
            .execute(&state.db_pool)
            .await
            .ok();
    }

    Ok(StatusCode::CREATED)
}

pub async fn list_products_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<ProductDto>>, AppError> {
    let products = sqlx::query_as::<_, ProductDto>(
        r#"
        SELECT
            id,
            slug,
            name,
            description,
            price,
            image_url,
            category,
            is_active,
            TO_CHAR(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at,
            TO_CHAR(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS updated_at
        FROM products
        WHERE is_active = TRUE
        ORDER BY id DESC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(products))
}

pub async fn get_product_by_slug_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<ProductDto>, AppError> {
    let product = sqlx::query_as::<_, ProductDto>(
        r#"
        SELECT
            id,
            slug,
            name,
            description,
            price,
            image_url,
            category,
            is_active,
            TO_CHAR(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at,
            TO_CHAR(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS updated_at
        FROM products
        WHERE slug = $1 AND is_active = TRUE
        "#,
    )
    .bind(slug)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("Product not found".into()))?;

    Ok(Json(product))
}

pub async fn admin_list_products_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ProductDto>>, AppError> {
    let user_id = current_user_id_from_headers(&headers)?;
    ensure_admin(&state, user_id).await?;

    let products = sqlx::query_as::<_, ProductDto>(
        r#"
        SELECT
            id,
            slug,
            name,
            description,
            price,
            image_url,
            category,
            is_active,
            TO_CHAR(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at,
            TO_CHAR(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS updated_at
        FROM products
        ORDER BY id DESC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(products))
}

pub async fn create_product_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateProductRequest>,
) -> Result<Json<ProductDto>, AppError> {
    let user_id = current_user_id_from_headers(&headers)?;
    ensure_admin(&state, user_id).await?;

    let name = payload.name.trim();
    let description = payload.description.trim();

    if name.is_empty() || description.is_empty() {
        return Err(AppError::BadRequest("Name and description are required".into()));
    }

    if payload.price <= 0.0 {
        return Err(AppError::BadRequest("Price must be greater than 0".into()));
    }

    let slug = sanitize_optional_text(&payload.slug)
        .unwrap_or_else(|| slugify(name));

    if slug.is_empty() {
        return Err(AppError::BadRequest("Slug is required".into()));
    }

    let product = sqlx::query_as::<_, ProductDto>(
        r#"
        INSERT INTO products (slug, name, description, price, image_url, category, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, TRUE)
        RETURNING
            id,
            slug,
            name,
            description,
            price,
            image_url,
            category,
            is_active,
            TO_CHAR(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at,
            TO_CHAR(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS updated_at
        "#,
    )
    .bind(slug)
    .bind(name)
    .bind(description)
    .bind(payload.price)
    .bind(sanitize_optional_text(&payload.image_url))
    .bind(sanitize_optional_text(&payload.category))
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(product))
}

pub async fn update_product_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(product_id): Path<i64>,
    Json(payload): Json<UpdateProductRequest>,
) -> Result<Json<ProductDto>, AppError> {
    let user_id = current_user_id_from_headers(&headers)?;
    ensure_admin(&state, user_id).await?;

    let name = payload.name.trim();
    let description = payload.description.trim();

    if name.is_empty() || description.is_empty() {
        return Err(AppError::BadRequest("Name and description are required".into()));
    }

    if payload.price <= 0.0 {
        return Err(AppError::BadRequest("Price must be greater than 0".into()));
    }

    let slug = sanitize_optional_text(&payload.slug)
        .unwrap_or_else(|| slugify(name));

    let product = sqlx::query_as::<_, ProductDto>(
        r#"
        UPDATE products
        SET
            slug = $2,
            name = $3,
            description = $4,
            price = $5,
            image_url = $6,
            category = $7,
            is_active = $8,
            updated_at = NOW()
        WHERE id = $1
        RETURNING
            id,
            slug,
            name,
            description,
            price,
            image_url,
            category,
            is_active,
            TO_CHAR(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at,
            TO_CHAR(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS updated_at
        "#,
    )
    .bind(product_id)
    .bind(slug)
    .bind(name)
    .bind(description)
    .bind(payload.price)
    .bind(sanitize_optional_text(&payload.image_url))
    .bind(sanitize_optional_text(&payload.category))
    .bind(payload.is_active)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("Product not found".into()))?;

    Ok(Json(product))
}

pub async fn delete_product_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(product_id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let user_id = current_user_id_from_headers(&headers)?;
    ensure_admin(&state, user_id).await?;

    let rows = sqlx::query(
        r#"
        UPDATE products
        SET is_active = FALSE, updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(product_id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound("Product not found".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_users_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<AdminUserDto>>, AppError> {
    let user_id = current_user_id_from_headers(&headers)?;
    ensure_admin(&state, user_id).await?;

    let users = sqlx::query_as::<_, AdminUserDto>(
        r#"
        SELECT
            id,
            email,
            role,
            full_name,
            avatar_url,
            auth_provider,
            TO_CHAR(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at,
            TO_CHAR(last_seen_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS last_seen_at
        FROM users
        ORDER BY id DESC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(users))
}

pub async fn admin_stats_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AdminStatsResponse>, AppError> {
    let user_id = current_user_id_from_headers(&headers)?;
    ensure_admin(&state, user_id).await?;

    let total_users = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let total_products = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM products")
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let total_visits = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM page_visits")
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let online_users = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM users
        WHERE last_seen_at >= NOW() - INTERVAL '5 minutes'
        "#,
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let auth_header = current_auth_header(&headers)?;

    let total_orders = match state
        .client
        .get(format!("{}/orders/admin/summary", state.order_service_url))
        .header("authorization", auth_header)
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => response
            .json::<OrderSummaryResponse>()
            .await
            .map(|data| data.total_orders)
            .unwrap_or(0),
        _ => 0,
    };

    Ok(Json(AdminStatsResponse {
        total_users,
        total_products,
        total_orders,
        total_visits,
        online_users,
    }))
}

pub async fn create_order_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, AppError> {
    let auth_header = current_auth_header(&headers)?;
    let (status, body) = proxy::forward_post_orders(&state, auth_header, body).await?;
    Ok((status, body))
}

pub async fn list_orders_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let auth_header = current_auth_header(&headers)?;
    let (status, body) = proxy::forward_get_orders(&state, auth_header).await?;
    Ok((status, body))
}

pub async fn get_order_by_id_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(order_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let auth_header = current_auth_header(&headers)?;
    let (status, body) = proxy::forward_get_order_by_id(&state, auth_header, order_id).await?;
    Ok((status, body))
}

pub async fn register_handler(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let hashed_password = hash(&payload.password, DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("Hash error: {}", e)))?;

    let user = sqlx::query_as::<_, UserRecord>(
        r#"
        INSERT INTO users (email, password_hash, role, auth_provider)
        VALUES ($1, $2, 'user', 'email')
        RETURNING id, email, password_hash, role, full_name, avatar_url, auth_provider
        "#,
    )
    .bind(&payload.email)
    .bind(&hashed_password)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let token =
        generate_jwt(user.id, &state.jwt_secret).map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(LoginResponse {
        token,
        user_id: user.id,
        email: user.email,
        role: user.role,
    }))
}

pub async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let user = sqlx::query_as::<_, UserRecord>(
        r#"
        SELECT id, email, password_hash, role, full_name, avatar_url, auth_provider
        FROM users
        WHERE email = $1
        "#,
    )
    .bind(&payload.email)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .ok_or(AppError::Unauthorized)?;

    if user.password_hash == "google_authenticated" {
        return Err(AppError::Unauthorized);
    }

    let is_valid = bcrypt_verify(&payload.password, &user.password_hash).unwrap_or(false);

    if !is_valid {
        return Err(AppError::Unauthorized);
    }

    sqlx::query(
        r#"
        UPDATE users
        SET last_seen_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(user.id)
    .execute(&state.db_pool)
    .await
    .ok();

    let token =
        generate_jwt(user.id, &state.jwt_secret).map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(LoginResponse {
        token,
        user_id: user.id,
        email: user.email,
        role: user.role,
    }))
}

pub async fn health_handler() -> (StatusCode, &'static str) {
    (StatusCode::OK, "api-gateway-ok")
}

