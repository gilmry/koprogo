==================================================================================
Issue #345: Diagnostic: Test-Driven Emergence — chainon manquant specs/seeds/tests
==================================================================================

:State: **CLOSED**
:Milestone: Jalon 2: Conformité Légale Belge 📋
:Labels: documentation,priority:high specs
:Assignees: Unassigned
:Created: 2026-03-28
:Updated: 2026-03-29
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/345>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Diagnostic : strategie Test-Driven Emergence
   
   ### Constat
   
   La session de travail du 26-28 mars 2026 (issue #343) a produit :
   - **Refactoring hexa frontend** : 13 fichiers utils/validators/services, 105 composants migres, -821 lignes nettes
   - **data-testid** : ~300 attributs ajoutes sur ~160 composants (11% -> 90%)
   - **i18n** : 776 -> 2378 cles, 4 locales en parite parfaite (11% -> 73%)
   - **12 scenarios Documentation Vivante** ecrits dont 6 passent
   - **Diagnostic multi-roles** : `docs/MULTIROLE_SPECIFICATIONS.rst` identifiant 9 postulats non conformes
   
   ### Probleme identifie
   
   Les scenarios E2E ont ete ecrits **avant** la formalisation des specs multi-roles et des seeds de donnees. Resultat : 6/12 scenarios echouent sur des problemes de setup (quorum, owner linking, page structure), pas de logique metier.
   
   ### Le chainon manquant
   
   ```
   Specs multi-roles (docs/legal/) -- EXISTE mais pas formalise en scenarios
     | devrait generer
   Scenarios metier alignes BDD + E2E -- MANQUE
     | devrait s'appuyer sur
   Seeds/fixtures avec faker + teardown -- MANQUE
     | devrait produire
   Tests qui passent + videos -- FRAGILE (6/12)
   ```
   
   ### 9 postulats implicites non conformes au droit belge
   
   1. **P1** : 4 roles code vs 7 roles legaux (commissaire, locataire manquent)
   2. **P2** : Seeds de test ne couvrent pas le quorum/AG
   3. **P3** : Procurations illimitees (max 3 + exception 10%, Art. 3.87 par.7)
   4. **P4** : Pas de plafonnement vote 50% (Art. 3.87 par.6)
   5. **P5** : Sequence AG non imposee par le code
   6. **P6** : Consentement convocation electronique non tracke (Art. 3.87 par.3)
   7. **P7** : Loi 22/10/2022 AG numerique non documentee
   8. **P8** : Mandat syndic max 3 ans non enforce
   9. **P9** : Lien agenda-resolution optionnel (devrait etre obligatoire)
   
   ### Plan de remediation
   
   5 issues detaillees suivent cette issue diagnostic :
   1. Formaliser les specs multi-roles dans docs/
   2. Creer les seeds backend avec faker + teardown
   3. Aligner les BDD sur les specs multi-roles
   4. Aligner les E2E sur les memes specs
   5. Combler les gaps legaux (procurations, plafonnement, consentement)
   
   ### References
   - `docs/MULTIROLE_SPECIFICATIONS.rst` (diagnostic complet)
   - `docs/legal/` (65 regles legales codifiees, 7 roles)
   - Issue #343 (refactoring hexa + testids + i18n)
   - Issue #344 (RFC RACE + Graph Social)

.. raw:: html

   </div>

