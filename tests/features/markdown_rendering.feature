Feature: Markdown rendering in web views
  As a developer using the browser UI
  I want task descriptions and comments rendered as HTML
  So that formatted text, code blocks, and headings display correctly

  Background:
    Given a tacks database is initialized
    And the web server is running

  # ---------------------------------------------------------------------------
  # Task detail page — description markdown rendering
  # ---------------------------------------------------------------------------

  Scenario: Task detail page renders markdown heading in description
    Given I created a task via API with title "Heading task" and description "## Heading" as "heading-task"
    When I GET the HTML task "heading-task"
    Then the response status is 200
    And the response body contains "<h2>Heading</h2>"

  Scenario: Task detail page renders inline code in description
    Given I created a task via API with title "Code task" and description "Use `cargo build` to compile." as "code-task"
    When I GET the HTML task "code-task"
    Then the response status is 200
    And the response body contains "<code>cargo build</code>"

  Scenario: Task detail page wraps rendered description in markdown-body class
    Given I created a task via API with title "Styled task" and description "## Styled" as "styled-task"
    When I GET the HTML task "styled-task"
    Then the response status is 200
    And the response body contains "markdown-body"

  Scenario: Task detail page shows fallback dash when description is absent
    Given I created a task via API with title "No-desc task" as "nodesc-task"
    When I GET the HTML task "nodesc-task"
    Then the response status is 200
    And the response body contains "—"

  # ---------------------------------------------------------------------------
  # Task detail modal fragment — HTMX markdown rendering
  # ---------------------------------------------------------------------------

  Scenario: Task modal fragment renders markdown heading in description
    Given I created a task via API with title "Modal heading task" and description "## Modal Heading" as "modal-md-task"
    When I HTMX GET the task "modal-md-task"
    Then the response status is 200
    And the response body contains "<h2>Modal Heading</h2>"

  Scenario: Task modal fragment shows fallback dash when description is absent
    Given I created a task via API with title "Modal no-desc task" as "modal-nodesc-task"
    When I HTMX GET the task "modal-nodesc-task"
    Then the response status is 200
    And the response body contains "—"

  # ---------------------------------------------------------------------------
  # Epic detail page — description markdown rendering
  # ---------------------------------------------------------------------------

  Scenario: Epic detail page renders markdown heading in description
    Given I created a task via API with title "Epic task" and description "## Epic Heading" as "epic-parent"
    And I created a subtask via API with title "Child task" under "epic-parent" as "epic-child"
    When I GET the HTML epic "epic-parent"
    Then the response status is 200
    And the response body contains "<h2>Epic Heading</h2>"

  Scenario: Epic detail page wraps rendered description in markdown-body class
    Given I created a task via API with title "Epic styled" and description "## Styled Epic" as "epic-styled"
    And I created a subtask via API with title "Epic child" under "epic-styled" as "epic-styled-child"
    When I GET the HTML epic "epic-styled"
    Then the response status is 200
    And the response body contains "markdown-body"

  # ---------------------------------------------------------------------------
  # Comments — markdown rendering
  # ---------------------------------------------------------------------------

  Scenario: Task detail page renders markdown in comments
    Given I created a task via API with title "Comment markdown task" as "comment-md-task"
    And I posted a comment "## Comment Heading" on API task "comment-md-task"
    When I GET the HTML task "comment-md-task"
    Then the response status is 200
    And the response body contains "<h2>Comment Heading</h2>"

  Scenario: Task detail page renders bold text in comment
    Given I created a task via API with title "Bold comment task" as "bold-comment-task"
    And I posted a comment "**important**" on API task "bold-comment-task"
    When I GET the HTML task "bold-comment-task"
    Then the response status is 200
    And the response body contains "<strong>important</strong>"
