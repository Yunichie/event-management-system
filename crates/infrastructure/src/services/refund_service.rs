use async_trait::async_trait;
use application::ports::refund_service::RefundService;
use domain::shared::value_objects::Money;
use uuid::Uuid;

pub struct BankRefundService;

impl BankRefundService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BankRefundService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RefundService for BankRefundService {
    async fn process_refund(&self, amount: &Money, reference: &str) -> Result<String, String> {
        println!(
            "Processing refund of {} {} for reference '{}'",
            amount.amount(),
            amount.currency(),
            reference,
        );

        let refund_reference = Uuid::new_v4().to_string();

        println!(
            "Refund successful — refund_reference: {}",
            refund_reference,
        );

        Ok(refund_reference)
    }
}
