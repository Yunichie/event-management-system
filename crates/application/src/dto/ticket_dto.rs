use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketDto {
    pub id: Uuid,
    pub booking_id: Uuid,
    pub event_id: Uuid,
    pub category_id: Uuid,
    pub code: String,
    pub status: String,
    pub checked_in_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
