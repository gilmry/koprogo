=========================================================================================
Issue #265: MCP Tool: énergie (campagne_list, inscrire, offre, comparer_tarif, ag_point)
=========================================================================================

:State: **OPEN**
:Milestone: Jalon 4: Automation & Intégrations 📅
:Labels: enhancement,track:mcp release:0.2.0
:Assignees: Unassigned
:Created: 2026-03-10
:Updated: 2026-03-15
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/265>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Description
   
   Implémenter les 5 outils MCP d'achat groupé d'énergie, permettant aux membres KoproGo de participer aux campagnes d'achat groupé directement via l'agent IA.
   
   ## Outils
   
   ### energie_campagne_list
   Liste les campagnes d'achat groupé actives ou passées. Filtrage par statut (inscription, enchère, offre, switching, terminée) et par région (Bruxelles, Wallonie, Flandre).
   
   ### energie_inscrire
   Inscrit un membre (copropriétaire individuel, propriétaire maison) ou une ACP (compteurs parties communes) à une campagne. Gratuit et sans engagement. Requiert : code EAN, consommation annuelle kWh, fournisseur actuel, type de compteur.
   
   ### energie_offre_personnalisee
   Récupère l'offre personnalisée d'un membre dans une campagne : fournisseur gagnant, prix/kWh, économie estimée, comparaison avec le contrat actuel.
   
   ### energie_comparer_tarif
   Compare le tarif actuel d'un membre avec l'offre groupée et le marché (données CREG). Décompose le prix en : énergie, transport, distribution, taxes.
   
   ### energie_ag_point
   Génère le point d'OdJ pour soumettre le changement de fournisseur énergie des parties communes à l'AG. Inclut le comparatif et la majorité requise (art. 3.88). Vérifie le seuil de mise en concurrence fixé par l'AG.
   
   ## Input Schemas
   
   Voir `backend/koprogo-mcp/README.md` section 9 pour les schemas JSON complets de chaque outil.
   
   ## Tâches
   
   - [ ] Créer `src/mcp/tools/energie.rs`
   - [ ] energie_campagne_list : brancher sur `EnergyCampaignUseCases`, filtrage statut/région
   - [ ] energie_inscrire : validation EAN (18 chiffres), consentement GDPR, création `EnergyBillUpload`
   - [ ] energie_offre_personnalisee : récupérer `ProviderOffer` sélectionnée + calcul économie
   - [ ] energie_comparer_tarif : comparaison tarif actuel vs offre groupée vs marché CREG
   - [ ] energie_ag_point : génération point OdJ avec comparatif + majorité requise
   - [ ] Respecter filtrage par rôle (voir matrice dans #253)
   - [ ] K-anonymity >= 5 participants pour les stats agrégées (GDPR)
   - [ ] Tests unitaires
   
   ## Matrice rôle × outil (énergie)
   
   | Outil | Syndic | Copropriétaire | Locataire | Commissaire | CdC |
   |-------|--------|----------------|-----------|-------------|-----|
   | energie_campagne_list | oui | oui | oui | - | oui |
   | energie_inscrire | oui | oui | oui | - | - |
   | energie_offre_personnalisee | oui | oui (son offre) | oui | - | - |
   | energie_comparer_tarif | oui | oui | oui | - | oui |
   | energie_ag_point | oui | - | - | - | - |
   
   > **Note** : Les locataires avec compteur individuel peuvent participer via le rôle « occupant énergie ».
   
   ## Dépendances
   
   - Bloqué par #252 (serveur SSE), #253 (auth)
   - Réutilise : `EnergyCampaignUseCases`, `EnergyBillUploadUseCases`
   - Réf légale : `docs/legal/achat-groupe-energie/module-specification.rst`

.. raw:: html

   </div>

