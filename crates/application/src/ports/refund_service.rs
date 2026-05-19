use async_trait::async_trait;
use domain::shared::value_objects::Money;

#[async_trait]
pub trait RefundService: Send + Sync {
    async fn process_refund(&self, amount: &Money, reference: &str) -> Result<String, String>;
}
