Feature: Task reparenting
  As a user
  I want to move existing tasks under epics or promote subtasks to top-level
  So that I can reorganize work after the fact

  Background:
    Given a tacks database is initialized

  Scenario: Reparent a top-level task under an epic
    Given I have a task called "epic" with title "Big project"
    And I have a task called "task1" with title "Some work"
    When I reparent "task1" under "epic"
    And I show task "task1" in JSON
    Then the JSON field "parent_id" equals the ID of "epic"

  Scenario: New parent is auto-tagged as epic on reparent
    Given I have a task called "parent" with title "Becomes epic"
    And I have a task called "child" with title "Move me"
    When I reparent "child" under "parent"
    And I show task "parent" in JSON
    Then the task details include tag "epic"

  Scenario: Promote a subtask to top-level
    Given I have a task called "epic" with title "Big project"
    And I have a subtask of "epic" called "sub" with title "Subtask"
    When I reparent "sub" under "none"
    And I show task "sub" in JSON
    Then the JSON field "parent_id" is null

  Scenario: Task retains original ID after reparent
    Given I have a task called "epic" with title "Big project"
    And I have a task called "task1" with title "Keep my ID"
    When I store the ID of "task1"
    And I reparent "task1" under "epic"
    And I show task "task1" in JSON
    Then the task ID matches the stored ID

  Scenario: Reject self-parenting
    Given I have a task called "task1" with title "Self ref"
    When I try to reparent "task1" under "task1"
    Then the command should fail
    And the error output contains "cannot reparent a task under itself"

  Scenario: Reject reparent under non-existent task
    Given I have a task called "task1" with title "Orphan"
    When I run tk update for "task1" with --parent "tk-9999"
    Then the command should fail
    And the error output contains "parent task not found"

  Scenario: Reject reparent under a subtask (max depth 1)
    Given I have a task called "epic" with title "Top level"
    And I have a subtask of "epic" called "sub" with title "Child"
    And I have a task called "task1" with title "Move me"
    When I try to reparent "task1" under "sub"
    Then the command should fail
    And the error output contains "already a subtask"

  Scenario: Reparent between epics
    Given I have a task called "epic1" with title "First epic" and tag "epic"
    And I have a subtask of "epic1" called "sub" with title "Shared work"
    And I have a task called "epic2" with title "Second epic" and tag "epic"
    When I reparent "sub" under "epic2"
    And I show task "sub" in JSON
    Then the JSON field "parent_id" equals the ID of "epic2"
