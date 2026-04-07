use common::errors::AppError;
use sqlx::PgPool;

use crate::models::Payment;

pub async fn insert_payment(
    db: &PgPool,
    order_id: i64,
    status: &str,
    transaction_id: Option<&str>,
) -> Result<Payment, AppError> {
    sqlx::query_as::<_, Payment>(
        r#"
        INSERT INTO payments (order_id, status, transaction_id)
        VALUES ($1, $2, $3)
        RETURNING id, order_id, status, transaction_id, created_at
        "#,
    )
    .bind(order_id)
    .bind(status)
    .bind(transaction_id)
    .fetch_one(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))
}