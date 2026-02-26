Feature: Agent workflow commands
  As an AI coding agent
  I want commands that support typical agent workflows
  So that I can claim tasks, comment on progress, and see aggregate stats

  Background:
    Given a tacks database is initialized

  Scenario: tk update --claim sets status to in_progress and assignee
    Given I have a task called "work" with title "Feature to implement"
    When I claim the task "work"
    Then the task "work" has status "in_progress"
    And the task "work" has assignee "agent"

  Scenario: tk comment adds a comment visible in tk show
    Given I have a task called "commented" with title "Task to comment on"
    When I add a comment "Making progress" to the task "commented"
    And I show the task "commented"
    Then the task details show a comment with body "Making progress"

  Scenario: tk stats shows correct counts by status
    Given I have a task called "t1" with title "Open task one"
    And I have a task called "t2" with title "Open task two"
    And I close the task "t2"
    When I run tk stats with json output
    Then the stats JSON shows "open" count of 1
    And the stats JSON shows "done" count of 1

  Scenario: tk stats --oneline outputs compact format
    Given I have a task called "t1" with title "One open task"
    When I run tk stats with oneline output
    Then the oneline output contains "open"

  Scenario: tk stats --json outputs structured JSON
    Given I have a task called "t1" with title "Tagged stats task" and tag "backend"
    When I run tk stats with json output
    Then the stats JSON has a "by_status" field
    And the stats JSON has a "by_priority" field
    And the stats JSON has a "by_tag" field

  Scenario: tk ready --limit 1 returns only one task
    Given I have a task called "a" with title "Ready task A"
    And I have a task called "b" with title "Ready task B"
    And I have a task called "c" with title "Ready task C"
    When I run tk ready with limit 1
    Then the ready list contains exactly 1 task

  Scenario: tk ready with no ready tasks returns empty list
    Given I have a task called "blocker" with title "The blocker"
    And I have a task called "blocked" with title "The blocked"
    When I add a dependency so "blocked" is blocked by "blocker"
    And I close the task "blocker"
    And I close the task "blocked"
    When I run tk ready with json output
    Then the ready list is empty
