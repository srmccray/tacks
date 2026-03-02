Feature: Board drag-and-drop status transitions
  As a user viewing the kanban board
  I want to drag tasks between status columns
  So that I can update task status visually
  The board drag-and-drop JS calls PATCH /api/tasks/:id with the new status.
  These scenarios verify that the underlying API correctly handles each transition.

  Background:
    Given a tacks database is initialized
    And the web server is running

  Scenario: Drag task from Open to In Progress
    Given I created a task via API with title "New feature" as "feature"
    When I PATCH the API task "feature" with body '{"status":"in_progress"}'
    Then the response status is 200
    And the response JSON field "status" equals "in_progress"
    When I GET the API task "feature"
    Then the response status is 200
    And the response JSON field "status" equals "in_progress"

  Scenario: Drag task from In Progress to Done
    Given I created a task via API with title "Work in progress" as "wip"
    When I PATCH the API task "wip" with body '{"status":"in_progress"}'
    And I PATCH the API task "wip" with body '{"status":"done"}'
    Then the response status is 200
    And the response JSON field "status" equals "done"
    When I GET the API task "wip"
    Then the response status is 200
    And the response JSON field "status" equals "done"

  Scenario: Drag task to Blocked column
    Given I created a task via API with title "Waiting on dependency" as "waiting"
    When I PATCH the API task "waiting" with body '{"status":"blocked"}'
    Then the response status is 200
    And the response JSON field "status" equals "blocked"
    When I GET the API task "waiting"
    Then the response status is 200
    And the response JSON field "status" equals "blocked"

  Scenario: Drag task back from Done to Open
    Given I created a task via API with title "Reopened task" as "reopened"
    When I PATCH the API task "reopened" with body '{"status":"done"}'
    And I PATCH the API task "reopened" with body '{"status":"open"}'
    Then the response status is 200
    And the response JSON field "status" equals "open"
    When I GET the API task "reopened"
    Then the response status is 200
    And the response JSON field "status" equals "open"

  Scenario: Drop on same column is a no-op
    Given I created a task via API with title "Stable task" as "stable"
    When I PATCH the API task "stable" with body '{"status":"open"}'
    Then the response status is 200
    And the response JSON field "status" equals "open"
    When I GET the API task "stable"
    Then the response status is 200
    And the response JSON field "status" equals "open"

  Scenario: Drag-and-drop on epic detail board updates only the subtask
    Given I created a task via API with title "My epic" as "epic"
    And I created a subtask via API with title "Subtask one" under "epic" as "subtask"
    When I PATCH the API task "subtask" with body '{"status":"in_progress"}'
    Then the response status is 200
    And the response JSON field "status" equals "in_progress"
    When I GET the API task "epic"
    Then the response status is 200
    And the response JSON field "status" equals "open"
    When I GET the API task "subtask"
    Then the response status is 200
    And the response JSON field "status" equals "in_progress"
