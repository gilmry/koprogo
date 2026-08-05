=======================================================================================
Issue #350: Gaps legaux: procurations, plafonnement 50%, consentement email (etape 5/5)
=======================================================================================

:State: **CLOSED**
:Milestone: Jalon 2: Conformité Légale Belge 📋
:Labels: enhancement,priority:high legal-compliance
:Assignees: Unassigned
:Created: 2026-03-28
:Updated: 2026-03-28
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/350>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Etape 5 : Combler les gaps legaux identifies
   
   Parent : #345
   Depend de : #346 (specs clarifient les regles exactes)
   
   ### Objectif
   
   Implementer les invariants du droit belge identifies comme manquants dans le diagnostic `docs/MULTIROLE_SPECIFICATIONS.rst`.
   
   ### Gaps par priorite
   
   #### Priorite haute (bloquants pour conformite Jalon 2)
   
   **P3 — Validation des procurations (Art. 3.87 par.7)**
   - Max 3 procurations par mandataire
   - Exception : si total voix representees (propres + mandats) < 10% du total
   - Implementation : validation dans `Resolution.close_voting()` ou `Vote.cast()`
   - Fichier : `backend/src/domain/entities/vote.rs`
   
   **P4 — Plafonnement vote a 50% (Art. 3.87 par.6)**
   - Si un coproprietaire detient > 50% des quotes-parts, ses voix sont ramenees a 50%-1
   - Les voix des autres sont recalculees proportionnellement
   - Implementation : dans `Resolution.close_voting()` avant le calcul de majorite
   - Fichier : `backend/src/domain/entities/resolution.rs`
   
   **P6 — Consentement convocation electronique (Art. 3.87 par.3)**
   - Chaque coproprietaire doit accepter "individuellement et expressement" la convocation par email
   - Implementation : champ `email_consent: bool` + `email_consent_date: Option<DateTime>` sur Owner
   - Migration : ajouter les champs a la table `owners`
   - Impact : le handler de convocation doit verifier le consentement avant envoi
   
   **P9 — Lien agenda-resolution obligatoire**
   - Seuls les points a l'OdJ peuvent faire l'objet d'un vote (Art. 3.87 par.2)
   - Le champ `agenda_item_index` dans Resolution est optionnel, il devrait etre obligatoire
   - OU validation que le titre de la resolution correspond a un point de l'agenda
   
   #### Priorite moyenne
   
   **P7 — Documenter la base legale AG numerique**
   - La loi du 22 octobre 2022 amende Art. 3.87 par.1
   - Ajouter la reference dans `docs/legal/assemblee-generale/`
   - Le module AgSession est deja conforme, il manque juste la doc
   
   **P8 — Mandat syndic max 3 ans**
   - Art. 3.89 par.1 al.4 : mandat max 3 ans, renouvelable
   - Validation dans BoardMember ou entite Syndic
   - Alerte quand le mandat approche de l'echeance
   
   **CC01 — Detection automatique seuil conseil >= 20 lots**
   - Art. 3.90 par.1 : conseil obligatoire pour immeubles >= 20 lots
   - Implementation : notification/alerte quand un building a >= 20 units et pas de BoardMember
   
   #### Priorite basse (Jalon 3+)
   
   **Commissaire aux comptes (CO01-CO05)**
   - Nouveau role ou sous-role de Owner avec permissions comptables
   - Workflow rapport annuel avant AG
   - A evaluer si necessaire pour Jalon 2 ou reporter a Jalon 3
   
   **Locataire (LO01-LO06)**
   - Nouveau role avec droits limites (info AG, observations)
   - Reporter a Jalon 3 (immeubles mixtes)
   
   ### Definition of Done
   
   - [ ] P3 : Test BDD qui verifie la limite 3 procurations
   - [ ] P4 : Test BDD qui verifie le plafonnement 50%
   - [ ] P6 : Migration + champ consentement + test
   - [ ] P9 : agenda_item_index obligatoire ou validation equivalente
   - [ ] P7 : Documentation ajoutee dans docs/legal/
   - [ ] P8 : Alerte mandat syndic
   - [ ] CC01 : Detection seuil 20 lots
   - [ ] Tous les tests existants continuent de passer

.. raw:: html

   </div>

