use async_trait::async_trait;
use domain::shared::value_objects::Money;

#[async_trait]
pub trait PaymentGateway: Send + Sync {
    async fn process_payment(&self, amount: &Money, reference: &str) -> Result<String, String>;
}
