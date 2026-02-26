Feature: List filtering
  As an AI coding agent
  I want to filter task lists
  So that I can see only the tasks relevant to my current context

  Background:
    Given a tacks database is initialized

  Scenario: tk list -s done shows only closed tasks
    Given I have a task called "open_task" with title "Still open"
    And I have a task called "done_task" with title "Already done"
    And I close the task "done_task"
    When I list tasks filtered by status "done"
    Then the filtered list contains "Already done"
    And the filtered list does not contain "Still open"

  Scenario: tk list -p 1 shows only priority 1 tasks
    Given I have a task called "p1" with title "High priority task" and priority 1
    And I have a task called "p2" with title "Medium priority task" and priority 2
    When I list tasks filtered by priority 1
    Then the filtered list contains "High priority task"
    And the filtered list does not contain "Medium priority task"

  Scenario: tk list -t backend shows only tasks with that tag
    Given I have a task called "be" with title "Backend service" and tag "backend"
    And I have a task called "fe" with title "Frontend UI" and tag "frontend"
    When I list tasks filtered by tag "backend"
    Then the filtered list contains "Backend service"
    And the filtered list does not contain "Frontend UI"

  Scenario: tk list -a shows all tasks including closed
    Given I have a task called "open_t" with title "Open one"
    And I have a task called "done_t" with title "Closed one"
    And I close the task "done_t"
    When I list all tasks including closed
    Then the filtered list contains "Open one"
    And the filtered list contains "Closed one"

  Scenario: tk list by default hides closed tasks
    Given I have a task called "active" with title "Active task"
    And I have a task called "finished" with title "Finished task"
    And I close the task "finished"
    When I list tasks with default settings
    Then the filtered list contains "Active task"
    And the filtered list does not contain "Finished task"
