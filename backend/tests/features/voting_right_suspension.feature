Feature: Voting right suspension for dismembered or undivided lots (Art. 3.87 §1 CC)
  As a syndic running a general assembly
  A lot that is dismembered (usufruct / bare ownership, emphyteusis, superficies)
  or held in indivision has its voting right suspended until the holders designate
  a single representative, so that no lot votes without a clear voting holder.

  @happy
  Scenario: A lot in sole full ownership can vote
    Given a lot held in full ownership by a single owner
    When the voting right status is evaluated
    Then the lot can vote

  @edge
  Scenario: A dismembered lot with a designated representative can vote
    Given a dismembered lot with the usufructuary designated as representative
    When the voting right status is evaluated
    Then the lot can vote

  @security
  Scenario: A dismembered lot without a representative is suspended
    Given a dismembered lot without a designated representative
    When the voting right status is evaluated
    Then the voting right is suspended

  @security
  Scenario: A lot in indivision without a representative is suspended
    Given a lot held in indivision without a designated representative
    When the voting right status is evaluated
    Then the voting right is suspended

  @negative
  Scenario: Designating two representatives for the same lot is rejected
    Given a lot in indivision with two designated representatives
    When the single representative rule is checked
    Then the designation is rejected
