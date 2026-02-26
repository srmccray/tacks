Feature: Task lifecycle
  As an AI coding agent
  I want to create, inspect, and close tasks
  So that I can track work from start to finish

  Background:
    Given a tacks database is initialized

  Scenario: Creating a task shows it in the task list
    When I create a task with title "Implement login"
    Then the task list contains "Implement login"

  Scenario: Showing a task displays its details
    When I create a task with title "Write tests"
    And I show the task
    Then the task details show title "Write tests"
    And the task details show status "open"

  Scenario: Closing a task removes it from the active list
    When I create a task with title "Fix bug"
    And I close the task
    Then the task list does not contain "Fix bug"

  Scenario: Closing a task marks it as done
    When I create a task with title "Deploy service"
    And I close the task
    And I show the task
    Then the task details show status "done"

  Scenario: Creating a task with priority and tags
    When I create a task with title "Urgent fix" and priority 1 and tags "backend,critical"
    Then the task list contains "Urgent fix"
    When I show the task
    Then the task details show title "Urgent fix"
    And the task details show priority 1
