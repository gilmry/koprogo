# Brief Cowork — recette 3, après corrections du 6 septembre 2026

**Version à tester** : `sha-1d79a9a` et suivantes, sur https://koprogo.com
**Vos deux rapports précédents** : recette du 4 septembre (`sha-7cf9d67a`), retest du 6 septembre
**Rédigé le** : 2026-09-06

---

## Méthode : recette humaine, au navigateur

**Tout se fait au navigateur, par de vrais clics et de vraies saisies, avec une capture
d'écran à l'appui.** Ce qu'on cherche à savoir, c'est ce qu'un syndic vit devant son
écran, pas ce que l'API sait faire.

- Ne créez **aucune** donnée par appel d'API. Si un écran ne permet pas de faire quelque
  chose, **c'est cela, le constat** — et il disparaîtrait si vous contourniez par l'API.
- Une capture d'écran par constat, avant et après l'action quand c'est utile.
- L'API sert **uniquement au diagnostic** : expliquer un comportement déjà observé à
  l'écran. Jamais à piloter l'application ni à préparer un jeu de données.

Les vérifications décrites plus bas citent parfois des noms de champs d'API. C'est pour
que vous sachiez quoi regarder si un chiffre affiché vous paraît faux, pas pour que vous
les appeliez à la place de l'interface.

---

## Avant tout : merci, et deux mises au point

Vos deux rapports ont trouvé de vraies choses, dont deux que personne n'avait vues et
qui sont maintenant corrigées. Ce brief commence pourtant par corriger **trois de vos
conclusions**, parce que la vérification a montré autre chose que ce que les symptômes
suggéraient. Ce n'est pas un reproche : dans les trois cas, ce que vous observiez était
exact, c'est l'explication qui ne l'était pas. Et dans deux cas sur trois, la cause
réelle était plus grave que celle que vous aviez supposée.

### 1. Il n'y a jamais eu de crash API, ni de régression CORS

**Vos constats** : R5-4 « API crash pendant les tests, totalement inaccessible, y
compris `/health` », et RN-1 « après environ 10 requêtes, l'API cesse de retourner les
headers CORS, le serveur répond toujours en serveur-à-serveur ».

**Ce qui se passait réellement** : le pare-feu applicatif du serveur, CrowdSec, **a banni
votre adresse IP pendant 4 heures**. Trace du 6 septembre à 07:21:51 :

```
Reason       : crowdsecurity/http-probing
Events Count : 12
Scope:Value  : Ip:159.26.118.22   (Belgique, Proton AG)
```

Les 12 événements sont douze `404` d'affilée sur les modules communautaires de votre
ACP de recette. Votre rapport dit « ~12 requêtes en ~60 secondes » ; la fenêtre CrowdSec
fait 32 secondes. C'est le même événement.

Le mécanisme est vicieux, et il vous piégeait sans que rien ne le signale :

> vous ouvrez le menu Communauté → l'application appelle huit endpoints qui n'existent
> pas → huit `404` en quelques secondes → CrowdSec conclut à un scanner → bannissement →
> **tout** le navigateur est coupé, `/health` compris → vous concluez au crash serveur.

Un contrôle depuis une autre machine répondait `200`, ce qui est exactement ce que vous
aviez observé et correctement noté. C'est notre propre incomplétude qui vous faisait
passer pour un attaquant.

**Ce qui a été fait** : votre adresse VPN est désormais en liste blanche CrowdSec, et le
bannissement en cours a été levé. Suivi en **issue #766**.

**Ce que ça change pour vous** : R5-4 et RN-1 sont à retirer de vos rapports. Et R5-5,
que vous aviez classé « bloqué par le crash », est **rejouable**. C'est même la priorité
n° 1 ci-dessous.

### 2. Les boutons fonctionnent : `onclick: null` est normal en Svelte 5

**Votre constat** : R7-1, « problème systémique — les boutons avec onclick ne
fonctionnent pas, `onclick: null` sur le DOM ».

**La vérification**, faite au navigateur avec Playwright sur la production :

| Bouton | `el.onclick` | Effet du clic |
|---|---|---|
| « + New budget » (`/budgets`) | `null` | **le formulaire s'ouvre**, DOM +2953 octets |
| « Edit » (`/syndic`) | `null` | DOM +1879 octets |
| Bascule de langue | `null` | DOM +1882 octets |

Aucune erreur JavaScript. Capture d'écran à l'appui : le formulaire « New budget »
complet s'affiche après le clic.

**Pourquoi `onclick` est `null`** : Svelte 5 n'attache pas les gestionnaires à la
propriété DOM `onclick`. Il utilise la **délégation d'événements**, un écouteur unique à
la racine qui route les clics. La propriété `onclick` d'un bouton parfaitement
fonctionnel vaut donc `null` **par construction**. Inspecter cette propriété ne dit rien
de l'état d'hydratation.

**Méthode à retenir pour la suite** : ne concluez jamais qu'un bouton est mort en
inspectant `onclick`. **Cliquez** et regardez si quelque chose bouge : URL, contenu du
DOM, requête réseau. C'est le seul test valide.

Cela dit, si un bouton précis ne réagit pas à un clic réel chez vous, c'est un vrai
défaut et il faut le rapporter — avec le nom du bouton, la page, et ce que vous
attendiez.

### 3. Le plafonnement Art. 3.87 était implémenté, mais invisible

**Vos constats** : R3-1 et R3-2, « vote capping non implémenté », Alice à 550 puis 800
au lieu de 450 puis 200.

**Ce que vous voyiez était juste.** Mais le plafonnement était bien codé, testé, et
correct — appliqué **uniquement au moment de la clôture du vote**. Comme rien ne
déclenche la clôture automatiquement (votre R3-3), et que le bouton de clôture était
inatteignable pour un syndic non copropriétaire, vous n'y arriviez jamais. L'API vous
servait donc le décompte brut pendant toute la séance.

**Ce qui a été corrigé aujourd'hui** : le décompte est maintenant plafonné **à la
lecture**, avant même la clôture. Voir la vérification n° 2 ci-dessous. Suivi en
**issue #767**.

---

## Ce qui a changé depuis votre base `sha-7cf9d67a`

Quatre livraisons. Ne perdez pas de temps à retester ce qui suit sans raison, mais
**vérifiez-le une fois** pour confirmer.

| Votre référence | Ce qui a été fait |
|---|---|
| R3-4 pourcentages par tête | Corrigé — vous l'aviez déjà confirmé |
| R3-1 / R3-2 plafonnement | Décompte plafonné dès la lecture |
| R1-1 / R1-2 syndic ne crée ni immeuble ni lot | Ouvert au syndic, avec cloisonnement par ACP |
| R2-3 à R2-5 champs perdus en silence | Un champ mal nommé rend maintenant `400` au lieu d'être jeté |
| R1-4 register bascule la session | L'appelant authentifié garde sa session |
| R4-3 modules communautaires | Les 6 sans backend sont retirés du menu |
| R0-2 pas de lien d'inscription | Deux points d'entrée ajoutés sur l'accueil |
| R0-4 mot de passe oublié | Page honnête : la fonction n'existe pas, voici le recours |
| R5-4 / RN-1 | Faux positifs, voir plus haut |
| R7-1 | Faux positif, voir plus haut |

---

## Ce que je vous demande de tester, par ordre de valeur

### 1. R5-5 — les tests IDOR inter-organisations ★ PRIORITÉ

C'est le test le plus important du produit et il n'a **jamais** pu aller au bout. Vos
actes 5 précédents montraient un cloisonnement correct sur les ACP, immeubles, lots et
copropriétaires. Restent non testés : **résolutions, votes, dépenses, budgets, appels de
fonds, états datés**.

Méthode, au navigateur : connectez-vous en Grand Place, puis **collez dans la barre
d'adresse** l'URL d'une page RECETTE relevée pendant votre session précédente — la fiche
d'une résolution, d'une dépense, d'un budget, d'un appel de fonds, d'un état daté. C'est
exactement ce que ferait un utilisateur curieux ayant reçu un lien.

Capture d'écran de ce que vous obtenez. Ce qu'on attend : un refus explicite et lisible.
Ce qui serait grave : voir la donnée s'afficher. Ce qui serait mauvais aussi : une page
blanche ou une erreur technique brute, parce que l'utilisateur ne saurait pas s'il est
en faute ou si l'application est cassée.

**Testez aussi l'écriture, et par l'interface.** Depuis Grand Place, essayez d'ajouter
une dépense en sélectionnant un immeuble RECETTE dans les listes déroulantes, ou de voter
sur une résolution RECETTE atteinte par URL. Une lecture cloisonnée avec une écriture
ouverte est un défaut classique et grave.

Si le sélecteur d'immeuble ne propose pas les immeubles de l'autre organisation, c'est
déjà une bonne nouvelle, et il faut le dire. Ouvrez alors la console réseau du navigateur
pour **expliquer** ce que vous avez vu : c'est le seul usage de l'API ici.

### 2. Le plafonnement, maintenant visible ★

Rejouez exactement votre scénario : Alice 550 ‰ « pour », Bob 250 ‰ « contre »,
Claire 200 ‰ « contre ».

**Sans clôturer la résolution**, faites `GET` sur la résolution. Vous devez voir :

- `total_voting_power_pour` = **450**, pas 550
- `total_voting_power_contre` = 450
- `pour_percentage` = **50.0**

Puis le scénario procuration : Alice 550 propres + procuration de Bob 250. Elle doit
apparaître à **200**, la seule voix restante (Claire), pas à 800.

Si vous voyez encore 550 ou 800, c'est un vrai défaut et il faut le dire fort.

### 3. R3-3 — la finalisation des résolutions

Toujours ouvert. L'endpoint de clôture **existe** (`close_voting`), contrairement à ce
que disait votre rapport, mais rien ne l'appelle automatiquement.

À tester : après avoir voté, le syndic peut-il clôturer depuis l'interface ? Le bouton
est-il visible pour un syndic **qui n'est pas copropriétaire** ? C'était le blocage
d'origine, corrigé le 4 septembre, jamais vérifié au navigateur.

### 4. R2-1 et R1-7 — la conformité qui bloque la comptabilité

Toujours ouvert, et c'est un piège pour tout syndic réel. `total_units` est saisi à la
création de l'immeuble et **jamais recalculé** quand on ajoute des lots. Si les deux
diffèrent, toute la comptabilité se ferme.

À caractériser précisément, parce que la correction dépend de votre réponse :

- Quelles opérations exactement sont bloquées ? Créer une dépense ? Une facture ? Un
  budget ? Une écriture comptable ? Un appel de fonds ?
- Le message d'erreur dit-il maintenant **combien** de lots et de millièmes manquent ? Il
  a été enrichi le 4 septembre, personne ne l'a vu à l'écran depuis.

Suivi en **issue #770**, où trois voies sont proposées. Votre observation tranchera.

### 5. Le reste de vos findings, non traités

Non corrigés, à confirmer et à préciser : R0-3 (pas de confirmation email), R0-6
(premier écran vide affiche une erreur technique), R0-7 (pas d'aide), R1-3 (pas de bouton
Ajouter dans le détail d'organisation), R1-5 (`ownership_percentage` en 0-100 alors que
les tantièmes sont en millièmes), R2-2 (dates simples rejetées), R3-5 (statut meeting
incohérent), R3-6 (enums incohérents), R3-7 (footer cite les articles pré-2020).

**R7-2, l'i18n, mérite d'être requalifié à la hausse.** Vous le classiez « modéré, pages
communautaires en anglais ». Sur la capture prise aujourd'hui, la barre latérale entière
est en anglais — « Expenses », « Invoice workflow », « Budgets », « Property statements »,
« Journal entries » — de même que les formulaires : « New budget », « Fiscal Year »,
« Ordinary budget », « Close ». Seuls les titres de page sont en français. Pour un
produit destiné à des syndics belges francophones, ce n'est pas cosmétique.

**R0-5 / R7-3 est confirmé** : le lien RGPD du pied de page pointe vers `/settings/gdpr`,
une route authentifiée. Un visiteur ne peut pas lire la politique de confidentialité,
ce qui est un manquement en soi. Aucune page publique de confidentialité n'existe :
`/privacy` et `/confidentialite` retombent sur l'accueil.

---

## Comment tester, concrètement

### Si l'application devient injoignable

**Ce n'est probablement pas un crash.** Avant de conclure quoi que ce soit :

1. Notez l'heure précise et ce que vous faisiez.
2. Vérifiez si le site répond depuis un autre réseau, ou demandez le contrôle serveur.
3. Signalez-le en donnant l'heure : la trace CrowdSec du serveur tranchera en une minute.

Votre adresse VPN est en liste blanche, donc cela ne devrait plus arriver. Mais **Proton
VPN change d'adresse de sortie** : si vous changez de serveur VPN en cours de session, la
nouvelle adresse n'est pas couverte. Dites-le si ça se produit.

### Pour juger si un élément d'interface fonctionne

Cliquez. Regardez si l'URL change, si le contenu bouge, si une requête part. N'inspectez
pas les propriétés DOM des gestionnaires d'événements, elles ne veulent rien dire avec
un framework moderne.

### Pour qualifier un défaut d'API

Donnez toujours ces trois choses, sans quoi le constat n'est pas reproductible :

1. La **requête** complète : méthode, chemin, corps.
2. Le **code HTTP** et le corps de la réponse.
3. Ce que vous **attendiez**, et pourquoi.

Distinguez bien `401` (il faut s'authentifier), `403` (interdit, et c'est souvent le bon
comportement), `404` (la route n'existe pas) et `500` (l'application a échoué). Cette
distinction change complètement le diagnostic. Un `404` sur une route d'API veut souvent
dire « pas encore implémenté », pas « bug ».

### Comptes de recette

Ceux de votre rapport du 4 septembre restent valables (mot de passe `Recette2026!`).
Signalez-le s'ils ne fonctionnent plus.

---

## Ce que j'attends de votre rapport

- **Séparez ce que vous avez observé de ce que vous en déduisez.** Vos deux rapports
  mélangeaient les deux, et c'est ce qui a produit les trois faux diagnostics ci-dessus.
  Écrivez « l'API a cessé de répondre à 07:21 » plutôt que « l'API a crashé ».
- **Une sévérité argumentée.** « Critique » doit vouloir dire qu'un utilisateur réel
  perd des données, de l'argent, ou se trouve en infraction. Le reste est majeur ou
  moindre.
- **Dites ce que vous n'avez pas pu tester**, et pourquoi. C'est aussi utile que le
  reste.
- **Dites ce qui marche.** Vos sections PASS sont précieuses : elles nous disent ce qu'on
  peut montrer sans crainte.

Merci. Les deux découvertes les plus utiles de vos rapports — la perte silencieuse des
lignes de facture, et le décompte des voix par tête — n'auraient été trouvées par aucun
test automatique. C'est exactement ce qu'on attend d'une recette humaine.
