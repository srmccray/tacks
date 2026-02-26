Feature: List child tasks
  As an AI coding agent
  I want to list children of a parent task
  So that I can see subtasks within an epic

  Background:
    Given a tacks database is initialized

  Scenario: Listing children of a parent task
    Given I have a task called "parent" with title "Epic task"
    When I create a subtask of "parent" with title "Child one"
    And I create a subtask of "parent" with title "Child two"
    And I run tk children for "parent"
    Then the output contains "Child one"
    And the output contains "Child two"

  Scenario: Children of a task with no children shows empty
    Given I have a task called "leaf" with title "Leaf task"
    When I run tk children for "leaf" with JSON
    Then the JSON output is an empty array

  Scenario: Children command with invalid ID fails
    When I run tk children for "tk-0000"
    Then the command should fail
    And the error output contains "not found"
