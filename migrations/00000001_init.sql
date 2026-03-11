-- if not exists create the nessicary tables

CREATE TABLE IF NOT EXISTS organisation (
    id   TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    default_auth      JSONB NOT NULL   -- ApiDispatcherConfig stored as JSON
);

CREATE TABLE IF NOT EXISTS organisation_users (
    organization_id   TEXT NOT NULL,
    user_id           TEXT NOT NULL PRIMARY KEY,
    role              TEXT,
    user_email        TEXT NOT NULL,
    name              TEXT NOT NULL,

    FOREIGN KEY (organization_id) REFERENCES organisation(id) ON DELETE CASCADE
);



CREATE TABLE IF NOT EXISTS pipeline_configs (
    organization_id   TEXT NOT NULL,
    customer_company_id TEXT NOT NULL,
    name              TEXT NOT NULL,
    image             TEXT,
    cron              TEXT NOT NULL,
    last_edited       TEXT NOT NULL DEFAULT '',
    pipeline          JSONB NOT NULL,  -- Manifest stored as JSON
    PRIMARY KEY (organization_id, customer_company_id),

    FOREIGN KEY (organization_id) REFERENCES organisation(id) ON DELETE CASCADE    
);