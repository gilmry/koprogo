# Issue #006 - Système de Paiement en Ligne

**Priorité**: 🟡 IMPORTANT
**Estimation**: 15-20 heures
**Labels**: `enhancement`, `backend`, `frontend`, `payments`, `important`

---

## 📋 Description

Intégrer un système de paiement en ligne pour permettre aux copropriétaires de régler leurs charges directement depuis l'application. Simplifier la gestion des encaissements pour les syndics.

**Bénéfices** :
- Réduction des impayés (paiement immédiat)
- Traçabilité automatique
- Réconciliation bancaire simplifiée
- Relances automatiques

---

## 🎯 Objectifs

- [ ] Intégration Stripe pour cartes bancaires
- [ ] Support SEPA Direct Debit (prélèvement automatique)
- [ ] Génération reçus automatiques
- [ ] Webhook pour notifications paiement
- [ ] Dashboard encaissements syndic
- [ ] Export comptable des transactions

---

## 📐 Architecture

### Entité `Payment`

```rust
pub struct Payment {
    pub id: Uuid,
    pub expense_id: Uuid,
    pub owner_id: Uuid,
    pub amount: Decimal,
    pub payment_method: PaymentMethod, // Card, BankTransfer, SEPA
    pub status: PaymentStatus, // Pending, Completed, Failed, Refunded
    pub stripe_payment_id: Option<String>,
    pub receipt_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

pub enum PaymentMethod {
    CreditCard,
    BankTransfer,
    SepaDirectDebit,
}

pub enum PaymentStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Refunded,
}
```

---

## 🔧 Endpoints

| Méthode | Endpoint | Description |
|---------|----------|-------------|
| `POST` | `/api/v1/payments/create-intent` | Créer PaymentIntent Stripe |
| `POST` | `/api/v1/payments/confirm` | Confirmer paiement |
| `GET` | `/api/v1/payments/:id` | Détails paiement |
| `GET` | `/api/v1/payments/owner/:owner_id` | Historique paiements |
| `POST` | `/api/v1/payments/sepa/mandate` | Créer mandat SEPA |
| `POST` | `/api/v1/webhooks/stripe` | Webhook Stripe |
| `GET` | `/api/v1/payments/receipt/:id` | Télécharger reçu PDF |

---

## 📝 User Stories

### US1 - Paiement par carte
```gherkin
En tant que copropriétaire
Je veux payer mes charges par carte bancaire
Afin d'être à jour rapidement

Scénario: Paiement réussi
  Étant donné une charge impayée de 450€
  Quand je clique sur "Payer par carte"
  Et je saisis mes informations bancaires
  Alors le paiement est traité
  Et je reçois un reçu par email
  Et la charge est marquée "Payée"
```

### US2 - Prélèvement automatique
```gherkin
En tant que copropriétaire
Je veux configurer un prélèvement automatique
Afin de ne jamais être en retard

Scénario: Activation SEPA
  Quand je fournis mon IBAN
  Et je signe le mandat électronique
  Alors mes charges futures sont prélevées automatiquement
  Et je reçois un email 2 jours avant chaque prélèvement
```

---

## 🔧 Implémentation

### 1. Intégration Stripe

**Cargo.toml** :
```toml
stripe-rust = { version = "0.26", features = ["async"] }
```

**Use Case** :
```rust
use stripe::{
    Client, CreatePaymentIntent, PaymentIntent, Currency
};

impl PaymentUseCases {
    pub async fn create_payment_intent(
        &self,
        expense_id: Uuid,
        owner_id: Uuid,
    ) -> Result<PaymentIntentResponse, String> {
        // 1. Récupérer expense
        let expense = self.expense_repo.find_by_id(expense_id).await?;

        // 2. Créer PaymentIntent Stripe
        let stripe_client = Client::new(env::var("STRIPE_SECRET_KEY")?);

        let mut create_intent = CreatePaymentIntent::new(
            (expense.amount * 100).to_u64(), // En centimes
            Currency::EUR,
        );
        create_intent.metadata = Some([
            ("expense_id".to_string(), expense_id.to_string()),
            ("owner_id".to_string(), owner_id.to_string()),
        ].into());

        let intent = PaymentIntent::create(&stripe_client, create_intent)
            .await
            .map_err(|e| e.to_string())?;

        // 3. Sauvegarder Payment en DB
        let payment = Payment::new(expense_id, owner_id, expense.amount);
        self.payment_repo.create(&payment).await?;

        Ok(PaymentIntentResponse {
            client_secret: intent.client_secret.unwrap(),
            payment_id: payment.id,
        })
    }
}
```

### 2. Webhook Handler

```rust
use stripe::{Event, EventObject, EventType};

pub async fn stripe_webhook(
    body: web::Bytes,
    headers: web::Header<HeaderMap>,
) -> Result<HttpResponse> {
    let signature = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or("Missing signature")?;

    let webhook_secret = env::var("STRIPE_WEBHOOK_SECRET")?;

    let event = Webhook::construct_event(
        &body,
        signature,
        &webhook_secret,
    ).map_err(|e| e.to_string())?;

    match event.type_ {
        EventType::PaymentIntentSucceeded => {
            if let EventObject::PaymentIntent(intent) = event.data.object {
                // Marquer payment comme completed
                let expense_id = intent.metadata.get("expense_id");
                // Update DB
            }
        }
        EventType::PaymentIntentPaymentFailed => {
            // Gérer échec
        }
        _ => {}
    }

    Ok(HttpResponse::Ok().finish())
}
```

---

## ✅ Critères d'Acceptation

- [ ] Paiement CB via Stripe Elements
- [ ] Webhook traite payment.succeeded
- [ ] Expense marquée "Paid" automatiquement
- [ ] Génération PDF reçu
- [ ] Mandats SEPA stockés sécurisés
- [ ] Refunds possibles (syndic only)

---

## 🚀 Checklist

- [ ] Créer entité Payment + migration
- [ ] Intégrer Stripe SDK
- [ ] Créer PaymentUseCases
- [ ] Handler webhook Stripe
- [ ] Frontend composant PaymentForm.svelte
- [ ] Tests E2E avec Stripe test mode
- [ ] Documentation

---

**Créé le** : 2025-10-23
**Dépend de** : Issue #003 (rapports financiers)
