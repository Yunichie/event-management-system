use std::sync::Arc;

use domain::booking::{repository::BookingRepository, value_objects::{BookingId, BookingStatus}};
use domain::refund::{aggregate::Refund, repository::RefundRepository, value_objects::RefundId};
use domain::shared::value_objects::UserId;
use domain::ticket::value_objects::TicketStatus;

use crate::dto::RefundDto;
use crate::errors::{ApplicationError, AppResult};
use crate::ports::refund_service::RefundService;
use crate::ports::notification::NotificationService;

use super::commands::{
    ApproveRefundCommand, PayoutRefundCommand, RejectRefundCommand, RequestRefundCommand,
};

pub struct RefundCommandHandler<
    RR: RefundRepository,
    BR: BookingRepository,
    RS: RefundService,
    NS: NotificationService,
> {
    refund_repo: Arc<RR>,
    booking_repo: Arc<BR>,
    refund_service: Arc<RS>,
    notification_service: Arc<NS>,
}

impl<
        RR: RefundRepository,
        BR: BookingRepository,
        RS: RefundService,
        NS: NotificationService,
    > RefundCommandHandler<RR, BR, RS, NS>
{
    pub fn new(
        refund_repo: Arc<RR>,
        booking_repo: Arc<BR>,
        refund_service: Arc<RS>,
        notification_service: Arc<NS>,
    ) -> Self {
        Self {
            refund_repo,
            booking_repo,
            refund_service,
            notification_service,
        }
    }

    pub async fn handle_request_refund(&self, cmd: RequestRefundCommand) -> AppResult<RefundDto> {
        let booking_id = BookingId::from(cmd.booking_id);
        let user_id = UserId::from(cmd.user_id);

        // Check if refund already exists
        if let Some(_existing) = self.refund_repo.find_by_booking(booking_id).await? {
            return Err(ApplicationError::Domain(
                domain::shared::errors::DomainError::BusinessRule(
                    "Refund already requested for this booking".to_string(),
                ),
            ));
        }

        // Load booking and validate
        let booking = self
            .booking_repo
            .find_by_id(booking_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Booking not found".to_string()))?;

        if booking.status != BookingStatus::Paid {
            return Err(ApplicationError::Domain(
                domain::shared::errors::DomainError::BusinessRule(
                    "Can only refund paid bookings".to_string(),
                ),
            ));
        }

        // Check if any ticket has been checked in
        let has_checked_in = booking
            .tickets
            .iter()
            .any(|t| t.status == TicketStatus::CheckedIn);

        if has_checked_in {
            return Err(ApplicationError::Domain(
                domain::shared::errors::DomainError::CheckedInTicketRefundDenied,
            ));
        }

        // Create refund
        let refund_id = RefundId::new();
        let mut refund = Refund::request(
            refund_id,
            booking_id,
            user_id,
            booking.total_amount.clone(),
            cmd.reason,
        );

        self.refund_repo.save(&mut refund).await?;

        Ok(self.refund_to_dto(&refund))
    }

    pub async fn handle_approve_refund(&self, cmd: ApproveRefundCommand) -> AppResult<RefundDto> {
        let refund_id = RefundId::from(cmd.refund_id);
        let mut refund = self
            .refund_repo
            .find_by_id(refund_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Refund not found".to_string()))?;

        refund.approve()?;

        // Load booking and mark as refunded
        let mut booking = self
            .booking_repo
            .find_by_id(refund.booking_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Booking not found".to_string()))?;

        booking.mark_refund_required()?;

        // Cancel all tickets
        for ticket in &mut booking.tickets {
            ticket.cancel()?;
        }

        // Save both
        self.refund_repo.save(&mut refund).await?;
        self.booking_repo.save(&mut booking).await?;

        // Send refund notification (don't fail if notification fails)
        if let Err(e) = self
            .notification_service
            .send_refund_notification(&refund.user_id, &refund_id.to_string())
            .await
        {
            eprintln!("Failed to send refund notification: {}", e);
        }

        Ok(self.refund_to_dto(&refund))
    }

    pub async fn handle_reject_refund(&self, cmd: RejectRefundCommand) -> AppResult<RefundDto> {
        let refund_id = RefundId::from(cmd.refund_id);
        let mut refund = self
            .refund_repo
            .find_by_id(refund_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Refund not found".to_string()))?;

        if cmd.rejection_reason.trim().is_empty() {
            return Err(ApplicationError::Domain(
                domain::shared::errors::DomainError::BusinessRule(
                    "Rejection reason is required".to_string(),
                ),
            ));
        }

        refund.reject(cmd.rejection_reason)?;
        self.refund_repo.save(&mut refund).await?;

        Ok(self.refund_to_dto(&refund))
    }

    pub async fn handle_payout_refund(&self, cmd: PayoutRefundCommand) -> AppResult<RefundDto> {
        let refund_id = RefundId::from(cmd.refund_id);
        let mut refund = self
            .refund_repo
            .find_by_id(refund_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Refund not found".to_string()))?;

        // Process refund through service
        let payment_ref = self
            .refund_service
            .process_refund(&refund.amount, &refund_id.to_string())
            .await
            .map_err(|e| ApplicationError::RefundFailed(e))?;

        refund.mark_paid_out(payment_ref)?;
        self.refund_repo.save(&mut refund).await?;

        Ok(self.refund_to_dto(&refund))
    }

    fn refund_to_dto(&self, refund: &Refund) -> RefundDto {
        RefundDto {
            id: refund.id.into_inner(),
            booking_id: refund.booking_id.into_inner(),
            user_id: refund.user_id.into_inner(),
            amount: refund.amount.amount(),
            currency: refund.amount.currency().to_string(),
            status: format!("{}", refund.status),
            reason: refund.reason.clone(),
            rejection_reason: refund.rejection_reason.clone(),
            payment_reference: refund.payment_reference.clone(),
            requested_at: refund.requested_at,
        }
    }
}
