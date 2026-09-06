#!/bin/bash
set -euo pipefail

# ============================================================
# login.sh - Login user and return session token
# ============================================================
# Usage:
#   ./scripts/login.sh                    # Uses saved credentials from .auth_state
#   ./scripts/login.sh -m "+1234567890" -p "password"  # Explicit credentials
#
# Logs in via the /auth/login endpoint with mobile platform.
# Returns the session token.
#
# Environment variables:
#   MERZAH_BASE_URL - Base URL of the server (default: http://127.0.0.1:3000)
# ============================================================

BASE_URL="${MERZAH_BASE_URL:-http://127.0.0.1:3000}"
ENDPOINT="/auth/login"
URL="${BASE_URL}${ENDPOINT}"
STATE_FILE="$(dirname "$0")/.auth_state"

MOBILE=""
PASSWORD=""

# Parse arguments
while getopts "m:p:" opt; do
    case $opt in
        m) MOBILE="$OPTARG" ;;
        p) PASSWORD="$OPTARG" ;;
        *) echo "Usage: $0 [-m mobile] [-p password]"; exit 1 ;;
    esac
done

# Load saved credentials if not provided
if [ -z "$MOBILE" ] || [ -z "$PASSWORD" ]; then
    if [ -f "$STATE_FILE" ]; then
        # shellcheck source=/dev/null
        source "$STATE_FILE"
        echo "Using saved credentials from $STATE_FILE"
    else
        echo "ERROR: No credentials provided and no saved state found."
        echo "Usage: $0 -m '+1234567890' -p 'password'"
        echo "Or run register.sh first to create a user."
        exit 1
    fi
fi

echo "===================================================="
echo "Logging in"
echo "===================================================="
echo "  Mobile: $MOBILE"
echo "===================================================="

RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$URL" \
    -H "Content-Type: application/json" \
    -d "{
        \"form\": {
            \"identifier\": {\"identifier_type\": \"mobile\", \"identifier_value\": \"$MOBILE\"},
            \"password\": \"$PASSWORD\",
            \"platform\": \"mobile\"
        }
    }")

HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | sed '$d')

echo ""
echo "Response (HTTP $HTTP_CODE):"
echo "$BODY" | head -c 500
echo ""

if [ "$HTTP_CODE" -eq 200 ]; then
    TOKEN=$(echo "$BODY" | grep -o '"data":"[^"]*"' | head -1 | sed 's/"data":"//;s/"$//')

    if [ -n "$TOKEN" ] && [ "$TOKEN" != "null" ]; then
        echo ""
        echo "SUCCESS: Logged in!"
        echo "Session Token: $TOKEN"

        # Update auth state
        cat > "$STATE_FILE" << EOF
TOKEN=$TOKEN
MOBILE=$MOBILE
PASSWORD=$PASSWORD
EOF
        echo "Auth state saved to $STATE_FILE"
    else
        echo "ERROR: Could not extract session token from response"
        exit 1
    fi
else
    echo "ERROR: Login failed with HTTP $HTTP_CODE"
    echo "Response: $BODY"
    exit 1
fi
