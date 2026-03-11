# Integration Guides

Version: 1.0.0

## Third-Party Integrations

### Email (Lettre + SMTP)

**Configuration** (`backend/.env`):

```env
SMTP_HOST=smtp.example.com
SMTP_PORT=587
SMTP_USERNAME=noreply@koprogo.com
SMTP_PASSWORD=secret
SMTP_FROM=noreply@koprogo.com
```

**Usage**:

```rust
use lettre::{Message, SmtpTransport, Transport};

let email = Message::builder()
    .from("noreply@koprogo.com".parse()?)
    .to("user@example.com".parse()?)
    .subject("Payment Reminder")
    .body("Your payment is overdue...")?;

let mailer = SmtpTransport::relay(&smtp_host)?
    .credentials(Credentials::new(username, password))
    .build();

mailer.send(&email)?;
```

### S3-Compatible Storage (MinIO/AWS)

**Configuration**:

```env
STORAGE_PROVIDER=s3
S3_BUCKET=koprogo-uploads
S3_REGION=eu-west-3
S3_ENDPOINT=https://s3.eu-west-3.amazonaws.com
S3_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE
S3_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
```

### Monitoring (Prometheus + Grafana)

**Scrape config** (`prometheus.yml`):

```yaml
scrape_configs:
  - job_name: 'koprogo-api'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
```

**Grafana Dashboard**: Import from `monitoring/grafana/dashboards/`

### Payment Gateway (Stripe) - Planned

**Configuration**:

```env
STRIPE_API_KEY=sk_test_...
STRIPE_WEBHOOK_SECRET=whsec_...
```

## API Clients

### cURL Examples

```bash
# Login
curl -X POST https://api.koprogo.com/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"syndic@example.com","password":"pass123"}'

# List buildings
curl https://api.koprogo.com/api/v1/buildings \
  -H "Authorization: Bearer <token>"
```

### Python Client

```python
import requests

class KoproGoClient:
    def __init__(self, base_url, email, password):
        self.base_url = base_url
        self.token = self._login(email, password)

    def _login(self, email, password):
        response = requests.post(
            f"{self.base_url}/auth/login",
            json={"email": email, "password": password}
        )
        return response.json()["access_token"]

    def list_buildings(self):
        response = requests.get(
            f"{self.base_url}/buildings",
            headers={"Authorization": f"Bearer {self.token}"}
        )
        return response.json()

client = KoproGoClient("https://api.koprogo.com/api/v1", "user@example.com", "pass")
buildings = client.list_buildings()
```

### JavaScript/TypeScript Client

```typescript
class KoproGoClient {
  constructor(private baseUrl: string, private token: string) {}

  static async login(baseUrl: string, email: string, password: string) {
    const response = await fetch(`${baseUrl}/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email, password })
    });
    const { access_token } = await response.json();
    return new KoproGoClient(baseUrl, access_token);
  }

  async listBuildings() {
    const response = await fetch(`${this.baseUrl}/buildings`, {
      headers: { 'Authorization': `Bearer ${this.token}` }
    });
    return response.json();
  }
}

const client = await KoproGoClient.login(
  'https://api.koprogo.com/api/v1',
  'user@example.com',
  'password'
);
const buildings = await client.listBuildings();
```

---

**Version**: 1.0.0
