#!/usr/bin/env python3
import requests
import json

# Login as superadmin
login_response = requests.post(
    'http://localhost:8080/api/v1/auth/login',
    headers={'Content-Type': 'application/json'},
    json={'email': 'admin@koprogo.com', 'password': 'admin123'}
)

if login_response.status_code != 200:
    print(f"❌ Login failed: {login_response.text}")
    exit(1)

token = login_response.json()['token']
print(f"✅ Logged in, token: {token[:50]}...")

# Seed demo data
seed_response = requests.post(
    'http://localhost:8080/api/v1/seed/demo',
    headers={
        'Authorization': f'Bearer {token}',
        'Content-Type': 'application/json'
    }
)

print(f"\n{'='*50}")
if seed_response.status_code == 200:
    print("✅ Seed successful!")
    result = seed_response.json()
    print(result.get('message', ''))
else:
    print(f"❌ Seed failed: HTTP {seed_response.status_code}")
    print(seed_response.text)
print(f"{'='*50}\n")
