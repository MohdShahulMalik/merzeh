DEFINE TABLE sessions SCHEMAFULL;

DEFINE FIELD user_id ON sessions TYPE record<users>;
DEFINE FIELD session_token ON sessions TYPE string;
DEFINE FIELD created_at ON sessions TYPE datetime DEFAULT time::now();
DEFINE FIELD expires_at ON sessions TYPE datetime;

-- Unique Session Tokens
DEFINE INDEX idx_session_token ON TABLE sessions COLUMNS session_token UNIQUE;
