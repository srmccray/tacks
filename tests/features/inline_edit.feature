Feature: Inline editing of task fields
  Tasks can be edited field-by-field via the PATCH /api/tasks/:id endpoint,
  supporting the click-to-edit inline editing UI.  Each scenario verifies that
  the change persists by fetching the task via GET after the PATCH.

  Background:
    Given a tacks database is initialized
    And the web server is running
    And a task "Sample task" exists via API

  Scenario: Edit title saves via API
    When I PATCH the task with '{"title": "Updated title"}'
    Then the task title should be "Updated title"

  Scenario: Edit status saves via API
    When I PATCH the task with '{"status": "in_progress"}'
    Then the task status should be "in_progress"

  Scenario: Edit priority saves via API
    When I PATCH the task with '{"priority": 2}'
    Then the task priority should be 2

  Scenario: Edit assignee saves via API
    When I PATCH the task with '{"assignee": "alice"}'
    Then the task assignee should be "alice"

  Scenario: Edit tags saves via API
    When I PATCH the task with '{"tags": ["ui", "backend"]}'
    Then the task tags should include "ui"
    And the task tags should include "backend"

  Scenario: Edit description saves via API
    When I PATCH the task with '{"description": "new desc"}'
    Then the task description should be "new desc"

  Scenario: Idempotent update with original value leaves field unchanged
    When I PATCH the task with '{"title": "Sample task"}'
    Then the task title should be "Sample task"

  Scenario: Datetime fields are not affected by title update
    Given the task created_at is stored
    When I PATCH the task with '{"title": "Different title"}'
    Then the task created_at should not change
