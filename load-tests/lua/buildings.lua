-- Buildings CRUD scenario for wrk
-- Simulates reading buildings list (most common operation)

init = function(args)
    wrk.headers["Content-Type"] = "application/json"
    wrk.headers["Accept"] = "application/json"

    -- In production, you'd set the auth token here
    -- wrk.headers["Authorization"] = "Bearer " .. token
end

-- Mix of operations weighted by real usage patterns
-- 70% GET list, 20% GET single, 10% POST
paths = {
    "/api/v1/buildings",           -- GET list
    "/api/v1/buildings",           -- GET list
    "/api/v1/buildings",           -- GET list
    "/api/v1/buildings",           -- GET list
    "/api/v1/buildings",           -- GET list
    "/api/v1/buildings",           -- GET list
    "/api/v1/buildings",           -- GET list
    "/api/v1/health",              -- Health check
    "/api/v1/health",              -- Health check
}

counter = 1

request = function()
    path = paths[counter]
    counter = counter + 1
    if counter > #paths then
        counter = 1
    end

    return wrk.format("GET", path)
end
