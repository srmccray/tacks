Feature: Dependency management
  As an AI coding agent
  I want to manage task dependencies
  So that I can track which tasks block other tasks

  Background:
    Given a tacks database is initialized

  Scenario: Adding a dependency blocks a task from the ready list
    Given I have a task called "blocker" with title "Prerequisite work"
    And I have a task called "blocked" with title "Depends on prerequisite"
    When I add a dependency so "blocked" is blocked by "blocker"
    Then the ready list does not contain "Depends on prerequisite"
    And the ready list contains "Prerequisite work"

  Scenario: Removing a dependency unblocks a task
    Given I have a task called "blocker" with title "Step one"
    And I have a task called "blocked" with title "Step two"
    When I add a dependency so "blocked" is blocked by "blocker"
    And I remove the dependency so "blocked" is no longer blocked by "blocker"
    Then the ready list contains "Step two"

  Scenario: A task with no dependencies appears in the ready list
    Given I have a task called "solo" with title "Independent task"
    Then the ready list contains "Independent task"

  Scenario: Adding a dependency with an invalid task ID fails gracefully
    Given I have a task called "real" with title "A real task"
    When I try to add a dependency so "real" is blocked by "tk-0000"
    Then the command should fail
    And the error output contains "not found"

  Scenario: Self-dependency is rejected
    Given I have a task called "self" with title "Self referencing task"
    When I try to add a self-dependency for "self"
    Then the command should fail

  Scenario: Show command displays dependents
    Given I have a task called "parent" with title "Core library"
    And I have a task called "child" with title "Uses core library"
    When I add a dependency so "child" is blocked by "parent"
    And I show task "parent" in JSON
    Then the task details include dependent "Uses core library"

  Scenario: Show command displays both blockers and dependents
    Given I have a task called "A" with title "Task A"
    And I have a task called "B" with title "Task B"
    And I have a task called "C" with title "Task C"
    When I add a dependency so "B" is blocked by "A"
    And I add a dependency so "C" is blocked by "B"
    And I show task "B" in JSON
    Then the task details include blocker "Task A"
    And the task details include dependent "Task C"
