Feature: Automatic epic status sync
  As a user
  I want epic status to automatically reflect child task progress
  So that I always see accurate epic state without manual updates

  Background:
    Given a tacks database is initialized

  Scenario: Epic stays open when all children are open
    Given I have a task called "epic" with title "Fresh epic" and tag "epic"
    When I create a subtask of "epic" with title "Task A"
    And I create a subtask of "epic" with title "Task B"
    And I show task "epic" in JSON
    Then the task details show status "open"

  Scenario: Epic becomes in_progress when first child closes
    Given I have a task called "epic" with title "Active epic" and tag "epic"
    When I create a subtask of "epic" with title "Done step"
    And I create a subtask of "epic" with title "Open step"
    And I force close subtask "Done step"
    And I show task "epic" in JSON
    Then the task details show status "in_progress"

  Scenario: Epic becomes done when all children close
    Given I have a task called "epic" with title "Complete epic" and tag "epic"
    When I create a subtask of "epic" with title "Step one"
    And I create a subtask of "epic" with title "Step two"
    And I force close subtask "Step one"
    And I force close subtask "Step two"
    And I show task "epic" in JSON
    Then the task details show status "done"

  Scenario: Adding a new subtask to done epic reverts to in_progress
    Given I have a task called "epic" with title "Reopened epic" and tag "epic"
    When I create a subtask of "epic" with title "Done step"
    And I force close subtask "Done step"
    And I show task "epic" in JSON
    Then the task details show status "done"
    When I create a subtask of "epic" with title "New work"
    And I show task "epic" in JSON
    Then the task details show status "in_progress"

  Scenario: Claiming a child does not change epic from open
    Given I have a task called "epic" with title "Open epic" and tag "epic"
    When I create a subtask of "epic" with title "WIP step"
    And I create a subtask of "epic" with title "Other step"
    And I claim subtask "WIP step"
    And I show task "epic" in JSON
    Then the task details show status "open"

  Scenario: Reopening a child reverts done epic to in_progress
    Given I have a task called "epic" with title "Reverted epic" and tag "epic"
    When I create a subtask of "epic" with title "Step one"
    And I force close subtask "Step one"
    And I show task "epic" in JSON
    Then the task details show status "done"
    When I reopen subtask "Step one"
    And I show task "epic" in JSON
    Then the task details show status "open"
