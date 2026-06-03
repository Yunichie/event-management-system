use async_trait::async_trait;
use application::ports::payment_gateway::PaymentGateway;
use domain::shared::value_objects::Money;
use uuid::Uuid;

pub struct StripePaymentGateway {
    api_key: String,
}

impl StripePaymentGateway {
    pub fn new() -> Self {
        Self {
            api_key: String::from("sk_test_mock_key"),
        }
    }
}

impl Default for StripePaymentGateway {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PaymentGateway for StripePaymentGateway {
    async fn process_payment(&self, amount: &Money, reference: &str) -> Result<String, String> {
        println!(
            "Processing payment of {} {} for reference '{}' (api_key: {}...)",
            amount.amount(),
            amount.currency(),
            reference,
            &self.api_key[..10],
        );

        let transaction_id = Uuid::new_v4().to_string();

        println!(
            "Payment successful — transaction_id: {}",
            transaction_id,
        );

        Ok(transaction_id)
    }
}
