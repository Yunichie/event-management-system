use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::shared::{errors::RepoError, value_objects::UserId};
use crate::event::value_objects::EventId;

use super::{aggregate::Booking, value_objects::BookingId};

#[async_trait]
pub trait BookingRepository: Send + Sync {
    async fn save(&self, booking: &mut Booking) -> Result<(), RepoError>;
    async fn find_by_id(&self, id: BookingId) -> Result<Option<Booking>, RepoError>;
    async fn find_by_customer_and_event(
        &self,
        customer_id: UserId,
        event_id: EventId,
    ) -> Result<Option<Booking>, RepoError>;
    async fn find_pending_expired(
        &self,
        now: DateTime<Utc>,
    ) -> Result<Vec<Booking>, RepoError>;
}
