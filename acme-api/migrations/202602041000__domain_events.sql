-- Domain events/outbox table for reliable event delivery.
--
-- Events are written within business transactions and processed
-- asynchronously by the outbox processor.

CREATE TABLE IF NOT EXISTS platform.domain_events (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  event_type text NOT NULL,
  payload jsonb NOT NULL,
  occurred_at timestamptz NOT NULL DEFAULT NOW(),
  processed_at timestamptz NULL
);

-- Index for efficient unprocessed event queries
CREATE INDEX IF NOT EXISTS domain_events_unprocessed_idx
  ON platform.domain_events (occurred_at)
  WHERE processed_at IS NULL;

-- LISTEN/NOTIFY trigger for efficient outbox processor wake-up
CREATE OR REPLACE FUNCTION platform.notify_domain_event_inserted()
RETURNS trigger AS $$
BEGIN
    PERFORM pg_notify('underlay_domain_event_notify', NEW.event_type);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS domain_event_inserted ON platform.domain_events;
CREATE TRIGGER domain_event_inserted
    AFTER INSERT ON platform.domain_events
    FOR EACH ROW
    EXECUTE FUNCTION platform.notify_domain_event_inserted();
