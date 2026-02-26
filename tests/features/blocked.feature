Feature: Blocked task listing
  As an AI coding agent
  I want to see which tasks are blocked
  So that I can identify and resolve blockers

  Background:
    Given a tacks database is initialized

  Scenario: Blocked tasks are shown when dependencies exist
    Given I have a task called "blocker" with title "Must finish first"
    And I have a task called "blocked" with title "Waiting on blocker"
    When I add a dependency so "blocked" is blocked by "blocker"
    And I run tk blocked with JSON
    Then the blocked output contains "Waiting on blocker"
    And the blocked output does not contain "Must finish first"

  Scenario: No blocked tasks shows empty list
    Given I have a task called "free" with title "Free task"
    When I run tk blocked with JSON
    Then the JSON output is an empty array

  Scenario: Resolved blocker removes task from blocked list
    Given I have a task called "blocker" with title "Resolve me"
    And I have a task called "blocked" with title "Waiting task"
    When I add a dependency so "blocked" is blocked by "blocker"
    And I close task "blocker" with reason "done"
    And I run tk blocked with JSON
    Then the JSON output is an empty array
