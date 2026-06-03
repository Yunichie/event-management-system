use async_trait::async_trait;
use sqlx::PgPool;

use domain::{
    booking::value_objects::BookingId,
    refund::{
        aggregate::Refund,
        repository::RefundRepository,
        value_objects::RefundId,
    },
    shared::errors::RepoError,
};

use super::{
    mappers::{refund_from_row, refund_to_row},
    models::RefundRow,
};

pub struct PgRefundRepository {
    pool: PgPool,
}

impl PgRefundRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RefundRepository for PgRefundRepository {
    async fn save(&self, refund: &mut Refund) -> Result<(), RepoError> {
        let row = refund_to_row(refund);

        sqlx::query(
            r#"
            INSERT INTO refunds (id, booking_id, customer_id, amount, currency, status,
                                 rejection_reason, payment_reference, requested_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (id) DO UPDATE SET
                status            = EXCLUDED.status,
                rejection_reason  = EXCLUDED.rejection_reason,
                payment_reference = EXCLUDED.payment_reference,
                updated_at        = NOW()
            "#,
        )
        .bind(row.id)
        .bind(row.booking_id)
        .bind(row.customer_id)
        .bind(row.amount)
        .bind(&row.currency)
        .bind(row.status)
        .bind(&row.rejection_reason)
        .bind(&row.payment_reference)
        .bind(row.requested_at)
        .bind(row.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| RepoError::Database(e.to_string()))?;

        let _events = refund.take_events();

        Ok(())
    }

    async fn find_by_id(&self, id: RefundId) -> Result<Option<Refund>, RepoError> {
        let uuid = id.into_inner();

        let maybe_row: Option<RefundRow> = sqlx::query_as(
            "SELECT id, booking_id, customer_id, amount, currency, status,
                    rejection_reason, payment_reference, requested_at, updated_at
             FROM refunds WHERE id = $1",
        )
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepoError::Database(e.to_string()))?;

        Ok(maybe_row.map(refund_from_row))
    }

    async fn find_by_booking(&self, booking_id: BookingId) -> Result<Option<Refund>, RepoError> {
        let uuid = booking_id.into_inner();

        let maybe_row: Option<RefundRow> = sqlx::query_as(
            "SELECT id, booking_id, customer_id, amount, currency, status,
                    rejection_reason, payment_reference, requested_at, updated_at
             FROM refunds WHERE booking_id = $1",
        )
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepoError::Database(e.to_string()))?;

        Ok(maybe_row.map(refund_from_row))
    }
}
