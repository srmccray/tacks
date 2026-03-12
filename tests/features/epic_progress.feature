Feature: Three-part epic progress
  As a user
  I want to see done/in-progress/open counts for epics
  So that I can gauge actual work activity at a glance

  Background:
    Given a tacks database is initialized

  Scenario: Epic with mixed status children shows three-part counts in JSON
    Given I have a task called "epic" with title "Mixed epic" and tag "epic"
    When I create a subtask of "epic" with title "Done step"
    And I create a subtask of "epic" with title "WIP step"
    And I create a subtask of "epic" with title "Open step"
    And I force close subtask "Done step"
    And I claim subtask "WIP step"
    And I run tk epic with JSON
    Then the epic JSON shows "Mixed epic" with 1 done, 1 in_progress, 1 open

  Scenario: All-done epic shows N/0/0 in JSON
    Given I have a task called "epic" with title "Finished epic" and tag "epic"
    When I create a subtask of "epic" with title "Step one"
    And I create a subtask of "epic" with title "Step two"
    And I force close subtask "Step one"
    And I force close subtask "Step two"
    And I run tk epic with JSON
    Then the epic JSON shows "Finished epic" with 2 done, 0 in_progress, 0 open

  Scenario: All-open epic shows 0/0/N in JSON
    Given I have a task called "epic" with title "Fresh epic" and tag "epic"
    When I create a subtask of "epic" with title "Step A"
    And I create a subtask of "epic" with title "Step B"
    And I create a subtask of "epic" with title "Step C"
    And I run tk epic with JSON
    Then the epic JSON shows "Fresh epic" with 0 done, 0 in_progress, 3 open

  Scenario: Human-readable epic output shows done/wip/open format
    Given I have a task called "epic" with title "Progress epic" and tag "epic"
    When I create a subtask of "epic" with title "Done step"
    And I create a subtask of "epic" with title "WIP step"
    And I create a subtask of "epic" with title "Open step"
    And I force close subtask "Done step"
    And I claim subtask "WIP step"
    And I run tk epic
    Then the epic output contains "1/1/1"
