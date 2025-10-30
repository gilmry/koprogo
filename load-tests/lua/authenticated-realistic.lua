-- Realistic authenticated workload scenario for wrk
-- Simulates production usage with 80% GET / 20% POST
--
-- Uses demo credentials: syndic@grandplace.be / syndic123

local jwt_token = nil
local building_id = "22e60793-a82f-495c-9d7f-25f298f94149" -- From production seed data (Résidence Grand Place)
local organization_id = "46705175-5e03-4fd4-b0a3-7aac10a7a663" -- From production seed data

-- Simple JSON token extractor (no external dependencies)
function extract_token(body)
    local token = body:match('"token"%s*:%s*"([^"]+)"')
    if token then
        -- Trim whitespace and newlines
        token = token:gsub("^%s+", ""):gsub("%s+$", ""):gsub("\n", ""):gsub("\r", "")
    end
    return token
end

-- Generate unique identifier (timestamp + random)
function generate_unique_id()
    local timestamp = os.time()
    local random = math.random(10000000, 99999999)
    return string.format("%d%d", timestamp, random)
end

init = function(args)
    wrk.headers["Content-Type"] = "application/json"
    wrk.headers["Accept"] = "application/json"

    -- Seed for randomness
    math.randomseed(os.time())
end

-- Weighted endpoints: 80% GET / 20% POST
-- Format: {weight, method, path, body_type (optional)}
-- body_type indicates what kind of POST body to generate dynamically
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
    {4, "POST", "/api/v1/expenses", "expense_maintenance"},
    {2, "POST", "/api/v1/expenses", "expense_utilities"},
    {2, "POST", "/api/v1/expenses", "expense_insurance"},

    -- Owners creation (5% POST)
    {3, "POST", "/api/v1/owners", "owner_jean"},
    {2, "POST", "/api/v1/owners", "owner_marie"},

    -- Meetings creation (4% POST)
    {2, "POST", "/api/v1/meetings", "meeting_ordinary"},
    {2, "POST", "/api/v1/meetings", "meeting_extraordinary"},

    -- Units creation (3% POST)
    {2, "POST", "/api/v1/units", "unit_apartment"},
    {1, "POST", "/api/v1/units", "unit_parking"},
}

-- Generate POST body based on type
function generate_post_body(body_type)
    local unique_id = generate_unique_id()

    if body_type == "expense_maintenance" then
        return string.format('{"organization_id":"%s","building_id":"%s","category":"Maintenance","description":"Entretien mensuel","amount":150.50,"expense_date":"2025-10-30T00:00:00Z","supplier":"Maintenance Plus","invoice_number":"INV-%s"}',
            organization_id, building_id, unique_id)

    elseif body_type == "expense_utilities" then
        return string.format('{"organization_id":"%s","building_id":"%s","category":"Utilities","description":"Eau et électricité","amount":250.00,"expense_date":"2025-10-30T00:00:00Z","supplier":"Energy SA","invoice_number":"ENE-%s"}',
            organization_id, building_id, unique_id)

    elseif body_type == "expense_insurance" then
        return string.format('{"organization_id":"%s","building_id":"%s","category":"Insurance","description":"Assurance copropriété","amount":500.00,"expense_date":"2025-10-30T00:00:00Z","supplier":"Assur Corp","invoice_number":"ASS-%s"}',
            organization_id, building_id, unique_id)

    elseif body_type == "owner_jean" then
        return string.format('{"first_name":"Jean","last_name":"Martin","email":"jean.martin.%s@example.com","phone":"+32499123456","address":"Avenue Louise 123","city":"Bruxelles","postal_code":"1000","country":"Belgique"}',
            unique_id)

    elseif body_type == "owner_marie" then
        return string.format('{"first_name":"Marie","last_name":"Dubois","email":"marie.dubois.%s@example.com","phone":"+32477654321","address":"Rue Royale 45","city":"Bruxelles","postal_code":"1000","country":"Belgique"}',
            unique_id)

    elseif body_type == "meeting_ordinary" then
        return string.format('{"organization_id":"%s","building_id":"%s","meeting_type":"Ordinary","title":"Assemblée Générale Ordinaire","description":"AG annuelle","scheduled_date":"2025-12-15T14:00:00Z","location":"Salle communale"}',
            organization_id, building_id)

    elseif body_type == "meeting_extraordinary" then
        return string.format('{"organization_id":"%s","building_id":"%s","meeting_type":"Extraordinary","title":"AG Extraordinaire","description":"Travaux urgents","scheduled_date":"2025-11-20T18:00:00Z","location":"Salle polyvalente"}',
            organization_id, building_id)

    elseif body_type == "unit_apartment" then
        return string.format('{"building_id":"%s","unit_number":"T%d","unit_type":"Apartment","floor":%d,"surface_area":%d.5,"quota":%d.0}',
            building_id, math.random(100, 999), math.random(1, 5), math.random(50, 120), math.random(50, 200))

    elseif body_type == "unit_parking" then
        return string.format('{"building_id":"%s","unit_number":"P%s","unit_type":"Parking","floor":-1,"surface_area":12.5,"quota":10.0}',
            building_id, unique_id)

    else
        return nil
    end
end

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
            return endpoint[2], endpoint[3], endpoint[4]  -- method, path, body_type
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
    local method, path, body_type = select_endpoint()

    if body_type then
        -- Generate unique POST body for each request
        local body = generate_post_body(body_type)
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
