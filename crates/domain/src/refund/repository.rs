use async_trait::async_trait;

use crate::shared::errors::RepoError;

use super::{aggregate::Refund, value_objects::RefundId};

#[async_trait]
pub trait RefundRepository: Send + Sync {
    async fn save(&self, refund: &mut Refund) -> Result<(), RepoError>;
    async fn find_by_id(&self, id: RefundId) -> Result<Option<Refund>, RepoError>;
}
