CREATE TABLE audit_events (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type      TEXT NOT NULL,
    actor_id        UUID REFERENCES users(id),
    payload         JSONB NOT NULL DEFAULT '{}',
    on_chain_index  BIGINT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_audit_events_type ON audit_events (event_type);
CREATE INDEX idx_audit_events_actor ON audit_events (actor_id);
CREATE INDEX idx_audit_events_created ON audit_events (created_at);
