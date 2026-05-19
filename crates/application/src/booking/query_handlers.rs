use std::sync::Arc;

use domain::booking::{aggregate::Booking, repository::BookingRepository, value_objects::BookingId};
use domain::event::value_objects::EventId;
use domain::shared::value_objects::UserId;

use crate::dto::BookingDto;
use crate::errors::{ApplicationError, AppResult};

use super::queries::{GetBookingQuery, GetCustomerBookingsQuery};

pub struct BookingQueryHandler<R: BookingRepository> {
    booking_repo: Arc<R>,
}

impl<R: BookingRepository> BookingQueryHandler<R> {
    pub fn new(booking_repo: Arc<R>) -> Self {
        Self { booking_repo }
    }

    pub async fn handle_get_booking(&self, query: GetBookingQuery) -> AppResult<BookingDto> {
        let booking_id = BookingId::from(query.booking_id);
        let booking = self
            .booking_repo
            .find_by_id(booking_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Booking not found".to_string()))?;

        Ok(self.booking_to_dto(&booking))
    }

    pub async fn handle_get_customer_bookings(
        &self,
        query: GetCustomerBookingsQuery,
    ) -> AppResult<Option<BookingDto>> {
        let customer_id = UserId::from(query.customer_id);
        let event_id = EventId::from(query.event_id);

        let booking = self
            .booking_repo
            .find_by_customer_and_event(customer_id, event_id)
            .await?;

        Ok(booking.as_ref().map(|b| self.booking_to_dto(b)))
    }

    fn booking_to_dto(&self, booking: &Booking) -> BookingDto {
        BookingDto {
            id: booking.id.into_inner(),
            user_id: booking.user_id.into_inner(),
            event_id: booking.event_id.into_inner(),
            category_id: booking.category_id.into_inner(),
            quantity: booking.quantity,
            total_amount: booking.total_amount.amount(),
            currency: booking.total_amount.currency().to_string(),
            status: format!("{}", booking.status),
            payment_deadline: booking.payment_deadline,
            created_at: booking.created_at,
            updated_at: booking.updated_at,
        }
    }
}
