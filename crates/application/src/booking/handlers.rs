use std::sync::Arc;

use chrono::Utc;
use domain::booking::{
    aggregate::Booking, repository::BookingRepository, value_objects::BookingId,
};
use domain::event::{
    repository::EventRepository,
    value_objects::{CategoryId, EventId, EventStatus},
};
use domain::shared::value_objects::{Money, UserId};
use domain::ticket::repository::TicketRepository;

use crate::dto::BookingDto;
use crate::errors::{ApplicationError, AppResult};
use crate::ports::payment_gateway::PaymentGateway;
use crate::ports::notification::NotificationService;

use super::commands::{CreateBookingCommand, ExpireBookingCommand, PayBookingCommand};

pub struct BookingCommandHandler<
    BR: BookingRepository,
    ER: EventRepository,
    TR: TicketRepository,
    PG: PaymentGateway,
    NS: NotificationService,
> {
    booking_repo: Arc<BR>,
    event_repo: Arc<ER>,
    ticket_repo: Arc<TR>,
    payment_gateway: Arc<PG>,
    notification_service: Arc<NS>,
}

impl<
        BR: BookingRepository,
        ER: EventRepository,
        TR: TicketRepository,
        PG: PaymentGateway,
        NS: NotificationService,
    > BookingCommandHandler<BR, ER, TR, PG, NS>
{
    pub fn new(
        booking_repo: Arc<BR>,
        event_repo: Arc<ER>,
        ticket_repo: Arc<TR>,
        payment_gateway: Arc<PG>,
        notification_service: Arc<NS>,
    ) -> Self {
        Self {
            booking_repo,
            event_repo,
            ticket_repo,
            payment_gateway,
            notification_service,
        }
    }

    pub async fn handle_create_booking(&self, cmd: CreateBookingCommand) -> AppResult<BookingDto> {
        let user_id = UserId::from(cmd.user_id);
        let event_id = EventId::from(cmd.event_id);
        let category_id = CategoryId::from(cmd.category_id);

        // Check if customer already has an active booking for this event
        if let Some(_existing) = self
            .booking_repo
            .find_by_customer_and_event(user_id, event_id)
            .await?
        {
            return Err(ApplicationError::Domain(
                domain::shared::errors::DomainError::BusinessRule(
                    "Customer already has a booking for this event".to_string(),
                ),
            ));
        }

        // Load event and validate
        let mut event = self
            .event_repo
            .find_by_id(event_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Event not found".to_string()))?;

        if event.status != EventStatus::Published {
            return Err(ApplicationError::Domain(
                domain::shared::errors::DomainError::BusinessRule(
                    "Event is not published".to_string(),
                ),
            ));
        }

        // Find category and validate availability
        let category = event
            .categories
            .iter_mut()
            .find(|c| c.id == category_id)
            .ok_or_else(|| ApplicationError::NotFound("Category not found".to_string()))?;

        let now = Utc::now();
        if !category.is_available_now(now) {
            return Err(ApplicationError::Domain(
                domain::shared::errors::DomainError::BusinessRule(
                    "Category is not available for purchase".to_string(),
                ),
            ));
        }

        if cmd.quantity > category.remaining_quota {
            return Err(ApplicationError::Domain(
                domain::shared::errors::DomainError::BusinessRule(format!(
                    "Requested quantity {} exceeds remaining quota {}",
                    cmd.quantity, category.remaining_quota
                )),
            ));
        }

        // Reserve tickets
        category.reserve(cmd.quantity)?;

        // Create booking
        let booking_id = BookingId::new();
        let mut booking = Booking::new(
            booking_id,
            user_id,
            event_id,
            category_id,
            cmd.quantity,
            category.price.clone(),
        )?;

        // Save both
        self.event_repo.save(&mut event).await?;
        self.booking_repo.save(&mut booking).await?;

        // Send booking confirmation notification (don't fail if notification fails)
        if let Err(e) = self
            .notification_service
            .send_booking_confirmation(&user_id, &booking_id.to_string())
            .await
        {
            eprintln!("Failed to send booking confirmation: {}", e);
        }

        Ok(self.booking_to_dto(&booking))
    }

    pub async fn handle_pay_booking(&self, cmd: PayBookingCommand) -> AppResult<BookingDto> {
        let booking_id = BookingId::from(cmd.booking_id);
        let mut booking = self
            .booking_repo
            .find_by_id(booking_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Booking not found".to_string()))?;

        let amount = Money::new(cmd.amount, cmd.currency)?;

        // Process payment through gateway
        let payment_ref = self
            .payment_gateway
            .process_payment(&amount, &booking_id.to_string())
            .await
            .map_err(|e| ApplicationError::PaymentFailed(e))?;

        // Mark booking as paid (this also creates tickets in the domain)
        let now = Utc::now();
        booking.pay(&amount, now)?;

        // Save booking with tickets
        self.booking_repo.save(&mut booking).await?;

        // Save tickets to ticket repository
        if !booking.tickets.is_empty() {
            self.ticket_repo.save_multiple(&mut booking.tickets).await?;
        }

        // Send payment confirmation notification (don't fail if notification fails)
        if let Err(e) = self
            .notification_service
            .send_payment_confirmation(&booking.user_id, &booking_id.to_string())
            .await
        {
            eprintln!("Failed to send payment confirmation: {}", e);
        }

        Ok(self.booking_to_dto(&booking))
    }

    pub async fn handle_expire_booking(&self, cmd: ExpireBookingCommand) -> AppResult<BookingDto> {
        let booking_id = BookingId::from(cmd.booking_id);
        let mut booking = self
            .booking_repo
            .find_by_id(booking_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Booking not found".to_string()))?;

        // Load event to release quota
        let mut event = self
            .event_repo
            .find_by_id(booking.event_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Event not found".to_string()))?;

        // Release quota
        if let Some(category) = event
            .categories
            .iter_mut()
            .find(|c| c.id == booking.category_id)
        {
            category.release(booking.quantity);
        }

        // Expire booking
        booking.expire()?;

        // Save both
        self.event_repo.save(&mut event).await?;
        self.booking_repo.save(&mut booking).await?;

        Ok(self.booking_to_dto(&booking))
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
