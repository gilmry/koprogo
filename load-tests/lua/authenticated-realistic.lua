-- Realistic authenticated workload scenario for wrk
-- Simulates production usage with 80% GET / 20% POST
--
-- Uses demo credentials: syndic@grandplace.be / syndic123

local jwt_token = nil
local building_id = "ed1c49c5-8434-48c8-8509-43fa202b7be6" -- From production seed data (Résidence Grand Place)
local organization_id = "718be179-675b-4a6b-a2a1-90f98e0c76a9" -- From production seed data

-- Simple JSON token extractor (no external dependencies)
function extract_token(body)
    local token = body:match('"token"%s*:%s*"([^"]+)"')
    if token then
        -- Trim whitespace and newlines
        token = token:gsub("^%s+", ""):gsub("%s+$", ""):gsub("\n", ""):gsub("\r", "")
    end
    return token
end

init = function(args)
    wrk.headers["Content-Type"] = "application/json"
    wrk.headers["Accept"] = "application/json"

    -- Seed for randomness
    math.randomseed(os.time())
end

-- Weighted endpoints: 80% GET / 20% POST
-- Format: {weight, method, path, body (optional)}
endpoints = {
    -- === 80% GET operations ===

    -- Health checks (5%)
    {5, "GET", "/api/v1/health"},

    -- Buildings (30% GET)
    {15, "GET", "/api/v1/buildings"},
    {10, "GET", "/api/v1/buildings"},
    {5, "GET", "/api/v1/buildings"},

    -- Units (20% GET)
    {12, "GET", "/api/v1/units"},
    {8, "GET", "/api/v1/units"},

    -- Owners (12% GET)
    {7, "GET", "/api/v1/owners"},
    {5, "GET", "/api/v1/owners"},

    -- Expenses (10% GET)
    {6, "GET", "/api/v1/expenses"},
    {4, "GET", "/api/v1/expenses"},

    -- Meetings (3% GET)
    {2, "GET", "/api/v1/meetings"},
    {1, "GET", "/api/v1/meetings"},

    -- === 20% POST operations ===

    -- Expenses creation (8% POST) - Most common write operation
    {4, "POST", "/api/v1/expenses", '{"organization_id":"' .. organization_id .. '","building_id":"' .. building_id .. '","category":"Maintenance","description":"Entretien mensuel","amount":150.50,"expense_date":"2025-10-30T00:00:00Z","supplier":"Maintenance Plus","invoice_number":"INV-' .. math.random(1000,9999) .. '"}'},
    {2, "POST", "/api/v1/expenses", '{"organization_id":"' .. organization_id .. '","building_id":"' .. building_id .. '","category":"Utilities","description":"Eau et électricité","amount":250.00,"expense_date":"2025-10-30T00:00:00Z","supplier":"Energy SA","invoice_number":"ENE-' .. math.random(1000,9999) .. '"}'},
    {2, "POST", "/api/v1/expenses", '{"organization_id":"' .. organization_id .. '","building_id":"' .. building_id .. '","category":"Insurance","description":"Assurance copropriété","amount":500.00,"expense_date":"2025-10-30T00:00:00Z","supplier":"Assur Corp","invoice_number":"ASS-' .. math.random(1000,9999) .. '"}'},

    -- Owners creation (5% POST)
    {3, "POST", "/api/v1/owners", '{"first_name":"Jean","last_name":"Martin","email":"jean.martin' .. math.random(1000, 9999) .. '@example.com","phone":"+32499123456","address":"Avenue Louise 123","city":"Bruxelles","postal_code":"1000","country":"Belgique"}'},
    {2, "POST", "/api/v1/owners", '{"first_name":"Marie","last_name":"Dubois","email":"marie.dubois' .. math.random(1000, 9999) .. '@example.com","phone":"+32477654321","address":"Rue Royale 45","city":"Bruxelles","postal_code":"1000","country":"Belgique"}'},

    -- Meetings creation (4% POST)
    {2, "POST", "/api/v1/meetings", '{"organization_id":"' .. organization_id .. '","building_id":"' .. building_id .. '","meeting_type":"Ordinary","title":"Assemblée Générale Ordinaire","description":"AG annuelle","scheduled_date":"2025-12-15T14:00:00Z","location":"Salle communale"}'},
    {2, "POST", "/api/v1/meetings", '{"organization_id":"' .. organization_id .. '","building_id":"' .. building_id .. '","meeting_type":"Extraordinary","title":"AG Extraordinaire","description":"Travaux urgents","scheduled_date":"2025-11-20T18:00:00Z","location":"Salle polyvalente"}'},

    -- Units creation (3% POST)
    {2, "POST", "/api/v1/units", '{"building_id":"' .. building_id .. '","unit_number":"T' .. math.random(100, 999) .. '","unit_type":"Apartment","floor":' .. math.random(1, 5) .. ',"surface_area":' .. math.random(50, 120) .. '.5,"quota":' .. math.random(50, 200) .. '.0}'},
    {1, "POST", "/api/v1/units", '{"building_id":"' .. building_id .. '","unit_number":"P' .. math.random(100, 999) .. '","unit_type":"Parking","floor":-1,"surface_area":12.5,"quota":10.0}'},
}

-- Calculate total weight
total_weight = 0
for i, endpoint in ipairs(endpoints) do
    total_weight = total_weight + endpoint[1]
end

-- Select endpoint based on weight
function select_endpoint()
    local rand = math.random(1, total_weight)
    local sum = 0

    for i, endpoint in ipairs(endpoints) do
        sum = sum + endpoint[1]
        if rand <= sum then
            return endpoint[2], endpoint[3], endpoint[4]
        end
    end

    -- Fallback
    return "GET", "/api/v1/health", nil
end

request = function()
    -- Keep trying to login until we have a token
    if not jwt_token then
        local login_body = '{"email":"syndic@grandplace.be","password":"syndic123"}'
        wrk.headers["Content-Type"] = "application/json"
        -- Don't set Authorization header for login
        wrk.headers["Authorization"] = nil
        return wrk.format("POST", "/api/v1/auth/login", nil, login_body)
    end

    -- Once we have a token, use it for authenticated requests
    wrk.headers["Authorization"] = "Bearer " .. jwt_token
    wrk.headers["Content-Type"] = "application/json"

    -- Select and execute endpoint
    local method, path, body = select_endpoint()

    if body then
        return wrk.format(method, path, nil, body)
    else
        return wrk.format(method, path)
    end
end

response = function(status, headers, body)
    -- Extract JWT token from login response
    if not jwt_token and status == 200 and body then
        jwt_token = extract_token(body)
        if jwt_token then
            io.write(string.format("✅ JWT token acquired: %s...\n", jwt_token:sub(1, 30)))
        else
            io.write("❌ Failed to extract token from response\n")
            if body then
                io.write(string.format("Response body (first 100 chars): %s\n", body:sub(1, 100)))
            end
        end
    elseif not jwt_token and status ~= 200 then
        io.write(string.format("⚠️  Login failed with status %d\n", status))
    end
end

-- Track response times
done = function(summary, latency, requests)
    io.write("------------------------------\n")
    io.write("Realistic Load Test Results (80/20):\n")
    io.write("------------------------------\n")
    io.write(string.format("  Total requests: %d\n", summary.requests))
    io.write(string.format("  Successful: %d\n", summary.requests - summary.errors.connect - summary.errors.read - summary.errors.write - summary.errors.status - summary.errors.timeout))
    io.write(string.format("  Errors: %d\n", summary.errors.connect + summary.errors.read + summary.errors.write + summary.errors.status + summary.errors.timeout))
    io.write(string.format("  Requests/sec: %.2f\n", summary.requests / summary.duration * 1e6))
    io.write("\nLatency Distribution:\n")
    io.write(string.format("  50%%: %.2fms\n", latency:percentile(50) / 1000))
    io.write(string.format("  75%%: %.2fms\n", latency:percentile(75) / 1000))
    io.write(string.format("  90%%: %.2fms\n", latency:percentile(90) / 1000))
    io.write(string.format("  95%%: %.2fms\n", latency:percentile(95) / 1000))
    io.write(string.format("  99%%: %.2fms\n", latency:percentile(99) / 1000))
    io.write(string.format("  99.9%%: %.2fms\n", latency:percentile(99.9) / 1000))

    -- Check authentication success based on error rate
    -- If most requests succeed, authentication worked
    local success_rate = (summary.requests - summary.errors.connect - summary.errors.read - summary.errors.write - summary.errors.status - summary.errors.timeout) / summary.requests * 100
    if success_rate > 90 then
        io.write(string.format("\n✅ Authentication: SUCCESS (%.2f%% success rate)\n", success_rate))
    else
        io.write(string.format("\n❌ Authentication: FAILED (only %.2f%% success rate)\n", success_rate))
    end

    io.write("\n📊 Workload: 80% GET / 20% POST (realistic production scenario)\n")
    io.write("------------------------------\n")
end
