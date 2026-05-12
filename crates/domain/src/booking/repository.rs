use async_trait::async_trait;

use crate::shared::errors::RepoError;

use super::{aggregate::Booking, value_objects::BookingId};

#[async_trait]
pub trait BookingRepository: Send + Sync {
    async fn save(&self, booking: &mut Booking) -> Result<(), RepoError>;
    async fn find_by_id(&self, id: BookingId) -> Result<Option<Booking>, RepoError>;
}
