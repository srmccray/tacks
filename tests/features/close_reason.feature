Feature: Close reason tracking
  As an AI coding agent
  I want to specify why a task was closed
  So that I can distinguish done, duplicate, absorbed, stale, and superseded tasks

  Background:
    Given a tacks database is initialized

  Scenario: Closing a task with default reason
    When I create a task with title "Default close"
    And I close the task
    And I show the task
    Then the task details show close_reason "done"

  Scenario: Closing a task with explicit reason
    Given I have a task called "dup" with title "Duplicate work"
    When I close task "dup" with reason "duplicate"
    And I show task "dup" in JSON
    Then the task details show close_reason "duplicate"

  Scenario: Closing a task with stale reason
    Given I have a task called "old" with title "Stale task"
    When I close task "old" with reason "stale"
    And I show task "old" in JSON
    Then the task details show close_reason "stale"

  Scenario: Invalid close reason is rejected
    Given I have a task called "bad" with title "Bad reason"
    When I try to close task "bad" with reason "invalid_reason"
    Then the command should fail
    And the error output contains "invalid close reason"
