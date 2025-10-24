-- Mixed workload scenario for wrk
-- Simulates realistic user behavior with multiple endpoints

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
    local method, path = select_endpoint()
    return wrk.format(method, path)
end

-- Track response times
done = function(summary, latency, requests)
    io.write("------------------------------\n")
    io.write("Request Statistics:\n")
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
    io.write("------------------------------\n")
end
