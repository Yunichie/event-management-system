use uuid::Uuid;

pub struct GetEventQuery {
    pub event_id: Uuid,
}

pub struct GetPublishedEventsQuery {
    pub date: Option<chrono::NaiveDate>,
    pub location: Option<String>,
}
