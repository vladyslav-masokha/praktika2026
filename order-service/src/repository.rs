use common::errors::AppError;
use sqlx::PgPool;

use crate::models::Order;

#[derive(Debug)]
pub struct OrderStatusUpdateResult {
    pub order: Order,
    pub changed: bool,
}

pub async fn insert_order(
    db: &PgPool,
    user_id: i64,
    amount: f64,
    status: &str,
    product_id: Option<i64>,
    product_slug: Option<&str>,
    product_name: Option<&str>,
    product_image_url: Option<&str>,
) -> Result<Order, AppError> {
    sqlx::query_as::<_, Order>(
        r#"
        INSERT INTO orders (
            user_id,
            amount,
            status,
            product_id,
            product_slug,
            product_name,
            product_image_url
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING
            id,
            user_id,
            amount,
            status,
            created_at,
            updated_at,
            product_id,
            product_slug,
            product_name,
            product_image_url
        "#,
    )
    .bind(user_id)
    .bind(amount)
    .bind(status)
    .bind(product_id)
    .bind(product_slug)
    .bind(product_name)
    .bind(product_image_url)
    .fetch_one(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))
}

pub async fn find_orders_by_user_id(db: &PgPool, user_id: i64) -> Result<Vec<Order>, AppError> {
    sqlx::query_as::<_, Order>(
        r#"
        SELECT
            id,
            user_id,
            amount,
            status,
            created_at,
            updated_at,
            product_id,
            product_slug,
            product_name,
            product_image_url
        FROM orders
        WHERE user_id = $1
        ORDER BY id DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))
}

pub async fn find_order_by_id(db: &PgPool, order_id: i64) -> Result<Order, AppError> {
    sqlx::query_as::<_, Order>(
        r#"
        SELECT
            id,
            user_id,
            amount,
            status,
            created_at,
            updated_at,
            product_id,
            product_slug,
            product_name,
            product_image_url
        FROM orders
        WHERE id = $1
        "#,
    )
    .bind(order_id)
    .fetch_optional(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or_else(|| AppError::NotFound(format!("Order {order_id} not found")))
}

pub async fn update_order_status(
    db: &PgPool,
    order_id: i64,
    new_status: &str,
) -> Result<OrderStatusUpdateResult, AppError> {
    let mut tx = db
        .begin()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let current_order = sqlx::query_as::<_, Order>(
        r#"
        SELECT
            id,
            user_id,
            amount,
            status,
            created_at,
            updated_at,
            product_id,
            product_slug,
            product_name,
            product_image_url
        FROM orders
        WHERE id = $1
        FOR UPDATE
        "#,
    )
    .bind(order_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or_else(|| AppError::NotFound(format!("Order {order_id} not found")))?;

    if current_order.status == new_status {
        tx.commit()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        return Ok(OrderStatusUpdateResult {
            order: current_order,
            changed: false,
        });
    }

    let updated_order = sqlx::query_as::<_, Order>(
        r#"
        UPDATE orders
        SET status = $2, updated_at = NOW()
        WHERE id = $1
        RETURNING
            id,
            user_id,
            amount,
            status,
            created_at,
            updated_at,
            product_id,
            product_slug,
            product_name,
            product_image_url
        "#,
    )
    .bind(order_id)
    .bind(new_status)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO order_status_history (order_id, old_status, new_status)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(order_id)
    .bind(&current_order.status)
    .bind(new_status)
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(OrderStatusUpdateResult {
        order: updated_order,
        changed: true,
    })
}

pub async fn count_all_orders(db: &PgPool) -> Result<i64, AppError> {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM orders")
        .fetch_one(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
}