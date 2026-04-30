=================================================
Workflow 01 : Vote en Assemblee Generale
=================================================

:Issue: #346
:Acteurs: Syndic, Coproprietaire (x3)
:Articles CC: Art. 3.87 (§1, §3, §5, §6, §7, §12), Art. 3.88 (§1, §1 1°, §1 2°, §1 3°)
:Priorite: Haute
:Voir aussi: docs/legal/assemblee-generale/majorites.rst

Resume
------

Le syndic organise une assemblee generale, valide le quorum, soumet des resolutions
au vote et calcule les resultats selon les 4 types de majorite du Code Civil belge
(Art. 3.88). Les coproprietaires votent avec leurs tantiemes et peuvent deleguer
leur pouvoir de vote par procuration.

Pre-conditions legales
-----------------------

- **Art. 3.87 §3** : Convocation envoyee au moins 15 jours avant l'AG
- **Art. 3.87 §5** : Quorum > 50% des quotes-parts pour 1re convocation.
  EXCEPTION : aucun quorum requis pour 2e convocation
- **Art. 3.87 §6** : Vote proportionnel aux tantiemes. Plafonnement a 50% des voix
- **Art. 3.87 §7** : Procurations : max 3 par mandataire (sauf si total < 10%).
  Le syndic ne peut pas etre mandataire
- **Art. 3.87 §12** : PV transmis dans les 30 jours
- **Art. 3.88 §1** : 4 types de majorite (voir section dediee ci-dessous)

Les 4 types de majorite (Art. 3.88)
--------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 15 15 20 50

   * - Type
     - Seuil
     - Base de calcul
     - Decisions types
   * - **Absolue** (defaut)
     - >50%
     - Presents/representes, **abstentions exclues**
     - Comptes, budget, syndic, commissaire, travaux loi, entretien courant
   * - **2/3**
     - >=66,67%
     - Presents/representes
     - Modif. statuts (jouissance/usage), travaux parties communes, mise en concurrence
   * - **4/5**
     - >=80%
     - Presents/representes
     - Modif. repartition charges, destination, reconstruction partielle, alienation
   * - **Unanimite**
     - 100%
     - **TOUS les tantiemes** (meme absents)
     - Modification des quotites de copropriete, reconstruction totale

.. warning::

   L'unanimite est la seule majorite calculee sur la **totalite** des quotes-parts,
   y compris les coproprietaires absents. Les 3 autres majorites se calculent
   uniquement sur les presents/representes.

.. note::

   Il n'existe PAS de "majorite simple" distincte en droit belge de la copropriete.
   La majorite absolue (>50% des presents, hors abstentions) est le regime de droit commun.

Etapes
------

1. **Syndic** — Cree la reunion (Meeting) avec date, lieu et ordre du jour.
   - Endpoint : ``POST /meetings``
   - Pour le seed de test : ``is_second_convocation = true`` (pas de quorum requis)

2. **Syndic** — Ajoute les points a l'ordre du jour.
   - Les resolutions seront liees aux items via ``agenda_item_index``

3. **Syndic** — Valide le quorum (ou bypass via 2e convocation).
   - 1re convocation : ``meeting.validate_quorum(present_quotas, total_quotas)``
   - 2e convocation : ``meeting.check_quorum_for_voting()`` retourne ``Ok(())``

4. **Syndic** — Cree une resolution liee a un point de l'OdJ.
   - Endpoint : ``POST /meetings/:meeting_id/resolutions``
   - Body : ``title``, ``description``, ``resolution_type``, ``majority_required``
     (absolute / two_thirds / four_fifths / unanimity)

5. **Coproprietaires** — Votent avec leurs tantiemes.
   - Endpoint : ``POST /resolutions/:id/vote``
   - Body : ``owner_id``, ``vote_choice`` (pour/contre/abstention), ``voting_power``
   - Option : ``proxy_owner_id`` pour vote par procuration

6. **Syndic** — Cloture le vote et le systeme calcule la majorite.
   - Endpoint : ``PUT /resolutions/:id/close``
   - Le calcul depend du type de majorite :
     - **Absolue** : ``pour > (pour + contre) / 2`` (abstentions exclues)
     - **2/3** : ``pour / (pour + contre) >= 0.6667`` (abstentions exclues)
     - **4/5** : ``pour / (pour + contre) >= 0.80`` (abstentions exclues)
     - **Unanimite** : ``pour == total_tantiemes_building`` (TOUS les tantiemes)

7. **Syndic** — Complete la reunion et distribue le PV dans les 30 jours.

Post-conditions
---------------

- Resolution statut ``Adopted`` ou ``Rejected``
- ``voted_at`` non-null
- Votes enregistres avec ``resolution_id``
- PV distribue dans 30 jours (Art. 3.87 §12)

Donnees seed requises
----------------------

- **Building** : 3 lots, total_quotas = 1000 milliemes
- **Unit A** : 300 milliemes (30%)
- **Unit B** : 200 milliemes (20%)
- **Unit C** : 500 milliemes (50%)
- **Owner A, B, C** : chacun proprietaire d'un lot
- **User Syndic** : role syndic
- **Meeting** : 2e convocation (``is_second_convocation = true``)

Scenarios BDD (Gherkin)
------------------------

.. code-block:: gherkin

   Feature: Vote en Assemblee Generale — 4 types de majorite (Art. 3.88)

     Background:
       Given un building "Residence des Tilleuls" avec 3 lots (300/200/500 milliemes)
       And 3 coproprietaires (Alice 300, Bob 200, Charlie 500) chacun proprietaire d'un lot
       And un syndic authentifie
       And une reunion AGO en 2e convocation (pas de quorum requis)

     # ===============================================================
     # MAJORITE ABSOLUE (>50% des presents, hors abstentions) — DEFAUT
     # ===============================================================

     Scenario: Majorite absolue — resolution adoptee
       Given une resolution "Approbation des comptes 2025" avec majorite "absolute"
       When Alice vote "Pour" avec 300 milliemes
       And Bob vote "Contre" avec 200 milliemes
       And Charlie vote "Pour" avec 500 milliemes
       And le syndic cloture le vote
       Then la resolution est "Adopted"
       And pour_power est 800 et contre_power est 200

     Scenario: Majorite absolue — resolution rejetee
       Given une resolution "Budget 2026" avec majorite "absolute"
       When Alice vote "Pour" avec 300 milliemes
       And Bob vote "Contre" avec 200 milliemes
       And Charlie vote "Contre" avec 500 milliemes
       And le syndic cloture le vote
       Then la resolution est "Rejected"
       And pour_power est 300 et contre_power est 700

     Scenario: Majorite absolue — abstentions exclues du calcul
       Given une resolution "Nomination commissaire" avec majorite "absolute"
       When Alice vote "Pour" avec 300 milliemes
       And Bob vote "Abstention" avec 200 milliemes
       And Charlie vote "Contre" avec 500 milliemes
       And le syndic cloture le vote
       Then la resolution est "Rejected"
       # Calcul : 300 pour / (300+500) exprimees = 37.5% < 50%
       # Les 200 milliemes d'abstention sont EXCLUES

     Scenario: Majorite absolue — adoptee grace aux abstentions exclues
       Given une resolution "Entretien courant" avec majorite "absolute"
       When Alice vote "Pour" avec 300 milliemes
       And Bob vote "Abstention" avec 200 milliemes
       And Charlie vote "Abstention" avec 500 milliemes
       And le syndic cloture le vote
       Then la resolution est "Adopted"
       # Calcul : 300 pour / (300+0) exprimees = 100% > 50%
       # Seul Alice a exprime un vote, il est Pour → Adopte

     # ===============================================================
     # MAJORITE DES 2/3 (>=66.67%) — Art. 3.88 §1, 1°
     # ===============================================================

     Scenario: Majorite 2/3 — resolution adoptee (travaux parties communes)
       Given une resolution "Travaux facade" avec majorite "two_thirds"
       When Alice vote "Pour" avec 300 milliemes
       And Bob vote "Pour" avec 200 milliemes
       And Charlie vote "Pour" avec 500 milliemes
       And le syndic cloture le vote
       Then la resolution est "Adopted"
       # 1000/1000 = 100% >= 66.67%

     Scenario: Majorite 2/3 — resolution adoptee de justesse
       Given une resolution "Modification statuts" avec majorite "two_thirds"
       When Alice vote "Contre" avec 300 milliemes
       And Bob vote "Pour" avec 200 milliemes
       And Charlie vote "Pour" avec 500 milliemes
       And le syndic cloture le vote
       Then la resolution est "Adopted"
       # 700 pour / (700+300) = 70% >= 66.67%

     Scenario: Majorite 2/3 — resolution rejetee (seuil non atteint)
       Given une resolution "Seuil mise en concurrence" avec majorite "two_thirds"
       When Alice vote "Pour" avec 300 milliemes
       And Bob vote "Pour" avec 200 milliemes
       And Charlie vote "Contre" avec 500 milliemes
       And le syndic cloture le vote
       Then la resolution est "Rejected"
       # 500 pour / (500+500) = 50% < 66.67%

     Scenario: Majorite 2/3 — abstentions exclues
       Given une resolution "Travaux privatifs" avec majorite "two_thirds"
       When Alice vote "Pour" avec 300 milliemes
       And Bob vote "Abstention" avec 200 milliemes
       And Charlie vote "Pour" avec 500 milliemes
       And le syndic cloture le vote
       Then la resolution est "Adopted"
       # 800 pour / (800+0 contre) = 100% >= 66.67% (Bob abstention exclue)

     # ===============================================================
     # MAJORITE DES 4/5 (>=80%) — Art. 3.88 §1, 2°
     # ===============================================================

     Scenario: Majorite 4/5 — resolution adoptee (alienation parties communes)
       Given une resolution "Vente parking commun" avec majorite "four_fifths"
       When Alice vote "Pour" avec 300 milliemes
       And Bob vote "Pour" avec 200 milliemes
       And Charlie vote "Pour" avec 500 milliemes
       And le syndic cloture le vote
       Then la resolution est "Adopted"
       # 1000/1000 = 100% >= 80%

     Scenario: Majorite 4/5 — resolution rejetee (seuil non atteint)
       Given une resolution "Changement destination" avec majorite "four_fifths"
       When Alice vote "Pour" avec 300 milliemes
       And Bob vote "Pour" avec 200 milliemes
       And Charlie vote "Contre" avec 500 milliemes
       And le syndic cloture le vote
       Then la resolution est "Rejected"
       # 500 pour / (500+500) = 50% < 80%

     Scenario: Majorite 4/5 — bloquee par minorite (Alice + Bob < 80%)
       Given une resolution "Reconstruction partielle" avec majorite "four_fifths"
       When Alice vote "Pour" avec 300 milliemes
       And Bob vote "Contre" avec 200 milliemes
       And Charlie vote "Pour" avec 500 milliemes
       And le syndic cloture le vote
       Then la resolution est "Rejected"
       # 800 pour / (800+200) = 80% — PAS ASSEZ, il faut STRICTEMENT > 80%
       # Note : verifier si Art. 3.88 dit >= ou > pour 4/5

     # ===============================================================
     # UNANIMITE (100% de TOUS les tantiemes) — Art. 3.88 §1, 3°
     # ===============================================================

     Scenario: Unanimite — resolution adoptee (tous votent Pour)
       Given une resolution "Modification quotites copropriete" avec majorite "unanimity"
       When Alice vote "Pour" avec 300 milliemes
       And Bob vote "Pour" avec 200 milliemes
       And Charlie vote "Pour" avec 500 milliemes
       And le syndic cloture le vote avec total_building_tantiemes 1000
       Then la resolution est "Adopted"
       # 1000 pour / 1000 total = 100%

     Scenario: Unanimite — rejetee car un coproprietaire vote Contre
       Given une resolution "Reconstruction totale" avec majorite "unanimity"
       When Alice vote "Pour" avec 300 milliemes
       And Bob vote "Contre" avec 200 milliemes
       And Charlie vote "Pour" avec 500 milliemes
       And le syndic cloture le vote avec total_building_tantiemes 1000
       Then la resolution est "Rejected"
       # 800 pour / 1000 total = 80% < 100%

     Scenario: Unanimite — rejetee car un coproprietaire est absent
       Given une resolution "Modification quotites" avec majorite "unanimity"
       When Alice vote "Pour" avec 300 milliemes
       And Charlie vote "Pour" avec 500 milliemes
       # Bob (200 milliemes) est absent et n'a pas vote
       And le syndic cloture le vote avec total_building_tantiemes 1000
       Then la resolution est "Rejected"
       # 800 pour / 1000 total = 80% < 100%
       # L'unanimite exige TOUS les tantiemes, y compris les absents

     Scenario: Unanimite — l'abstention equivaut a un rejet
       Given une resolution "Modification quotites" avec majorite "unanimity"
       When Alice vote "Pour" avec 300 milliemes
       And Bob vote "Abstention" avec 200 milliemes
       And Charlie vote "Pour" avec 500 milliemes
       And le syndic cloture le vote avec total_building_tantiemes 1000
       Then la resolution est "Rejected"
       # 800 pour / 1000 total = 80% < 100%
       # Pour l'unanimite, l'abstention N'EST PAS exclue du calcul

     # ===============================================================
     # REGLES TRANSVERSALES
     # ===============================================================

     Scenario: Quorum bloque le vote en 1ere convocation
       Given une reunion AGO en 1ere convocation
       And le quorum est valide avec 400 milliemes sur 1000
       When le syndic tente de creer une resolution
       Then une erreur "Quorum not reached" est retournee

     Scenario: 2e convocation permet le vote sans quorum
       Given une reunion AGO en 2e convocation
       When le syndic cree une resolution
       Then la creation reussit sans validation de quorum

     Scenario: Plafonnement a 50% des voix (Art. 3.87 §6)
       Given un building avec 2 lots (800/200 milliemes)
       And Owner A possede 800 milliemes (80%)
       And une resolution avec majorite "absolute"
       When le systeme applique le plafonnement
       Then Owner A ne peut voter qu'avec 499 milliemes (50% - 1)
       And Owner B vote avec ses 200 milliemes recalcules proportionnellement

     Scenario: Procuration limitee a 3 mandats (Art. 3.87 §7)
       Given 5 coproprietaires
       And Owner A a deja 3 procurations
       When Owner E tente de donner procuration a Owner A
       Then une erreur "Maximum 3 procurations par mandataire" est retournee

     Scenario: Exception procuration >3 si total < 10% (Art. 3.87 §7)
       Given 10 coproprietaires avec 100 milliemes chacun
       And Owner A (100 milliemes) a deja 3 procurations (3x100 = 300 milliemes)
       # Total represente : 100 (propre) + 300 (mandats) = 400 = 40% > 10%
       When Owner E tente de donner procuration a Owner A
       Then une erreur "Total voix representees depasse 10%" est retournee

     Scenario: PV en retard apres 30 jours
       Given une reunion completee il y a 31 jours sans PV distribue
       Then is_minutes_overdue retourne true

     Scenario: Double vote refuse
       Given une resolution avec majorite "absolute"
       And Alice a deja vote "Pour"
       When Alice tente de voter a nouveau
       Then une erreur "Already voted" est retournee

Scenario E2E (narratif Documentation Vivante)
-----------------------------------------------

**Titre video** : "Comment voter en AG — demonstration multi-roles"

1. ``humanLogin(syndic)`` → Naviguer vers Assemblees → Ouvrir l'AG
2. Voir la section Resolutions avec la resolution pre-seedee
3. ``stepPause`` — montrer l'etat initial (Pending)
4. ``humanLogin(owner_a)`` → Naviguer vers la meme AG
5. Voter "Pour" sur la resolution → voir la confirmation
6. ``humanLogin(owner_b)`` → Voter "Contre"
7. ``stepPause`` — montrer le decompte partiel
8. ``humanLogin(syndic)`` → Cloturer le vote
9. Voir le resultat : Adopted avec le detail des voix
10. ``finalPause`` — montrer le resultat final
