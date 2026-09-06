#!/bin/bash
set -euo pipefail

# ============================================================
# populate_events.sh - Populate events for existing mosques
# ============================================================
# Usage:
#   ./scripts/populate_events.sh
#   ./scripts/populate_events.sh -t "session_token"
#
# Creates sample events for mosques already in the database.
# Requires authentication (any authenticated user can create events).
#
# Events are created for "test mosques" that should already exist
# in the database (from populate_mosques.sh or other means).
#
# Environment variables:
#   MERZAH_BASE_URL  - Base URL of the server (default: http://127.0.0.1:3000)
#   MERZAH_TOKEN     - Session token (or loaded from .auth_state)
# ============================================================

BASE_URL="${MERZAH_BASE_URL:-http://127.0.0.1:3000}"
ENDPOINT="/mosques/events/add-event"
URL="${BASE_URL}${ENDPOINT}"
STATE_FILE="$(dirname "$0")/.auth_state"
TOKEN="${MERZAH_TOKEN:-}"

# Parse arguments
while getopts "t:" opt; do
    case $opt in
        t) TOKEN="$OPTARG" ;;
        *) echo "Usage: $0 [-t token]"; exit 1 ;;
    esac
done

# Load saved token if not provided
if [ -z "$TOKEN" ]; then
    if [ -f "$STATE_FILE" ]; then
        # shellcheck source=/dev/null
        source "$STATE_FILE"
        echo "Using saved token from $STATE_FILE"
    fi
fi

if [ -z "${TOKEN:-}" ]; then
    echo "ERROR: No token available. Please login first."
    exit 1
fi

# ============================================================
# Helper function to add an event
# ============================================================
add_event() {
    local title="$1"
    local description="$2"
    local category="$3"
    local date="$4"
    local mosque_id="$5"
    local speaker="${6:-}"

    local speaker_field=""
    if [ -n "$speaker" ]; then
        speaker_field="\"speaker\": \"$speaker\","
    fi

    echo "  Adding: $title ($category)"

    local response
    response=$(curl -s -w "\n%{http_code}" -X POST "$URL" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $TOKEN" \
        -d "{
            \"create_event\": {
                \"title\": \"$title\",
                \"description\": \"$description\",
                \"category\": \"$category\",
                \"date\": \"$date\",
                \"mosque\": \"$mosque_id\",
                $speaker_field
                \"recurrence_pattern\": null,
                \"recurrence_duration\": null
            }
        }")

    local http_code
    http_code=$(echo "$response" | tail -n1)
    local body
    body=$(echo "$response" | sed '$d')

    if [ "$http_code" -eq 200 ] || [ "$http_code" -eq 201 ]; then
        if echo "$body" | grep -q '"error":null' || ( ! echo "$body" | grep -q '"error":' && echo "$body" | grep -q '"data":' ); then
            echo "    OK"
            return 0
        else
            echo "    FAILED: $body"
            return 1
        fi
    else
        echo "    FAILED: HTTP $http_code - $body"
        return 1
    fi
}

echo "===================================================="
echo "Populating events"
echo "===================================================="
echo ""

# ============================================================
# NOTE: You must replace these mosque IDs with actual IDs
# from your database. Run a query like:
#   SELECT id, name FROM mosques;
# to find the mosque IDs.
#
# Format: mosques:<record_id>
# Example: mosques:4h3k7j8m9n0p
# ============================================================

# Placeholder mosque IDs - REPLACE THESE with actual IDs
MOSQUE_1="${MOSQUE_1_ID:-mosques:placeholder1}"
MOSQUE_2="${MOSQUE_2_ID:-mosques:placeholder2}"
MOSQUE_3="${MOSQUE_3_ID:-mosques:placeholder3}"

# Generate dates relative to now
TODAY=$(date -u +"%Y-%m-%d")
TOMORROW=$(date -u -d "+1 day" +"%Y-%m-%d" 2>/dev/null || date -u -v+1d +"%Y-%m-%d" 2>/dev/null || echo "$TODAY")
NEXT_FRIDAY=$(date -u -d "+$(( (5 - $(date -u +%u)) % 7 )) days" +"%Y-%m-%d" 2>/dev/null || echo "$TOMORROW")
NEXT_SATURDAY=$(date -u -d "+$(( (6 - $(date -u +%u)) % 7 )) days" +"%Y-%m-%d" 2>/dev/null || echo "$TOMORROW")
NEXT_SUNDAY=$(date -u -d "+$(( (7 - $(date -u +%u)) % 7 )) days" +"%Y-%m-%d" 2>/dev/null || echo "$TOMORROW")

echo "Creating events..."
echo ""

# Featured Events
add_event \
    "Jumu'ah: The Prophetic Character" \
    "Join us for this week's Jumu'ah khutbah focusing on the noble character of Prophet Muhammad and how we can embody these teachings in our daily lives. Sheikh Ahmed will lead the congregation." \
    "lecture" \
    "${NEXT_FRIDAY}T13:00:00+00:00" \
    "$MOSQUE_1" \
    "Sheikh Ahmed"

add_event \
    "Eid al-Adha Celebration" \
    "Community-wide Eid prayers followed by breakfast and activities for the whole family. Children's activities, communal meal, and opportunities to connect with fellow community members." \
    "eid" \
    "${NEXT_SATURDAY}T08:00:00+00:00" \
    "$MOSQUE_2"

# All Events
add_event \
    "Understanding Surah Al-Kahf" \
    "Deep dive into the stories and lessons of Surah Al-Kahf with Sheikh Omar. Weekly series covering practical reflections and timeless guidance." \
    "halaqah" \
    "${NEXT_SATURDAY}T10:00:00+00:00" \
    "$MOSQUE_1" \
    "Sheikh Omar"

add_event \
    "Resume Building Workshop" \
    "Professional career workshop helping young professionals craft compelling resumes and prepare for upcoming interviews." \
    "workshop" \
    "${NEXT_SUNDAY}T14:00:00+00:00" \
    "$MOSQUE_3"

add_event \
    "Youth Halaqah: Living Faith" \
    "Interactive discussion circle for young adults exploring how to live faith authentically in everyday life." \
    "halaqah" \
    "${NEXT_SUNDAY}T18:30:00+00:00" \
    "$MOSQUE_2"

add_event \
    "Sisters' Study Circle" \
    "Monthly gathering for sisters to study Islamic history, discuss contemporary issues, and build community." \
    "halaqah" \
    "${TOMORROW}T19:30:00+00:00" \
    "$MOSQUE_1"

add_event \
    "Food Drive Volunteer Day" \
    "Help organize and distribute food packages to families in need. All ages welcome. Light refreshments provided." \
    "volunteer" \
    "${NEXT_SATURDAY}T09:00:00+00:00" \
    "$MOSQUE_3"

add_event \
    "Family Game Night" \
    "Bring the whole family for an evening of board games, activities, and fellowship. Dinner and snacks included." \
    "community" \
    "${NEXT_FRIDAY}T18:00:00+00:00" \
    "$MOSQUE_2"

echo ""
echo "===================================================="
echo "Events populated successfully!"
echo "===================================================="
echo ""
echo "NOTE: If you see placeholder mosque IDs, set these environment variables:"
echo "  MOSQUE_1_ID=mosques:your_actual_id"
echo "  MOSQUE_2_ID=mosques:your_actual_id"
echo "  MOSQUE_3_ID=mosques:your_actual_id"
echo ""
echo "To find mosque IDs, query your database:"
echo "  surreal sql --endpoint ws://127.0.0.1:5100 --username root --password root \\"
echo "    --database merzah --namespace merzah --query 'SELECT id, name FROM mosques'"
