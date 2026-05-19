use async_trait::async_trait;
use domain::shared::value_objects::UserId;

#[async_trait]
pub trait NotificationService: Send + Sync {
    async fn send_booking_confirmation(&self, user_id: &UserId, booking_id: &str) -> Result<(), String>;
    async fn send_payment_confirmation(&self, user_id: &UserId, booking_id: &str) -> Result<(), String>;
    async fn send_refund_notification(&self, user_id: &UserId, refund_id: &str) -> Result<(), String>;
    async fn send_event_cancellation(&self, user_id: &UserId, event_id: &str) -> Result<(), String>;
}
