DEFINE TABLE IF NOT EXISTS mosques SCHEMAFULL;

DEFINE FIELD IF NOT EXISTS name ON mosques TYPE string;
DEFINE FIELD IF NOT EXISTS location ON mosques TYPE geometry<point>;
DEFINE FIELD IF NOT EXISTS street ON mosques TYPE string;
DEFINE FIELD IF NOT EXISTS city ON mosques TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON mosques TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS updated_at ON mosques TYPE datetime DEFAULT time::now();

DEFINE INDEX IF NOT EXISTS mosque_location_idx ON TABLE mosques COLUMNS location MTREE DIMENSION 2;
DEFINE INDEX IF NOT EXISTS idx_mosque_name ON TABLE mosques COLUMNS name;
DEFINE INDEX IF NOT EXISTS idx_mosque_city ON TABLE mosques COLUMNS city;
