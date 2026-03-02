Feature: Task creation modal
  As a developer using the browser UI
  I want a modal form for creating new tasks without leaving the current page
  So that I can quickly add tasks from anywhere in the UI

  Background:
    Given a tacks database is initialized
    And the web server is running

  # ---------------------------------------------------------------------------
  # GET /tasks/new/modal — modal form fragment
  # ---------------------------------------------------------------------------

  Scenario: Modal endpoint returns HTML fragment
    When I GET "/tasks/new/modal"
    Then the response status is 200
    And the response content type is "text/html"

  Scenario: Modal form contains a form element
    When I GET "/tasks/new/modal"
    Then the response status is 200
    And the response body contains "<form"

  Scenario: Modal form has a title input field
    When I GET "/tasks/new/modal"
    Then the response status is 200
    And the response body contains "title"

  Scenario: Modal form has a description input field
    When I GET "/tasks/new/modal"
    Then the response status is 200
    And the response body contains "description"

  Scenario: Modal form has a priority input field
    When I GET "/tasks/new/modal"
    Then the response status is 200
    And the response body contains "priority"

  Scenario: Modal form has a status input field
    When I GET "/tasks/new/modal"
    Then the response status is 200
    And the response body contains "status"

  Scenario: Modal form has a tags input field
    When I GET "/tasks/new/modal"
    Then the response status is 200
    And the response body contains "tags"

  Scenario: Modal form has an assignee input field
    When I GET "/tasks/new/modal"
    Then the response status is 200
    And the response body contains "assignee"

  Scenario: Modal form has a parent field
    When I GET "/tasks/new/modal"
    Then the response status is 200
    And the response body contains "parent"

  Scenario: Modal fragment does not include full HTML page wrapper
    When I GET "/tasks/new/modal"
    Then the response status is 200
    And the response body does not contain "<!DOCTYPE"
    And the response body does not contain "<html"

  # ---------------------------------------------------------------------------
  # Parent dropdown populated with epics
  # ---------------------------------------------------------------------------

  Scenario: Modal form shows available epics in parent dropdown
    Given I created a task via API with title "Epic task" as "epic-task"
    And I created a subtask via API with title "Child of epic" under "epic-task" as "epic-child"
    When I GET "/tasks/new/modal"
    Then the response status is 200
    And the response body contains "Epic task"

  # ---------------------------------------------------------------------------
  # Form POST — task creation via modal form
  # ---------------------------------------------------------------------------

  Scenario: POST to modal create endpoint with all fields creates a task
    When I POST the modal create form with title "Modal task" and priority 1
    Then the response status is 201
    And the response body contains "Modal task"

  Scenario: POST to modal create endpoint with missing title returns validation error
    When I POST the modal create form with an empty title
    Then the response status is 422

  Scenario: POST to modal create endpoint with parent_id sets parent
    Given I created a task via API with title "Parent epic" as "modal-parent"
    When I POST the modal create form with title "Child task" under "modal-parent"
    Then the response status is 201
    And the response body contains "Child task"

  Scenario: POST with parent_id auto-tags parent as epic
    Given I created a task via API with title "Future epic" as "future-epic"
    When I POST the modal create form with title "Subtask" under "future-epic"
    Then the response status is 201
    And the parent task "future-epic" has tag "epic"

  # ---------------------------------------------------------------------------
  # Nav bar — New Issue button
  # ---------------------------------------------------------------------------

  Scenario: Task list page nav bar contains a New Issue button
    When I GET "/tasks"
    Then the response status is 200
    And the response body contains "New Issue"

  Scenario: Board page nav bar contains a New Issue button
    When I GET "/board"
    Then the response status is 200
    And the response body contains "New Issue"

  Scenario: Epics page nav bar contains a New Issue button
    When I GET "/epics"
    Then the response status is 200
    And the response body contains "New Issue"

  Scenario: New Issue button links to the modal endpoint
    When I GET "/tasks"
    Then the response status is 200
    And the response body contains "/tasks/new/modal"
