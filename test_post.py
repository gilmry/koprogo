#!/usr/bin/env python3
import requests
import json
import uuid
from datetime import datetime

# Login
login_response = requests.post(
    'https://api.koprogo.com/api/v1/auth/login',
    headers={'Content-Type': 'application/json'},
    json={'email': 'syndic@grandplace.be', 'password': 'syndic123'}
)

token = login_response.json()['token']
print(f"✅ Logged in\n")

# Test POST expense (CORRECTED - with organization_id)
timestamp = datetime.now().strftime('%Y%m%d%H%M%S')
unique_id = str(uuid.uuid4())[:8]

expense_data = {
    "organization_id": "718be179-675b-4a6b-a2a1-90f98e0c76a9",
    "building_id": "ed1c49c5-8434-48c8-8509-43fa202b7be6",
    "category": "Maintenance",
    "description": "Test expense corrected",
    "amount": 100.0,
    "expense_date": "2025-10-30T00:00:00Z",
    "supplier": "Test Supplier",
    "invoice_number": f"INV-{timestamp}-{unique_id}"
}

print(f"📝 POST expense data (CORRECTED):")
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
print(f"Response: {expense_response.text[:500]}")
print()

# Test POST owner
print("="*50)
# Generate unique email with UUID + timestamp
owner_email = f"jean.testload.{timestamp}.{unique_id}@example.com"

owner_data = {
    "first_name": "Jean",
    "last_name": "TestLoad",
    "email": owner_email,
    "phone": "+32499123456",
    "address": "Avenue Louise 123",
    "city": "Bruxelles",
    "postal_code": "1000",
    "country": "Belgique"
}

print(f"📝 POST owner data:")
print(json.dumps(owner_data, indent=2))
print()

owner_response = requests.post(
    'https://api.koprogo.com/api/v1/owners',
    headers={
        'Authorization': f'Bearer {token}',
        'Content-Type': 'application/json'
    },
    json=owner_data
)

print(f"Status: {owner_response.status_code}")
print(f"Response: {owner_response.text[:500]}")
print()

# Test POST meeting (CORRECTED - with organization_id)
print("="*50)
meeting_data = {
    "organization_id": "718be179-675b-4a6b-a2a1-90f98e0c76a9",
    "building_id": "ed1c49c5-8434-48c8-8509-43fa202b7be6",
    "meeting_type": "Ordinary",
    "title": f"Test Meeting {timestamp}",
    "description": "Test description",
    "scheduled_date": "2025-12-15T14:00:00Z",
    "location": "Salle test"
}

print(f"📝 POST meeting data (CORRECTED):")
print(json.dumps(meeting_data, indent=2))
print()

meeting_response = requests.post(
    'https://api.koprogo.com/api/v1/meetings',
    headers={
        'Authorization': f'Bearer {token}',
        'Content-Type': 'application/json'
    },
    json=meeting_data
)

print(f"Status: {meeting_response.status_code}")
print(f"Response: {meeting_response.text[:500]}")
