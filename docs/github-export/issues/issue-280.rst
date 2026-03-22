================================================================================================================================
Issue #280: feat: Orchestrateur neutre achat groupé énergie — zéro commission + maisons individuelles + CER (extension BC8)
================================================================================================================================

:State: **OPEN**
:Milestone: Jalon 3: Features Différenciantes 🎯
:Labels: enhancement,track:software release:0.1.0
:Assignees: Unassigned
:Created: 2026-03-12
:Updated: 2026-03-14
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/280>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte stratégique
   
   KoproGo se positionne comme **orchestrateur neutre et transparent** d'achat groupé d'énergie, sans aucune commission des fournisseurs — le seul acteur belge à pouvoir faire cette promesse (revenus = SaaS uniquement).
   
   **Faiblesses des acteurs actuels** (source : Étude CREG F1827, 2018) :
   - iChoosr, Pricewise, Wikipower reçoivent 15-30 €/contrat des fournisseurs → biais de sélection
   - Calcul d'économies basé sur prix moyen marché, pas le contrat réel du consommateur
   - Concentration : 3 acteurs depuis 2012, label CREG quasi-absent (Wikipower expiré 2023)
   
   KoproGo peut se différencier sur tous ces axes.
   
   ## Principes non-négociables
   
   1. **Zéro commission fournisseur** — déclarée publiquement et vérifiable
   2. **Calcul économies honnête** — basé sur l'upload de la vraie facture (EnergyBillUpload déjà implémenté)
   3. **Toutes les offres affichées** — aucune offre cachée ou sponsorisée
   4. **Licences fournisseurs vérifiées** — VREG/CWaPE/BRUGEL avant invitation
   5. **Label CREG ciblé** — accréditation Charte B1614 (décision B2282 montre le chemin)
   
   ## Extensions demandées
   
   ### 1. Campagnes ouvertes aux maisons individuelles
   - Extension champ `audience_type` sur EnergyCampaign : CoProprietiesOnly / OpenToIndividuals / Public
   - Nouvelle entité `IndividualMember` : email, postal_code, consent, campaign_id
   - Endpoint public : `POST /energy-campaigns/:id/join-as-individual` (sans auth complète)
   - Parcours d'inscription simplifié (email + code postal + consentement RGPD)
   - Compte "énergie solo" minimal (voir offres, économies individuelles, rétractation)
   
   **Pourquoi** : Les voisins des copropriétés clientes = canal d'acquisition naturel, masse négociante augmentée, pouvoir de négociation accru.
   
   ### 2. Module Communauté d'Énergie (CER/CEC)
   Nouveau sous-module BC8 pour les copropriétés avec production locale (PV).
   
   Entité `EnerCommunity` :
   - `members[]` (copropriétaires + maisons individuelles + PME locales)
   - `production_installation_id` (panneaux PV, éolien)
   - `sharing_rules` (quote-part par membre)
   - `region` (Wallonie/Flandre/Bruxelles — règles différentes)
   - `grD_contract_ref` (contrat avec Fluvius/Ores/Sibelga)
   - `status` (Draft/Registered/Active/Suspended)
   
   Use cases :
   - `create_community(building_id, energy_type, members)`
   - `calculate_sharing(production_kwh, member_shares)`
   - `generate_monthly_report(community_id)` → rapport AG annuel
   - `link_to_iot(community_id, linky_device_ids)` → suivi temps réel
   
   **Base légale** :
   - Directive RED II (2018/2001), Art. 22
   - Wallonie : Décret du 05/05/2022
   - Flandre : Décret flamand d'électricité (VREG)
   - Bruxelles : Ordonnance électricité (BRUGEL)
   
   ### 3. Algorithme de sélection transparent et publié
   Scoring multi-critères avec pondération définie par les membres :
   - Prix énergie : 50% (défaut)
   - % renouvelable : 25% (défaut)
   - Service client (historique) : 15% (défaut)
   - Type contrat (fixe/variable) : 10% (défaut)
   
   Les membres peuvent voter sur les pondérations en début de campagne.
   
   ### 4. Registre des fournisseurs agréés
   Table `energy_providers` : name, licence_vreg, licence_cwape, licence_brugel,
   green_pct_certified, last_verified_at.
   Vérification manuelle ou via API régulateurs avant invitation à une campagne.
   
   ### 5. Rapport CREG-compliant
   GET /energy-campaigns/:id/creg-report → rapport PDF conforme Charte B1614 :
   - Description du service et de la rémunération (zéro commission)
   - Méthode de calcul des économies (basée sur vraie facture)
   - Liste complète des offres reçues
   - Critères de sélection et pondérations utilisées
   
   ## Conformité CREG (Charte B1614)
   
   | Principe | Exigence | Statut |
   |----------|----------|--------|
   | Clarté du service | Déclaration zéro commission | À ajouter |
   | Calcul honnête | Basé sur upload facture réelle | Déjà implémenté |
   | Toutes les offres | Affichage complet | À ajouter |
   | RGPD | AES-256-GCM, k-anonymité ≥ 5 | Déjà implémenté |
   
   ## Corpus de référence
   Voir `docs/energie/` pour l'intégralité du cadre légal, l'analyse concurrentielle
   et la stratégie de différenciation.
   
   ## Definition of Done
   - [ ] EnergyCampaign.audience_type gérant copropriétés + maisons individuelles
   - [ ] IndividualMember entity + endpoint inscription publique
   - [ ] EnerCommunity entity (CER/CEC) avec use cases sharing calculation
   - [ ] Algorithme scoring publié (endpoint GET /energy-campaigns/:id/scoring-method)
   - [ ] energy_providers table avec vérification licence
   - [ ] Page 'Comment ça marche' avec déclaration zéro commission
   - [ ] Tests unitaires EnerCommunity (partage proportionnel, rapport mensuel)

.. raw:: html

   </div>

