DEFINE TABLE IF NOT EXISTS users SCHEMAFULL;

DEFINE FIELD IF NOT EXISTS password_hash ON users TYPE string
	ASSERT string::len($value) > 0;
DEFINE FIELD IF NOT EXISTS role ON users TYPE string
	ASSERT $value IN ['app_admin', 'mosque_admin', 'educator', 'regular']
	DEFAULT 'regular';
DEFINE FIELD IF NOT EXISTS display_name ON users TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON users TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS updated_at ON users TYPE datetime DEFAULT time::now();
