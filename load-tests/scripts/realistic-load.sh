#!/bin/bash
################################################################################
# KoproGo Realistic Load Test - Mixed POST/GET Scenarios
# Simulates real user behavior with CRUD operations
################################################################################

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="${SCRIPT_DIR}/../results"
BASE_URL="${BASE_URL:-http://localhost:8080}"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo "========================================="
echo -e "${BLUE}🎯 KoproGo REALISTIC Load Test${NC}"
echo "========================================="
echo "Base URL: $BASE_URL"
echo "Duration: 3 minutes"
echo "Scenario: Mixed 70% GET / 30% POST (realistic ratio)"
echo ""
echo "Test parameters:"
echo "- Threads: 4"
echo "- Connections: 20"
echo "- Duration: 3 minutes"
echo ""
echo "Starting test..."
echo ""

# Create results directory
mkdir -p "$RESULTS_DIR"

# Test login endpoint to get token and organization_id
AUTH_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
    -H "Content-Type: application/json" \
    -d '{"email":"admin@small.be","password":"admin123"}')

TOKEN=$(echo "$AUTH_RESPONSE" | jq -r '.token')
ORG_ID=$(echo "$AUTH_RESPONSE" | jq -r '.user.organization_id')

if [ -z "$TOKEN" ] || [ "$TOKEN" == "null" ]; then
    echo "❌ Authentication: FAILED - No token received"
    exit 1
fi

if [ -z "$ORG_ID" ] || [ "$ORG_ID" == "null" ]; then
    echo "❌ Authentication: FAILED - No organization_id received"
    exit 1
fi

echo "✅ JWT token acquired: ${TOKEN:0:30}..."
echo "✅ Organization ID: $ORG_ID"

# Fetch 10 real building IDs to use in POST requests
echo "🔍 Fetching real building IDs..."
BUILDINGS_JSON=$(curl -s -X GET "$BASE_URL/api/v1/buildings" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json")

# Extract up to 10 building IDs
BUILDING_IDS=$(echo "$BUILDINGS_JSON" | jq -r '.data[0:10][].id' | tr '\n' ',' | sed 's/,$//')

if [ -z "$BUILDING_IDS" ] || [ "$BUILDING_IDS" == "null" ]; then
    echo "⚠️  No existing buildings found - POST operations for units/expenses will be limited"
    BUILDING_IDS="00000000-0000-0000-0000-000000000001"
fi

echo "✅ Found building IDs: ${BUILDING_IDS:0:80}..."

# Create wrk2 Lua script for realistic mixed workload with dynamic ORG_ID and real building IDs
cat > /tmp/realistic_workload.lua << LUA_SCRIPT
-- Realistic workload: 70% reads (GET), 30% writes (POST/PUT)
math.randomseed(os.time())

-- Track user's organization context
local org_id = nil
local unit_ids = {}
local owner_ids = {}

-- Real building IDs fetched from API (comma-separated string converted to Lua table)
local building_ids_str = "$BUILDING_IDS"
local building_ids = {}
for id in string.gmatch(building_ids_str, "[^,]+") do
    table.insert(building_ids, id)
end

-- Helper function to get a random building ID from real buildings
function get_random_building_id()
    if #building_ids > 0 then
        return building_ids[math.random(#building_ids)]
    else
        return "00000000-0000-0000-0000-000000000001"  -- Fallback
    end
end

-- API endpoints with realistic distribution
local read_operations = {
    { method = "GET", path = "/api/v1/buildings", weight = 30 },
    { method = "GET", path = "/api/v1/units", weight = 25 },
    { method = "GET", path = "/api/v1/owners", weight = 20 },
    { method = "GET", path = "/api/v1/expenses", weight = 20 },
    { method = "GET", path = "/api/v1/auth/me", weight = 5 },
}

local write_operations = {
    { method = "POST", path = "/api/v1/buildings", weight = 30 },
    { method = "POST", path = "/api/v1/units", weight = 30 },
    { method = "POST", path = "/api/v1/owners", weight = 20 },
    { method = "POST", path = "/api/v1/expenses", weight = 20 },
}

-- Generate sample data for POST requests
function generate_building_data()
    local streets = {"Rue de la Paix", "Avenue Louise", "Boulevard du Roi", "Place du Marché"}
    local cities = {"Bruxelles", "Anvers", "Gand", "Liège"}
    return string.format([[{
        "organization_id": "$ORG_ID",
        "name": "Résidence Test %d",
        "address": "%s %d",
        "city": "%s",
        "postal_code": "%d",
        "country": "Belgium",
        "total_units": %d,
        "construction_year": %d
    }]],
        math.random(1000, 9999),
        streets[math.random(#streets)],
        math.random(1, 200),
        cities[math.random(#cities)],
        math.random(1000, 9999),
        math.random(10, 50),
        math.random(1960, 2024)
    )
end

function generate_unit_data(building_id)
    -- Use lowercase for unit_type ENUM (apartment, parking, cellar, commercial, other)
    local types = {"apartment", "apartment", "apartment", "parking", "cellar"}
    -- If no building_id provided, get a random real one
    local bid = building_id or get_random_building_id()
    return string.format([[{
        "organization_id": "$ORG_ID",
        "building_id": "%s",
        "unit_number": "%d.%d",
        "unit_type": "%s",
        "floor": %d,
        "surface_area": %.2f,
        "quota": %d
    }]],
        bid,
        math.random(0, 10),
        math.random(1, 4),
        types[math.random(#types)],
        math.random(0, 10),
        math.random(45, 150) + math.random(),
        math.random(50, 200)
    )
end

function generate_owner_data()
    local first_names = {"Pierre", "Marie", "Jean", "Sophie", "Luc"}
    local last_names = {"Dupont", "Martin", "Bernard", "Dubois", "Laurent"}
    local cities = {"Bruxelles", "Anvers", "Gand", "Liège"}
    local streets = {"Rue de la Paix", "Avenue Louise", "Boulevard du Roi"}
    -- Add timestamp to email for uniqueness
    local timestamp = os.time() + math.random(0, 999999)
    return string.format([[{
        "organization_id": "$ORG_ID",
        "first_name": "%s",
        "last_name": "%s",
        "email": "test%d_%d@example.be",
        "phone": "+32 2 %d %d %d",
        "address": "%s %d",
        "city": "%s",
        "postal_code": "%d",
        "country": "Belgium"
    }]],
        first_names[math.random(#first_names)],
        last_names[math.random(#last_names)],
        math.random(1000, 9999),
        timestamp,
        math.random(100, 999),
        math.random(10, 99),
        math.random(10, 99),
        streets[math.random(#streets)],
        math.random(1, 200),
        cities[math.random(#cities)],
        math.random(1000, 9999)
    )
end

function generate_expense_data(building_id)
    local descriptions = {
        "Entretien ascenseur",
        "Nettoyage parties communes",
        "Chauffage collectif",
        "Assurance immeuble"
    }
    local categories = {"Maintenance", "Repairs", "Insurance", "Utilities", "Cleaning"}
    -- If no building_id provided, get a random real one
    local bid = building_id or get_random_building_id()
    return string.format([[{
        "organization_id": "$ORG_ID",
        "building_id": "%s",
        "category": "%s",
        "description": "%s",
        "amount": %.2f,
        "expense_date": "%s"
    }]],
        bid,
        categories[math.random(#categories)],
        descriptions[math.random(#descriptions)],
        math.random(300, 5000) + math.random(),
        os.date("!%Y-%m-%dT%H:%M:%SZ")
    )
end

function weighted_choice(operations)
    local total_weight = 0
    for _, op in ipairs(operations) do
        total_weight = total_weight + op.weight
    end

    local rand = math.random() * total_weight
    local cumulative = 0

    for _, op in ipairs(operations) do
        cumulative = cumulative + op.weight
        if rand <= cumulative then
            return op
        end
    end

    return operations[1]
end

-- Setup function
function setup(thread)
    thread:set("id", thread.id)
end

function init(args)
    token = args[1]  -- JWT token passed as argument
end

-- Main request function (70% GET, 30% POST)
function request()
    local headers = {
        ["Authorization"] = "Bearer " .. token,
        ["Content-Type"] = "application/json"
    }

    -- 70% chance for read operations
    if math.random() < 0.7 then
        local op = weighted_choice(read_operations)
        return wrk.format(op.method, op.path, headers, nil)
    else
        -- 30% chance for write operations
        local op = weighted_choice(write_operations)
        local body = nil

        if string.find(op.path, "buildings") then
            body = generate_building_data()
        elseif string.find(op.path, "units") then
            body = generate_unit_data()
        elseif string.find(op.path, "owners") then
            body = generate_owner_data()
        elseif string.find(op.path, "expenses") then
            body = generate_expense_data()
        end

        return wrk.format(op.method, op.path, headers, body)
    end
end

-- Response processing
function response(status, headers, body)
    if status >= 400 then
        -- Track errors but don't spam
        if math.random() < 0.01 then  -- Log 1% of errors
            print("Error " .. status .. ": " .. body:sub(1, 100))
        end
    end
end

-- Statistics
function done(summary, latency, requests)
    io.write("------------------------------\n")
    io.write("Realistic Mixed Workload Results:\n")
    io.write("------------------------------\n")
    io.write(string.format("  Total requests: %d\n", summary.requests))
    io.write(string.format("  Successful: %d\n", summary.requests - summary.errors.status - summary.errors.timeout))
    io.write(string.format("  Errors: %d\n", summary.errors.status + summary.errors.timeout))
    io.write(string.format("  Requests/sec: %.2f\n\n", summary.requests / summary.duration * 1000000))

    io.write("Latency Distribution:\n")
    io.write(string.format("  50%%: %.2fms\n", latency:percentile(50)))
    io.write(string.format("  75%%: %.2fms\n", latency:percentile(75)))
    io.write(string.format("  90%%: %.2fms\n", latency:percentile(90)))
    io.write(string.format("  95%%: %.2fms\n", latency:percentile(95)))
    io.write(string.format("  99%%: %.2fms\n", latency:percentile(99)))
    io.write(string.format("  99.9%%: %.2fms\n", latency:percentile(99.9)))
    io.write("------------------------------\n\n")
end
LUA_SCRIPT

# Run wrk2 with realistic workload
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULT_FILE="$RESULTS_DIR/realistic-load_${TIMESTAMP}.txt"

wrk -t4 -c20 -d3m --latency -s /tmp/realistic_workload.lua "$BASE_URL" -- "$TOKEN" | tee "$RESULT_FILE"

echo ""
echo "✅ Test complete!"
echo "Results saved to: $RESULT_FILE"
echo ""
echo "Expected results for 1 vCPU / 2GB RAM:"
echo "  ✅ P99 latency: < 100ms (with POST operations)"
echo "  ✅ Throughput: > 200 req/s"
echo "  ✅ Error rate: < 1%"
