DEFINE TABLE IF NOT EXISTS prayer_times SCHEMAFULL;

DEFINE FIELD IF NOT EXISTS fajr ON TABLE prayer_times TYPE string;
DEFINE FIELD IF NOT EXISTS dhuhr ON TABLE prayer_times TYPE string;
DEFINE FIELD IF NOT EXISTS asr ON TABLE prayer_times TYPE string;
DEFINE FIELD IF NOT EXISTS maghrib ON TABLE prayer_times TYPE string;
DEFINE FIELD IF NOT EXISTS isha ON TABLE prayer_times TYPE string;
DEFINE FIELD IF NOT EXISTS jummah ON TABLE prayer_times TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON TABLE prayer_times TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS updated_at ON TABLE prayer_times TYPE datetime DEFAULT time::now();
