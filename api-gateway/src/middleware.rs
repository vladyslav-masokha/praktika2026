use axum::{
    body::Body,
    extract::State,
    http::{header::AUTHORIZATION, Request, StatusCode, Method},
    middleware::Next,
    response::Response,
};
use common::jwt::verify_jwt;

use crate::app_state::AppState;

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    if req.method() == Method::OPTIONS {
        return Ok(next.run(req).await);
    }

    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = verify_jwt(token, &state.jwt_secret)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    req.headers_mut().insert(
        "x-user-id",
        claims
            .sub
            .to_string()
            .parse()
            .map_err(|_| StatusCode::UNAUTHORIZED)?,
    );

    Ok(next.run(req).await)
}