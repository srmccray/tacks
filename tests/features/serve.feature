Feature: Web server
  The tk serve command starts a local web server for the browser UI.

  Background:
    Given a tacks database is initialized

  Scenario: Server responds to index page
    Given the web server is running
    When I GET "/"
    Then the response status is 200
    And the response body contains "Tacks"

  Scenario: Server serves static assets
    Given the web server is running
    When I GET "/static/htmx.min.js"
    Then the response status is 200
    When I GET "/static/pico.min.css"
    Then the response status is 200

  Scenario: Server returns 404 for unknown routes
    Given the web server is running
    When I GET "/nonexistent"
    Then the response status is 404

  Scenario: Server responds with HTML content type
    Given the web server is running
    When I GET "/"
    Then the response status is 200
    And the response body contains "<html"
    And the response body contains "</html>"
