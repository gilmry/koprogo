# Brief Cowork — recette navigateur, parcours de création complet

**Cible** : https://koprogo.com — API https://api.koprogo.com/api/v1
**Version** : `sha-7cf9d67a` (`feature/dev`, 2026-09-04)

**Un seul compte suffit. Tout le reste se crée depuis l'interface.**

```
admin@koprogo.com / admin123     (SuperAdmin)
```

> Ces identifiants sont déductibles du dépôt public : défaut connu, suivi en
> #763. Ne pas le rapporter.

---

## Le rôle à tenir

**Vous êtes l'administrateur SaaS qui s'apprête à faire entrer les premiers
bêta-testeurs.** Ce sont de vrais syndics et de vrais copropriétaires. Ils
vont tout voir : les écrans vides, les messages d'erreur, les libellés
approximatifs, les boutons qui ne font rien.

Cette recette a donc deux exigences superposées, et la seconde est la plus
exigeante :

1. **Est-ce que ça marche ?** — le fonctionnel, le droit, les montants.
2. **Est-ce montrable à un professionnel ?** — un syndic belge qui découvre
   l'outil doit comprendre où il est, quoi faire, et pourquoi on lui refuse
   quelque chose. S'il reste bloqué sans savoir pourquoi, c'est un défaut
   même si le code est juste.

**Tout ce qui choque l'œil compte** : un `TODO` affiché, un `Lorem ipsum`,
un texte anglais dans une interface française, une date au format
américain, un bouton sans effet, une page blanche, un écran vide sans
explication de ce qu'il faut faire pour le remplir.

## Principe de la recette

On ne vérifie pas des données existantes : **on bâtit une copropriété
complète depuis rien**, comme le ferait un bêta-testeur le premier jour, et
on regarde si chaque étape tient debout. Les données déjà présentes peuvent
être ignorées.

Le parcours suit l'ordre de la vie réelle d'une ACP : elle naît, se dote
d'un syndic, tient sa comptabilité, délibère, puis s'organise en communauté.

Trois domaines sont couverts : le **jumeau numérique du Code civil**
(Livre 3), le **noyau comptable** (AR du 12/07/2012), l'**économie
circulaire**.

**Nommer tout ce qui est créé avec un préfixe reconnaissable**, par exemple
`RECETTE-`, pour retrouver et nettoyer ensuite.

---

## Acte 0 — Ce que voit un bêta-testeur en arrivant

À faire **avant** de créer quoi que ce soit, une seule fois, l'œil neuf.

### 0.1 Les pages publiques

Ouvrir https://koprogo.com **sans être connecté**.

- La page d'accueil explique-t-elle ce qu'est le produit à un syndic qui ne
  connaît pas ?
- `/mentions-legales` et `/privacy-policy` : présentes, en français, sans
  mention manquante ? Ce sont des obligations, pas du décor.
- Les liens du pied de page mènent-ils quelque part ?
- Le site est-il utilisable **au téléphone** ? Un syndic consulte souvent
  depuis un chantier.

### 0.2 L'inscription

`/register` — s'inscrire comme un nouveau venu le ferait.

- Que se passe-t-il après l'inscription ? Où atterrit-on ?
- Le mot de passe a-t-il des exigences annoncées **avant** d'être refusé ?
- Un courriel déjà pris est-il signalé clairement ?
- Y a-t-il une confirmation par courriel ? Si oui, arrive-t-elle ?

### 0.3 Le premier écran d'un compte vide ★

C'est le moment qui décide si un bêta-testeur reste ou part.

Se connecter avec le compte fraîchement inscrit, **sans aucune donnée**.

- L'écran d'accueil dit-il **quoi faire en premier** ?
- Les listes vides affichent-elles « aucun élément » avec une invitation à
  créer, ou une zone blanche sans explication ?
- Y a-t-il des compteurs à `0`, des graphiques vides, des `NaN` ?
- Un bouton « créer » est-il visible, ou faut-il deviner l'URL ?

**Noter le nombre de clics nécessaires pour créer sa première copropriété.**
Si le chemin n'est pas évident, c'est le défaut le plus coûteux de tous.

### 0.4 La récupération de mot de passe

Un bêta-testeur oubliera son mot de passe. Le parcours existe-t-il ? Le
courriel part-il ?

### 0.5 L'aide

Y a-t-il une aide, une documentation, un contact ? Un syndic bloqué doit
savoir à qui s'adresser.

---

## Acte 1 — Naissance de la copropriété

### 1.1 Créer une organisation (le cabinet de syndic)

`/admin/organizations` en SuperAdmin.

**À vérifier** : les champs obligatoires, le numéro d'entreprise (BCE), et
qu'une organisation sans nom est refusée.

### 1.2 Créer un utilisateur syndic dans cette organisation

**À vérifier** : le rôle est bien `syndic`, l'utilisateur est rattaché à
l'organisation, et il peut se connecter.

> Garder ces identifiants : tout l'acte 2 se joue avec eux.

### 1.3 Créer l'ACP ★ le cœur du modèle

**Art. 3.85 § 1er** : l'ACP naît de l'acte de base. Elle est la personne
morale, distincte du syndic qui la gère.

**À vérifier** :
- Le formulaire demande bien un **total de tantièmes** (base de l'acte : 1000,
  10 000, ou autre). Il ne doit **pas** être codé en dur.
- **Art. 3.86 § 1er** : la personnalité juridique suppose deux conditions
  cumulatives — acte de base transcrit **et** naissance de l'indivision.
  L'écran en tient-il compte ?
- Le **numéro d'entreprise** est-il demandé ? L'**Art. 3.89 § 5, 5°** impose
  qu'il figure sur tous les documents émanant de l'ACP : le vérifier plus
  tard sur les convocations et appels de fonds.

### 1.4 Créer l'immeuble, puis les lots

**Art. 3.85 § 1er al. 2** : une ACP peut compter **plusieurs immeubles**
(« immeuble ou groupe d'immeubles »). Créer **deux** immeubles sous la même
ACP pour l'éprouver.

Puis les lots. **Créer les cinq natures** : `Appartement`, `Parking`, `Cave`,
`Local commercial`, `Autre`.

**À vérifier** :
- Les cinq natures sont proposées. *(Avant aujourd'hui il n'y en avait que
  trois, et éditer un lot commercial le basculait en appartement.)*
- **Éditer un lot commercial et le ré-enregistrer sans y toucher : sa nature
  doit être préservée.**
- La somme des quotes-parts doit atteindre le total déclaré. Tant qu'elle ne
  l'atteint pas, l'ACP est **non conforme** et les opérations financières
  seront refusées en **422** — c'est voulu.
- **Le message de refus est-il compréhensible ?** Un syndic doit savoir quoi
  corriger. C'est un point de jugement, pas de conformité.

### 1.5 Créer les copropriétaires et les rattacher aux lots

**À vérifier** :
- Un lot peut avoir **plusieurs** propriétaires (indivision) et la somme des
  parts doit faire 100 %.
- **Art. 3.86 § 3 al. 8** : usufruit et nue-propriété — les titulaires de
  droits réels sont **solidaires** des charges. Le modèle le permet-il ?

---

## Acte 2 — La comptabilité

> Bascule sur le compte **syndic** créé en 1.2. Tout ce qui suit doit être
> faisable sans repasser SuperAdmin.

### 2.1 Le régime comptable ★

**Art. 3.89 § 5, 15°** : comptabilité simplifiée sous **20 lots**, et
**caves, garages et parkings ne comptent pas** dans ce décompte.

**À éprouver** : créer 15 appartements, 15 caves et 20 parkings. Le décompte
légal doit valoir **15**, pas 50. L'ACP doit rester en régime simplifié.

### 2.2 Le plan comptable

Vérifier qu'un plan comptable (PCMN, AR du 12/07/2012) est disponible et que
les comptes sont utilisables à la saisie.

### 2.3 Le budget

Créer un budget ordinaire et un budget extraordinaire. Vérifier qu'ils sont
rattachés à l'**ACP** et non au cabinet.

### 2.4 Le fonds de réserve ★

**Art. 3.86 § 3 al. 4** : fonds de réserve **obligatoire** après cinq ans,
alimenté d'au moins **5 % des charges ordinaires** de l'exercice précédent.

**À vérifier** : l'écran l'expose-t-il ? Le plancher est-il calculé ?

### 2.5 L'appel de fonds ★

Créer un appel de fonds sur l'ACP conforme.

**Art. 3.86 § 3 al. 7** : la part affectée au **fonds de réserve** doit être
communiquée **lors de l'appel**. Le formulaire propose-t-il ce champ ?
Apparaît-il sur le document produit ?

**À vérifier** : la ventilation par lot au **prorata des tantièmes**, et que
la somme des parts égale **exactement** le montant appelé, sans centime
perdu à l'arrondi.

### 2.6 Les dépenses et leur répartition

Créer une dépense, la répartir. Même exigence sur l'exactitude à l'euro
près. Vérifier la TVA si l'écran la propose.

### 2.7 Les écritures comptables ★

**Scénario A** : créer une écriture manuelle **sans désigner d'immeuble**.

**Attendu** : refus en **400**, message expliquant qu'il faut un immeuble
pour déterminer l'ACP.

> C'était un **500** jusqu'à aujourd'hui. **Si un 500 apparaît ici, c'est une
> régression : signalez-la.**

**Scénario B** : créer une écriture déséquilibrée (débit ≠ crédit). Refus
attendu, message clair.

**Scénario C** : créer une écriture équilibrée et vérifier qu'elle apparaît
au grand livre de la bonne ACP.

### 2.8 Les soldes de comptes ★ POINT SENSIBLE

La vue des soldes était groupée par **organisation** : un cabinet gérant
cinq ACP voyait **un seul bilan pour les cinq**.

**À éprouver** : créer une **seconde ACP** dans la même organisation, avec
ses propres écritures. Les balances des deux ACP doivent être **séparées**,
et aucun total ne doit mélanger les deux.

### 2.9 État daté, relances

- **État daté** (Art. 3.89 § 9) : arriérés, fonds de réserve, procédures en
  cours. À produire pour une mutation.
- **Relances** : niveaux successifs, pénalités, mode d'envoi.

### 2.10 Les montants sont des Decimal ★

Traquer partout : `NaN`, `undefined`, `[object Object]`, un montant en
`1000.0000000001`, un arrondi qui perd des centimes.

Les quotes-parts doivent afficher `250/1000èmes`, ou **`—`** si elles
manquent. **Jamais `NaN`. Jamais `0`** — zéro voudrait dire « ce lot ne vote
pas », ce qui est une tout autre affirmation.

---

## Acte 3 — La vie de l'assemblée

### 3.1 Convoquer

**Art. 3.87 § 2** : l'AG se réunit dans la **période de quinze jours** fixée
au règlement (Art. 3.85 § 3, 3°).

**Art. 3.87 § 2** aussi : un cinquième des copropriétaires en quotes-parts
peut **requérir** une AG ; le syndic convoque sous **trente jours**.

**Art. 3.89 § 5, 5°** : le **numéro d'entreprise** de l'ACP doit figurer sur
la convocation. **Le vérifier sur le document produit.**

**À vérifier** : les délais d'envoi, le mode d'envoi accepté par écrit par
le destinataire, l'ordre du jour.

### 3.2 Le quorum

**Art. 3.87 § 5** : première convocation — plus de la moitié des
copropriétaires **et** au moins la moitié des quotes-parts. À défaut,
seconde AG après **quinze jours au moins**, qui délibère quel que soit le
nombre de présents.

### 3.3 Les procurations ★

**Art. 3.87 § 7** :
- Plus de **trois** procurations : refusé, **sauf** si le total des voix du
  mandataire et de ses mandants reste sous **10 %** du total des lots.
- Le **syndic** ne peut être mandataire d'un copropriétaire. Il vote pour son
  propre lot s'il est copropriétaire.

**À tester** : franchir chacun de ces plafonds et vérifier que le refus
**cite son article**, pour que le président de séance sache quoi corriger.

### 3.4 Le plafonnement des voix ★★ LE POINT LE PLUS NOUVEAU

> « Nul ne peut prendre part au vote, même comme **mandant** ou mandataire,
> pour un nombre de voix supérieur à la somme des voix dont disposent les
> autres copropriétaires présents ou représentés. »

Ce n'est **pas** un refus : c'est un **plafonnement**. La séance reste
valide, le décompte est corrigé.

**Scénario A — le majoritaire ne décide pas seul**
Un copropriétaire à 600 tantièmes vote **pour**, un autre à 400 vote
**contre**. Clore.

→ Les 600 sont ramenées à **400**. Décompte 400 contre 400. Majorité absolue
non atteinte : **résolution rejetée**.

**Scénario B — le plafond joue même à l'unanimité**
Les deux votent **pour** (600 et 400). Clore.

→ Total retenu **800**, pas 1000. Résolution adoptée.

**Scénario C — contournement par procurations éclatées**
Un copropriétaire majoritaire confie ses lots à **trois mandataires
différents**, pour qu'aucun ne dépasse le seuil isolément.

→ Il doit **quand même** être plafonné : le texte vise « même comme
**mandant** ». C'est un montage tenté en pratique et jugé non conforme.

**Scénario D — votant unique**
Un seul votant présent ne doit **pas** être ramené à zéro.

**À juger, et c'est le plus utile** :
- Un syndic comprend-il pourquoi le décompte diffère de la somme des voix ?
- L'écran montre-t-il le décompte **retenu**, le **brut**, ou les deux ?
- *(La trace du plafonnement existe en base, `resolutions.voix_plafonnees`,
  mais aucun écran ne l'explique. Dire si ça manque.)*

### 3.5 Les majorités

**Art. 3.88** : majorité absolue, deux tiers, quatre cinquièmes, unanimité
selon l'objet. **Art. 3.87 § 8** : abstentions, votes blancs et nuls **ne
comptent pas** dans le calcul.

**À tester** : une résolution avec des abstentions — vérifier qu'elles sont
exclues du dénominateur.

### 3.6 Le conflit d'intérêts

**Art. 3.87 § 9** : un prestataire ne délibère pas sur sa propre mission.
Désigner un prestataire sur une résolution, le faire voter, clore.

### 3.7 Le contrat avec le syndic ou ses proches

**Art. 3.89 § 5, 13°** : un contrat entre l'ACP et le syndic, ou une société
liée, exige l'autorisation **préalable** de l'AG. Vérifier que l'antériorité
est contrôlée.

### 3.8 Le procès-verbal

**Art. 3.87 § 10 et § 12** : signatures, inscription au **registre**, et
transmission sous **trente jours**.

### 3.9 Conseil de copropriété et commissaire aux comptes

**Art. 3.90** et **Art. 3.91** : élection, composition, compétences.

---

## Acte 4 — La communauté et l'économie circulaire

### 4.1 Campagnes d'énergie groupée

`/energy-campaigns` — créer une campagne, ajouter des offres fournisseur.

**Attention aux types** : prix du kWh et redevance mensuelle sont des
**montants**, le pourcentage d'énergie verte est un **affichage**. Aucun des
trois ne doit produire `NaN`.

Vérifier le classement des offres, l'économie estimée, et qu'une offre ne
s'ajoute qu'en statut **négociation**.

### 4.2 Partage, échanges, place de marché

`/sharing`, `/exchanges`, `/marketplace` — proposer, réserver, rendre.

### 4.3 Réservation de ressources communes

`/bookings` — créneaux, conflits de réservation, annulation.

### 4.4 Compétences et entraide

`/skills` — proposer, chercher, solliciter.

### 4.5 Tickets, sondages, annonces

`/tickets`, `/polls`, `/notices` — cycle de vie complet.

### 4.6 Gamification

`/gamification` — points, badges, classement.

---

## Acte 5 — Le cloisonnement ★★ LE TEST QUI COMPTE LE PLUS

C'est le défaut de fond corrigé cette semaine : **le dossier de gestion
appartenait au syndic, pas à la copropriété.**

**Art. 3.89 § 5, 7°** : à la fin de son mandat, le syndic transmet
**l'ensemble du dossier** sous trente jours.

### Le scénario de passation

1. Créer une **seconde organisation** (cabinet B) et un **second syndic**.
2. Lui confier l'ACP créée à l'acte 1.
3. Se connecter en **syndic B**.

**Attendu** : il voit **tout** — budgets, appels de fonds, écritures,
assemblées, procès-verbaux, copropriétaires, arriérés.

4. Se reconnecter en **syndic A**.

**Attendu** : il ne voit **plus rien** de cette ACP.

### Où chercher les fuites

Le cloisonnement doit tenir partout, pas seulement sur les listes
principales :

- Compteurs et graphiques du **tableau de bord**
- **Recherches** globales et filtres
- **Exports** (PDF, Excel, CSV)
- **Notifications**
- **Classements** de gamification
- Objets partagés, réservations, sondages
- URLs devinées : ouvrir une pièce de l'autre ACP par son identifiant direct

**Toute fuite, même un simple compteur, est un défaut majeur.**

---

## Acte 6 — Le métier d'administrateur SaaS

C'est votre rôle propre : faire entrer des bêta-testeurs et garder la main.

### 6.1 Faire entrer un testeur

Depuis le compte SuperAdmin, faire entrer un nouveau syndic dans une
organisation.

- Existe-t-il une **invitation**, ou faut-il créer le compte et transmettre
  un mot de passe en clair par un autre canal ?
- Peut-on rattacher un utilisateur existant à une organisation ?
- Que voit le testeur à sa première connexion : son organisation est-elle
  déjà peuplée, ou repart-il d'un écran vide ?

### 6.2 Voir ce que voit le testeur

Un administrateur SaaS doit pouvoir diagnostiquer un problème rapporté.

- Peut-on **consulter** l'espace d'une organisation sans en usurper le
  compte ? Si oui, est-ce tracé ?
- Si la seule façon d'aider un testeur est de se connecter à sa place avec
  son mot de passe, **le dire** : c'est un manque structurant pour une bêta.

### 6.3 Reprendre la main

- Désactiver un compte, changer un rôle, retirer un accès.
- Supprimer une organisation de test : que deviennent ses données ? Le
  refus est-il expliqué s'il y a des dépendances ?

### 6.4 Les limites et les quotas

Y a-t-il une limite au nombre d'ACP, d'immeubles, d'utilisateurs par
organisation ? Si oui, le message est-il clair quand on l'atteint ?

### 6.5 Ce qu'un testeur ne doit jamais voir

- Une **trace technique** : `panicked at`, un chemin `src/...`, un nom de
  table, un identifiant SQL, une pile d'appels.
- Le **contenu d'une autre organisation**, sous quelque forme que ce soit.
- Un message d'erreur en **anglais** dans une interface en français.

Tout écran qui expose l'un de ces trois est à signaler en priorité : ce
n'est pas seulement inélégant, ça renseigne un attaquant.

---

## Acte 7 — Transverse

### 7.1 Le sélecteur de périmètre ★ NOUVEAU

Les immeubles sont désormais **groupés sous le nom de leur ACP**. Avec deux
immeubles sous une même ACP (acte 1.4), le groupement doit être visible.

Un immeuble sans ACP identifiable doit apparaître dans un groupe « ACP
inconnue », **et non être masqué**.

Ce sélecteur est présent sur **27 pages** : en tester quelques-unes de
familles différentes.

### 7.2 Multilingue

`fr`, `en`, `nl`, `de`. Basculer et traquer les clés brutes affichées telles
quelles, du genre `units.commercial` au lieu de « Local commercial ».

### 7.3 RGPD

Export et effacement des données. **L'auto-effacement exige le mot de
passe** : c'est la seule preuve que la demande vient de la personne et non
d'un jeton volé.

### 7.4 Rôles

Rejouer les écrans clés avec chaque rôle créé : syndic, comptable,
copropriétaire. Un copropriétaire ne doit pas atteindre les écrans syndic ;
un comptable ne doit pas modifier la gouvernance.

---

## Ce qu'il faut rapporter

**Toujours** :
- Tout **500** — c'est toujours un défaut
- Toute **fuite entre ACP ou organisations**, même un compteur
- Tout `NaN`, `undefined`, `[object Object]`, clé de traduction brute
- Tout écart de centime dans une répartition
- Tout refus **incompréhensible** pour un syndic — même si le refus est juste
- Toute étape du parcours **impossible à faire depuis l'interface** et qui
  exigerait de passer par l'API

**Format** : l'URL, le compte utilisé, ce qui était attendu, ce qui s'est
produit, une capture, et le code HTTP si visible.

**Ne pas rapporter** : les identifiants faibles (#763, connu).

---

## Le verdict attendu à la fin

Au-delà de la liste d'anomalies, répondre à trois questions. Ce sont elles
qui décident si la bêta peut s'ouvrir.

1. **Un syndic belge pourrait-il gérer une vraie copropriété avec ça
   aujourd'hui ?** Si non, qu'est-ce qui manque en premier ?
2. **Où s'est-on trouvé bloqué sans comprendre pourquoi ?** Ce sont les
   points qui feront abandonner un bêta-testeur, bien avant les bugs.
3. **Qu'est-ce qui donnerait mauvaise impression à un professionnel ?** Un
   libellé approximatif, une erreur en anglais, un montant mal formaté, un
   écran vide sans explication.

Un compte rendu qui répond à ces trois questions vaut mieux qu'une liste de
cinquante anomalies mineures.

---

## Une incohérence déjà repérée, à confirmer

En modifiant un immeuble par l'API, la réponse du `PUT` renvoie
`units_count: 0`, `quota_sum: "0"`, `is_conformant: false`, alors qu'un
`GET` immédiat renvoie les vraies valeurs. La réponse de modification ne
recalcule pas les agrégats.

**À confirmer depuis l'interface** : après édition d'un immeuble, l'écran
affiche-t-il brièvement « non conforme » à tort ?
