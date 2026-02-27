Feature: REST API endpoints
  As an API consumer or web UI
  I want a JSON REST API over HTTP
  So that I can create, read, update, and manage tasks programmatically

  Background:
    Given a tacks database is initialized
    And the web server is running

  # ---------------------------------------------------------------------------
  # Task creation — POST /api/tasks
  # ---------------------------------------------------------------------------

  Scenario: POST /api/tasks creates a task and returns 201
    When I POST "/api/tasks" with body '{"title":"Buy milk","priority":2}'
    Then the response status is 201
    And the response JSON has field "id"
    And the response JSON field "title" equals "Buy milk"
    And the response JSON field "status" equals "open"

  Scenario: POST /api/tasks with tags stores them on the task
    When I POST "/api/tasks" with body '{"title":"Tagged task","tags":["backend","urgent"]}'
    Then the response status is 201
    And the response JSON field "title" equals "Tagged task"

  Scenario: POST /api/tasks with missing title returns 422
    When I POST "/api/tasks" with body '{"priority":1}'
    Then the response status is 422

  # ---------------------------------------------------------------------------
  # Task listing — GET /api/tasks
  # ---------------------------------------------------------------------------

  Scenario: GET /api/tasks returns all open tasks
    Given I created a task via API with title "Alpha task" as "alpha"
    And I created a task via API with title "Beta task" as "beta"
    When I GET "/api/tasks"
    Then the response status is 200
    And the response JSON array contains a task with title "Alpha task"
    And the response JSON array contains a task with title "Beta task"

  Scenario: GET /api/tasks?status=done excludes open tasks
    Given I created a task via API with title "Open task" as "open"
    And I created a task via API with title "Done task" as "done"
    And I closed the API task "done"
    When I GET "/api/tasks?status=done"
    Then the response status is 200
    And the response JSON array contains a task with title "Done task"
    And the response JSON array does not contain a task with title "Open task"

  Scenario: GET /api/tasks?priority=1 filters by priority
    Given I created a task via API with title "P1 task" and priority 1 as "p1"
    And I created a task via API with title "P3 task" and priority 3 as "p3"
    When I GET "/api/tasks?priority=1"
    Then the response status is 200
    And the response JSON array contains a task with title "P1 task"
    And the response JSON array does not contain a task with title "P3 task"

  Scenario: GET /api/tasks?tag=backend filters by tag
    Given I created a task via API with title "Backend task" and tag "backend" as "be"
    And I created a task via API with title "Frontend task" and tag "frontend" as "fe"
    When I GET "/api/tasks?tag=backend"
    Then the response status is 200
    And the response JSON array contains a task with title "Backend task"
    And the response JSON array does not contain a task with title "Frontend task"

  # ---------------------------------------------------------------------------
  # Task detail — GET /api/tasks/:id
  # ---------------------------------------------------------------------------

  Scenario: GET /api/tasks/:id returns task details
    Given I created a task via API with title "Detailed task" as "detail"
    When I GET the API task "detail"
    Then the response status is 200
    And the response JSON field "title" equals "Detailed task"
    And the response JSON has field "id"
    And the response JSON has field "status"
    And the response JSON has field "priority"

  Scenario: GET /api/tasks/:id returns 404 for unknown id
    When I GET "/api/tasks/tk-0000"
    Then the response status is 404

  # ---------------------------------------------------------------------------
  # Task update — PATCH /api/tasks/:id
  # ---------------------------------------------------------------------------

  Scenario: PATCH /api/tasks/:id updates the title
    Given I created a task via API with title "Old title" as "updatable"
    When I PATCH the API task "updatable" with body '{"title":"New title"}'
    Then the response status is 200
    And the response JSON field "title" equals "New title"

  Scenario: PATCH /api/tasks/:id updates the status
    Given I created a task via API with title "Status task" as "status-task"
    When I PATCH the API task "status-task" with body '{"status":"in_progress"}'
    Then the response status is 200
    And the response JSON field "status" equals "in_progress"

  Scenario: PATCH /api/tasks/:id updates the priority
    Given I created a task via API with title "Priority task" as "prio-task"
    When I PATCH the API task "prio-task" with body '{"priority":1}'
    Then the response status is 200
    And the response JSON field "priority" equals 1

  Scenario: PATCH /api/tasks/:id returns 404 for unknown id
    When I PATCH "/api/tasks/tk-0000" with body '{"title":"Ghost"}'
    Then the response status is 404

  # ---------------------------------------------------------------------------
  # Task close — POST /api/tasks/:id/close
  # ---------------------------------------------------------------------------

  Scenario: POST /api/tasks/:id/close closes a task
    Given I created a task via API with title "Task to close" as "closeable"
    When I POST the close endpoint for API task "closeable" with body '{"reason":"done"}'
    Then the response status is 200
    And the response JSON field "status" equals "done"
    And the response JSON field "close_reason" equals "done"

  Scenario: POST /api/tasks/:id/close with a comment stores it
    Given I created a task via API with title "Task with comment" as "comment-task"
    When I POST the close endpoint for API task "comment-task" with body '{"reason":"duplicate","comment":"Covered by tk-0001"}'
    Then the response status is 200
    And the response JSON field "status" equals "done"

  Scenario: POST /api/tasks/:id/close with invalid reason returns 422
    Given I created a task via API with title "Bad close task" as "bad-close"
    When I POST the close endpoint for API task "bad-close" with body '{"reason":"bogus"}'
    Then the response status is 422

  Scenario: POST /api/tasks/:id/close returns 404 for unknown id
    When I POST "/api/tasks/tk-0000/close" with body '{"reason":"done"}'
    Then the response status is 404

  # ---------------------------------------------------------------------------
  # Ready list — GET /api/tasks/ready
  # ---------------------------------------------------------------------------

  Scenario: GET /api/tasks/ready returns unblocked tasks
    Given I created a task via API with title "Unblocked task" as "free"
    When I GET "/api/tasks/ready"
    Then the response status is 200
    And the response JSON array contains a task with title "Unblocked task"

  Scenario: GET /api/tasks/ready excludes blocked tasks
    Given I created a task via API with title "Blocker" as "blocker"
    And I created a task via API with title "Blocked by blocker" as "blocked"
    And I added API dependency so "blocked" is blocked by "blocker"
    When I GET "/api/tasks/ready"
    Then the response status is 200
    And the response JSON array does not contain a task with title "Blocked by blocker"

  Scenario: GET /api/tasks/ready?limit=1 returns at most one task
    Given I created a task via API with title "Ready A" as "ra"
    And I created a task via API with title "Ready B" as "rb"
    When I GET "/api/tasks/ready?limit=1"
    Then the response status is 200
    And the response JSON array has length 1

  # ---------------------------------------------------------------------------
  # Blocked list — GET /api/tasks/blocked
  # ---------------------------------------------------------------------------

  Scenario: GET /api/tasks/blocked returns tasks with open blockers
    Given I created a task via API with title "Prerequisite" as "prereq"
    And I created a task via API with title "Waiting task" as "waiter"
    And I added API dependency so "waiter" is blocked by "prereq"
    When I GET "/api/tasks/blocked"
    Then the response status is 200
    And the response JSON array contains a task with title "Waiting task"

  Scenario: GET /api/tasks/blocked excludes unblocked tasks
    Given I created a task via API with title "Free task" as "free2"
    When I GET "/api/tasks/blocked"
    Then the response status is 200
    And the response JSON array does not contain a task with title "Free task"

  # ---------------------------------------------------------------------------
  # Dependencies — POST and DELETE /api/tasks/:id/deps
  # ---------------------------------------------------------------------------

  Scenario: POST /api/tasks/:id/deps adds a dependency
    Given I created a task via API with title "Dep parent" as "dep-parent"
    And I created a task via API with title "Dep child" as "dep-child"
    When I POST the deps endpoint for API task "dep-child" with body '{"parent_id":"dep-parent"}'
    Then the response status is 201

  Scenario: POST /api/tasks/:id/deps returns 409 on cycle
    Given I created a task via API with title "Cycle A" as "cyc-a"
    And I created a task via API with title "Cycle B" as "cyc-b"
    And I added API dependency so "cyc-b" is blocked by "cyc-a"
    When I POST the deps endpoint for API task "cyc-a" with body '{"parent_id":"cyc-b"}'
    Then the response status is 409

  Scenario: DELETE /api/tasks/:child/deps/:parent removes a dependency
    Given I created a task via API with title "Parent dep" as "rem-parent"
    And I created a task via API with title "Child dep" as "rem-child"
    And I added API dependency so "rem-child" is blocked by "rem-parent"
    When I DELETE the API dependency from "rem-child" to "rem-parent"
    Then the response status is 204

  # ---------------------------------------------------------------------------
  # Comments — POST and GET /api/tasks/:id/comments
  # ---------------------------------------------------------------------------

  Scenario: POST /api/tasks/:id/comments adds a comment
    Given I created a task via API with title "Commented task" as "comm-task"
    When I POST the comments endpoint for API task "comm-task" with body '{"body":"Work in progress"}'
    Then the response status is 201
    And the response JSON has field "id"
    And the response JSON field "body" equals "Work in progress"

  Scenario: GET /api/tasks/:id/comments lists all comments
    Given I created a task via API with title "Multi-comment task" as "multi-comm"
    And I posted a comment "First note" on API task "multi-comm"
    And I posted a comment "Second note" on API task "multi-comm"
    When I GET the comments endpoint for API task "multi-comm"
    Then the response status is 200
    And the response JSON array contains a comment with body "First note"
    And the response JSON array contains a comment with body "Second note"

  Scenario: GET /api/tasks/:id/comments returns empty array when no comments
    Given I created a task via API with title "No-comment task" as "no-comm"
    When I GET the comments endpoint for API task "no-comm"
    Then the response status is 200
    And the response JSON is an empty array

  # ---------------------------------------------------------------------------
  # Stats — GET /api/stats
  # ---------------------------------------------------------------------------

  Scenario: GET /api/stats returns structured counts
    Given I created a task via API with title "Stats task" as "stats1"
    When I GET "/api/stats"
    Then the response status is 200
    And the response JSON has field "by_status"
    And the response JSON has field "by_priority"
    And the response JSON has field "by_tag"

  Scenario: GET /api/stats reflects closed tasks in done count
    Given I created a task via API with title "Open one" as "so1"
    And I created a task via API with title "Done one" as "sd1"
    And I closed the API task "sd1"
    When I GET "/api/stats"
    Then the response status is 200
    And the response JSON nested field "by_status.done" equals 1

  # ---------------------------------------------------------------------------
  # Children — GET /api/tasks/:id/children
  # ---------------------------------------------------------------------------

  Scenario: GET /api/tasks/:id/children returns subtasks
    Given I created a task via API with title "Epic parent" as "epic"
    And I created a subtask via API with title "Child one" under "epic" as "child1"
    And I created a subtask via API with title "Child two" under "epic" as "child2"
    When I GET the children endpoint for API task "epic"
    Then the response status is 200
    And the response JSON array contains a task with title "Child one"
    And the response JSON array contains a task with title "Child two"

  Scenario: GET /api/tasks/:id/children returns empty array for task with no children
    Given I created a task via API with title "Childless task" as "childless"
    When I GET the children endpoint for API task "childless"
    Then the response status is 200
    And the response JSON is an empty array
