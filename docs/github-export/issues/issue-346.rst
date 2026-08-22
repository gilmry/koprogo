============================================================================
Issue #346: Specs multi-roles: formaliser 8 workflows dans docs/ (etape 1/5)
============================================================================

:State: **CLOSED**
:Milestone: Jalon 2: Conformité Légale Belge 📋
:Labels: documentation,priority:high specs
:Assignees: Unassigned
:Created: 2026-03-28
:Updated: 2026-03-28
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/346>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Etape 1 : Formaliser les specs multi-roles dans docs/
   
   Parent : #345
   
   ### Objectif
   
   Ecrire 8 specifications de workflows multi-roles dans `docs/specs/`, chacune decrivant :
   - Le narratif metier (qui fait quoi, dans quel ordre)
   - Les acteurs (roles) a chaque etape
   - Les pre-conditions legales (articles du Code Civil)
   - Les post-conditions attendues
   - Les donnees necessaires (seeds)
   
   ### Workflows a formaliser (par priorite)
   
   | # | Workflow | Acteurs | Articles CC |
   |---|---------|---------|------------|
   | 1 | **Vote AG (resolution)** | Syndic -> Owners -> Syndic | Art. 3.87 par.4-6 |
   | 2 | **Ticket maintenance complet** | Owner -> Syndic -> Contractor -> Board | Art. 3.89 par.5 2 |
   | 3 | **Demande AGE** | Owner -> Cosignataires -> Syndic | Art. 3.87 par.2 |
   | 4 | **SEL echange local** | Owner A <-> Owner B | - |
   | 5 | **Sondage entre AG** | Syndic -> Owners -> Syndic | Art. 577-8/4 par.4 |
   | 6 | **Approbation facture** | Syndic/Comptable -> Board | Art. 3.89 par.5 15 |
   | 7 | **Convocation + attendance** | Syndic -> Owners | Art. 3.87 par.3 |
   | 8 | **Annonce communautaire** | Syndic/Owner -> Owners | - |
   
   ### Format de chaque spec
   
   ```rst
   Workflow : Vote AG
   ==================
   
   Acteurs : Syndic, Coproprietaire(s), Commissaire (optionnel)
   
   Pre-conditions legales :
   - Quorum > 50% des quotes-parts (Art. 3.87 par.5)
     OU 2e convocation (pas de quorum requis)
   - Convocation envoyee >= 15j avant (Art. 3.87 par.3)
   - Points inscrits a l'OdJ (Art. 3.87 par.2)
   
   Etapes :
   1. [Syndic] Cree la reunion + OdJ + convocation
   2. [System] Envoie les convocations (15j min)
   3. [Owners] Confirment presence / procuration
   4. [Syndic] Verifie le quorum a l'ouverture
   5. [Syndic] Presente chaque resolution
   6. [Owners] Votent (tantiemes, procurations, plafonnement 50%)
   7. [Syndic] Cloture chaque vote, calcule la majorite
   8. [Syndic] Redige le PV (30j max, Art. 3.87 par.12)
   
   Post-conditions :
   - Resolutions adoptees ou rejetees
   - PV distribue dans les 30 jours
   - Delai de recours 4 mois (Art. 3.92 par.3)
   
   Donnees seed requises :
   - 1 building + 3 units (tantiemes 300/200/500)
   - 3 owners assignes aux units
   - 1 meeting avec quorum valide (2e convocation)
   - 1+ resolutions en statut Pending
   ```
   
   ### Sources
   
   - `docs/legal/coproprietaire/droits_obligations.rst` (CP01-CP15)
   - `docs/legal/syndic/missions_legales.rst` (L01-L18)
   - `docs/legal/assemblee-generale/sequence_odj.rst` (12 etapes)
   - `docs/legal/commissaire/droits_obligations.rst` (CO01-CO05)
   - `docs/MULTIROLE_SPECIFICATIONS.rst` (diagnostic)
   
   ### Definition of Done
   
   - [ ] 8 fichiers .rst dans docs/specs/
   - [ ] Chaque spec reference les articles du CC
   - [ ] Chaque spec liste les donnees seed requises
   - [ ] Review avec le PO pour validation metier

.. raw:: html

   </div>

