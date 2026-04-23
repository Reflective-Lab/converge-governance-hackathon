Feature: Enterprise AI vendor selection

  Background:
    Given truth "vendor-selection"
    And vendors from "seed-vendors.json"
    And criteria from "seed-criteria.json"
    And policy "vendor-selection-policy.cedar"

  Scenario: Happy path — governed vendor shortlist
    Given vendors "Acme AI, Beta ML, Epsilon AI"
    And min_score 75
    And max_risk 30
    When vendor-selection is executed
    Then intent is admitted
    And formation is assembled with 5 roles
    And all vendors are screened for compliance
    And a ranked shortlist is produced
    And policy authorizes commitment
    And the truth converges

  Scenario: HITL gate — high-value commitment requires approval
    Given vendors from "seed-vendors.json"
    And min_score 60
    And max_risk 40
    When vendor-selection is executed with total_amount > 50000
    Then shortlist is produced
    And policy returns Blocked with approval reference
    And the truth does not fully converge

  Scenario: Non-compliant vendor filtered from shortlist
    Given vendors "Acme AI, Beta ML, Gamma LLM"
    And min_score 75
    And max_risk 30
    When vendor-selection is executed
    Then Gamma LLM is excluded from shortlist
    And rejection reason includes "non-compliant"
    And remaining vendors are ranked

  Scenario: Insufficient authority — policy reject
    Given vendors "Acme AI"
    And principal authority "advisory"
    When vendor-selection is executed
    Then policy returns Unmet
    And commitment is not authorized
