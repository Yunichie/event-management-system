use async_trait::async_trait;
use sqlx::PgPool;

use domain::{
    event::{
        aggregate::Event,
        repository::{EventFilter, EventRepository},
        value_objects::EventId,
    },
    shared::errors::RepoError,
};

use super::{
    mappers::{event_from_rows, event_to_row, ticket_category_to_row},
    models::{EventRow, EventStatusDb, TicketCategoryRow},
};

pub struct PgEventRepository {
    pool: PgPool,
}

impl PgEventRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventRepository for PgEventRepository {
    async fn save(&self, event: &mut Event) -> Result<(), RepoError> {
        let row = event_to_row(event);

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| RepoError::Database(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO events (id, organizer_id, name, description, start_date, end_date,
                                location, max_capacity, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (id) DO UPDATE SET
                name          = EXCLUDED.name,
                description   = EXCLUDED.description,
                start_date    = EXCLUDED.start_date,
                end_date      = EXCLUDED.end_date,
                location      = EXCLUDED.location,
                max_capacity  = EXCLUDED.max_capacity,
                status        = EXCLUDED.status,
                updated_at    = NOW()
            "#,
        )
        .bind(row.id)
        .bind(row.organizer_id)
        .bind(&row.name)
        .bind(&row.description)
        .bind(row.start_date)
        .bind(row.end_date)
        .bind(&row.location)
        .bind(row.max_capacity)
        .bind(row.status)
        .bind(row.created_at)
        .bind(row.updated_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| RepoError::Database(e.to_string()))?;

        for cat in &event.categories {
            let cat_row = ticket_category_to_row(cat);

            sqlx::query(
                r#"
                INSERT INTO ticket_categories (id, event_id, name, price_amount, price_currency,
                                               quota, remaining_quota, sales_start, sales_end,
                                               is_active, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                ON CONFLICT (id) DO UPDATE SET
                    name            = EXCLUDED.name,
                    price_amount    = EXCLUDED.price_amount,
                    price_currency  = EXCLUDED.price_currency,
                    quota           = EXCLUDED.quota,
                    remaining_quota = EXCLUDED.remaining_quota,
                    sales_start     = EXCLUDED.sales_start,
                    sales_end       = EXCLUDED.sales_end,
                    is_active       = EXCLUDED.is_active
                "#,
            )
            .bind(cat_row.id)
            .bind(cat_row.event_id)
            .bind(&cat_row.name)
            .bind(cat_row.price_amount)
            .bind(&cat_row.price_currency)
            .bind(cat_row.quota)
            .bind(cat_row.remaining_quota)
            .bind(cat_row.sales_start)
            .bind(cat_row.sales_end)
            .bind(cat_row.is_active)
            .bind(cat_row.created_at)
            .execute(&mut *tx)
            .await
            .map_err(|e| RepoError::Database(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| RepoError::Database(e.to_string()))?;

        let _events = event.take_events();

        Ok(())
    }

    async fn find_by_id(&self, id: EventId) -> Result<Option<Event>, RepoError> {
        let uuid = id.into_inner();

        let maybe_row: Option<EventRow> = sqlx::query_as(
            "SELECT id, organizer_id, name, description, start_date, end_date,
                    location, max_capacity, status, created_at, updated_at
             FROM events WHERE id = $1",
        )
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepoError::Database(e.to_string()))?;

        let row = match maybe_row {
            Some(r) => r,
            None => return Ok(None),
        };

        let category_rows: Vec<TicketCategoryRow> = sqlx::query_as(
            "SELECT id, event_id, name, price_amount, price_currency,
                    quota, remaining_quota, sales_start, sales_end, is_active, created_at
             FROM ticket_categories WHERE event_id = $1",
        )
        .bind(uuid)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepoError::Database(e.to_string()))?;

        Ok(Some(event_from_rows(row, category_rows)))
    }

    async fn find_published(&self, filter: EventFilter) -> Result<Vec<Event>, RepoError> {
        let event_rows: Vec<EventRow> = match (&filter.date, &filter.location) {
            (Some(date), Some(loc)) => {
                sqlx::query_as(
                    "SELECT id, organizer_id, name, description, start_date, end_date,
                            location, max_capacity, status, created_at, updated_at
                     FROM events
                     WHERE status = 'published'
                       AND start_date <= $1 AND end_date >= $1
                       AND location ILIKE '%' || $2 || '%'",
                )
                .bind(date)
                .bind(loc)
                .fetch_all(&self.pool)
                .await
            }
            (Some(date), None) => {
                sqlx::query_as(
                    "SELECT id, organizer_id, name, description, start_date, end_date,
                            location, max_capacity, status, created_at, updated_at
                     FROM events
                     WHERE status = 'published'
                       AND start_date <= $1 AND end_date >= $1",
                )
                .bind(date)
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(loc)) => {
                sqlx::query_as(
                    "SELECT id, organizer_id, name, description, start_date, end_date,
                            location, max_capacity, status, created_at, updated_at
                     FROM events
                     WHERE status = 'published'
                       AND location ILIKE '%' || $1 || '%'",
                )
                .bind(loc)
                .fetch_all(&self.pool)
                .await
            }
            (None, None) => {
                sqlx::query_as(
                    "SELECT id, organizer_id, name, description, start_date, end_date,
                            location, max_capacity, status, created_at, updated_at
                     FROM events
                     WHERE status = 'published'",
                )
                .fetch_all(&self.pool)
                .await
            }
        }
        .map_err(|e| RepoError::Database(e.to_string()))?;

        let mut events = Vec::with_capacity(event_rows.len());
        for row in event_rows {
            let uuid = row.id;
            let category_rows: Vec<TicketCategoryRow> = sqlx::query_as(
                "SELECT id, event_id, name, price_amount, price_currency,
                        quota, remaining_quota, sales_start, sales_end, is_active, created_at
                 FROM ticket_categories WHERE event_id = $1",
            )
            .bind(uuid)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepoError::Database(e.to_string()))?;

            events.push(event_from_rows(row, category_rows));
        }

        Ok(events)
    }
}
