DEFINE TABLE IF NOT EXISTS sessions SCHEMAFULL;

DEFINE FIELD IF NOT EXISTS user_id ON sessions TYPE record<users>;
DEFINE FIELD IF NOT EXISTS session_token ON sessions TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON sessions TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS expires_at ON sessions TYPE datetime;

-- Unique Session Tokens
DEFINE INDEX IF NOT EXISTS idx_session_token ON TABLE sessions COLUMNS session_token UNIQUE;
