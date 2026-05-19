use uuid::Uuid;

pub struct GetTicketQuery {
    pub ticket_id: Uuid,
}

pub struct GetTicketByCodeQuery {
    pub ticket_code: String,
}
