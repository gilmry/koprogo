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

# Test login endpoint to get token
TOKEN=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
    -H "Content-Type: application/json" \
    -d '{"email":"admin@small.be","password":"admin123"}' | jq -r '.access_token')

if [ -z "$TOKEN" ] || [ "$TOKEN" == "null" ]; then
    echo "❌ Authentication: FAILED - No token received"
    exit 1
fi

echo "✅ JWT token acquired: ${TOKEN:0:30}..."

# Create wrk2 Lua script for realistic mixed workload
cat > /tmp/realistic_workload.lua << 'LUA_SCRIPT'
-- Realistic workload: 70% reads (GET), 30% writes (POST/PUT)
math.randomseed(os.time())

-- Track user's organization context
local org_id = nil
local building_ids = {}
local unit_ids = {}
local owner_ids = {}

-- API endpoints with realistic distribution
local read_operations = {
    { method = "GET", path = "/api/v1/buildings", weight = 25 },
    { method = "GET", path = "/api/v1/units", weight = 25 },
    { method = "GET", path = "/api/v1/owners", weight = 15 },
    { method = "GET", path = "/api/v1/expenses", weight = 15 },
    { method = "GET", path = "/api/v1/users", weight = 5 },
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
    local types = {"apartment", "studio", "duplex", "penthouse"}
    return string.format([[{
        "building_id": "%s",
        "unit_number": "%d.%d",
        "unit_type": "%s",
        "floor": %d,
        "surface_area": %.2f,
        "quota": %d
    }]],
        building_id or "00000000-0000-0000-0000-000000000001",
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
    return string.format([[{
        "first_name": "%s",
        "last_name": "%s",
        "email": "test%d@example.be",
        "phone": "+32 2 %d %d %d"
    }]],
        first_names[math.random(#first_names)],
        last_names[math.random(#last_names)],
        math.random(1000, 9999),
        math.random(100, 999),
        math.random(10, 99),
        math.random(10, 99)
    )
end

function generate_expense_data(building_id)
    local descriptions = {
        "Entretien ascenseur",
        "Nettoyage parties communes",
        "Chauffage collectif",
        "Assurance immeuble"
    }
    return string.format([[{
        "building_id": "%s",
        "description": "%s",
        "amount": %.2f,
        "expense_date": "%s",
        "due_date": "%s"
    }]],
        building_id or "00000000-0000-0000-0000-000000000001",
        descriptions[math.random(#descriptions)],
        math.random(300, 5000) + math.random(),
        os.date("%Y-%m-%d"),
        os.date("%Y-%m-%d", os.time() + 30*24*60*60)
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
