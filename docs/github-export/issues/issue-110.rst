=============================================================================
Issue #110: feat: Energy Buying Groups Platform (Groupements d'Achat Énergie)
=============================================================================

:State: **OPEN**
:Milestone: Jalon 6: Intelligence & Expansion (PropTech 2.0) 🤖
:Labels: enhancement,phase:k8s track:software,priority:medium finance,automation community,proptech:energy
:Assignees: Unassigned
:Created: 2025-11-07
:Updated: 2025-11-13
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/110>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## ⚡ Energy Buying Groups Platform
   
   **Priority**: 🟡 Medium | **Phase**: 3 (K8s Production) | **Track**: Software
   
   ### Description
   Plateforme de groupements d'achat d'énergie pour mutualiser les contrats et obtenir meilleurs tarifs.
   
   ### PropTech 2.0 - Pilier Énergie
   - 💰 Négociation collective contrats gaz/électricité
   - 📊 Comparateur fournisseurs (Engie, Luminus, TotalEnergies, etc.)
   - 🔄 Switch collectif automatisé (campagnes annuelles)
   - 📈 Analytics consommations + prévisions factures
   
   ### Cas d'usage concret
   **Exemple**: 500 copropriétés KoproGo = 10,000+ lots
   → Pouvoir négociation énorme auprès fournisseurs\!
   → Économies estimées: **15-25% vs contrats individuels**
   
   ### Modèle Participatif - Double Économie
   1. **Plateforme KoproGo**: Prix 0.40€/mois (dilué par l'échelle)
   2. **Énergie**: -20% sur factures (groupement achat)
   → **ROI immédiat pour copropriétaires\!**
   
   ### Tâches Techniques
   
   #### Backend (Rust)
   - [ ] Entity `EnergyContract` (provider, start_date, end_date, rate_kwh, fixed_fee)
   - [ ] Entity `EnergyConsumption` (building_id, period, kwh_total, cost, source)
   - [ ] Entity `BuyingGroup` (campaign_id, buildings[], status, target_kwh, negotiated_rate)
   - [ ] Port `EnergyRepository` + PostgreSQL impl
   - [ ] Use Case `CreateBuyingGroupCampaign`
   - [ ] Use Case `CalculatePotentialSavings`
   - [ ] Endpoints: `POST /api/v1/energy/campaigns`, `GET /energy/buildings/:id/consumption`
   
   #### Intégrations Fournisseurs
   - [ ] Enedis API (consommations Linky)
   - [ ] VREG/CREG API (tarifs régulés Belgique)
   - [ ] Web scraping tarifs fournisseurs (si pas d'API)
   - [ ] Export CSV pour courtiers énergie
   
   #### Frontend
   - [ ] Dashboard consommations (graphiques mensuels)
   - [ ] Comparateur fournisseurs (tableau tarifs)
   - [ ] Inscription campagnes groupement (wizard)
   - [ ] Simulateur économies (formulaire + résultat temps-réel)
   - [ ] Suivi campagnes (timeline + participants)
   
   #### Workflow Campagnes
   1. **Lancement campagne** (AG vote: oui/non participation)
   2. **Collecte données** (consommations annuelles via Linky)
   3. **Négociation** (KoproGo → courtier → fournisseurs)
   4. **Vote final** (acceptation offre négociée)
   5. **Switch automatisé** (dossiers fournisseur générés)
   
   ### Livrables
   - ✅ Intégration Enedis Linky (consommations temps-réel)
   - ✅ Comparateur 5+ fournisseurs belges
   - ✅ Workflow campagne complet (vote + switch)
   - ✅ Simulateur économies (input kWh → output €)
   - ✅ Dashboard analytics consommations
   - ✅ Export rapport courtier énergie
   
   ### Effort estimé
   **32-40 heures** (4-5 jours dev)
   
   ### Dépend de
   - Phase 3 K8s
   - IoT platform (#109) pour consommations temps-réel
   - Voting system (#046) pour votes campagnes
   
   ### Impact Social & Écologique
   🌍 **Triple impact**:
   1. **Économique**: -20% factures énergie
   2. **Social**: Pouvoir négociation collectif vs oligopole
   3. **Écologique**: Incentive switch vers fournisseurs verts (Lampiris, Greenpeace Energy)
   
   ### Labels
   `enhancement`, `phase:k8s`, `track:software`, `priority:medium`, `proptech:energy`, `finance`, `automation`, `community`

.. raw:: html

   </div>

