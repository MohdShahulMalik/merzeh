DEFINE TABLE IF NOT EXISTS mosque_details SCHEMAFULL;

-- The mosque these details belong to
DEFINE FIELD IF NOT EXISTS mosque ON TABLE mosque_details TYPE record<mosques> ASSERT $value != NONE;

-- Array of users who are admins for this mosque
DEFINE FIELD IF NOT EXISTS admins ON TABLE mosque_details TYPE array;
DEFINE FIELD IF NOT EXISTS admins.* ON TABLE mosque_details TYPE record<users>;

-- Congregational prayer times (object with time fields)
DEFINE FIELD IF NOT EXISTS jamat_time ON TABLE mosque_details TYPE object;
DEFINE FIELD IF NOT EXISTS jamat_time.fajr ON TABLE mosque_details TYPE time;
DEFINE FIELD IF NOT EXISTS jamat_time.dhuhr ON TABLE mosque_details TYPE time;
DEFINE FIELD IF NOT EXISTS jamat_time.asr ON TABLE mosque_details TYPE time;
DEFINE FIELD IF NOT EXISTS jamat_time.maghrib ON TABLE mosque_details TYPE time;
DEFINE FIELD IF NOT EXISTS jamat_time.isha ON TABLE mosque_details TYPE time;
DEFINE FIELD IF NOT EXISTS jamat_time.jummah ON TABLE mosque_details TYPE time;  -- Optional

-- Call to prayer times (object with time fields)
DEFINE FIELD IF NOT EXISTS adhan_time ON TABLE mosque_details TYPE object;
DEFINE FIELD IF NOT EXISTS adhan_time.fajr ON TABLE mosque_details TYPE time;
DEFINE FIELD IF NOT EXISTS adhan_time.dhuhr ON TABLE mosque_details TYPE time;
DEFINE FIELD IF NOT EXISTS adhan_time.asr ON TABLE mosque_details TYPE time;
DEFINE FIELD IF NOT EXISTS adhan_time.maghrib ON TABLE mosque_details TYPE time;
DEFINE FIELD IF NOT EXISTS adhan_time.isha ON TABLE mosque_details TYPE time;
DEFINE FIELD IF NOT EXISTS adhan_time.jummah ON TABLE mosque_details TYPE time;  -- Optional

-- A mosque should have only one details entry
DEFINE INDEX IF NOT EXISTS idx_mosque_unique ON mosque_details FIELDS mosque UNIQUE;

-- Index for faster lookups by mosque
DEFINE INDEX IF NOT EXISTS idx_mosque_lookup ON mosque_details FIELDS mosque;
