use std::sync::Arc;

use domain::booking::value_objects::BookingId;
use domain::refund::{aggregate::Refund, repository::RefundRepository, value_objects::RefundId};

use crate::dto::RefundDto;
use crate::errors::{AppResult, ApplicationError};

use super::queries::{GetRefundByBookingQuery, GetRefundQuery};

pub struct RefundQueryHandler<R: RefundRepository> {
    refund_repo: Arc<R>,
}

impl<R: RefundRepository> RefundQueryHandler<R> {
    pub fn new(refund_repo: Arc<R>) -> Self {
        Self { refund_repo }
    }

    pub async fn handle_get_refund(&self, query: GetRefundQuery) -> AppResult<RefundDto> {
        let refund_id = RefundId::from(query.refund_id);
        let refund = self
            .refund_repo
            .find_by_id(refund_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Refund not found".to_string()))?;

        Ok(self.refund_to_dto(&refund))
    }

    pub async fn handle_get_refund_by_booking(
        &self,
        query: GetRefundByBookingQuery,
    ) -> AppResult<Option<RefundDto>> {
        let booking_id = BookingId::from(query.booking_id);
        let refund = self.refund_repo.find_by_booking(booking_id).await?;

        Ok(refund.as_ref().map(|r| self.refund_to_dto(r)))
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
