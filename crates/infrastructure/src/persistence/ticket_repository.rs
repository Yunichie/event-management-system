use async_trait::async_trait;
use sqlx::PgPool;

use domain::{
    shared::errors::RepoError,
    ticket::{
        entity::Ticket,
        repository::TicketRepository,
        value_objects::{TicketCode, TicketId},
    },
};

use super::{
    mappers::{ticket_from_row_with_category, ticket_to_row},
    models::TicketWithCategoryRow,
};

pub struct PgTicketRepository {
    pool: PgPool,
}

impl PgTicketRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TicketRepository for PgTicketRepository {
    async fn save(&self, ticket: &mut Ticket) -> Result<(), RepoError> {
        let row = ticket_to_row(ticket);

        sqlx::query(
            r#"
            INSERT INTO tickets (id, booking_id, event_id, code, status, checked_in_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE SET
                status        = EXCLUDED.status,
                checked_in_at = EXCLUDED.checked_in_at
            "#,
        )
        .bind(row.id)
        .bind(row.booking_id)
        .bind(row.event_id)
        .bind(&row.code)
        .bind(row.status)
        .bind(row.checked_in_at)
        .bind(row.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| RepoError::Database(e.to_string()))?;

        let _events = ticket.take_events();

        Ok(())
    }

    async fn save_multiple(&self, tickets: &mut [Ticket]) -> Result<(), RepoError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| RepoError::Database(e.to_string()))?;

        for ticket in tickets.iter() {
            let row = ticket_to_row(ticket);

            sqlx::query(
                r#"
                INSERT INTO tickets (id, booking_id, event_id, code, status, checked_in_at, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (id) DO UPDATE SET
                    status        = EXCLUDED.status,
                    checked_in_at = EXCLUDED.checked_in_at
                "#,
            )
            .bind(row.id)
            .bind(row.booking_id)
            .bind(row.event_id)
            .bind(&row.code)
            .bind(row.status)
            .bind(row.checked_in_at)
            .bind(row.created_at)
            .execute(&mut *tx)
            .await
            .map_err(|e| RepoError::Database(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| RepoError::Database(e.to_string()))?;

        for ticket in tickets.iter_mut() {
            let _events = ticket.take_events();
        }

        Ok(())
    }

    async fn find_by_id(&self, id: TicketId) -> Result<Option<Ticket>, RepoError> {
        let uuid = id.into_inner();

        let maybe_row: Option<TicketWithCategoryRow> = sqlx::query_as(
            "SELECT t.id, t.booking_id, t.event_id, t.code, t.status,
                    t.checked_in_at, t.created_at, b.category_id
             FROM tickets t
             JOIN bookings b ON t.booking_id = b.id
             WHERE t.id = $1",
        )
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepoError::Database(e.to_string()))?;

        Ok(maybe_row.map(ticket_from_row_with_category))
    }

    async fn find_by_code(&self, code: &TicketCode) -> Result<Option<Ticket>, RepoError> {
        let code_str = code.as_str();

        let maybe_row: Option<TicketWithCategoryRow> = sqlx::query_as(
            "SELECT t.id, t.booking_id, t.event_id, t.code, t.status,
                    t.checked_in_at, t.created_at, b.category_id
             FROM tickets t
             JOIN bookings b ON t.booking_id = b.id
             WHERE t.code = $1",
        )
        .bind(code_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepoError::Database(e.to_string()))?;

        Ok(maybe_row.map(ticket_from_row_with_category))
    }
}
