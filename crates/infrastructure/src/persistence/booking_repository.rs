use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use domain::{
    booking::{
        aggregate::Booking,
        repository::BookingRepository,
        value_objects::BookingId,
    },
    event::value_objects::{CategoryId, EventId},
    shared::{errors::RepoError, value_objects::UserId},
};

use super::{
    mappers::{booking_from_row, booking_to_row, ticket_from_row},
    models::{BookingRow, TicketRow},
};

pub struct PgBookingRepository {
    pool: PgPool,
}

impl PgBookingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

async fn load_tickets_for_booking(
    pool: &PgPool,
    booking_id: uuid::Uuid,
    category_id: CategoryId,
) -> Result<Vec<domain::ticket::entity::Ticket>, RepoError> {
    let ticket_rows: Vec<TicketRow> = sqlx::query_as(
        "SELECT id, booking_id, event_id, code, status, checked_in_at, created_at
         FROM tickets WHERE booking_id = $1",
    )
    .bind(booking_id)
    .fetch_all(pool)
    .await
    .map_err(|e| RepoError::Database(e.to_string()))?;

    Ok(ticket_rows
        .into_iter()
        .map(|r| ticket_from_row(r, category_id))
        .collect())
}

#[async_trait]
impl BookingRepository for PgBookingRepository {
    async fn save(&self, booking: &mut Booking) -> Result<(), RepoError> {
        let row = booking_to_row(booking);

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| RepoError::Database(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO bookings (id, customer_id, event_id, category_id, quantity,
                                  total_amount, total_currency, status, payment_deadline,
                                  created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (id) DO UPDATE SET
                quantity         = EXCLUDED.quantity,
                total_amount     = EXCLUDED.total_amount,
                total_currency   = EXCLUDED.total_currency,
                status           = EXCLUDED.status,
                payment_deadline = EXCLUDED.payment_deadline,
                updated_at       = NOW()
            "#,
        )
        .bind(row.id)
        .bind(row.customer_id)
        .bind(row.event_id)
        .bind(row.category_id)
        .bind(row.quantity)
        .bind(row.total_amount)
        .bind(&row.total_currency)
        .bind(row.status)
        .bind(row.payment_deadline)
        .bind(row.created_at)
        .bind(row.updated_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| RepoError::Database(e.to_string()))?;

        for ticket in &booking.tickets {
            let t_row = super::mappers::ticket_to_row(ticket);

            sqlx::query(
                r#"
                INSERT INTO tickets (id, booking_id, event_id, code, status, checked_in_at, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (id) DO UPDATE SET
                    status        = EXCLUDED.status,
                    checked_in_at = EXCLUDED.checked_in_at
                "#,
            )
            .bind(t_row.id)
            .bind(t_row.booking_id)
            .bind(t_row.event_id)
            .bind(&t_row.code)
            .bind(t_row.status)
            .bind(t_row.checked_in_at)
            .bind(t_row.created_at)
            .execute(&mut *tx)
            .await
            .map_err(|e| RepoError::Database(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| RepoError::Database(e.to_string()))?;

        let _events = booking.take_events();

        Ok(())
    }

    async fn find_by_id(&self, id: BookingId) -> Result<Option<Booking>, RepoError> {
        let uuid = id.into_inner();

        let maybe_row: Option<BookingRow> = sqlx::query_as(
            "SELECT id, customer_id, event_id, category_id, quantity,
                    total_amount, total_currency, status, payment_deadline,
                    created_at, updated_at
             FROM bookings WHERE id = $1",
        )
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepoError::Database(e.to_string()))?;

        let row = match maybe_row {
            Some(r) => r,
            None => return Ok(None),
        };

        let category_id = CategoryId::from(row.category_id);
        let tickets = load_tickets_for_booking(&self.pool, uuid, category_id).await?;

        Ok(Some(booking_from_row(row, tickets)))
    }

    async fn find_by_customer_and_event(
        &self,
        customer_id: UserId,
        event_id: EventId,
    ) -> Result<Option<Booking>, RepoError> {
        let cid = customer_id.into_inner();
        let eid = event_id.into_inner();

        let maybe_row: Option<BookingRow> = sqlx::query_as(
            "SELECT id, customer_id, event_id, category_id, quantity,
                    total_amount, total_currency, status, payment_deadline,
                    created_at, updated_at
             FROM bookings
             WHERE customer_id = $1 AND event_id = $2
             AND status IN ('pending_payment', 'paid')
             ORDER BY created_at DESC
             LIMIT 1",
        )
        .bind(cid)
        .bind(eid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepoError::Database(e.to_string()))?;

        let row = match maybe_row {
            Some(r) => r,
            None => return Ok(None),
        };

        let booking_uuid = row.id;
        let category_id = CategoryId::from(row.category_id);
        let tickets = load_tickets_for_booking(&self.pool, booking_uuid, category_id).await?;

        Ok(Some(booking_from_row(row, tickets)))
    }

    async fn find_pending_expired(
        &self,
        now: DateTime<Utc>,
    ) -> Result<Vec<Booking>, RepoError> {
        let rows: Vec<BookingRow> = sqlx::query_as(
            "SELECT id, customer_id, event_id, category_id, quantity,
                    total_amount, total_currency, status, payment_deadline,
                    created_at, updated_at
             FROM bookings
             WHERE status = 'pending_payment' AND payment_deadline < $1",
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepoError::Database(e.to_string()))?;

        let mut bookings = Vec::with_capacity(rows.len());
        for row in rows {
            let booking_uuid = row.id;
            let category_id = CategoryId::from(row.category_id);
            let tickets =
                load_tickets_for_booking(&self.pool, booking_uuid, category_id).await?;
            bookings.push(booking_from_row(row, tickets));
        }

        Ok(bookings)
    }
}
