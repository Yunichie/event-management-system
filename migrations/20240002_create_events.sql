CREATE TABLE events (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  organizer_id  UUID NOT NULL,
  name          TEXT NOT NULL,
  description   TEXT NOT NULL DEFAULT '',
  start_date    DATE NOT NULL,
  end_date      DATE NOT NULL,
  location      TEXT NOT NULL,
  max_capacity  INT  NOT NULL CHECK (max_capacity > 0),
  status        event_status NOT NULL DEFAULT 'draft',
  created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE INDEX events_status_dates_idx ON events (status, start_date, end_date);
CREATE INDEX events_location_trgm_idx ON events USING GIN (location gin_trgm_ops);
CREATE INDEX events_organizer_id_idx ON events (organizer_id);
