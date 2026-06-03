use async_trait::async_trait;
use application::ports::notification::NotificationService;
use domain::shared::value_objects::UserId;

pub struct EmailNotificationService;

impl EmailNotificationService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EmailNotificationService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NotificationService for EmailNotificationService {
    async fn send_booking_confirmation(
        &self,
        user_id: &UserId,
        booking_id: &str,
    ) -> Result<(), String> {
        println!(
            "Sending booking confirmation to user '{}' for booking '{}'",
            user_id.into_inner(),
            booking_id,
        );
        Ok(())
    }

    async fn send_payment_confirmation(
        &self,
        user_id: &UserId,
        booking_id: &str,
    ) -> Result<(), String> {
        println!(
            "Sending payment confirmation to user '{}' for booking '{}'",
            user_id.into_inner(),
            booking_id,
        );
        Ok(())
    }

    async fn send_refund_notification(
        &self,
        user_id: &UserId,
        refund_id: &str,
    ) -> Result<(), String> {
        println!(
            "Sending refund notification to user '{}' for refund '{}'",
            user_id.into_inner(),
            refund_id,
        );
        Ok(())
    }

    async fn send_event_cancellation(
        &self,
        user_id: &UserId,
        event_id: &str,
    ) -> Result<(), String> {
        println!(
            "Sending event cancellation notice to user '{}' for event '{}'",
            user_id.into_inner(),
            event_id,
        );
        Ok(())
    }
}
