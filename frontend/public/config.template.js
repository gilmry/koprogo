// Runtime configuration for KoproGo Frontend
// Automatically detects API URL based on current domain
(function () {
  const hostname = window.location.hostname;

  // Development: use localhost backend
  if (hostname === "localhost" || hostname === "127.0.0.1") {
    window.__ENV__ = {
      API_URL: "http://localhost:8080/api/v1",
    };
  }
  // Production: use api.{domain}
  else {
    const protocol = window.location.protocol;
    window.__ENV__ = {
      API_URL: protocol + "//api." + hostname + "/api/v1",
    };
  }
})();
