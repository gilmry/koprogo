#!/bin/sh
# Generate runtime configuration for KoproGo Frontend
# This script is executed at container startup to inject environment variables

cat > /usr/share/nginx/html/config.js <<EOF
// Runtime configuration for KoproGo Frontend
// Generated at: $(date -Iseconds)
window.__ENV__ = {
  API_URL: "${PUBLIC_API_URL:-http://localhost:8080/api/v1}"
};
EOF

echo "✅ Generated config.js with API_URL=${PUBLIC_API_URL:-http://localhost:8080/api/v1}"
