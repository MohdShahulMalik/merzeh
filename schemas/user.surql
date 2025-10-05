DEFINE TABLE user SCHEMAFUL;

DEFINE FIELD password_hash ON users TYPE string;
DEFINE FIELD role ON users TYPE string ASSERT $value IN ['app_admin', 'mosque_admin', 'educator', 'regular'];
DEFINE FIELD display_name ON users TYPE string;
DEFINE FIELD created_at ON users TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON users TYPE datetime DEFAULT time::now();
