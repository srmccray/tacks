Feature: Board done column filtering by done_since
  As a user viewing the kanban board
  I want to filter old completed tasks out of the Done column
  So that the board stays focused on recent work

  Background:
    Given a tacks database is initialized
    And the web server is running

  # ---------------------------------------------------------------------------
  # Main board — GET /board
  # ---------------------------------------------------------------------------

  Scenario: Default filter hides done tasks older than 3 days
    Given I created a task via API with title "Recent task" as "recent"
    And I created a task via API with title "Old task" as "old"
    And I closed the API task "recent"
    And I closed the API task "old" 4 days ago
    When I GET "/board"
    Then the response status is 200
    And the response body contains "Recent task"
    And the response body does not contain "Old task"

  Scenario: done_since=7d shows recently closed task
    Given I created a task via API with title "Week-old task" as "week-task"
    And I closed the API task "week-task"
    When I GET "/board?done_since=7d"
    Then the response status is 200
    And the response body contains "Week-old task"

  Scenario: done_since=7d hides task closed 8 days ago
    Given I created a task via API with title "Stale task" as "stale"
    And I closed the API task "stale" 8 days ago
    When I GET "/board?done_since=7d"
    Then the response status is 200
    And the response body does not contain "Stale task"

  Scenario: done_since=all shows all done tasks regardless of age
    Given I created a task via API with title "Very old task" as "very-old"
    And I closed the API task "very-old" 60 days ago
    When I GET "/board?done_since=all"
    Then the response status is 200
    And the response body contains "Very old task"

  Scenario: done_since=3d shows task closed 2 days ago
    Given I created a task via API with title "Three-day-window task" as "threeday"
    And I closed the API task "threeday" 2 days ago
    When I GET "/board?done_since=3d"
    Then the response status is 200
    And the response body contains "Three-day-window task"

  Scenario: done_since=3d hides task closed 8 days ago
    Given I created a task via API with title "Older than 3 days" as "old-3"
    And I closed the API task "old-3" 8 days ago
    When I GET "/board?done_since=3d"
    Then the response status is 200
    And the response body does not contain "Older than 3 days"

  Scenario: done_since=30d shows task closed 20 days ago
    Given I created a task via API with title "Month window task" as "monthly"
    And I closed the API task "monthly" 20 days ago
    When I GET "/board?done_since=30d"
    Then the response status is 200
    And the response body contains "Month window task"

  Scenario: Board with no done tasks renders without error
    Given I created a task via API with title "Undone task" as "undone"
    When I GET "/board"
    Then the response status is 200
    And the response body contains "Undone task"

  # ---------------------------------------------------------------------------
  # Epic board — GET /epics/:id?view=board
  # ---------------------------------------------------------------------------

  Scenario: Default filter on epic board shows all done subtasks regardless of age
    Given I created a task via API with title "My epic" as "epic"
    And I created a subtask via API with title "Old subtask" under "epic" as "old-sub"
    And I closed the API task "old-sub" 8 days ago
    When I GET the epic board for "epic"
    Then the response status is 200
    And the response body contains "Old subtask"

  Scenario: done_since=all on epic board shows old done subtask
    Given I created a task via API with title "Full epic" as "full-epic"
    And I created a subtask via API with title "Ancient subtask" under "full-epic" as "ancient-sub"
    And I closed the API task "ancient-sub" 60 days ago
    When I GET the epic board for "full-epic" with done_since "all"
    Then the response status is 200
    And the response body contains "Ancient subtask"

  Scenario: Epic completion stats are unaffected by done_since filter
    Given I created a task via API with title "Stats epic" as "stats-epic"
    And I created a subtask via API with title "New done subtask" under "stats-epic" as "new-sub"
    And I created a subtask via API with title "Old done subtask" under "stats-epic" as "old-sub2"
    And I closed the API task "new-sub"
    And I closed the API task "old-sub2" 30 days ago
    When I GET the API task "stats-epic"
    Then the response status is 200
    And the response JSON field "status" equals "done"
