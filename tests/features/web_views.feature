Feature: Web view pages
  As a developer using the browser UI
  I want HTML views for tasks, board, epics, and forms
  So that I can manage my backlog without using the CLI

  Background:
    Given a tacks database is initialized
    And the web server is running

  # ---------------------------------------------------------------------------
  # Task list page — GET /tasks
  # ---------------------------------------------------------------------------

  Scenario: Task list page returns HTML
    When I GET "/tasks"
    Then the response status is 200
    And the response content type is "text/html"
    And the response body contains "<html"

  Scenario: Task list page contains a task table
    When I GET "/tasks"
    Then the response status is 200
    And the response body contains "<table"

  Scenario: Task list page shows created task titles
    Given I created a task via API with title "Write documentation" as "doc-task"
    And I created a task via API with title "Fix login bug" as "bug-task"
    When I GET "/tasks"
    Then the response status is 200
    And the response body contains "Write documentation"
    And the response body contains "Fix login bug"

  Scenario: Task list page filters by status=open
    Given I created a task via API with title "Open task" as "open-t"
    And I created a task via API with title "Done task" as "done-t"
    And I closed the API task "done-t"
    When I GET "/tasks?status=open"
    Then the response status is 200
    And the response body contains "Open task"

  Scenario: Task list page filters by priority
    Given I created a task via API with title "High priority task" and priority 1 as "p1-view"
    And I created a task via API with title "Low priority task" and priority 3 as "p3-view"
    When I GET "/tasks?priority=1"
    Then the response status is 200
    And the response body contains "High priority task"

  # ---------------------------------------------------------------------------
  # Task detail page — GET /tasks/:id
  # ---------------------------------------------------------------------------

  Scenario: Task detail page returns HTML with task title
    Given I created a task via API with title "Detail view task" as "detail-task"
    When I GET the HTML task "detail-task"
    Then the response status is 200
    And the response content type is "text/html"
    And the response body contains "Detail view task"

  Scenario: Task detail page shows task status and priority
    Given I created a task via API with title "Status check task" and priority 2 as "status-view"
    When I GET the HTML task "status-view"
    Then the response status is 200
    And the response body contains "open"
    And the response body contains "2"

  Scenario: Task detail page shows task description
    Given I created a task via API with title "Described task" and description "This task has a detailed description" as "desc-task"
    When I GET the HTML task "desc-task"
    Then the response status is 200
    And the response body contains "This task has a detailed description"

  Scenario: Task detail page returns 404 for unknown id
    When I GET "/tasks/tk-0000"
    Then the response status is 404

  # ---------------------------------------------------------------------------
  # Board view — GET /board
  # ---------------------------------------------------------------------------

  Scenario: Board page returns HTML
    When I GET "/board"
    Then the response status is 200
    And the response content type is "text/html"
    And the response body contains "<html"

  Scenario: Board page has four status column headers
    When I GET "/board"
    Then the response status is 200
    And the response body contains "Open"
    And the response body contains "In Progress"
    And the response body contains "Blocked"
    And the response body contains "Done"

  Scenario: Board page shows tasks in their correct columns
    Given I created a task via API with title "Open board task" as "board-open"
    When I GET "/board"
    Then the response status is 200
    And the response body contains "Open board task"

  # ---------------------------------------------------------------------------
  # Epic view — GET /epics
  # ---------------------------------------------------------------------------

  Scenario: Epics page returns HTML
    When I GET "/epics"
    Then the response status is 200
    And the response content type is "text/html"
    And the response body contains "<html"

  Scenario: Epics page shows epic progress table
    When I GET "/epics"
    Then the response status is 200
    And the response body contains "<table"

  Scenario: Epics page lists epics with subtasks
    Given I created a task via API with title "Main epic" as "epic-parent"
    And I created a subtask via API with title "Epic subtask" under "epic-parent" as "epic-child"
    When I GET "/epics"
    Then the response status is 200
    And the response body contains "Main epic"

  # ---------------------------------------------------------------------------
  # Create task form — GET /tasks/new
  # ---------------------------------------------------------------------------

  Scenario: Create task form returns HTML
    When I GET "/tasks/new"
    Then the response status is 200
    And the response content type is "text/html"
    And the response body contains "<html"

  Scenario: Create task form contains a form element
    When I GET "/tasks/new"
    Then the response status is 200
    And the response body contains "<form"

  Scenario: Create task form has title input field
    When I GET "/tasks/new"
    Then the response status is 200
    And the response body contains "title"

  Scenario: Create task form has priority input field
    When I GET "/tasks/new"
    Then the response status is 200
    And the response body contains "priority"

  Scenario: Create task form has description input field
    When I GET "/tasks/new"
    Then the response status is 200
    And the response body contains "description"

  # ---------------------------------------------------------------------------
  # Navigation — all pages include nav links
  # ---------------------------------------------------------------------------

  Scenario: Task list page includes navigation
    When I GET "/tasks"
    Then the response status is 200
    And the response body contains "<nav"

  Scenario: Board page includes navigation
    When I GET "/board"
    Then the response status is 200
    And the response body contains "<nav"

  Scenario: Epics page includes navigation
    When I GET "/epics"
    Then the response status is 200
    And the response body contains "<nav"

  # ---------------------------------------------------------------------------
  # Full page base template — direct navigation returns complete HTML
  # ---------------------------------------------------------------------------

  Scenario: Task list page includes full HTML document structure
    When I GET "/tasks"
    Then the response status is 200
    And the response body contains "<!DOCTYPE html"
    And the response body contains "<head"
    And the response body contains "<body"
    And the response body contains "</html>"

  Scenario: Board page includes full HTML document structure
    When I GET "/board"
    Then the response status is 200
    And the response body contains "<!DOCTYPE html"
    And the response body contains "</html>"

  Scenario: Task list page has link to board view
    When I GET "/tasks"
    Then the response status is 200
    And the response body contains "/board"

  Scenario: Task list page has link to epics view
    When I GET "/tasks"
    Then the response status is 200
    And the response body contains "/epics"
