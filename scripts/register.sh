#!/bin/bash
set -euo pipefail

# ============================================================
# register.sh - Register a random user and return session token
# ============================================================
# Usage: ./scripts/register.sh
#
# Generates random credentials (name, email, mobile, password)
# and registers a user via the /auth/register endpoint.
# Returns the session token for mobile platform.
#
# Saves credentials and token to scripts/.auth_state for
# other scripts to use.
#
# Environment variables:
#   MERZAH_BASE_URL - Base URL of the server (default: http://127.0.0.1:3000)
# ============================================================

BASE_URL="${MERZAH_BASE_URL:-http://127.0.0.1:3000}"
ENDPOINT="/auth/register"
URL="${BASE_URL}${ENDPOINT}"
STATE_FILE="$(dirname "$0")/.auth_state"

# Generate random alphanumeric string
random_string() {
    local length="${1:-8}"
    tr -dc 'a-z0-9' < /dev/urandom | head -c "$length"
}

# Generate random digits
random_digits() {
    local length="${1:-10}"
    tr -dc '0-9' < /dev/urandom | head -c "$length"
}

# Generate random name (capitalize first letter)
generate_name() {
    local suffix
    suffix=$(random_string 6)
    echo "User_${suffix}"
}

# Generate random email
generate_email() {
    local suffix
    suffix=$(random_string 8)
    echo "user_${suffix}@test.com"
}

# Generate random mobile number (E.164 format)
generate_mobile() {
    local digits
    digits=$(random_digits 10)
    echo "+1${digits}"
}

# Generate random password (meets min 8 char requirement)
generate_password() {
    local pass
    pass="Pass_$(random_string 8)!"
    echo "$pass"
}

# Generate credentials
USERNAME=$(generate_name)
EMAIL=$(generate_email)
MOBILE=$(generate_mobile)
PASSWORD=$(generate_password)

echo "===================================================="
echo "Registering new user"
echo "===================================================="
echo "  Name:     $USERNAME"
echo "  Email:    $EMAIL"
echo "  Mobile:   $MOBILE"
echo "  Password: $PASSWORD"
echo "===================================================="

# Use mobile identifier for mobile platform
# Leptos server functions wrap params in an object keyed by the parameter name
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$URL" \
    -H "Content-Type: application/json" \
    -d "{
        \"form\": {
            \"name\": \"$USERNAME\",
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
    # Extract session token from ApiResponse.data
    TOKEN=$(echo "$BODY" | grep -o '"data":"[^"]*"' | head -1 | sed 's/"data":"//;s/"$//')

    if [ -n "$TOKEN" ] && [ "$TOKEN" != "null" ]; then
        echo ""
        echo "SUCCESS: User registered!"
        echo "Session Token: $TOKEN"

        # Save auth state for other scripts
        cat > "$STATE_FILE" << EOF
TOKEN=$TOKEN
USERNAME=$USERNAME
EMAIL=$EMAIL
MOBILE=$MOBILE
PASSWORD=$PASSWORD
EOF
        echo "Auth state saved to $STATE_FILE"
    else
        echo "ERROR: Could not extract session token from response"
        echo "Full response: $BODY"
        exit 1
    fi
else
    echo "ERROR: Registration failed with HTTP $HTTP_CODE"
    echo "Response: $BODY"
    exit 1
fi
