#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DATA_DIR="$PROJECT_ROOT/scripts/data"
EDUCATORS_FILE="$DATA_DIR/educators.json"

SURREAL_USER=$(grep '^SURREAL_USER' "$PROJECT_ROOT/.env" | sed 's/.*= *//')
SURREAL_PASS=$(grep '^SURREAL_PASS' "$PROJECT_ROOT/.env" | sed 's/.*= *//')
SURREAL_URL=$(grep '^SURREAL_URL' "$PROJECT_ROOT/.env" | sed 's/.*= *//')
SURREAL_NS=$(grep '^SURREAL_NS' "$PROJECT_ROOT/.env" | sed 's/.*= *//')
SURREAL_DB=$(grep '^SURREAL_DB' "$PROJECT_ROOT/.env" | sed 's/.*= *//')

echo "===================================================="
echo "Creating Educator Records"
echo "===================================================="
echo ""

EDUCATORS=(
    "SeekersGuidance"
    "Shaykh Faraz Rabbani"
    "Ustadh Abdullah Misra"
    "Shaykh Yahya Rhodus"
    "Shaykh Abdul-Rahim Reasat"
    "Ustadha Shireen Ahmed"
    "Quran.com"
    "Sunnah.com"
    "Imam al-Bukhari"
    "Imam Muslim"
    "Imam an-Nawawi"
    "Imam Malik"
    "Imam at-Tirmidhi"
    "Imam Ibn Majah"
    "Imam an-Nasa'i"
    "Imam Ahmad"
)

EDUCATOR_IDS_FILE="$DATA_DIR/educator_ids.json"
echo "[" > "$EDUCATOR_IDS_FILE"
FIRST=true

for name in "${EDUCATORS[@]}"; do
    echo "  Creating: $name"

    response=$(curl -s -X POST "http://${SURREAL_URL}/sql" \
        -u "${SURREAL_USER}:${SURREAL_PASS}" \
        -H "surreal-ns: ${SURREAL_NS}" \
        -H "surreal-db: ${SURREAL_DB}" \
        -H "Accept: application/json" \
        -d "CREATE users SET display_name = '${name}', password_hash = 'external', role = 'educator', created_at = time::now(), updated_at = time::now();")

    id=$(echo "$response" | python3 -c "import sys,json; data=json.load(sys.stdin); print(data[0]['result'][0]['id'])" 2>/dev/null || echo "")
    echo "    ID: $id"

    if [ -n "$id" ]; then
        if [ "$FIRST" = true ]; then
            FIRST=false
        else
            echo "," >> "$EDUCATOR_IDS_FILE"
        fi
        printf '  {"name": "%s", "id": "%s"}' "$name" "$id" >> "$EDUCATOR_IDS_FILE"
    fi
done

echo "" >> "$EDUCATOR_IDS_FILE"
echo "]" >> "$EDUCATOR_IDS_FILE"

echo ""
echo "===================================================="
echo "Educator creation complete!"
echo "===================================================="
echo ""
echo "Educator IDs saved to: $EDUCATOR_IDS_FILE"
cat "$EDUCATOR_IDS_FILE"
