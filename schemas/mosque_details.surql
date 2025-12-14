DEFINE TABLE IF NOT EXISTS mosque_details SCHEMAFULL;

-- The mosque these details belong to
DEFINE FIELD IF NOT EXISTS mosque ON TABLE mosque_details TYPE record<mosques> ASSERT $value != NONE;

-- Array of users who are admins for this mosque
DEFINE FIELD IF NOT EXISTS admins ON TABLE mosque_details TYPE array<record<users>>;

-- Reference to congregational prayer times record
DEFINE FIELD IF NOT EXISTS jamat_times ON TABLE mosque_details TYPE record<prayer_times>;

-- Reference to call to prayer times record
DEFINE FIELD IF NOT EXISTS adhan_times ON TABLE mosque_details TYPE record<prayer_times>;

-- A mosque should have only one details entry
DEFINE INDEX IF NOT EXISTS idx_mosque_unique ON mosque_details FIELDS mosque UNIQUE;

-- Index for faster lookups by mosque
DEFINE INDEX IF NOT EXISTS idx_mosque_lookup ON mosque_details FIELDS mosque;
