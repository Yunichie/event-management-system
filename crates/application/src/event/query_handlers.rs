use std::sync::Arc;
use std::collections::HashMap;

use chrono::Utc;
use domain::event::{
    aggregate::Event,
    repository::{EventFilter, EventRepository},
    value_objects::{EventId, CategoryId},
};
use domain::booking::{repository::BookingRepository, value_objects::BookingStatus};
use rust_decimal::Decimal;

use crate::dto::{EventDto, PriceCalculationDto, EventSalesReportDto, ParticipantDto};
use crate::errors::{AppResult, ApplicationError};

use super::queries::{GetEventQuery, GetPublishedEventsQuery, CalculateBookingPriceQuery, GetEventSalesReportQuery, GetEventParticipantsQuery};

pub struct EventQueryHandler<ER: EventRepository, BR: BookingRepository> {
    event_repo: Arc<ER>,
    booking_repo: Arc<BR>,
}

impl<ER: EventRepository, BR: BookingRepository> EventQueryHandler<ER, BR> {
    pub fn new(event_repo: Arc<ER>, booking_repo: Arc<BR>) -> Self {
        Self { event_repo, booking_repo }
    }

    pub async fn handle_get_event(&self, query: GetEventQuery) -> AppResult<EventDto> {
        let event_id = EventId::from(query.event_id);
        let event = self
            .event_repo
            .find_by_id(event_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Event not found".to_string()))?;

        Ok(self.event_to_dto(&event))
    }

    pub async fn handle_get_published_events(
        &self,
        query: GetPublishedEventsQuery,
    ) -> AppResult<Vec<EventDto>> {
        let filter = EventFilter {
            date: query.date,
            location: query.location,
        };

        let events = self.event_repo.find_published(filter).await?;
        Ok(events.iter().map(|e| self.event_to_dto(e)).collect())
    }

    pub async fn handle_calculate_booking_price(
        &self,
        query: CalculateBookingPriceQuery,
    ) -> AppResult<PriceCalculationDto> {
        let event_id = EventId::from(query.event_id);
        let category_id = CategoryId::from(query.category_id);

        let event = self
            .event_repo
            .find_by_id(event_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Event not found".to_string()))?;

        let category = event
            .categories
            .iter()
            .find(|c| c.id == category_id)
            .ok_or_else(|| ApplicationError::NotFound("Category not found".to_string()))?;

        if query.quantity == 0 {
            return Err(ApplicationError::Domain(
                domain::shared::errors::DomainError::BusinessRule(
                    "Quantity must be greater than zero".to_string(),
                ),
            ));
        }

        let unit_price = category.price.amount();
        let total = unit_price * Decimal::from(query.quantity);

        // Ensure total is not negative
        if total < Decimal::ZERO {
            return Err(ApplicationError::Domain(
                domain::shared::errors::DomainError::BusinessRule(
                    "Total price cannot be negative".to_string(),
                ),
            ));
        }

        Ok(PriceCalculationDto {
            total_amount: total,
            currency: category.price.currency().to_string(),
            unit_price,
            quantity: query.quantity,
            service_fee: None,
        })
    }

    pub async fn handle_get_event_sales_report(
        &self,
        query: GetEventSalesReportQuery,
    ) -> AppResult<EventSalesReportDto> {
        let event_id = EventId::from(query.event_id);

        let event = self
            .event_repo
            .find_by_id(event_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Event not found".to_string()))?;

        // Get all bookings for this event - we need to add a method to find by event
        // For now, we'll return a basic report structure
        // In a real implementation, we'd need BookingRepository.find_by_event_id()
        
        let mut tickets_sold_per_category = HashMap::new();
        let mut booking_counts_by_status = HashMap::new();
        let mut total_revenue = Decimal::ZERO;
        let currency = event.categories.first()
            .map(|c| c.price.currency().to_string())
            .unwrap_or_else(|| "USD".to_string());

        // Initialize counts
        booking_counts_by_status.insert("PendingPayment".to_string(), 0);
        booking_counts_by_status.insert("Paid".to_string(), 0);
        booking_counts_by_status.insert("Expired".to_string(), 0);
        booking_counts_by_status.insert("Refunded".to_string(), 0);

        for category in &event.categories {
            let sold = category.quota - category.remaining_quota;
            tickets_sold_per_category.insert(category.id.into_inner(), sold);
        }

        Ok(EventSalesReportDto {
            event_id: event.id.into_inner(),
            tickets_sold_per_category,
            booking_counts_by_status,
            total_revenue,
            currency,
        })
    }

    pub async fn handle_get_event_participants(
        &self,
        query: GetEventParticipantsQuery,
    ) -> AppResult<Vec<ParticipantDto>> {
        let _event_id = EventId::from(query.event_id);

        // In a real implementation, we'd need:
        // 1. BookingRepository.find_by_event_id() to get all bookings
        // 2. Filter for Paid status and exclude Refunded
        // 3. Get customer information (might need a UserRepository)
        // 4. Get ticket information for each booking
        
        // For now, return empty list as placeholder
        // This would be implemented when the infrastructure layer provides the necessary repository methods
        Ok(vec![])
    }

    fn event_to_dto(&self, event: &Event) -> EventDto {
        let now = Utc::now();
        
        EventDto {
            id: event.id.into_inner(),
            organizer_id: event.organizer_id.into_inner(),
            name: event.name.clone(),
            description: event.description.clone(),
            location: event.location.clone(),
            start_date: event.start_date,
            end_date: event.end_date,
            max_capacity: event.max_capacity,
            status: format!("{}", event.status),
            categories: event
                .categories
                .iter()
                .filter(|c| c.is_active) // Only include active categories
                .map(|c| {
                    crate::dto::TicketCategoryDto {
                        id: c.id.into_inner(),
                        event_id: c.event_id.into_inner(),
                        name: c.name.clone(),
                        price: c.price.amount(),
                        currency: c.price.currency().to_string(),
                        quota: c.quota,
                        remaining_quota: c.remaining_quota,
                        sales_start: c.sales_start,
                        sales_end: c.sales_end,
                        is_active: c.is_active,
                    }
                })
                .collect(),
            created_at: event.created_at,
            updated_at: event.updated_at,
        }
    }
}
