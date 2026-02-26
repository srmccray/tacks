Feature: Filter tasks by parent
  As an AI coding agent
  I want to list tasks under a specific parent
  So that I can see subtasks within an epic

  Background:
    Given a tacks database is initialized

  Scenario: Listing tasks with --parent shows only children
    Given I have a task called "epic" with title "Big project"
    And I have a task called "other" with title "Unrelated task"
    When I create a subtask of "epic" with title "Sub A"
    And I create a subtask of "epic" with title "Sub B"
    And I list tasks with parent "epic"
    Then the filtered list contains "Sub A"
    And the filtered list contains "Sub B"
    And the filtered list does not contain "Unrelated task"
    And the filtered list does not contain "Big project"
