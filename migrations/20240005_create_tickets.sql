-- Tickets table: entities created when a booking is paid
CREATE TABLE tickets (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  booking_id    UUID NOT NULL REFERENCES bookings(id),
  event_id      UUID NOT NULL REFERENCES events(id),
  code          TEXT NOT NULL UNIQUE,
  status        ticket_status NOT NULL DEFAULT 'active',
  checked_in_at TIMESTAMPTZ,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX tickets_booking_id_idx ON tickets (booking_id);
CREATE INDEX tickets_event_id_idx ON tickets (event_id);
