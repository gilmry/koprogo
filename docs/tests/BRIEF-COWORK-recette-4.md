# Brief Cowork — recette 4, vérification des corrections du 6 septembre

**Rédigé le** : 2026-09-06, après votre rapport de recette 3
**À tester sur** : https://koprogo.com, une fois la version annoncée déployée
**Vos rapports précédents** : recette du 4 septembre, retest du 6 septembre matin, recette 3 du 6 septembre

---

## La méthode, et elle n'est pas négociable

**Tout se fait au navigateur, par de vrais clics, de vraies saisies, et une capture d'écran par constat.**

- **Cliquez réellement.** Pas de requête forgée, pas d'appel d'API pour déclencher une action, pas de navigation programmatique là où un utilisateur cliquerait.
- **Capturez.** Une image par constat, et deux quand l'action a un avant et un après. Une capture vaut mieux qu'une description : elle montre ce que l'utilisateur voit, y compris ce que vous n'aviez pas remarqué.
- **Saisissez comme un humain.** Remplissez les champs, choisissez dans les listes, cochez les cases. Si un écran ne permet pas de faire quelque chose, **c'est cela le constat** — et il disparaîtrait si vous contourniez par l'API.
- **L'API sert uniquement au diagnostic.** Elle explique un comportement déjà vu à l'écran. Elle ne le provoque jamais et ne prépare aucune donnée.

Cette règle a une raison précise. Sur votre recette 3, une partie des constats venait de sondages d'API sur des chemins devinés. C'est ce qui a produit le finding R4-3 — « 7 modules communautaires sur 8 sans backend » — qui s'est révélé **entièrement faux**, et c'est aussi ce qui vous a fait bannir. Voir plus bas.

---

## Trois corrections à vos rapports, dont une qui m'est imputable

### R4-3 était faux, et ma première correction l'était aussi

Vous aviez conclu que sept modules communautaires sur huit n'avaient pas de backend. J'ai d'abord confirmé en sondant `/api/v1/sel`, `/api/v1/skills` et consorts, et j'ai retiré six entrées du menu.

**Nous avions tort tous les deux, et pour la même raison : nous sondions des chemins que personne n'appelle.**

Vérification faite au navigateur le 2026-09-06, en visitant les huit pages comme un utilisateur :

| Page | Appels API | Réponses non-2xx |
|---|---|---|
| Annonces | 10 | 1 |
| Compétences | 9 | 0 |
| Partage | 9 | 0 |
| Réservations | 9 | 1 |
| SEL | 12 | 1 |
| Gamification | 16 | 0 |
| Sondages | 9 | 0 |
| Énergie | 7 | 0 |

**Les huit pages fonctionnent.** Le serveur sert 111 points d'entrée pour ces modules, tous enregistrés. J'ai annulé mon retrait du menu, qui aurait enterré du code qui marche.

Les 404 de votre bannissement — `/acps/{id}/sel`, `/acps/{id}/community/skills` — ne sont émis par **aucune page de l'application**. Ils venaient de votre propre exploration d'API. D'où la règle ci-dessus.

### Le crash API et la régression CORS n'ont jamais existé

CrowdSec avait banni votre adresse 4 heures, sur douze 404 en trente-deux secondes. Votre adresse VPN est maintenant en liste blanche et le bannissement a été levé.

Attention : **Proton VPN change d'adresse de sortie**. Si vous changez de serveur en cours de session, la nouvelle adresse n'est pas couverte. Dites-le si l'application redevient injoignable, avec l'heure précise.

### `onclick: null` est normal en Svelte 5

Vérifié au navigateur : trois boutons dont la propriété `onclick` vaut `null` ouvrent bien leur formulaire au clic. Svelte 5 délègue les événements à un écouteur unique ; la propriété DOM est donc vide **par construction**. Ne concluez jamais à un bouton mort en l'inspectant : cliquez.

---

## Ce qui a été corrigé et que je vous demande de vérifier

### 1. Le décompte des voix à l'écran ★ PRIORITÉ

**Ce qui était faux** : l'écran affichait « Pour 1 vote (33,3 %) / Contre 2 votes (66,7 %) » quand l'API renvoyait 55 % et 45 %. Le panneau recalculait les pourcentages par nombre de bulletins.

**À vérifier** : rejouez votre scénario. Alice 550 ‰ pour, Bob 250 ‰ contre, Claire 200 ‰ contre.

L'écran doit maintenant afficher, pour chaque camp, **le nombre de votes, les millièmes, et le pourcentage** — par exemple « 1 vote · 450 ‰ (50,0 %) ». Et la barre de progression doit suivre les millièmes, pas les têtes.

Deux points d'attention :

- **Les millièmes affichés sont plafonnés.** Alice pesant 550 doit apparaître à **450**, la somme des autres présents (Art. 3.87 § 7). Si vous voyez 550, c'est un vrai défaut.
- **Les trois pourcentages ne font pas 100, et c'est voulu.** « Pour » et « contre » se rapportent aux voix exprimées, abstentions exclues (Art. 3.87 § 8) ; l'abstention se rapporte à toutes les voix présentes. Ne le rapportez pas comme une erreur de calcul.

Testez aussi le cas procuration : Alice 550 propres plus la procuration de Bob 250 doit apparaître à **200**, pas 800.

### 2. La création d'annonce

**Ce qui se passait** : le serveur créait bien l'annonce et rendait 201. Mais la page se rechargeait entièrement, ce qui détruisait le message de succès avant qu'il soit lisible, et le filtre revenait à « publiées seulement » — or une annonce naît **en brouillon**. Trois défauts derrière un seul symptôme.

**À vérifier** : créez une annonce. Vous devez voir le message de succès, la fenêtre se fermer, et **l'annonce apparaître immédiatement**, la liste basculant sur les brouillons.

Vérifiez ensuite qu'elle est bien là après un rechargement manuel, en sélectionnant « Brouillon » dans le filtre de statut.

### 3. Les libellés du formulaire d'annonce

**Ce qui était faux** : le formulaire affichait `LostAndFound` et `ClassifiedAd`, les valeurs internes, là où le filtre affichait « Objets trouvés » et « Petite annonce ». La catégorie affichait `General`, `Maintenance`… Et le libellé du champ titre disait « Annonces * » au lieu de « Titre * ».

**À vérifier** : les deux listes déroulantes du formulaire et le libellé du champ titre, dans la langue de votre session.

### 4. La page RGPD

**Ce qui était faux** : quatre intitulés de droits sur cinq et tous les boutons d'action étaient en anglais, dont « Request Data Erasure » qui déclenche un effacement irréversible.

**À vérifier** : la page entière dans votre langue. Vingt-neuf libellés ont été traduits en français, anglais, néerlandais et allemand, avec la terminologie officielle du RGPD. Le bouton d'effacement annonce désormais explicitement que l'action est irréversible.

Basculez la langue et revérifiez : c'est le seul écran où la langue engage la conformité.

### 5. Le lien RGPD du pied de page

**Ce qui était faux** : il pointait vers `/settings/gdpr`, une route authentifiée. Un visiteur non connecté ne pouvait pas lire la politique de confidentialité.

**À vérifier**, **déconnecté** : le lien « Confidentialité & RGPD » du pied de page doit ouvrir une page lisible sans compte.

### 6. Les références légales

« Articles 577-7 et suivants » et « Art. 577-8/1 » étaient la numérotation d'avant la réforme de 2020. Corrigés en 3.88 et 3.90, dans les quatre langues.

**Non corrigé, et je vous le dis** : « Article 577-8/4 § 4 » sur la page Sondages. Je ne connais pas sa correspondance avec certitude, et une référence légale fausse est pire qu'une référence périmée. Si vous savez, dites-le.

### 7. Boutons de création d'immeuble et de lot

Vous aviez reclassé R1-1 « non fait », à juste titre : j'avais ouvert la route serveur au syndic sans toucher à l'interface, où le bouton restait réservé au SuperAdmin.

**À vérifier**, connecté comme syndic : le bouton de création doit être visible sur `/buildings`, et sur la liste des lots d'un immeuble.

**Attention** : les boutons de **modification** et de **suppression** restent réservés au SuperAdmin, délibérément. Les routes serveur correspondantes n'ont pas encore de contrôle de périmètre ; les ouvrir exposerait des écritures entre organisations. Ne le rapportez pas comme une incohérence.

### 8. La fuite entre organisations ★ PRIORITÉ

C'est votre trouvaille la plus importante, RN-2. `GET /meetings/{id}/resolutions` et `GET /resolutions/{id}/votes` rendaient 200 sur les données d'une autre organisation. Les deux gestionnaires ne demandaient **aucune identité**.

Les deux routes sont fermées, avec deux tests automatisés qui lisent depuis une organisation étrangère.

**À vérifier au navigateur** : connecté en Grand Place, collez dans la barre d'adresse l'URL d'une page RECETTE relevée lors de votre session précédente. Vous devez obtenir un refus lisible, jamais la donnée.

**L'audit qui suit est plus inquiétant que la fuite.** Sur 310 routes imbriquées, **73 restent sans identité** : paiements, documents, états datés, convocations, avis, répartitions de charges. Elles sont suivies dans l'issue #772. Si votre exploration en croise une, signalez-la avec l'URL exacte.

---

## Ce qui n'est PAS corrigé, et que je ne vous demande pas de retester à blanc

Autant le dire pour que vous n'y perdiez pas de temps :

| Votre réf | Sujet | Issue |
|---|---|---|
| RN-8 / R3-3 | Bouton « Clôturer le vote » sans effet | #776 |
| RN-7 | Filtre « Brouillon » en chargement infini | #775 |
| R2-1 / R1-7 | `total_units` jamais recalculé, bloque la comptabilité | #770 |
| R3-5 | Statut d'AG incohérent malgré le quorum validé | ouvert |
| R3-6 | Énumérations d'API incohérentes | ouvert |
| R1-5 | `ownership_percentage` en 0-100 quand les tantièmes sont en millièmes | ouvert |
| R2-2 | Dates simples rejetées | ouvert |
| R0-3, R0-6, R0-7, R1-3 | Confirmation email, premier écran vide, aide, boutons d'ajout | ouverts |

En revanche, **si l'une d'elles se comporte différemment de ce que vous aviez noté**, dites-le : cela signifierait qu'une correction a eu un effet de bord.

---

## Ce que je vous demande d'explorer en plus

### Le cycle de vie complet d'une assemblée

Vous aviez montré que le bouton de clôture est visible pour un syndic non copropriétaire, mais sans effet. Reprenez le parcours entier, de la convocation à la clôture de l'AG, et **dites précisément où il casse**. C'est le seul parcours du produit qui ne peut pas aller à son terme, et il bloque la vérification du plafonnement à la clôture.

### Les huit pages communautaires, pour de bon

Elles fonctionnent, mais personne ne les a exercées à la main. Créez une annonce, une offre de compétence, un objet partagé, une réservation. **Cliquez, saisissez, capturez.** C'est le module qui distingue KoproGo, et il n'a jamais été éprouvé par un humain.

Deux 404 que j'ai relevés au passage et que vous croiserez peut-être : `bookable-resources/available` n'existe pas côté serveur, sur la page Réservations. `/owners/me` rendait 404 pour un syndic non copropriétaire, c'est corrigé.

### Ce qui marche

Continuez à le dire. Vos sections PASS sont ce qui permet de savoir ce qu'on peut montrer sans crainte, et elles ont plus de valeur que vous ne le pensez.

---

## Ce que j'attends de votre rapport

- **Une capture par constat.** C'est la règle principale.
- **Séparez l'observation de la déduction.** Écrivez « l'écran affiche 33,3 % » puis, à part, « j'en déduis que le calcul se fait par tête ». Trois faux diagnostics de vos rapports précédents venaient de ce mélange, et à chaque fois l'observation était juste.
- **Distinguez les codes HTTP** quand vous les citez : `401` il faut s'authentifier, `403` c'est interdit et souvent correct, `404` la route n'existe pas, `500` l'application a échoué.
- **Dites ce que vous n'avez pas pu tester, et pourquoi.**

Merci. Vos trois rapports ont trouvé une fuite de données réelle, une perte silencieuse de lignes de facture et un décompte de voix contraire au droit belge. Aucun de ces trois défauts n'aurait été trouvé par un test automatique.
