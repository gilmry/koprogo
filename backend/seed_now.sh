#!/bin/bash
# Quick seed script using psql and the seed logic

set -e

# Get superadmin token
TOKEN=$(curl -s -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@koprogo.com","password":"admin123"}' | jq -r '.token')

echo "Token: ${TOKEN:0:50}..."

# Seed demo data
curl -X POST http://localhost:8080/api/v1/seed/demo \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" | jq

echo "✅ Seed complete!"
