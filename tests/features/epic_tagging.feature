Feature: Epic auto-tagging
  As an AI coding agent
  I want parent tasks to be automatically tagged as epics
  So that I can identify epic tasks without manual tagging

  Background:
    Given a tacks database is initialized

  Scenario: Creating a subtask auto-tags parent as epic
    Given I have a task called "parent" with title "Parent task"
    When I create a subtask of "parent" with title "Child task"
    And I show task "parent" in JSON
    Then the task details include tag "epic"

  Scenario: Auto-tag does not duplicate epic tag
    Given I have a task called "parent" with title "Already epic" and tag "epic"
    When I create a subtask of "parent" with title "Another child"
    And I show task "parent" in JSON
    Then the task details show exactly one "epic" tag

  Scenario: Creating a subtask assigns parent ID
    Given I have a task called "parent" with title "Top level"
    When I create a subtask of "parent" with title "Sub level"
    Then the subtask has parent ID matching "parent"
