use std::sync::Arc;

use domain::event::{
    aggregate::Event,
    repository::EventRepository,
    ticket_category::TicketCategory,
    value_objects::{CategoryId, EventId, EventStatus},
};
use domain::shared::value_objects::{Money, UserId};
use crate::ports::notification::NotificationService;

use crate::dto::EventDto;
use crate::errors::{ApplicationError, AppResult};

use super::commands::{
    AddTicketCategoryCommand, CancelEventCommand, CreateEventCommand, PublishEventCommand,
    DisableTicketCategoryCommand,
};

pub struct EventCommandHandler<R: EventRepository, NS: NotificationService> {
    event_repo: Arc<R>,
    notification_service: Arc<NS>,
}

impl<R: EventRepository, NS: NotificationService> EventCommandHandler<R, NS> {
    pub fn new(event_repo: Arc<R>, notification_service: Arc<NS>) -> Self {
        Self { event_repo, notification_service }
    }

    pub async fn handle_create_event(&self, cmd: CreateEventCommand) -> AppResult<EventDto> {
        let event_id = EventId::new();
        let organizer_id = UserId::from(cmd.organizer_id);

        let mut event = Event::new(
            event_id,
            organizer_id,
            cmd.name,
            cmd.description,
            cmd.location,
            cmd.start_date,
            cmd.end_date,
            cmd.max_capacity,
        )?;

        self.event_repo.save(&mut event).await?;

        Ok(EventDto {
            id: event.id.into_inner(),
            organizer_id: event.organizer_id.into_inner(),
            name: event.name,
            description: event.description,
            location: event.location,
            start_date: event.start_date,
            end_date: event.end_date,
            max_capacity: event.max_capacity,
            status: format!("{}", event.status),
            categories: vec![],
            created_at: event.created_at,
            updated_at: event.updated_at,
        })
    }

    pub async fn handle_publish_event(&self, cmd: PublishEventCommand) -> AppResult<EventDto> {
        let event_id = EventId::from(cmd.event_id);
        let mut event = self
            .event_repo
            .find_by_id(event_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Event not found".to_string()))?;

        event.publish()?;
        self.event_repo.save(&mut event).await?;

        Ok(self.event_to_dto(&event))
    }

    pub async fn handle_cancel_event(&self, cmd: CancelEventCommand) -> AppResult<EventDto> {
        let event_id = EventId::from(cmd.event_id);
        let mut event = self
            .event_repo
            .find_by_id(event_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Event not found".to_string()))?;

        event.cancel()?;
        self.event_repo.save(&mut event).await?;

        // Note: In a complete implementation, we would:
        // 1. Find all paid bookings for this event using BookingRepository.find_by_event_id()
        // 2. Mark each booking as requiring refund
        // 3. Cancel all tickets associated with those bookings
        // 4. Send event cancellation notifications to all affected customers
        // This requires additional repository methods that would be implemented in the infrastructure layer

        Ok(self.event_to_dto(&event))
    }

    pub async fn handle_add_ticket_category(
        &self,
        cmd: AddTicketCategoryCommand,
    ) -> AppResult<EventDto> {
        let event_id = EventId::from(cmd.event_id);
        let mut event = self
            .event_repo
            .find_by_id(event_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Event not found".to_string()))?;

        let category_id = CategoryId::new();
        let price = Money::new(cmd.price, cmd.currency)?;

        event.add_ticket_category(
            category_id,
            cmd.name,
            price,
            cmd.quota,
            cmd.sales_start,
            cmd.sales_end,
        )?;

        self.event_repo.save(&mut event).await?;

        Ok(self.event_to_dto(&event))
    }

    pub async fn handle_disable_ticket_category(
        &self,
        cmd: DisableTicketCategoryCommand,
    ) -> AppResult<EventDto> {
        let event_id = EventId::from(cmd.event_id);
        let category_id = CategoryId::from(cmd.category_id);
        
        let mut event = self
            .event_repo
            .find_by_id(event_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Event not found".to_string()))?;

        event.disable_category(&category_id)?;
        self.event_repo.save(&mut event).await?;

        Ok(self.event_to_dto(&event))
    }

    fn event_to_dto(&self, event: &Event) -> EventDto {
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
                .map(|c| crate::dto::TicketCategoryDto {
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
                })
                .collect(),
            created_at: event.created_at,
            updated_at: event.updated_at,
        }
    }
}
