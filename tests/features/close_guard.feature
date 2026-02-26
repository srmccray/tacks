Feature: Close guard for tasks with open subtasks
  As an AI coding agent
  I want to be warned when closing a task that has open subtasks
  So that I don't accidentally close an epic with unfinished work

  Background:
    Given a tacks database is initialized

  Scenario: Closing a task with open subtasks is rejected
    Given I have a task called "parent" with title "Core module"
    When I create a subtask of "parent" with title "Open subtask"
    And I try to close task "parent"
    Then the command should fail
    And the error output contains "open dependent"

  Scenario: Closing a task with --force bypasses the guard
    Given I have a task called "parent" with title "Forceable task"
    When I create a subtask of "parent" with title "Dependent subtask"
    And I force close task "parent"
    And I show task "parent" in JSON
    Then the task details show status "done"

  Scenario: Closing a task with no subtasks works normally
    Given I have a task called "solo" with title "No subtasks"
    When I close task "solo" with reason "done"
    And I show task "solo" in JSON
    Then the task details show status "done"

  Scenario: Closing a task whose subtasks are already done works
    Given I have a task called "parent" with title "Already clear"
    When I create a subtask of "parent" with title "Already done child"
    And I force close subtask "Already done child"
    And I close task "parent" with reason "done"
    And I show task "parent" in JSON
    Then the task details show status "done"
