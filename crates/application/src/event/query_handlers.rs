use std::sync::Arc;

use domain::event::{
    aggregate::Event,
    repository::{EventFilter, EventRepository},
    value_objects::EventId,
};

use crate::dto::EventDto;
use crate::errors::{AppResult, ApplicationError};

use super::queries::{GetEventQuery, GetPublishedEventsQuery};

pub struct EventQueryHandler<R: EventRepository> {
    event_repo: Arc<R>,
}

impl<R: EventRepository> EventQueryHandler<R> {
    pub fn new(event_repo: Arc<R>) -> Self {
        Self { event_repo }
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
