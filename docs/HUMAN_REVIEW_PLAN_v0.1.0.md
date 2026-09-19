# Plan de Revue Humaine — KoproGo v0.1.0

**Pour** : le relecteur qui signe G1
**Où** : `http://localhost:8090` — la **pile de recette**, jetable
**Durée estimée** : 3-4 sessions de 2-3h (total ~8-10h)
**Date** : 2026-04-01, en-tête corrigé le 2026-09-18
**WBS** : Phase 8 de `docs/archive/WBS_RELEASE_0_1_0.md`

> ## ⚠️ Deux corrections d'en-tête, et pourquoi elles comptent
>
> **1. L'adresse.** Ce document a désigné pendant des mois le **port 80 nu**
> de cet hôte, avec un `staging.koprogo.com` en second choix qui n'a jamais
> existé. Or ce port 80 **est le Traefik de la démo** : un relecteur qui
> suivait ce plan créait des assemblées générales, envoyait des convocations
> et enregistrait des procurations **dans la démo publique**, en croyant
> travailler sur un bac à sable. L'ADR 0050 a décalé la recette sur **8090**
> précisément pour que les deux ne puissent plus être confondues.
>
> **2. Le mot de passe.** La table ci-dessous donnait en clair celui du
> superadministrateur. Il ne marche plus, et c'est voulu : il tourne
> au déploiement. Les identifiants se lisent dans l'environnement —
> `KOPROGO_SUPERADMIN_EMAIL` et `KOPROGO_SUPERADMIN_PASSWORD` — comme la
> suite e2e les lit (`frontend/tests/e2e/helpers/identifiants.ts`).
>
> ## Ce document a été refondu le 2026-09-18
>
> Il s'arrêtait au milieu de sa session 1, sur un « *(voir document
> complet)* » qui ne renvoyait nulle part. Le signer en l'état aurait fait
> attester une couverture qui n'existait pas.
>
> Il ne recopie plus les parcours : ceux-ci vivent dans
> `docs/personas/<role>.md`, y sont tenus à jour, et les dupliquer ici
> créerait une seconde source de vérité qui dériverait. Ce plan porte **ce
> qu'il faut juger** ; le persona porte **ce qu'il faut faire**.
>
> Les deux workflows d'assemblée générale restent écrits en détail, parce
> qu'ils portent des règles chiffrées où une erreur d'un jour ou d'un
> pourcent est une faute de droit, pas un défaut d'ergonomie.

---
## Ce que G1 atteste, et ce qu'il n'atteste PAS

**G1 n'est pas une seconde campagne de tests.** La campagne automatique en
exécute 356 contre la recette, à chaque fois, et elle est meilleure qu'un
humain pour ce qu'elle fait : cliquer partout sans se lasser, et comparer au
caractère près.

Rejouer ses parcours à la main n'ajouterait rien, et coûterait les huit
heures de ce plan.

Ce qu'un humain apporte, et que rien n'automatise :

| G1 atteste | G1 n'atteste pas |
|---|---|
| que le droit belge est **correctement interprété**, pas seulement implémenté | que les routes répondent — les gates le vérifient |
| que le produit est **utilisable** par la personne à qui il est destiné | que les écrans s'affichent — la campagne le vérifie |
| que les **libellés** disent ce que la loi dit | que les traductions existent — une garde le vérifie |
| que les **lacunes connues** sont acceptables pour une v0.1.0 | qu'il n'y en a pas |

La dernière ligne est la plus importante. Signer G1 ne veut pas dire « tout
marche ». Cela veut dire : **j'ai vu ce qui manque, et je considère qu'on
peut sortir avec.**

---

## Avant de commencer

### Où

`http://localhost:8090` — la pile de **recette**, jetable. Jamais le port 80
nu : c'est le Traefik de la démo publique (ADR 0050).

### Les identifiants

Ils se lisent dans l'environnement, jamais dans ce document :

```bash
export KOPROGO_SUPERADMIN_EMAIL=...
export KOPROGO_SUPERADMIN_PASSWORD=...
```

Les comptes de persona (Alice, François, Gisèle…) sont créés par le semis
ci-dessous et leurs mots de passe sont ceux du monde de scénario.

### Le monde de scénario

```bash
make seed-reset      # vide puis recrée ; échoue si l'API refuse
```

Environ vingt secondes. Il crée les vingt et un personas sur l'immeuble
« Résidence du Parc Royal », avec leurs quotités.

### Combien de temps

Trois à quatre sessions de deux à trois heures. **À faire en plusieurs
fois** : la fatigue d'une revue est ce qui fait signer ce qu'on n'a pas lu.

---

## Ce que la machine vérifie déjà — ne le refaites pas

Mesuré le 2026-09-18, campagne complète contre la recette :

```text
340 réussis / 2 échoués / 14 sautés en 1 h 06
backend stable, aucun redémarrage
```

Les deux échecs restants ouvrent tous deux `/admin/users` et tiennent à une
seule cause mesurée (#953). Le registre `RELEASE.md` porte le détail, gate
par gate, avec pour chacun la commande et le code de sortie.

**Si un gate est 🟡 ou 🔴 au registre le jour de la revue, lisez sa note
avant de commencer.** Un gate rouge n'interdit pas de relire, mais il change
ce que la relecture peut conclure.

---

## Ce qui n'a JAMAIS été éprouvé, et qu'il faut regarder en premier

Deux rôles sur six n'ont **aucun compte** dans le monde de recette. Leur
persona le déclare : `population_recette: 0`, `eprouve: false`.

| Rôle | Persona | État |
|---|---|---|
| **Administrateur** | `docs/personas/admin.md` | 0 compte de recette, non éprouvé |
| **Modérateur communauté** | `docs/personas/community-moderator.md` | 0 compte de recette, non éprouvé |

Cela ne veut pas dire que ces rôles sont cassés. Cela veut dire que
**personne ne sait**, ni la machine ni un humain. Pour une v0.1.0, c'est une
information de signature, pas un détail.

Commencez par eux : c'est là que la revue a le plus de chances de trouver
quelque chose, et c'est là que son verdict a le plus de valeur.

---

## Les sessions, et d'où elles sortent

Chaque session suit **le persona**, pas une liste de clics. Les parcours
nominaux, ce que chaque rôle ne peut pas faire, et les références légales
vivent dans `docs/personas/<role>.md` et y sont tenus à jour.

Les recopier ici créerait une seconde source de vérité qui dériverait — le
défaut que ce dépôt traque partout ailleurs. Ce plan ne porte donc que **ce
qu'il faut juger**, et renvoie au persona pour **ce qu'il faut faire**.

| Session | Rôle | Persona | Ce que vous jugez |
|---|---|---|---|
| 1 | Syndic | `syndic.md` | la conformité AG : délais, majorités, procurations |
| 2 | Copropriétaire | `owner.md` | ce qu'il voit de ses charges, et ce qu'il ne doit PAS voir |
| 3 | Comptable | `accountant.md` | la séparation des rôles : il saisit, il n'approuve pas |
| 4 | Administrateur + Modérateur | `admin.md`, `community-moderator.md` | tout, puisque rien n'a été éprouvé |

Pour chaque étape du persona, trois questions, et une seule compte
vraiment :

1. Est-ce que ça marche ? *(la machine l'a déjà dit, passez vite)*
2. **Est-ce que c'est juste au regard du droit ?**
3. Est-ce qu'un humain qui découvre l'écran comprend quoi faire ?

---

## Les deux workflows écrits en détail

Ils le sont parce qu'ils portent des règles de droit **chiffrées**, où une
erreur d'un jour ou d'un pourcent est une faute et non un défaut
d'ergonomie. Le reste des sessions se déroule par les personas.

## SESSION 1 — Conformité Légale AG (Votes & Convocations)

> **Objectif** : Vérifier que le droit belge de la copropriété (Art. 3.87-3.92 CC) est respecté.

---

### WORKFLOW 1 — Convocation AG (Art. 3.87 §3)

**Règle légale** : La convocation doit partir ≥ 15 jours avant la date de l'AG.

**[SYNDIC : François]**

- [ ] → `/meetings` → "Nouvelle réunion"
- [ ] Créer une AG ordinaire avec date = dans **13 jours**
  - `✓ Attendu :` Le bouton "Envoyer la convocation" est **désactivé** ou affiche une erreur "Délai légal non respecté (15 jours minimum)"
  - `✗ Bug :` ___________
- [ ] Changer la date à dans **16 jours**
  - `✓ Attendu :` Le bouton "Envoyer la convocation" est disponible
- [ ] Envoyer la convocation → `/convocations`
  - `✓ Attendu :` Statut passe à "Envoyée", liste des destinataires visible avec statut "En attente"
- [ ] Vérifier le suivi → cliquer sur la convocation → "Destinataires"
  - `✓ Attendu :` 10 copropriétaires listés avec statut e-mail (Envoyé/Ouvert/Échec)
  - `✓ Attendu :` Jeanne Devos marquée "Courrier recommandé" (pas d'e-mail)

**[COPROPRIÉTAIRE : Alice]** (switch de compte)

- [ ] → `/convocations` ou notification reçue
  - `✓ Attendu :` La convocation est visible avec l'ordre du jour
- [ ] Confirmer présence → "Je serai présent(e)"
  - `✓ Attendu :` Statut mise à jour "Sera présent(e)"

**[COPROPRIÉTAIRE : Philippe]** (investisseur absent)

- [ ] → convocation → "Désigner un mandataire"
  - `✓ Attendu :` Formulaire de procuration disponible
- [ ] Donner procuration à Alice Dubois
  - `✓ Attendu :` Procuration enregistrée

**[SYNDIC : François]**

- [ ] → suivi convocation
  - `✓ Attendu :` "Taux d'ouverture : X%" et Alice = "Sera présent(e)", Philippe = "Procuration → Alice"

---

### WORKFLOW 2 — Vote AG : 4 types de majorités (Art. 3.88 CC)

> **Contexte** : L'immeuble "Résidence du Parc Royal" tient son AG.
> François préside, Alice est dans la salle avec procuration de Philippe (1 800) et la sienne (450) = 2 250 tantièmes.
> **Total votants** = Alice(450) + Bob(430) + Charlie(660) + Diane(580) + Marcel(450) + Nadia(320) + Marguerite(380) + Jeanne(290) + Philippe-via-Alice(1 800) + Emmanuel(1 280) = **6 640**

**Rappel règles belges :**
| Majorité | Seuil | Exemple |
|----------|-------|---------|
| Absolue | > 50% des votes exprimés | Budget annuel |
| 2/3 | ≥ 66,67% des votes exprimés | Travaux façade |
| 4/5 | ≥ 80% des votes exprimés | Changement d'affectation |
| Unanimité | 100% de TOUS les tantièmes (présents + absents) | Modification parts |

**[SYNDIC : François]**

- [ ] → `/meetings/:id` → "Résolutions" → "Nouvelle résolution"

Les quatre tests ci-dessous se jouent sur le **même** total de votants,
**6 640 tantièmes**. Ce qui change est le seuil, et c'est le seuil qu'il faut
vérifier — pas que le bouton réagisse.

> ⚠️ **Le piège de l'unanimité, et il est dans la loi.** Les trois premières
> majorités se calculent sur les **votes exprimés** (6 640). La quatrième se
> calcule sur **TOUS les tantièmes de l'ACP** (10 000), présents ou non.
> Un logiciel qui applique 100 % des exprimés au lieu de 100 % du total
> déclarerait adoptée une résolution qui ne l'est pas. C'est le seul de ces
> quatre tests où une erreur produit une décision **juridiquement nulle**.

#### Test 2.1 — Majorité absolue (> 50 % des exprimés)

**[SYNDIC : François]** Résolution « Budget annuel 2026 », majorité absolue.

**[COPROPRIÉTAIRES]** Faire voter POUR : Alice (450 + procuration Philippe
1 800), Charlie (660) = **2 910**. CONTRE : les autres = **3 730**.

- `✓ Attendu :` 2 910 / 6 640 = **43,8 %** → **REJETÉE**
- `✓ Attendu :` l'écran affiche le pourcentage ET le seuil, pas seulement le
  verdict. Un syndic doit pouvoir expliquer le résultat en assemblée.
- `✗ Bug :` ___________

Refaire en inversant : POUR = 3 730 → **56,2 %** → **ADOPTÉE**.

#### Test 2.2 — Majorité des 2/3 (≥ 66,67 % des exprimés)

**[SYNDIC]** Résolution « Travaux de façade », majorité 2/3.

**[COPROPRIÉTAIRES]** POUR : Alice (2 250), Charlie (660), Diane (580),
Emmanuel (1 280) = **4 770**.

- `✓ Attendu :` 4 770 / 6 640 = **71,8 %** → **ADOPTÉE**
- `✓ Attendu :` retirer Emmanuel → 3 490 / 6 640 = **52,6 %** → **REJETÉE**
- `✗ Bug :` ___________

**Le cas limite à essayer**, et c'est celui qui distingue une implémentation
correcte d'une approximation : construire un vote à **exactement 66,67 %**.
La loi dit « les deux tiers », donc `≥ 2/3`, et non `> 2/3`. Un logiciel qui
rejette à 66,67 % rejette une résolution adoptée.

#### Test 2.3 — Majorité des 4/5 (≥ 80 % des exprimés)

**[SYNDIC]** Résolution « Changement d'affectation d'une partie commune »,
majorité 4/5.

**[COPROPRIÉTAIRES]** POUR : tout le monde sauf Jeanne (290) et Nadia (320)
= **6 030**.

- `✓ Attendu :` 6 030 / 6 640 = **90,8 %** → **ADOPTÉE**
- `✓ Attendu :` ajouter Marguerite (380) aux CONTRE → 5 650 / 6 640 =
  **85,1 %** → toujours **ADOPTÉE**
- `✗ Bug :` ___________

#### Test 2.4 — Unanimité (100 % de TOUS les tantièmes)

**[SYNDIC]** Résolution « Modification des quotités », unanimité.

**[COPROPRIÉTAIRES]** Faire voter POUR **tous les présents** : **6 640**.

- `✓ Attendu :` **REJETÉE**. Les présents totalisent 6 640 sur 10 000 ; les
  3 360 tantièmes absents comptent comme non-acquis.
- `✓ Attendu :` le motif du refus **nomme les absents**, ou au moins le total
  manquant. « Unanimité non atteinte » sans chiffre laisse le syndic sans
  rien à dire à l'assemblée.
- `✗ Bug :` ___________

> Si ce test rend **ADOPTÉE**, arrêtez la revue et signalez-le. Ce n'est pas
> un défaut d'affichage : c'est une décision d'assemblée générale prise sans
> la majorité que la loi exige.

---

## Les zones rouges connues au 2026-09-18

À lire **avant** de commencer, pour ne pas les découvrir à la troisième
heure et croire les avoir trouvées.

| Sujet | Issue | Ce que ça change pour la revue |
|---|---|---|
| Administrateur et modérateur jamais éprouvés | — | aucun compte de recette ; tout est à découvrir |
| Le syndic ne peut plus saisir de vote en séance | #850 | chacun vote depuis son compte. Est-ce tenable en assemblée réelle ? **C'est une question de produit, et elle attend votre arbitrage.** |
| Parcours multi-rôles jamais joués à largeur de téléphone | #869 | le banc mobile n'a ni compte ni base |
| La base de recette grossit à chaque campagne | #954 | les écrans d'administration ralentissent avec le temps, indépendamment du code |

---

## Signature

Signer, c'est attester **ce qui a été vu**, pas ce qui fonctionne.

```yaml
signature_humaine:
  date: null
  nom: null
  role: "Product Owner / superviseur"
  etat: null          # VALIDÉ | VALIDÉ AVEC RÉSERVES | REFUSÉ
  sessions_parcourues: []   # lesquelles, honnêtement
  reserves: |
    Ce qui n'a pas été parcouru, et ce qu'on accepte de ne pas savoir.
```

Une signature qui ne nomme pas ce qu'elle n'a pas couvert n'est pas une
validation : c'est une signature.

---

*Écrit le 2026-04-01, refondu le 2026-09-18.*

