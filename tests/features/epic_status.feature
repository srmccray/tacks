Feature: Epic progress tracking
  As an AI coding agent
  I want to see completion progress for all epics
  So that I can track overall project health

  Background:
    Given a tacks database is initialized

  Scenario: Epic shows child completion progress
    Given I have a task called "epic" with title "Big feature" and tag "epic"
    When I create a subtask of "epic" with title "Step one"
    And I create a subtask of "epic" with title "Step two"
    And I run tk epic with JSON
    Then the epic output shows "Big feature" with 0 of 2 done

  Scenario: Epic progress updates when children close
    Given I have a task called "epic" with title "Progressing feature" and tag "epic"
    When I create a subtask of "epic" with title "Done step"
    And I create a subtask of "epic" with title "Open step"
    And I force close subtask "Done step"
    And I run tk epic with JSON
    Then the epic output shows "Progressing feature" with 1 of 2 done

  Scenario: No epics shows empty output
    Given I have a task called "task" with title "Regular task"
    When I run tk epic with JSON
    Then the JSON output is an empty array
