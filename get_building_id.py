#!/usr/bin/env python3
import requests
import json

# Login
login_response = requests.post(
    'https://api.koprogo.com/api/v1/auth/login',
    headers={'Content-Type': 'application/json'},
    json={'email': 'syndic@grandplace.be', 'password': 'syndic123'}
)

if login_response.status_code != 200:
    print(f"❌ Login failed: {login_response.text}")
    exit(1)

token = login_response.json()['token']
print(f"✅ Logged in")

# Get buildings
buildings_response = requests.get(
    'https://api.koprogo.com/api/v1/buildings',
    headers={'Authorization': f'Bearer {token}'}
)

if buildings_response.status_code != 200:
    print(f"❌ Failed to get buildings: {buildings_response.text}")
    exit(1)

data = buildings_response.json()
print(f"Response: {json.dumps(data, indent=2)[:200]}...")

# Handle both list and dict responses
if isinstance(data, list):
    buildings = data
elif isinstance(data, dict):
    buildings = data.get('buildings', []) or data.get('data', []) or [data]
else:
    buildings = []

if not buildings:
    print("❌ No buildings found")
    exit(1)

building = buildings[0]
print(f"\n📊 First Building from seed:")
print(f"  ID: {building['id']}")
print(f"  Name: {building['name']}")
print(f"\n✅ Use this ID in the script: {building['id']}")
