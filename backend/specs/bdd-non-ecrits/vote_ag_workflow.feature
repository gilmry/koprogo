# Feature: Vote en Assemblee Generale — 4 types de majorite (Art. 3.88)
# Workflow multi-roles avec personas du seed Residence du Parc Royal
# Issue #346 — Spec: docs/specs/01-vote-ag.rst
# Articles CC: Art. 3.87 (§1, §3, §5, §6, §7, §12), Art. 3.88 (§1, §1 1°, §1 2°, §1 3°)

Feature: Vote en Assemblee Generale — Workflow multi-roles (Art. 3.88 CC)
  As Francois Leroy (syndic)
  I want to create resolutions, let owners vote with their tantiemes, and close voting
  So that decisions comply with Belgian copropriete law and the 4 majority types

  Background:
    Given the system is initialized
    And the building "Residence du Parc Royal" with 182 lots and 10000 tantiemes
    And Francois Leroy is syndic of the building
    And the following owners exist with their tantiemes:
      | name                    | lot   | tantiemes | shares_pct |
      | Alice Dubois            | 2A    | 450       | 0.045      |
      | Bob Janssen             | 2B    | 430       | 0.043      |
      | Charlie Martin          | 3B    | 660       | 0.066      |
      | Diane Peeters           | 3A    | 580       | 0.058      |
      | Emmanuel Claes          | 5A    | 1280      | 0.128      |
      | Philippe Vandermeulen   | 6A-6C | 1800      | 0.180      |
      | Marcel Dupont           | 4B    | 450       | 0.045      |
      | Nadia Benali            | 4A    | 320       | 0.032      |
      | Marguerite Lemaire      | 1A    | 380       | 0.038      |
      | Jeanne Devos            | 1B    | 290       | 0.029      |
    And a meeting "AG Ordinaire 2026" exists in 2nd convocation (no quorum required)

  # ===============================================================
  # MAJORITE ABSOLUE (>50% des presents, hors abstentions) — DEFAUT
  # Art. 3.88 §1 — Comptes, budget, syndic, commissaire, entretien
  # ===============================================================

  Scenario: Majorite absolue — resolution adoptee (approbation des comptes)
    Given Francois creates a resolution "Approbation des comptes 2025" with majority "absolute"
    When Alice (450) votes "Pour" on the resolution
    And Bob (430) votes "Pour" on the resolution
    And Charlie (660) votes "Contre" on the resolution
    And Diane (580) votes "Pour" on the resolution
    And Francois closes voting on the resolution
    Then the resolution status should be "Adopted"
    # Calcul: 1460 Pour / (1460 + 660) = 68.9% > 50%

  Scenario: Majorite absolue — resolution rejetee (budget 2026)
    Given Francois creates a resolution "Budget previsionnel 2026" with majority "absolute"
    When Alice (450) votes "Pour" on the resolution
    And Bob (430) votes "Pour" on the resolution
    And Charlie (660) votes "Contre" on the resolution
    And Diane (580) votes "Contre" on the resolution
    And Emmanuel (1280) votes "Contre" on the resolution
    And Francois closes voting on the resolution
    Then the resolution status should be "Rejected"
    # Calcul: 880 Pour / (880 + 2520) = 25.9% < 50%

  Scenario: Majorite absolue — abstentions exclues du calcul
    # Art. 3.88 §1: les abstentions ne comptent pas dans la base de calcul
    Given Francois creates a resolution "Nomination commissaire aux comptes" with majority "absolute"
    When Alice (450) votes "Pour" on the resolution
    And Bob (430) votes "Abstention" on the resolution
    And Charlie (660) votes "Contre" on the resolution
    And Francois closes voting on the resolution
    Then the resolution status should be "Rejected"
    # Calcul: 450 Pour / (450 + 660) = 40.5% < 50%
    # Bob's 430 abstention milliemes are EXCLUDED from calculation

  Scenario: Majorite absolue — adoptee grace aux abstentions exclues
    Given Francois creates a resolution "Entretien courant parties communes" with majority "absolute"
    When Alice (450) votes "Pour" on the resolution
    And Bob (430) votes "Abstention" on the resolution
    And Charlie (660) votes "Abstention" on the resolution
    And Francois closes voting on the resolution
    Then the resolution status should be "Adopted"
    # Calcul: 450 Pour / (450 + 0 Contre) = 100% > 50%
    # Only Alice expressed a vote and it is Pour -> Adopted

  # ===============================================================
  # MAJORITE DES 2/3 (>=66.67%) — Art. 3.88 §1, 1°
  # Modification statuts, travaux parties communes, mise en concurrence
  # ===============================================================

  Scenario: Majorite 2/3 — resolution adoptee (travaux facade)
    Given Francois creates a resolution "Ravalement facade batiment principal" with majority "two_thirds"
    When Alice (450) votes "Pour" on the resolution
    And Bob (430) votes "Pour" on the resolution
    And Charlie (660) votes "Pour" on the resolution
    And Diane (580) votes "Pour" on the resolution
    And Francois closes voting on the resolution
    Then the resolution status should be "Adopted"
    # Calcul: 2120 / 2120 = 100% >= 66.67%

  Scenario: Majorite 2/3 — adoptee de justesse (modification statuts)
    Given Francois creates a resolution "Modification reglement copropriete — animaux" with majority "two_thirds"
    When Alice (450) votes "Contre" on the resolution
    And Bob (430) votes "Pour" on the resolution
    And Charlie (660) votes "Pour" on the resolution
    And Diane (580) votes "Pour" on the resolution
    And Francois closes voting on the resolution
    Then the resolution status should be "Adopted"
    # Calcul: 1670 Pour / (1670 + 450) = 78.8% >= 66.67%

  Scenario: Majorite 2/3 — rejetee par coalition investisseurs
    # Philippe + Emmanuel = 3080 tantiemes (46.4% des presents)
    Given Francois creates a resolution "Mise en concurrence contrat syndic" with majority "two_thirds"
    When Alice (450) votes "Pour" on the resolution
    And Bob (430) votes "Pour" on the resolution
    And Charlie (660) votes "Pour" on the resolution
    And Diane (580) votes "Pour" on the resolution
    And Emmanuel (1280) votes "Contre" on the resolution
    And Philippe (1800) votes "Contre" on the resolution
    And Francois closes voting on the resolution
    Then the resolution status should be "Rejected"
    # Calcul: 2120 Pour / (2120 + 3080) = 40.8% < 66.67%
    # The investor bloc (Philippe + Emmanuel) blocks the qualified majority

  Scenario: Majorite 2/3 — abstentions exclues du calcul
    Given Francois creates a resolution "Travaux privatifs sur parties communes" with majority "two_thirds"
    When Alice (450) votes "Pour" on the resolution
    And Bob (430) votes "Abstention" on the resolution
    And Charlie (660) votes "Pour" on the resolution
    And Diane (580) votes "Pour" on the resolution
    And Francois closes voting on the resolution
    Then the resolution status should be "Adopted"
    # Calcul: 1690 Pour / (1690 + 0 Contre) = 100% >= 66.67% (Bob abstention excluded)

  # ===============================================================
  # MAJORITE DES 4/5 (>=80%) — Art. 3.88 §1, 2°
  # Modification repartition charges, destination, reconstruction,
  # alienation parties communes
  # ===============================================================

  Scenario: Majorite 4/5 — resolution adoptee (vente parking commun)
    Given Francois creates a resolution "Alienation du parking commun P3" with majority "four_fifths"
    When Alice (450) votes "Pour" on the resolution
    And Bob (430) votes "Pour" on the resolution
    And Charlie (660) votes "Pour" on the resolution
    And Diane (580) votes "Pour" on the resolution
    And Marcel (450) votes "Pour" on the resolution
    And Nadia (320) votes "Pour" on the resolution
    And Francois closes voting on the resolution
    Then the resolution status should be "Adopted"
    # Calcul: 2890 / 2890 = 100% >= 80%

  Scenario: Majorite 4/5 — rejetee (changement destination immeuble)
    Given Francois creates a resolution "Changement affectation rez-de-chaussee en commerce" with majority "four_fifths"
    When Alice (450) votes "Pour" on the resolution
    And Bob (430) votes "Pour" on the resolution
    And Charlie (660) votes "Contre" on the resolution
    And Diane (580) votes "Pour" on the resolution
    And Francois closes voting on the resolution
    Then the resolution status should be "Rejected"
    # Calcul: 1460 Pour / (1460 + 660) = 68.9% < 80%

  Scenario: Majorite 4/5 — bloquee par Philippe seul (18% des tantiemes)
    # Philippe's 1800 tantiemes alone can block a 4/5 majority
    Given Francois creates a resolution "Reconstruction partielle aile sud" with majority "four_fifths"
    When Alice (450) votes "Pour" on the resolution
    And Bob (430) votes "Pour" on the resolution
    And Charlie (660) votes "Pour" on the resolution
    And Diane (580) votes "Pour" on the resolution
    And Marcel (450) votes "Pour" on the resolution
    And Nadia (320) votes "Pour" on the resolution
    And Philippe (1800) votes "Contre" on the resolution
    And Francois closes voting on the resolution
    Then the resolution status should be "Rejected"
    # Calcul: 2890 Pour / (2890 + 1800) = 61.6% < 80%

  # ===============================================================
  # UNANIMITE (100% de TOUS les tantiemes) — Art. 3.88 §1, 3°
  # Modification quotites copropriete, reconstruction totale
  # Note: Unanimite = all 10000 tantiemes, not just those present
  # ===============================================================

  Scenario: Unanimite — rejetee car seuls les presents votent (absents comptent)
    # Even if all present owners vote Pour, unanimity requires ALL 10000 tantiemes
    Given Francois creates a resolution "Modification quotites de copropriete" with majority "unanimity"
    When Alice (450) votes "Pour" on the resolution
    And Bob (430) votes "Pour" on the resolution
    And Charlie (660) votes "Pour" on the resolution
    And Diane (580) votes "Pour" on the resolution
    And Marcel (450) votes "Pour" on the resolution
    And Nadia (320) votes "Pour" on the resolution
    And Marguerite (380) votes "Pour" on the resolution
    And Jeanne (290) votes "Pour" on the resolution
    And Emmanuel (1280) votes "Pour" on the resolution
    And Philippe (1800) votes "Pour" on the resolution
    And Francois closes voting on the resolution with total_building_tantiemes 10000
    Then the resolution status should be "Rejected"
    # Calcul: 6640 Pour / 10000 total = 66.4% < 100%
    # The 172 absent lots (3360 tantiemes) make unanimity impossible

  Scenario: Unanimite — rejetee car Emmanuel vote Contre
    Given Francois creates a resolution "Reconstruction totale immeuble" with majority "unanimity"
    When Alice (450) votes "Pour" on the resolution
    And Bob (430) votes "Pour" on the resolution
    And Charlie (660) votes "Pour" on the resolution
    And Emmanuel (1280) votes "Contre" on the resolution
    And Francois closes voting on the resolution with total_building_tantiemes 10000
    Then the resolution status should be "Rejected"
    # 1540 Pour / 10000 total = 15.4% < 100%

  Scenario: Unanimite — abstention equivaut a un rejet
    # For unanimity, abstentions are NOT excluded — only Pour counts
    Given Francois creates a resolution "Modification quotites lot 2A" with majority "unanimity"
    When Alice (450) votes "Pour" on the resolution
    And Bob (430) votes "Pour" on the resolution
    And Charlie (660) votes "Abstention" on the resolution
    And Francois closes voting on the resolution with total_building_tantiemes 10000
    Then the resolution status should be "Rejected"
    # 880 Pour / 10000 total = 8.8% < 100%
    # Charlie's abstention is NOT excluded for unanimity

  # ===============================================================
  # REGLES TRANSVERSALES
  # ===============================================================

  Scenario: Quorum bloque le vote en 1ere convocation (Art. 3.87 §5)
    Given a meeting "AG 2026 — 1ere convocation" in 1st convocation
    And quorum is validated with 3500 milliemes present out of 10000
    # 3500 / 10000 = 35% < 50% quorum threshold
    When Francois tries to create a resolution for this meeting
    Then an error "Quorum not reached" is returned

  Scenario: 2e convocation permet le vote sans quorum (Art. 3.87 §5)
    Given a meeting "AG 2026 — 2e convocation" in 2nd convocation
    When Francois creates a resolution for this meeting
    Then the resolution is created without quorum validation

  Scenario: Plafonnement a 50% des voix — Art. 3.87 §6
    # A single owner cannot hold more than 50% of voting power
    # Philippe (1800 = 18%) is under the cap, but this scenario tests the rule
    Given a building with 2 lots (8000/2000 milliemes)
    And Owner A owns 8000 milliemes (80%)
    And Owner B owns 2000 milliemes (20%)
    And a resolution with majority "absolute"
    When the system applies vote capping
    Then Owner A can only vote with 4999 milliemes (50% - 1)
    # Art. 3.87 §6: Nul ne peut prendre part au vote pour un nombre de voix
    # superieur a la moitie du nombre total

  Scenario: Procuration limitee a 3 mandats — Art. 3.87 §7
    Given 5 owners exist for the building
    And Philippe already holds 3 proxy delegations
    When Emmanuel tries to delegate proxy to Philippe
    Then an error "Maximum 3 procurations par mandataire" is returned

  Scenario: Exception procuration si total represente < 10% — Art. 3.87 §7
    Given 20 owners with 50 milliemes each (total 1000)
    And Marguerite (50 milliemes) already holds 3 proxies (3x50 = 150 milliemes)
    # Total represented: 50 (own) + 150 (proxies) = 200 = 20% > 10%
    When Jeanne tries to delegate proxy to Marguerite
    Then an error "Total voix representees depasse 10%" is returned

  Scenario: Le syndic ne peut pas etre mandataire — Art. 3.89 §9
    When Emmanuel tries to delegate proxy to Francois (syndic)
    Then an error "Le syndic ne peut pas etre mandataire" is returned

  Scenario: Double vote refuse
    Given Francois creates a resolution "Budget 2026" with majority "absolute"
    And Alice (450) has already voted "Pour" on the resolution
    When Alice tries to vote "Pour" again on the resolution
    Then an error "Already voted" is returned

  Scenario: PV en retard apres 30 jours — Art. 3.87 §12
    Given a meeting completed 31 days ago without distributed minutes
    Then is_minutes_overdue returns true

  Scenario: Vote par procuration — Emmanuel delegue a Philippe
    Given Francois creates a resolution "Approbation des comptes" with majority "absolute"
    And Emmanuel (1280) has delegated proxy to Philippe
    When Philippe votes "Contre" as proxy for Emmanuel with 1280 milliemes
    And Philippe votes "Contre" with his own 1800 milliemes
    And Alice (450) votes "Pour" on the resolution
    And Bob (430) votes "Pour" on the resolution
    And Charlie (660) votes "Pour" on the resolution
    And Diane (580) votes "Pour" on the resolution
    And Francois closes voting on the resolution
    Then the resolution status should be "Rejected"
    # Pour: 2120 / (2120 + 3080) = 40.8% < 50%
    # Philippe + Emmanuel proxy = 3080 tantiemes (30.8%) — bloc investisseur
