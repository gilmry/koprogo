=================================================
Workflow 08 : Tableau d'affichage communautaire
=================================================

:Issue: #346
:Personas: Voir 00-personas-et-seed.rst
:Acteurs: Francois Leroy (syndic publie travaux), Alice Dubois (publie SEL), Charlie Martin (petite annonce), Marguerite Lemaire (lectrice), Ahmed Mansouri (locataire lecteur)
:Articles CC: Aucun (fonctionnalite communautaire, pas d'obligation legale stricte)
:Priorite: Moyenne

Resume
------

Le syndic ou un coproprietaire cree une annonce sur le tableau d'affichage
communautaire de l'immeuble. L'annonce passe par un workflow Draft -> Published
avant d'etre visible par tous les membres du building. Le syndic peut epingler
les annonces importantes. Les annonces expirent automatiquement si une date
d'expiration est definie, et peuvent etre archivees manuellement.

Quatre types d'annonces sont supportes : Announcement (informations generales),
Event (evenements communautaires avec date et lieu), LostAndFound (objets
trouves/perdus), et ClassifiedAd (petites annonces).

Dimension humaine — Le tableau d'affichage comme lien social
--------------------------------------------------------------

Le tableau d'affichage est bien plus qu'un outil technique. C'est parfois
le **seul lien numerique** entre des residents isoles et la vie de l'immeuble :

- **Francois** utilise le tableau pour les annonces officielles (travaux,
  fermetures, decisions de l'AG). C'est un canal de communication formel.
- **Alice** utilise le tableau pour le SEL — elle propose un cours de
  cuisine le samedi. C'est son role de presidente CdC : animer la vie
  communautaire au-dela des questions administratives.
- **Charlie** publie une petite annonce — il vend la poussette de ses
  enfants. Pour un couple dont les charges representent 48% du revenu,
  chaque euro compte.
- **Marguerite** (78 ans, veuve) lit les annonces chaque matin sur sa
  tablette. C'est son **seul lien social numerique** avec l'immeuble.
  Elle ne publie pas, mais elle lit tout — les travaux, les evenements,
  les petites annonces. Sans le tableau, elle serait encore plus isolee.
- **Ahmed** (locataire de Philippe) peut lire les annonces meme s'il n'a
  pas de droit de vote. Il apprend ainsi que le parking sera ferme pendant
  2 semaines — information vitale pour son quotidien. Le tableau d'affichage
  est inclusif la ou l'AG ne l'est pas.

Pre-conditions
--------------

1. Le building "Residence du Parc Royal" existe dans l'organisation.
2. L'auteur est authentifie avec le role ``syndic``, ``owner``, ou ``superadmin``.
3. L'auteur est membre du building concerne.
4. Les locataires (comme Ahmed) ont un acces en lecture seule.

Etapes
------

**Etape 1 — Francois publie une annonce travaux (Announcement)**

- Francois appelle ``POST /notices`` avec :

  .. code-block:: json

     {
       "building_id": "<residence-du-parc-royal-uuid>",
       "notice_type": "Announcement",
       "category": "Parking",
       "title": "Fermeture du parking souterrain pour travaux",
       "content": "Le parking souterrain sera ferme du 15 au 30 avril pour travaux de refection du sol et mise aux normes de l'eclairage. Veuillez utiliser le parking exterieur pendant cette periode. Les places exterieur seront attribuees par ordre d'arrivee. Un affichage sera place a l'entree du parking 5 jours avant la fermeture.",
       "expires_at": "2026-05-01T00:00:00Z"
     }

- L'annonce est creee en statut **Draft**, ``is_pinned = false``.
- Francois publie immediatement via ``PUT /notices/:id/publish``.
- Francois epingle l'annonce via ``PUT /notices/:id/pin`` — elle apparaitra
  en tete de la liste pendant toute la duree des travaux.
- Audit events : ``NoticeCreated``, ``NoticePublished``, ``NoticePinned``.

**Etape 2 — Alice publie un evenement SEL (Event)**

- Alice appelle ``POST /notices`` avec :

  .. code-block:: json

     {
       "building_id": "<residence-du-parc-royal-uuid>",
       "notice_type": "Event",
       "category": "Social",
       "title": "Cours de cuisine — Atelier pain maison (SEL)",
       "content": "Rejoignez-moi pour apprendre a faire du pain au levain ! Atelier ouvert a tous les residents. Ingredients fournis. Valeur SEL : 2 credits. Nombre de places : 6. Inscription par email ou en personne au 2A.",
       "event_date": "2026-04-05T10:00:00Z",
       "event_location": "Salle commune, rez-de-chaussee",
       "contact_info": "alice@residence-parc.be ou apt 2A",
       "expires_at": "2026-04-05T23:59:59Z"
     }

- Alice publie ensuite via ``PUT /notices/:id/publish``.
- L'annonce expire automatiquement apres la date de l'evenement.

**Etape 3 — Charlie publie une petite annonce (ClassifiedAd)**

- Charlie appelle ``POST /notices`` avec :

  .. code-block:: json

     {
       "building_id": "<residence-du-parc-royal-uuid>",
       "notice_type": "ClassifiedAd",
       "category": "General",
       "title": "Vends poussette Bugaboo Fox 3 — bon etat",
       "content": "Poussette Bugaboo Fox 3, utilisee 2 ans, tres bon etat. Nacelle + hamac + habillage pluie inclus. Prix : 250 EUR (neuf : 1.100 EUR). Visible sur place. Priorite aux residents de l'immeuble.",
       "contact_info": "charlie@residence-parc.be ou apt 3B",
       "expires_at": "2026-04-30T00:00:00Z"
     }

- Charlie publie via ``PUT /notices/:id/publish``.
- Pour Charlie dont les charges representent 48% du revenu, 250 EUR
  recuperes sur la poussette allegent un peu le budget.

**Etape 4 — Marguerite lit les annonces (consultation)**

- Marguerite consulte ``GET /buildings/{id}/notices`` chaque matin.
- Les annonces epinglees (travaux parking de Francois) apparaissent en premier.
- Elle lit le cours de cuisine d'Alice — elle ira peut-etre, ca lui ferait
  du bien de sortir de chez elle.
- Elle voit la poussette de Charlie — elle en parlera a sa petite-fille
  qui attend un bebe.

**Etape 5 — Ahmed (locataire) consulte les annonces**

- Ahmed consulte ``GET /buildings/{id}/notices`` et decouvre la fermeture
  du parking pour 2 semaines. Information vitale : il devra reorganiser
  ses deplacements.
- Ahmed voit aussi le cours de cuisine d'Alice — il pourrait y participer.
  Le tableau d'affichage l'integre a la vie de l'immeuble la ou l'AG l'exclut.

**Etape 6 — Expiration automatique**

- L'annonce du cours de cuisine d'Alice expire le 5 avril a 23h59.
- L'annonce travaux parking de Francois expire le 1er mai.
- La petite annonce de Charlie expire le 30 avril (ou plus tot si vendue).
- Le job d'expiration marque les annonces comme **Expired** et retire
  l'epingle automatiquement.

**Etape 7 — Archivage**

- Apres la fin des travaux, Francois archive l'annonce parking via
  ``PUT /notices/:id/archive``. L'annonce n'apparait plus dans la liste
  active mais reste consultable dans l'historique.

**Etape 8 — Suppression (optionnel)**

- Charlie supprime sa petite annonce via ``DELETE /notices/:id`` une fois
  la poussette vendue.

Post-conditions
---------------

1. Les annonces Published sont visibles par tous les membres du building,
   y compris Ahmed (locataire).
2. L'annonce travaux de Francois est epinglee en tete de liste.
3. Les annonces expirees ne sont plus visibles dans la liste active.
4. Les annonces archivees sont consultables uniquement dans l'historique.
5. L'annonce d'Alice (Event) a obligatoirement ``event_date`` et
   ``event_location`` renseignes.

Donnees seed requises
----------------------

.. note::

   Ce workflow utilise le seed partage defini dans ``00-personas-et-seed.rst``.
   Building : **Residence du Parc Royal** (182 lots, 10000 tantiemes).

Donnees specifiques au workflow :

.. code-block:: sql

   -- 3 notices publiees
   INSERT INTO notices (id, building_id, author_id, notice_type, category,
       title, content, status, is_pinned, published_at, expires_at) VALUES
   -- Annonce travaux parking par Francois (syndic) — epinglee
   ('n0800000-0000-0000-0000-000000000001', '<residence-du-parc-royal-uuid>',
    '<francois-user-id>', 'Announcement', 'Parking',
    'Fermeture du parking souterrain pour travaux',
    'Le parking souterrain sera ferme du 15 au 30 avril pour travaux de refection du sol et mise aux normes de l eclairage. Veuillez utiliser le parking exterieur pendant cette periode.',
    'Published', true, NOW() - INTERVAL '2 days', '2026-05-01T00:00:00Z'),
   -- Evenement SEL par Alice (cours cuisine)
   ('n0800000-0000-0000-0000-000000000002', '<residence-du-parc-royal-uuid>',
    '<alice-owner-id>', 'Event', 'Social',
    'Cours de cuisine — Atelier pain maison (SEL)',
    'Rejoignez-moi pour apprendre a faire du pain au levain ! Atelier ouvert a tous les residents. Valeur SEL : 2 credits. Places : 6. Inscription : alice@residence-parc.be ou apt 2A.',
    'Published', false, NOW() - INTERVAL '1 day', '2026-04-05T23:59:59Z'),
   -- Petite annonce par Charlie (vends poussette)
   ('n0800000-0000-0000-0000-000000000003', '<residence-du-parc-royal-uuid>',
    '<charlie-owner-id>', 'ClassifiedAd', 'General',
    'Vends poussette Bugaboo Fox 3 — bon etat',
    'Poussette Bugaboo Fox 3, 2 ans, tres bon etat. Nacelle + hamac + pluie. 250 EUR (neuf 1100). charlie@residence-parc.be ou apt 3B.',
    'Published', false, NOW(), '2026-04-30T00:00:00Z');

   -- Event metadata pour le cours d'Alice
   UPDATE notices
   SET event_date = '2026-04-05T10:00:00Z',
       event_location = 'Salle commune, rez-de-chaussee'
   WHERE id = 'n0800000-0000-0000-0000-000000000002';

   -- Contact info pour la poussette de Charlie
   UPDATE notices
   SET contact_info = 'charlie@residence-parc.be ou apt 3B'
   WHERE id = 'n0800000-0000-0000-0000-000000000003';

BDD
---

.. code-block:: gherkin

   Feature: Tableau d'affichage communautaire

     Background:
       Given l'immeuble "Residence du Parc Royal" avec 182 lots
       And le syndic "Francois Leroy"
       And les coproprietaires Alice Dubois, Charlie Martin, Marguerite Lemaire
       And le locataire Ahmed Mansouri

     Scenario: Francois publie une annonce travaux parking
       Given Francois est connecte en tant que syndic
       When Francois cree une annonce "Announcement" avec titre "Fermeture du parking souterrain pour travaux"
       Then the notice status is "Draft"
       And is_pinned is false

       When Francois publie l'annonce
       Then the notice status is "Published"
       And published_at is recorded

       When Francois epingle l'annonce
       Then is_pinned is true

     Scenario: Alice publie un evenement SEL (cours de cuisine)
       Given Alice est connectee en tant que coproprietaire
       When Alice cree une annonce "Event" avec titre "Cours de cuisine — Atelier pain maison (SEL)"
         And event_date le 5 avril a 10h
         And event_location "Salle commune, rez-de-chaussee"
       Then the notice status is "Draft"

       When Alice publie l'annonce
       Then the notice status is "Published"

     Scenario: Alice cree un evenement sans date echoue
       Given Alice est connectee
       When Alice cree une annonce "Event" sans event_date
       Then I receive an error "Event notices must have an event_date"

     Scenario: Alice cree un evenement sans lieu echoue
       Given Alice est connectee
       When Alice cree une annonce "Event" avec event_date mais sans event_location
       Then I receive an error "Event notices must have an event_location"

     Scenario: Charlie publie une petite annonce (vends poussette)
       Given Charlie est connecte en tant que coproprietaire
       When Charlie cree une annonce "ClassifiedAd" avec titre "Vends poussette Bugaboo Fox 3"
       And Charlie publie l'annonce
       Then the notice status is "Published"
       # Pour Charlie (charges = 48% du revenu), 250 EUR recuperes comptent

     Scenario: Titre trop court echoue
       When on cree une annonce avec titre "Sale"
       Then I receive an error "at least 5 characters"

     Scenario: Contenu vide echoue
       When on cree une annonce avec contenu vide
       Then I receive an error "cannot be empty"

     Scenario: Marguerite consulte les annonces (son lien social numerique)
       Given 3 annonces publiees : travaux (epinglee), cuisine, poussette
       When Marguerite consulte les annonces de l'immeuble
       Then l'annonce travaux de Francois (epinglee) apparait en premier
       And Marguerite voit 3 annonces au total
       # Pour Marguerite (78 ans, veuve), c'est son seul lien numerique avec l'immeuble

     Scenario: Ahmed (locataire) consulte les annonces
       Given 3 annonces publiees
       When Ahmed consulte les annonces de l'immeuble
       Then Ahmed voit les 3 annonces (acces en lecture)
       And Ahmed decouvre la fermeture du parking (information vitale)
       # Le tableau d'affichage est inclusif — meme sans droit de vote

     Scenario: Filtrage par type Event
       Given 1 Announcement (travaux) et 1 Event (cuisine) publiees
       When on filtre les annonces par type "Event"
       Then on voit 1 annonce de type "Event" (cours d'Alice)

     Scenario: Francois epingle l'annonce travaux
       Given l'annonce travaux de Francois en statut "Published"
       When Francois epingle l'annonce
       Then is_pinned is true

     Scenario: Epingler un brouillon echoue
       Given une annonce en statut "Draft"
       When on tente d'epingler l'annonce
       Then I receive an error "Only Published notices can be pinned"

     Scenario: Annonce cours de cuisine expire automatiquement apres l'evenement
       Given l'annonce d'Alice avec expires_at le 5 avril 23h59
       When la date du 6 avril est atteinte
       Then the notice status is "Expired"
       And is_pinned is false

     Scenario: Francois archive l'annonce travaux apres la fin des travaux
       Given l'annonce travaux en statut "Published"
       When Francois archive l'annonce
       Then the notice status is "Archived"
       And archived_at is recorded
       And is_pinned is false

     Scenario: Archiver un brouillon echoue
       Given une annonce en statut "Draft"
       When on tente d'archiver l'annonce
       Then I receive an error "Only Published or Expired notices can be archived"

     Scenario: Charlie modifie sa petite annonce (prix reduit)
       Given l'annonce poussette de Charlie en statut "Draft"
       When Charlie modifie le titre en "Vends poussette Bugaboo Fox 3 — PRIX REDUIT 200 EUR"
       Then le titre est mis a jour

     Scenario: Modification d'une annonce publiee echoue
       Given une annonce en statut "Published"
       When on tente de modifier le titre
       Then I receive an error "Only Draft notices can be updated"

     Scenario: Charlie supprime sa petite annonce (poussette vendue)
       Given l'annonce poussette de Charlie publiee
       When Charlie supprime l'annonce
       Then l'annonce est supprimee

E2E
---

.. code-block:: text

   Test 1 : Francois publie et epingle une annonce travaux
     1. POST /notices (Francois, Announcement, "Fermeture parking", Draft)
     2. PUT /notices/:id/publish (Published)
     3. PUT /notices/:id/pin (epinglee)
     4. GET /buildings/:id/notices (epinglee en premier)
     5. PUT /notices/:id/unpin (desepinglee)
     6. PUT /notices/:id/archive (Archived, apres travaux)
     7. GET /buildings/:id/notices (disparue de la liste active)

   Test 2 : Alice publie un evenement SEL (cours de cuisine)
     1. POST /notices (Alice, Event, "Cours cuisine SEL", event_date, event_location)
     2. PUT /notices/:id/publish (Published)
     3. GET /buildings/:id/notices?type=Event (filtrage)
     4. Verifier event_date et event_location dans la reponse
     5. Attendre expiration (apres event_date)
     6. Verifier statut Expired

   Test 3 : Charlie publie et supprime une petite annonce
     1. POST /notices (Charlie, ClassifiedAd, "Vends poussette")
     2. PUT /notices/:id/publish (Published)
     3. DELETE /notices/:id (poussette vendue)

   Test 4 : Marguerite et Ahmed consultent (lecture seule)
     1. GET /buildings/:id/notices en tant que Marguerite -> 3 annonces
     2. GET /buildings/:id/notices en tant qu'Ahmed (locataire) -> 3 annonces
     3. Verifier que l'annonce epinglee est en premier pour les deux

   Test 5 : Validations
     1. POST /notices avec titre < 5 chars -> 400
     2. POST /notices avec contenu vide -> 400
     3. POST /notices type Event sans event_date -> 400
     4. POST /notices type Event sans event_location -> 400
     5. PUT /notices/:id (Published) -> 400 (modification interdite)
     6. PUT /notices/:id/pin (Draft) -> 400 (epinglage interdit)

   Test 6 : Expiration automatique
     1. POST /notices avec expires_at dans 1 seconde
     2. PUT /notices/:id/publish
     3. Attendre expiration
     4. Verifier statut Expired et is_pinned false

   Test 7 : Multi-auteurs — Francois (syndic) + Alice + Charlie (coproprietaires)
     1. Francois cree une annonce -> 201
     2. Alice cree une annonce -> 201
     3. Charlie cree une annonce -> 201
     4. GET /buildings/:id/notices -> les trois annonces visibles

Diagramme de workflow
---------------------

.. code-block:: text

   Notice:
     Draft ---publish---> Published ---expire---> Expired ---archive---> Archived
       |                      |                                              ^
       |                      '---archive--->-----------Archived-------------'
       |                      |
       |                      '---pin/unpin (toggle epingle)
       |
       '---update (modification du contenu)

   Auteurs:
     Francois (syndic)  --> Announcement (travaux parking, epinglee)
     Alice (presidente) --> Event (cours cuisine SEL)
     Charlie (CP)       --> ClassifiedAd (poussette)

   Lecteurs:
     Marguerite (CP, 78 ans)  --> lecture quotidienne, lien social
     Ahmed (locataire)        --> lecture, information pratique (parking)

   Types:
     Announcement  : titre + contenu (Francois : travaux parking)
     Event         : titre + contenu + event_date + event_location (Alice : cuisine SEL)
     LostAndFound  : titre + contenu + contact_info (recommande)
     ClassifiedAd  : titre + contenu + contact_info (Charlie : poussette)

   Categories:
     General | Maintenance | Social | Security | Environment | Parking | Other
