-- Authenticated mixed workload scenario for wrk
-- Simulates realistic user behavior with JWT authentication
--
-- Uses demo credentials: syndic@grandplace.be / syndic123

local jwt_token = nil
local login_requested = false

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

-- Weighted endpoints based on real usage patterns
-- Format: {weight, method, path}
endpoints = {
    -- Health checks (5%)
    {5, "GET", "/api/v1/health"},

    -- Buildings (40%)
    {20, "GET", "/api/v1/buildings"},
    {10, "GET", "/api/v1/buildings"},
    {10, "GET", "/api/v1/buildings"},

    -- Units (25%)
    {15, "GET", "/api/v1/units"},
    {10, "GET", "/api/v1/units"},

    -- Owners (15%)
    {10, "GET", "/api/v1/owners"},
    {5, "GET", "/api/v1/owners"},

    -- Expenses (10%)
    {7, "GET", "/api/v1/expenses"},
    {3, "GET", "/api/v1/expenses"},

    -- Meetings (5%)
    {3, "GET", "/api/v1/meetings"},
    {2, "GET", "/api/v1/meetings"},
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
            return endpoint[2], endpoint[3]
        end
    end

    -- Fallback
    return "GET", "/api/v1/health"
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
    local method, path = select_endpoint()
    return wrk.format(method, path)
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
    io.write("Authenticated Load Test Results:\n")
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

    if jwt_token then
        io.write("\n✅ Authentication: SUCCESS\n")
    else
        io.write("\n❌ Authentication: FAILED - No token received\n")
    end
    io.write("------------------------------\n")
end
