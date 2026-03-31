Feature: Enterprise AI vendor selection

  Scenario: Evaluate candidate vendors for governed rollout
    Given truth "evaluate-vendor"
    And vendors:
      | name      |
      | Acme AI   |
      | Beta ML   |
      | Gamma LLM |
