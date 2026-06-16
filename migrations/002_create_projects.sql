CREATE TYPE project_status AS ENUM ('pending', 'verified', 'suspended');

CREATE TABLE projects (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id    UUID NOT NULL REFERENCES users(id),
    registry    TEXT NOT NULL,
    cert_id     TEXT NOT NULL,
    project_name TEXT NOT NULL,
    project_type TEXT,
    location    TEXT,
    vintage_year INTEGER,
    status      project_status NOT NULL DEFAULT 'pending',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(registry, cert_id)
);

CREATE INDEX idx_projects_owner ON projects (owner_id);
CREATE INDEX idx_projects_status ON projects (status);
CREATE INDEX idx_projects_registry ON projects (registry);
