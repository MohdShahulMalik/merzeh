DEFINE TABLE IF NOT EXISTS user_identifier SCHEMAFULL;

DEFINE FIELD IF NOT EXISTS user_id ON user_identifier TYPE record<users>;
DEFINE FIELD IF NOT EXISTS identifier_type ON user_identifier TYPE string ASSERT $value IN ['email', 'mobile'];
DEFINE FIELD IF NOT EXISTS identifier_value ON user_identifier TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON user_identifier TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS updated_at ON user_identifier TYPE datetime DEFAULT time::now();

-- Allow only one user per identifier_value globally
DEFINE INDEX IF NOT EXISTS idx_identifier_value ON TABLE user_identifier COLUMNS identifier_value UNIQUE;

-- Only one identifier of each type per user
DEFINE INDEX IF NOT EXISTS idx_user_identifier_type ON TABLE user_identifier COLUMNS user_id, identifier_type UNIQUE;
