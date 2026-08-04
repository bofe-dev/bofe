CREATE TYPE report_target AS ENUM ('board', 'card', 'user');

CREATE TABLE reports (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL,
    ip_address varchar(255) NOT NULL,
    target report_target NOT NULL,
    target_id uuid NOT NULL,
    data jsonb NOT NULL,
    message text NOT NULL,
    reviewed_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_reports PRIMARY KEY (id),
    CONSTRAINT fkey_reports_to_users FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

SELECT manage_updated_at('reports');
