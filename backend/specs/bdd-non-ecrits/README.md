# Scénarios BDD sans code : des spécifications, pas des tests

Ces 28 fichiers vivaient dans `backend/tests/features/`. **Aucun harnais ne les
chargeait.** Le chargement y est nominatif — `run_and_exit("tests/features/x.feature")` —
et il n'existe aucun glob de répertoire : un fichier que personne ne cite ne s'exécute
jamais. Il ne produit alors aucun signal, ni vert, ni rouge, ni « skipped ». Il disparaît
du décompte tout en continuant à figurer dans les scénarios que le projet revendique.

Ils sont ici pour que leur statut soit lisible : ce sont des **spécifications écrites
d'avance**. Elles gardent leur valeur de cahier des charges. Elles n'ont simplement
jamais eu de code.

## Pourquoi ils n'ont pas été branchés

Mesure exécutée, pas estimée : les harnais déclarent **1056 pas**. Sur les
**1387 phrases uniques** de ces 28 fichiers, **39 seulement**
correspondent à un pas existant. Les brancher demanderait donc d'écrire environ
**1348 définitions de pas**, chacune avec une vraie assertion contre un cas d'usage.

Les écrire vite et creux reproduirait exactement le défaut que ce déplacement corrige :
une capacité écrite, testée, et sans effet.

## Le coût, fichier par fichier

| Pas à écrire | Fichier | Scénarios | Phrases uniques |
|---:|---|---:|---:|
| 146 | `legal_compliance.feature` | 41 | 147 |
| 80 | `sel_workflow.feature` | 14 | 86 |
| 71 | `vote_ag_workflow.feature` | 23 | 73 |
| 68 | `ticket_workflow.feature` | 10 | 70 |
| 65 | `notice_board_workflow.feature` | 18 | 68 |
| 63 | `poll_workflow.feature` | 11 | 66 |
| 63 | `mcp_sse.feature` | 21 | 64 |
| 60 | `security_incidents.feature` | 21 | 61 |
| 56 | `second_convocation.feature` | 18 | 57 |
| 54 | `legal_api.feature` | 20 | 55 |
| 49 | `api_keys.feature` | 17 | 50 |
| 48 | `role_assignment_endpoint.feature` | 15 | 49 |
| 44 | `technical_spec.feature` | 14 | 45 |
| 44 | `marketplace.feature` | 17 | 45 |
| 39 | `gdpr_art30.feature` | 14 | 40 |
| 39 | `contractor_evaluation.feature` | 16 | 40 |
| 37 | `service_providers.feature` | 12 | 38 |
| 33 | `ticket_complaint.feature` | 11 | 34 |
| 33 | `syndic_response_sla.feature` | 10 | 34 |
| 32 | `individual_members.feature` | 16 | 33 |
| 32 | `consent.feature` | 14 | 33 |
| 31 | `contract_evaluation.feature` | 12 | 32 |
| 30 | `role_delegation.feature` | 10 | 31 |
| 30 | `mandate.feature` | 9 | 31 |
| 29 | `work_orders.feature` | 10 | 30 |
| 29 | `magic_link.feature` | 8 | 30 |
| 24 | `resolution_agenda.feature` | 10 | 26 |
| 19 | `unit_owner_validation.feature` | 7 | 19 |
| **1348** | **28 fichiers** | **419** | **1387** |

## Le cas de `legal_compliance.feature`

Il se présente en tête comme « le POINT CENTRAL de suivi de conformité juridique »,
quarante et un scénarios rattachés chacun à un article du Code civil. Aucun ne
s'exécutait. C'est ce qui a permis à la matrice de conformité de déclarer six règles
« NON implémentée (bloquant pour production) » pendant six mois alors qu'elles
l'étaient (#837) : rien ne pouvait la démentir.

C'est le fichier qui mérite le plus d'être branché en premier, et le plus coûteux.

## Comment en rebrancher un

1. Le remettre dans `backend/tests/features/`.
2. Écrire ses pas dans un harnais, ou l'ajouter à la liste d'un harnais existant.
3. Le harnais doit citer `tests/features/<nom>.feature` : c'est ce que la garde
   `garde_features_orphelines.rs` vérifie.

Cette garde est désormais à **zéro**. Tout fichier `.feature` présent dans
`tests/features/` y est chargé par un harnais. Suivi en #838.
