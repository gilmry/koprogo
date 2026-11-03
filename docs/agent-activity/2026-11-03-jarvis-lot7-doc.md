# 2026-11-03 — Jarvis : lot 7, exactitude fonctionnelle et juridique (1re passe)

Persona : agent Jarvis, lettre de mission v0.1.0, lot 7. La doc est alignée
sur le code, jamais l'inverse. Aucun code, aucun test modifié.

## Corrigé

| Sujet | Correction | Source de vérité |
|---|---|---|
| Trois devis | « obligation légale au-delà de 5 000 € » → l'AG fixe aux 2/3 le seuil de mise en concurrence (art. 3.88 § 1er, 1°, c) | `domain/copropriete/majorites.rs` (`SeuilDeMiseEnConcurrence`) |
| Bornes de recharge (blog) | majorité simple → 2/3 (travaux aux parties communes) | idem (`TravauxAuxPartiesCommunes`) |
| Conseil de copropriété | « conseil syndical » → « conseil de copropriété » (22 occurrences) ; « plus de 20 lots » → « au moins 20 lots », art. 3.90 | lettre de mission, art. 3.90 |
| Taux de pénalité | 8 % → 4,5 % (2026), exemples recalculés, avertissement « à confirmer chaque année » | `domain/comptabilite/payment_reminder.rs:104-106` |
| FEC | sections retirées | aucun export FEC dans `backend/src` ni `frontend/src` (0 occurrence) |
| Linky | « obligatoire en Belgique » → compteur français, API Enedis visée | lettre de mission |
| CREG | « label » → « accréditation à la charte, visée » | lettre de mission |
| SMS / Push | SMS retiré, Push « prévu », Email « suivi d'envoi, transport SMTP à brancher » | `convocation_use_cases.rs:299` |
| Sondages | base légale « 577-8/4 » retirée : consultatif, seule l'AG décide | lettre de mission |
| Jalon 6 MCP | bandeau « vision » | — |
| Appel de fonds | « email + PDF » → suivi d'envoi, pas de PDF | exporteurs réels : `domain/services/*_exporter.rs` |

## Non fait, et pourquoi

- **Renumérotation 577-x → Livre 3** : les renvois actuels sont parfois faux
  (« 577-2 » pour l'état daté). Une table de correspondance doit être validée
  avant de remplacer ; proposée dans la PR.
- **`legal_compliance.feature`, `expenses.feature`, matrice de conformité,
  script de chiffres** : ce sont des tests ou du code, hors du périmètre doc
  du lot 7 ; ils demandent une story signée.
