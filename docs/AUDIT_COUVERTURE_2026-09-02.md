# Audit de couverture — 2026-09-02

Établi en marge du rapport de test « workflows financiers KoproGo » du
2026-09-01. Le rapport comptait 21 constats ; ce document répond à la
question qui vient après : **pourquoi personne ne les avait vus.**

Trois mesures, dans l'ordre de ce qu'elles coûtent à réparer.

---

## 1. Le contrat d'API — 27 % des routes

`utoipa` ne documente que ce qui porte explicitement `#[utoipa::path]`.
Une route non annotée est absente de `docs/api/openapi.json`, donc absente
de `frontend/src/types/api.d.ts`, donc **absente des trois barrières CI**
qui vérifient la cohérence du contrat (`ci.yml:595-625`).

C'est l'angle mort central : ces barrières comparent deux fichiers qui
ignorent tous les deux la route. « Aucun drift » ne veut pas dire « aucun
désaccord ».

| | routes | documentées |
|---|---:|---:|
| **Total** | 604 | **164 (27 %)** |

### Les dix modules les plus exposés

| module | documentées | hors contrat |
|---|---:|---:|
| `gamification_handlers` | 0/27 | 27 |
| `resource_booking_handlers` | 0/19 | 19 |
| `expense_handlers` | 0/17 | **17** |
| `local_exchange_handlers` | 0/17 | 17 |
| `notice_handlers` | 0/17 | 17 |
| `shared_object_handlers` | 0/17 | 17 |
| `budget_handlers` | 0/16 | **16** |
| `convocation_handlers` | 0/16 | 16 |
| `payment_reminder_handlers` | 0/16 | **16** |
| `contractor_report_handlers` | 0/15 | 15 |

En gras : les modules directement visés par le rapport du 2026-09-01.

### Ce que cela coûte concrètement

Faute de type généré, le frontend écrit le sien à la main. Trois exemples
mesurés, tous des défauts réels :

- **`lib/types.ts` déclarait `quota: number`.** Le contrat dit
  `quota: string` (`Decimal` sérialisé en chaîne, ADR-0008). En
  JavaScript, `+` concatène les chaînes : la somme des tantièmes affichait
  « NaN/1000èmes », et `Math.abs(NaN - 1000) > 0.5` valant *faux*,
  l'indicateur de conformité des quotités annonçait « correct » quel que
  soit l'encodage. C'est le constat F14.
- **`lib/api/payments.ts` redéclarait `CreatePaymentDto`** alors que
  `/payments` est documenté à 100 %. Le type recopié avait divergé sur
  trois points : `building_id` optionnel là où le contrat le dit requis,
  `currency` et `stripe_payment_intent_id` inventés côté client,
  `payment_method_id` et `description` inatteignables.
- **`lib/types.ts` ne déclarait pas `amount_excl_vat`, `vat_rate`,
  `vat_amount`** — pourtant renvoyés par l'API. La fiche dépense ne
  pouvait afficher que le TTC. C'est le constat F20.

### Le cliquet, et son erreur de mesure

`scripts/check-openapi-coverage.sh` empêche la dette d'augmenter. Il
fonctionnait, mais **sous-comptait de 118 routes** : sa détection exigeait
moins de 12 lignes entre `#[utoipa::path]` et la macro de route, seuil qui
punissait la documentation soignée — une route décrivant cinq réponses le
dépasse. La couverture réelle n'était pas de 9 % mais de 27 %.

Corrigé le 2026-09-02 : une annotation couvre la prochaine route, quelle
que soit sa longueur. Cliquet abaissé de 558 à 440.

### Sens de marche

23 routes annotées le 2026-09-02 (`journal-entries`, `units`,
`owner-contributions`, `call-for-funds`) : 15 nouveaux chemins, 20
nouveaux schémas, aucun chemin perdu. Priorité suivante : `expense`,
`budget`, `payment-reminder` — le reste du module financier.

### Ce qui reste hors contrat et mérite d'être su

Deux constats trouvés en remboursant la dette, tous deux du même type
« champ envoyé, champ jeté » :

- **`POST /expenses` recevait `amount_excl_vat`, `vat_rate` et `due_date`
  sans les accepter.** Ces champs n'existaient que sur
  `CreateInvoiceDraftDto`, servi par `POST /invoices/draft` — une autre
  route, que l'interface n'appelle pas. Corrigé le 2026-09-02 : ce sont
  les constats F12 et F20.
- **Le mode « détaillé » du formulaire de facture envoie `line_items`, qui
  n'est accepté par aucune route.** La table `invoice_line_items` existe
  depuis novembre 2025, `CreateInvoiceLineItemDto` aussi, mais **aucun
  endpoint ne les expose**. Le détail ligne à ligne saisi par
  l'utilisateur ne quitte donc jamais le navigateur.

  Ce n'est pas un défaut visible aujourd'hui : les totaux HT / TVA / TTC
  sont correctement agrégés et enregistrés, et aucun écran ne relit les
  lignes. Le mode détaillé fonctionne donc comme une calculatrice. Mais
  c'est un piège : le jour où un écran voudra afficher le détail, la
  donnée sera absente pour tout l'historique. **Non corrigé** — exposer
  ces lignes est une fonctionnalité à part entière, pas une correction.

---

## 2. Les tests unitaires — 3 modules sur 67 non couverts

| module | lignes | état |
|---|---:|---|
| `unit_owner_use_cases` | 322 | **couvert** par `unit_owner_use_cases_test.rs` (18 tests) |
| `individual_member_use_cases` | 113 | aucun test |
| `service_provider_use_cases` | 89 | aucun test |

`payment_reminder_use_cases` (670 lignes) figurait dans cette liste
jusqu'au 2026-09-02. Il portait la logique de recouvrement — pénalités,
escalade à quatre niveaux, création en masse — c'est-à-dire le module
même que visent les constats F10 et F18.

Il contenait un mock complet de 170 lignes, marqué `#[allow(dead_code)]`
et suivi d'un `// TODO: Add more comprehensive tests`. L'échafaudage avait
été monté puis jamais utilisé ; l'annotation faisait taire l'avertissement
qui l'aurait signalé.

**Les 11 tests écrits ont révélé deux défauts réels** dès la première
exécution :

1. `escalate_reminder` persistait le statut `Escalated` **avant** de
   valider le niveau suivant. Une escalade déclenchée avant le délai légal
   (J+30 pour une relance ferme, J+60 pour une mise en demeure) laissait
   la relance marquée escaladée, sans successeur, définitivement bloquée —
   et `process_automatic_escalations`, appelé par cron, journalise
   l'erreur sur stderr avant de poursuivre sa boucle.
2. Les deux méthodes de statistiques du mock renvoyaient des vecteurs
   vides. Tout test bâti dessus aurait passé quelle que soit la logique.

Reste à couvrir : `individual_member` et `service_provider`, deux
passe-plats CRUD portant chacun une seule règle réelle (unicité de l'email
dans une campagne, validation de `TradeCategory`).

---

## 3. Ce que les barrières existantes ne peuvent pas voir

Le dépôt a déjà de bons garde-fous : trois barrières de contrat en CI, un
cliquet de couverture OpenAPI, `check-no-f64-money.sh`,
`check-no-local-helpers.sh`. Le rapport du 2026-09-01 est quand même
passé au travers, pour trois raisons distinctes qui méritent d'être
nommées.

**La CI de `feature/dev` n'a volontairement aucune barrière de tests.**
C'est un choix assumé, pour itérer vite, les tests tournant en local dans
la boucle de développement. Le corollaire est réel : trois tests RGPD sont
restés cassés du 2026-08-31 au 2026-09-02, cassés par un commit qui ne les
avait pas relancés.

**Un test peut être un leurre.** Un mock qui renvoie un vecteur vide fait
passer n'importe quelle assertion. Les nouveaux garde-fous statiques
(`money-guards.test.ts`) ont donc été vérifiés en réintroduisant
délibérément les défauts qu'ils sont censés attraper — les deux sont
attrapés, avec un message qui nomme la cause.

**Le protocole de test compte autant que le code testé.** Une part
notable des constats du rapport ne se reproduit pas :

- **F5** « boutons inopérants sur toutes les pages financières » : les
  boutons de `/payment-reminders` et `/invoice-workflow` sont gardés par
  un `confirm()`, que toute automatisation de navigateur **rejette par
  défaut**. Le clic aboutissait, la confirmation était refusée, rien ne
  partait. Sur `/journal-entries` et `/owner-contributions`, les boutons
  sont désactivés tant que le formulaire est invalide.
- **F3** « rapports comptables en échec » : testé avec un compte sans
  organisation, qui reçoit un 401 sur toute route scopée.
- **F1/F2** « impossible de lier propriétaires aux lots » : le testeur
  passait `owner_id` à `PUT /units`, champ **déprécié** depuis la
  migration `20250127000000_refactor_owners_multitenancy`. La relation
  vit dans `unit_owners`.

Ces trois-là ne sont pas des faux constats sans valeur : dans les trois
cas, l'API répondait 2xx en jetant l'information, ce qui rendait l'erreur
indétectable côté client. C'est ce silence qui a été corrigé
(`deny_unknown_fields` + corps d'erreur JSON), pas seulement le
malentendu.

`FinancialRegressions.spec.ts` couvre aussi ces contre-épreuves : un
constat écarté sans preuve revient au prochain audit.
