Feature: Dependency visualization
  As an AI coding agent
  I want to see dependency relationships clearly
  So that I can understand what is blocking or depending on tasks

  Background:
    Given a tacks database is initialized

  Scenario: Task with no dependencies shows empty blockers and dependents
    Given I have a task called "solo" with title "Standalone task"
    When I show task "solo" in JSON
    Then the task details have no blockers
    And the task details have no dependents

  Scenario: Task with a blocker shows the blocker in show output
    Given I have a task called "A" with title "Foundation work"
    And I have a task called "B" with title "Depends on foundation"
    When I add a dependency so "B" is blocked by "A"
    And I show task "B" in JSON
    Then the task details include blocker "Foundation work"

  Scenario: Task with a dependent shows the dependent in show output
    Given I have a task called "A" with title "Library task"
    And I have a task called "B" with title "Consumer task"
    When I add a dependency so "B" is blocked by "A"
    And I show task "A" in JSON
    Then the task details include dependent "Consumer task"

  Scenario: Blocked task appears in tk list output
    Given I have a task called "blocker" with title "Must do first"
    And I have a task called "blocked" with title "Waiting for blocker"
    When I add a dependency so "blocked" is blocked by "blocker"
    Then the task list contains "Waiting for blocker"

  Scenario: Linear chain shows all intermediate tasks in blocked list
    Given I have a task called "A" with title "Step one"
    And I have a task called "B" with title "Step two"
    And I have a task called "C" with title "Step three"
    When I add a dependency so "B" is blocked by "A"
    And I add a dependency so "C" is blocked by "B"
    And I run tk blocked with JSON
    Then the blocked output contains "Step two"
    And the blocked output contains "Step three"
    And the blocked output does not contain "Step one"

  Scenario: Closing middle blocker in chain unblocks downstream task
    Given I have a task called "A" with title "First step"
    And I have a task called "B" with title "Middle step"
    And I have a task called "C" with title "Last step"
    When I add a dependency so "B" is blocked by "A"
    And I add a dependency so "C" is blocked by "B"
    And I close task "A" with reason "done"
    And I run tk blocked with JSON
    Then the blocked output does not contain "Middle step"
    And the blocked output contains "Last step"

  Scenario: Task is unblocked once all its blockers are closed
    Given I have a task called "dep1" with title "Prerequisite one"
    And I have a task called "dep2" with title "Prerequisite two"
    And I have a task called "main" with title "Main work"
    When I add a dependency so "main" is blocked by "dep1"
    And I add a dependency so "main" is blocked by "dep2"
    And I close task "dep1" with reason "done"
    And I run tk blocked with JSON
    Then the blocked output contains "Main work"
    When I close task "dep2" with reason "done"
    And I run tk blocked with JSON
    Then the JSON output is an empty array
