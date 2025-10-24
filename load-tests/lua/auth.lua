-- Authentication scenario for wrk
-- Simulates real user authentication flow

-- Global token storage
token = nil

-- Setup function (called once per thread)
setup = function(thread)
    thread:set("id", counter())
end

-- Initialize function (called once per thread)
init = function(args)
    -- Default credentials for load testing
    -- Make sure to create this user or use seed data
    email = "loadtest@example.com"
    password = "LoadTest123!"

    -- Login body
    login_body = string.format([[{"email": "%s", "password": "%s"}]], email, password)

    -- Headers
    wrk.headers["Content-Type"] = "application/json"
    wrk.headers["Accept"] = "application/json"
end

-- Request function (called for each request)
request = function()
    -- Login endpoint
    path = "/api/v1/auth/login"

    return wrk.format("POST", path, nil, login_body)
end

-- Response function (called for each response)
response = function(status, headers, body)
    -- Extract token from response (simplified)
    if status == 200 then
        -- In real scenario, parse JSON and extract token
        -- token = extract_token(body)
    end
end

-- Counter for unique thread IDs
counter = function()
    local i = 0
    return function()
        i = i + 1
        return i
    end
end
