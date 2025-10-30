#!/usr/bin/env python3
import requests
import json

# Login
login_response = requests.post(
    'https://api.koprogo.com/api/v1/auth/login',
    headers={'Content-Type': 'application/json'},
    json={'email': 'syndic@grandplace.be', 'password': 'syndic123'}
)

token = login_response.json()['token']
print(f"✅ Logged in\n")

# Test POST expense
expense_data = {
    "building_id": "ed1c49c5-8434-48c8-8509-43fa202b7be6",
    "category": "maintenance",
    "description": "Test expense",
    "amount": 100.0,
    "expense_date": "2025-10-30T00:00:00Z",
    "payment_status": "pending"
}

print(f"📝 POST expense data:")
print(json.dumps(expense_data, indent=2))
print()

expense_response = requests.post(
    'https://api.koprogo.com/api/v1/expenses',
    headers={
        'Authorization': f'Bearer {token}',
        'Content-Type': 'application/json'
    },
    json=expense_data
)

print(f"Status: {expense_response.status_code}")
print(f"Response: {expense_response.text}")
