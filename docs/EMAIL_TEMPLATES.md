# Email Templates Documentation

Version: 1.0.0

## Templates

### Payment Reminder (Gentle)

**Subject**: Rappel aimable - Charges {{quarter}} {{year}}

**Body**:
```
Bonjour {{owner_name}},

Nous vous rappelons amicalement que le paiement des charges du {{quarter}} {{year}}
d'un montant de {{amount}}€ est attendu avant le {{due_date}}.

Référence de paiement: {{reference}}

Si vous avez déjà effectué ce paiement, veuillez ignorer ce message.

Cordialement,
{{syndic_name}}
{{building_name}}
```

### Meeting Invitation

**Subject**: Convocation AG - {{meeting_date}}

**Body**:
```
Chers copropriétaires,

Vous êtes convoqués à l'Assemblée Générale qui se tiendra le {{meeting_date}} à {{meeting_time}}.

Lieu: {{meeting_location}}

Ordre du jour:
{{agenda}}

Veuillez trouver ci-joint les documents nécessaires.

Cordialement,
Le Syndic
```

## Configuration

```env
# backend/.env
SMTP_HOST=smtp.example.com
SMTP_PORT=587
SMTP_FROM=noreply@koprogo.com
```

---

**Version**: 1.0.0
