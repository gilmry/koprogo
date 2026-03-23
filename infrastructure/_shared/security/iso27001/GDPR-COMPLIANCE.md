# Conformite GDPR - KoproGo

## ISO 27001:2022 - A.18.1 + RGPD (EU 2016/679)

### Articles implementes

| Article | Droit | Endpoint | Status |
|---------|-------|----------|--------|
| Art. 15 | Acces | GET /gdpr/export | Implemente |
| Art. 16 | Rectification | PUT /gdpr/rectify | Implemente |
| Art. 17 | Effacement | DELETE /gdpr/erase | Implemente |
| Art. 18 | Limitation traitement | PUT /gdpr/restrict-processing | Implemente |
| Art. 20 | Portabilite | GET /gdpr/export (JSON) | Implemente |
| Art. 21 | Opposition marketing | PUT /gdpr/marketing-preference | Implemente |

### Mesures techniques

| Mesure | Implementation |
|--------|----------------|
| Chiffrement au repos | LUKS (AES-XTS-512) |
| Chiffrement en transit | TLS 1.3 (Let's Encrypt) |
| Pseudonymisation | Anonymisation dans /gdpr/erase |
| Minimisation | Seules les donnees necessaires stockees |
| Retention | Politique de retention par type de donnee |
| Breach notification | Alertes automatiques + processus 72h |
| DPO | A designer |
| DPIA | A realiser pour le traitement principal |
| Registre traitements | Audit trail complet (audit.rs) |

### Autorite de controle

- **APD** (Autorite de Protection des Donnees belge)
- Contact: contact@apd-gba.be
- Site: https://www.autoriteprotectiondonnees.be
