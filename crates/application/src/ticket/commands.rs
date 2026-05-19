use uuid::Uuid;

pub struct CheckInTicketCommand {
    pub ticket_code: String,
    pub event_id: Uuid,
}
