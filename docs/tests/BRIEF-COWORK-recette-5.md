# Brief Cowork — recette 5

**Rédigé le** : 2026-09-06, après votre recette 4
**À tester sur** : https://koprogo.com, une fois la version annoncée déployée

---

## La méthode, inchangée

**Tout au navigateur, par de vrais clics et de vraies saisies, une capture d'écran par constat.** L'API sert au diagnostic, jamais à provoquer une action ni à préparer une donnée.

Votre recette 4 a suivi cette règle et c'est ce qui l'a rendue utile : les trois verrous du cycle de vie d'une AG ne pouvaient être trouvés qu'en cliquant.

**Sur les captures.** Vous nous dites ne pas pouvoir les joindre au document. Votre transcription mot pour mot des écrans a parfaitement fait l'affaire — les messages d'erreur cités littéralement ont permis de retrouver chaque cause dans le code en quelques minutes. Continuez ainsi : **citez le texte exact affiché**, y compris les messages en anglais et les codes techniques. Si vous pouvez en plus décrire ce qui est à l'écran autour, c'est encore mieux.

---

## Ce que votre recette 4 a permis, en une phrase

Vos deux constats les plus utiles ne sont pas les plus spectaculaires. « Le filtre Brouillon ne charge jamais » nous a menés à une exception de rendu qui cassait la liste des annonces **dès qu'une annonce existe** — un défaut plus large que le symptôme. Et « Owner not found for this user in the organization » nous a fait découvrir que le message était le seul vrai défaut : le refus, lui, était légitime.

---

## Corrigé, à vérifier

### 1. Le filtre « Brouillon » et la création d'annonce ★

**La cause, que vous ne pouviez pas voir.** Deux points d'entrée servent la même liste d'annonces et ne rendaient pas la même forme : la liste complète omet le champ `content`, que le composant lisait sans précaution. `undefined.length` levait une exception **pendant le rendu**, donc hors du bloc qui remet l'indicateur de chargement à faux. D'où « Chargement des annonces… » à l'infini.

Vous aviez raison de dire que les deux défauts se composaient : la correction de RN-6 fait atterrir l'utilisateur précisément dans la vue cassée.

**À vérifier** : créez une annonce, puis constatez qu'elle **apparaît**. Changez le filtre de statut dans les deux sens. Aucune vue ne doit rester en chargement.

Note : l'annonce que vous aviez créée en recette 4 **existait bien** en base, sous le titre « RECETTE4-Entretien des communs », statut Brouillon. Vous devriez la voir.

**Un test automatisé garde maintenant ce cas.** Il a été vérifié par la méthode du témoin : remis le défaut, le test échoue sur l'exception exacte ; correction rétablie, il passe.

### 2. Le message des modules communautaires

**Votre déduction était juste, et notre correction ne va pas là où vous l'attendiez.** Le refus est légitime : offrir une compétence ou prêter un objet engage une personne nommée, pas la copropriété. La preuve que ce n'est pas un blocage général, c'est que **votre création d'annonce a fonctionné** — un avis, lui, émane de la copropriété.

Ce qui n'allait pas, c'est le message. Il dit maintenant, en français, que l'action est réservée aux copropriétaires, pourquoi, et que le syndic agissant pour le compte de l'ACP n'existe pas encore.

**À vérifier** : le message affiché sur Compétences et Partage. Dites-nous s'il vous paraît compréhensible pour un syndic qui n'a pas lu ce brief.

**Reste ouvert, et c'est une décision produit** (#781) : le syndic devrait probablement pouvoir **réserver** la salle commune au nom de l'ACP, contrairement aux deux autres modules. Votre avis d'usage nous intéresse.

### 3. Les deux pages en anglais

Compétences et Partage sont passées en français : titres, sous-titres, bouton, filtres, états vides. Les libellés de listes des composants sont désormais traduits en quatre langues.

**À vérifier** : les deux pages en français, puis **basculez en néerlandais** et regardez ce qui reste.

### 4. Les paragraphes de la page RGPD

Vos cinq paragraphes explicatifs sont traduits, en quatre langues, ainsi que l'aide du formulaire de rectification. Celui de l'effacement dit maintenant explicitement que l'action est IRRÉVERSIBLE.

**Limite connue, à ne pas rapporter comme un défaut nouveau** : le titre de la page et le bloc « À propos de vos droits RGPD » restent en français quelle que soit la langue. Ils vivent dans la coquille Astro de la page, et **toutes** les coquilles du produit sont en français en dur. C'est une limite d'architecture, pas un oubli sur cette page. Votre constat des trois langues sur un écran reste juste et il est consigné.

### 5. L'encadré légal des Sondages

Vous aviez raison de porter le doute sur le fond plutôt que sur la numérotation. L'affirmation « le syndic peut consulter les copropriétaires sur toute décision ne nécessitant pas de vote formel » n'a pas de base établie, et le droit belge encadre étroitement la décision hors assemblée.

L'encadré ne cite plus aucun article. Il dit maintenant que les sondages sont une **consultation informelle, sans valeur de vote**, et qu'un avis recueilli doit être repris en assemblée pour produire un effet. Corrigé aussi dans les quatre langues.

**À vérifier** : la page Sondages et le formulaire de création.

---

## Ce que je vous demande d'explorer

### 1. Le parcours complet d'une AG, dans les deux sens ★ PRIORITÉ

Les trois verrous que vous avez trouvés sont consignés dans l'issue **#780** et **ne sont pas corrigés**. Je ne vous demande pas de les reconstater, mais de cerner leur périmètre exact :

- **Le report d'AG.** Vous l'avez vu inerte sur une AG à date passée. L'est-il aussi sur une AG lointaine, où le report est anodin ? Si oui, le bouton est mort partout ; si non, il échoue seulement dans le cas qui compte.
- **L'envoi de convocation.** Existe-t-il un autre écran — fiche de convocation, détail d'immeuble, liste des copropriétaires — où l'on peut désigner des destinataires ? Nous ne l'avons pas trouvé dans le code, mais un chemin d'interface nous a déjà échappé.
- **La clôture de vote est corrigée**, et sa cause mérite d'être connue de vous. Le champ `total_voting_power` était obligatoire côté serveur et le frontend envoyait un corps vide : la requête échouait en `400` avant d'atteindre la moindre logique. Le total est désormais lu sur l'immeuble, jamais reçu du client.

  **Mais votre observation avait une seconde cause, qui vous concerne directement.** Le bouton ouvre une boîte `confirm()` avant d'envoyer. Un navigateur piloté la rejette par défaut : **aucune requête ne partait**, ce qui explique mot pour mot « aucun dialogue, aucun changement, aucun message ». Un syndic humain aurait vu la boîte.

  À retenir pour toutes vos recettes : **un bouton protégé par une confirmation paraîtra toujours inerte** si votre outil n'accepte pas le dialogue. Acceptez-le explicitement, et dites-nous quand un bouton en ouvre un.

  À vérifier maintenant : clôturez une résolution de bout en bout, et regardez si la condition « 1 résolution en cours » disparaît de l'écran d'AG.

### 2. Le blocage de conformité, que vous proposiez de tester

**Oui, allez-y.** Créez un immeuble volontairement incohérent — un nombre de lots déclaré supérieur au réel — maintenant que le bouton de création existe. Puis dites-nous précisément :

- quelles opérations exactement sont bloquées : dépense, facture, budget, écriture comptable, appel de fonds ?
- le message d'erreur nomme-t-il l'écart en lots et en millièmes ? Il a été enrichi le 4 septembre et personne ne l'a jamais vu à l'écran.

C'est l'issue **#770**, où trois voies sont proposées. Votre observation tranchera. Préfixez les données en `RECETTE5-` pour qu'on les distingue.

### 3. Les rôles autres que syndic

Toutes vos recettes sont passées par un compte syndic. Les comptes `recette-comptable@erables.be` et `recette-copro1/2/3@erables.be` existent.

Un **copropriétaire** devrait pouvoir faire ce que le syndic ne peut pas : offrir une compétence, prêter un objet, réserver. C'est le complément naturel de RN-11, et personne ne l'a vérifié.

Un **comptable** devrait pouvoir saisir des écritures là où le syndic ne le peut pas.

### 4. Le premier écran d'un compte vide

R0-6 de votre première recette : « erreur technique affichée au lieu d'un onboarding ». Jamais revérifié depuis, et c'est la première chose que voit un prospect.

---

## Ce qui n'est pas corrigé

| Réf | Sujet | Issue |
|---|---|---|
| RN-9, RN-10 | Deux des trois verrous d'AG : report impossible, destinataires de convocation | #780 |
| RN-13 | « API Error: 404 » à l'ouverture des Réservations | #766 |
| — | Le syndic agissant pour le compte de l'ACP | #781, #588 |
| R2-1 / R1-7 | `total_units` jamais recalculé | #770 |
| — | 73 routes imbriquées sans identité | #772 |
| R3-5, R3-6, R1-5, R2-2, R0-3, R0-7, R1-3 | Constats de vos premières recettes | ouverts |

Sur les **73 routes** : vous avez raison de ne pas les sonder, c'est un travail de test automatisé et le brief vous l'interdit à juste titre. Ne changez rien.

---

## Une remarque sur votre méthode

Trois choses dans votre recette 4 méritent d'être dites, parce qu'elles ont directement changé ce qui a été corrigé.

Vous avez **séparé l'observation de la déduction** partout. Cela a permis de garder vos observations quand vos déductions se révélaient incomplètes — sur RN-11 notamment, où le constat était juste et la conclusion trop large.

Vous avez **dit ce que vous ne pouviez pas affirmer**, sur l'article 577-8/4. C'est ce qui a conduit à retirer une affirmation juridique invérifiée plutôt qu'à la renuméroter.

Et vous avez **maintenu une section « ce qui marche »**. Elle est ce qui permet de dire à un syndic ce qu'on peut lui montrer sans crainte.

Continuez exactement ainsi.
