/// Marker trait for all domain events.
/// Domain events represent something meaningful that happened in the domain.
pub trait DomainEvent: Send + Sync + std::fmt::Debug {
    fn event_name(&self) -> &'static str;
}

/// Type-erased boxed domain event for collection in aggregates.
pub type DomainEventBox = Box<dyn DomainEvent>;
