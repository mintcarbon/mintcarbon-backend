CREATE TABLE token_holdings (
    user_id    UUID NOT NULL REFERENCES users(id),
    token_id   TEXT NOT NULL,
    quantity   BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, token_id)
);

CREATE INDEX idx_token_holdings_token ON token_holdings (token_id);
