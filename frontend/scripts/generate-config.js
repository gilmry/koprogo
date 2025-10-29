#!/usr/bin/env node
import { writeFileSync } from "fs";
import { fileURLToPath } from "url";
import { dirname, join } from "path";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Generate config.js content with dynamic API URL detection
const configContent = `// Runtime configuration for KoproGo Frontend
// Automatically detects API URL based on current domain
(function () {
  const hostname = window.location.hostname;
  const port = window.location.port;

  // Development with Traefik/proxy on port 80
  if (
    (hostname === "localhost" || hostname === "127.0.0.1") &&
    (!port || port === "80")
  ) {
    window.__ENV__ = {
      API_URL: "http://localhost/api/v1",
    };
  }
  // For port 3000 (dev server), don't set __ENV__ - let it use import.meta.env.PUBLIC_API_URL
  // This allows Playwright to control the API URL via environment variables
  // Production: use api.{domain}
  else if (hostname !== "localhost" && hostname !== "127.0.0.1") {
    const protocol = window.location.protocol;
    window.__ENV__ = {
      API_URL: protocol + "//api." + hostname + "/api/v1",
    };
  }
})();
`;

// Write to public directory
const publicPath = join(__dirname, "..", "public", "config.js");
writeFileSync(publicPath, configContent, "utf-8");

console.log("✅ config.js generated with dynamic API URL detection");
