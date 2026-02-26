Feature: Notes as mutable working context
  As an AI coding agent
  I want to set and update working notes on tasks
  So that I can track current context distinct from append-only comments

  Background:
    Given a tacks database is initialized

  Scenario: Setting notes on a task
    Given I have a task called "work" with title "Feature work"
    When I update task "work" with notes "Started implementation"
    And I show task "work" in JSON
    Then the task details show notes "Started implementation"

  Scenario: Notes overwrite previous value
    Given I have a task called "work" with title "Iterating"
    When I update task "work" with notes "First draft"
    And I update task "work" with notes "Revised approach"
    And I show task "work" in JSON
    Then the task details show notes "Revised approach"

  Scenario: Task without notes shows null
    When I create a task with title "No notes task"
    And I show the task
    Then the task details have no notes
