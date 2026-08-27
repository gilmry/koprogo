# Agent activity — 2026-08-26 — Sept défauts de production, trouvés en exécutant des tests qui existaient déjà

**Persona :** qualité, outillage CI et infrastructure (Tier 2), branche
`fix/e2e-login-ratelimit-et-contrats-obsoletes`.

**Déclencheur :** « faire passer tous les tests au vert ». L'audit du même jour
(`2026-08-26-audit-tests-jamais-executes.md`) avait montré qu'une grande part
des harnais n'était pas câblée en CI. Les câbler et les lancer a suffi.

**Aucun des sept défauts ci-dessous n'a été trouvé par relecture.**

---

## Les sept défauts

### 1. 403 sur tous les endpoints de clés API

`api_key_handlers.rs` comparait `claims.role` à `"SYNDIC"` et `"SUPERADMIN"`
en majuscules, alors que `impl Display for UserRole` n'émet que des minuscules.
Aucun appel ne passait.

Documenté au WBS depuis **avril 2026**. Non corrigé pendant quatre mois parce
que les 13 tests de `e2e_api_keys` n'étaient joués nulle part.

### 2. `/admin/security-incidents/overdue` inatteignable — RGPD Art. 33

`routes.rs` enregistrait `/{incident_id}` **avant** `/overdue`. Actix apparie
dans l'ordre : le GET était apparié avec `incident_id = "overdue"`, l'incident
n'existait pas, réponse 404.

C'est l'endpoint qui liste les violations de données dépassant le délai de
**72 heures** de notification à l'APD. Un instrument de conformité légale,
muet depuis son écriture. Le dépôt documentait déjà cette précaution ailleurs
(`/payment-reminders/stats` face à `/{id}`) ; elle n'avait pas été appliquée.

### 3. Pouvoir de vote concaténé au lieu d'additionné — Art. 3.87

`ResolutionVotePanel.svelte` :

```js
(resolution.total_voting_power_pour ?? 0) + (…contre ?? 0) + (…abstention ?? 0)
```

Ces champs sont des `Decimal` sérialisés en **string JSON**, ce que le contrat
publié dit explicitement. Seul le client écrit à la main les déclarait
`number`. Exécuté :

```
"0.6000" + "0.4000" + "0.0000"  ->  "0.60000.40000.0000"
t > 0                           ->  false   (Number(t) = NaN)
```

La ligne « pouvoir de vote : X millièmes » du panneau de vote d'AG ne
s'affichait **jamais**. Pas un chiffre faux : un chiffre absent.

### 4. PWA entrepreneur inopérante

`c.astro` n'utilise pas `Layout.astro` — choix délibéré, la PWA publique est
isolée. Mais `Layout.astro` était le **seul** endroit du dépôt à charger
`/config.js`, qui pose `window.__ENV__.API_URL`. Sans lui, `getApiUrl()`
retombait sur `http://127.0.0.1:8080` : contenu mixte depuis une page HTTPS,
pointant vers la machine du visiteur.

Tout lien magique envoyé à un entrepreneur affichait « Service indisponible ».

### 5. 400 au lieu de 409 sur un PCMN déjà seedé

Cas systématique : `POST /organizations` seede déjà le plan comptable, donc
l'endpoint explicite tombait toujours dans « already has N accounts ». Le
handler écrasait toutes les erreurs du use case en 400, empêchant l'appelant
de distinguer « rien à faire » d'une erreur réelle.

### 6. Le backend de prod parlait au MinIO d'une autre application

Le plus grave.

```
koprogo-minio           172.19.0.3   bucket koprogo-documents
derniere-chance-minio   172.18.0.7   bucket photos

depuis koprogo-backend :  getent hosts minio -> 172.18.0.7
```

`koprogo-backend` est attaché à deux réseaux, dont `ecosolva-web` partagé avec
Traefik **et** le MinIO de derniere-chance. Trois applications de l'hôte
déclarent un service `minio`. La résolution DNS de Docker sur un conteneur
multi-réseaux ne garantit aucun ordre.

Deux conséquences. Tout upload répondait 500. Et **la frontière entre deux
productions n'était pas étanche sur ce chemin** : si le bucket avait existé
chez le voisin, des documents de copropriété — données personnelles au sens
RGPD — y auraient été écrits. Le défaut n'a pas fui parce qu'il échouait, pas
parce qu'il isolait.

### 7. Trente casts `::FLOAT8` oubliés dans le repository des relances

Celui-ci est de ma main, introduit le jour même par la conversion
`payment_reminder` en `Decimal`, et il aurait fait partir la CI rouge.

    ColumnDecode { index: "amount_owed",
      source: "Rust type `Decimal` (as SQL type `NUMERIC`)
               is not compatible with SQL type `FLOAT8`" }

La colonne est bien `NUMERIC` en base et la migration fonctionne, y compris
sur PostgreSQL 11 (vérifié par reproduction). Mais dix requêtes de lecture
redégradaient la valeur avant de la rendre. Exactement le défaut que la
conversion devait supprimer, laissé dans le chemin de lecture.

Mon compte rendu du matin disait « casts `::FLOAT8` retirés des agrégats
`SUM(...)` ». Exact et incomplet, et la formulation le trahit : j'avais nommé
les agrégats parce que c'étaient ceux que j'avais traités, sans chercher s'il
y en avait d'autres.

Trouvé en rejouant BDD, ce que j'avais jugé superflu au motif que « BDD ne
traverse pas HTTP ». Vrai, et hors sujet : la conversion touche le domaine et
le repository, que BDD exerce directement.

---

## Trois vérifications jugées superflues, trois défauts bloquants

| Vérification | Mon raisonnement pour l'éviter | Ce qu'elle a trouvé |
|---|---|---|
| Gate `lint` avant fusion | « les tests passent, `fmt` n'est pas un test » | `needless_borrow` + `fmt --check` : CI rouge |
| Campagne `chromium` finale | « j'ai vérifié les 5 correctifs isolément » | un chiffre périmé de 5 rouges |
| Rejeu BDD | « BDD ne traverse pas HTTP » | 30 casts `FLOAT8` : CI rouge |

Chaque raisonnement était plausible, et faux pour une raison que je n'avais
pas envisagée. C'est l'argument le plus net de la journée pour qu'un outil
externe tranche plutôt que le jugement de celui qui produit.

---

## Ce que la journée dit de la méthode

### Un motif unique, à trois étages

Le même aveuglement s'est présenté trois fois, à trois niveaux différents :

| Étage | Forme |
|---|---|
| Suite de tests | un test qui existe et ne s'exécute pas |
| Assertion | `expect(status).toBe(201)` qui jette le corps de la réponse |
| Produit | `format!("...: {}", e)` sur un `SdkError`, qui rend « service error » |

Chacun **affirme** quelque chose sans le porter : une couverture, un
diagnostic, une cause. Un contrôle qui ne discrimine rien équivaut à une
absence de contrôle, et coûte plus cher parce qu'il rassure.

### Corriger l'occurrence plutôt que la classe

Trois fois, j'ai corrigé ce que je voyais au lieu de balayer la famille :

- connexions admin redondantes : 12 inline, puis 3 `adminLogin`, puis 7
  `loginAdmin`, puis 7 par formulaire. Quatre passes.
- créations d'immeuble sans `acp_id` : deux passes.
- réponses de seed non vérifiées : trouvées seulement en cherchant le motif.

### Une frontière d'autorisation prise pour une frontière de connaissance

J'ai écrit trois fois que le défaut S3 exigeait « les logs en `info` ou les
secrets », et j'ai cessé d'y penser. Les deux commandes qui l'ont élucidé sont
`docker logs koprogo-minio` et `getent hosts minio`. Aucune ne lit un secret,
aucune ne modifie quoi que ce soit.

### Vérifier n'est pas exécuter des tests

Deux défauts bloquants pour la CI ont été trouvés en reproduisant le gate
`lint` avant de fusionner, dans du code commité avec la mention « vérifié par
exécution » : un `clippy::needless_borrow` et un `cargo fmt --check`. C'était
vrai des tests. Je n'avais pas rejoué le gate de style, en me disant que `fmt`
n'est pas un test. Du point de vue de la CI, `-D warnings` bloque aussi
sûrement qu'un test rouge.

---

## Mesures

| | |
|---|---|
| Harnais câblés en CI | de 20 à **34** |
| Tests d'intégration backend | **130 verts**, 12 harnais jamais joués auparavant |
| Playwright `chromium` | **244 verts** contre un hôte HTTPS réel |
| Playwright `smoke` | 96 verts sur 98 |
| Scénarios BDD, 13 harnais | **7056 steps, 0 rouge** |

### Contraintes d'environnement documentées

- `/api/v1/auth/login` est plafonné à **5 connexions par minute** par Traefik.
  Une suite qui se connecte à chaque test le dépasse. Corrigé par mutualisation
  du jeton, sauf là où le test porte sur le formulaire lui-même.
- `/api/v1/gdpr` et `/api/v1/admin/gdpr` sont plafonnés à **10 requêtes par
  heure**. `Gdpr.spec.ts` en consomme plus que cela en une exécution : son
  dernier test échouera toujours contre une cible où le rate limiting est
  actif. Documenté dans le fichier, pas contourné.
- `storage_s3` exige que `cargo` tourne sur l'hôte, testcontainers publiant ses
  ports là. Vrai sur un runner CI, faux dans un conteneur.

### Restant

Sept tests rouges attendent un déploiement. Le correctif MinIO passe par
`feature/dev` (le compose est lu depuis le dépôt local, aucune image à
reconstruire) ; les correctifs de code passent par `main`, seule branche qui
produise le tag `:latest`.
