use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundDto {
    pub id: Uuid,
    pub booking_id: Uuid,
    pub user_id: Uuid,
    pub amount: Decimal,
    pub currency: String,
    pub status: String,
    pub reason: Option<String>,
    pub rejection_reason: Option<String>,
    pub payment_reference: Option<String>,
    pub requested_at: DateTime<Utc>,
}
