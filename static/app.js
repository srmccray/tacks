// Tacks keyboard shortcuts
// Global shortcuts fire only when focus is not inside an input, textarea, or select.

(function () {
  'use strict';

  // --- Help overlay ---

  function createOverlay() {
    var dlg = document.getElementById('help-overlay');
    if (dlg) return dlg;

    dlg = document.createElement('dialog');
    dlg.id = 'help-overlay';
    dlg.innerHTML = [
      '<article>',
      '  <header>',
      '    <button aria-label="Close" rel="prev" id="help-close"></button>',
      '    <h3>Keyboard Shortcuts</h3>',
      '  </header>',
      '  <table>',
      '    <tbody>',
      '      <tr><td><kbd>n</kbd></td><td>New task</td></tr>',
      '      <tr><td><kbd>/</kbd></td><td>Focus tag filter</td></tr>',
      '      <tr><td><kbd>?</kbd></td><td>Show / hide this help</td></tr>',
      '      <tr><td><kbd>j</kbd> / <kbd>&darr;</kbd></td><td>Next row (task list)</td></tr>',
      '      <tr><td><kbd>k</kbd> / <kbd>&uarr;</kbd></td><td>Previous row (task list)</td></tr>',
      '      <tr><td><kbd>Enter</kbd></td><td>Open focused task</td></tr>',
      '      <tr><td><kbd>Esc</kbd></td><td>Close this overlay / blur focus</td></tr>',
      '    </tbody>',
      '  </table>',
      '</article>',
    ].join('\n');
    document.body.appendChild(dlg);

    document.getElementById('help-close').addEventListener('click', function () {
      dlg.close();
    });

    return dlg;
  }

  function toggleHelp() {
    var dlg = createOverlay();
    if (dlg.open) {
      dlg.close();
    } else {
      dlg.showModal();
    }
  }

  // --- Utilities ---

  function isTypingTarget(el) {
    var tag = el.tagName;
    return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || el.isContentEditable;
  }

  function currentPath() {
    return window.location.pathname;
  }

  // --- Task list navigation ---

  function getListRows() {
    var tbody = document.querySelector('table tbody');
    if (!tbody) return [];
    return Array.from(tbody.querySelectorAll('tr[data-href]'));
  }

  function getFocusedRowIndex(rows) {
    var active = document.activeElement;
    return rows.indexOf(active);
  }

  function focusRow(rows, index) {
    if (rows.length === 0) return;
    var clamped = Math.max(0, Math.min(index, rows.length - 1));
    // Remove tabindex from all, set on target
    rows.forEach(function (r) { r.setAttribute('tabindex', '-1'); });
    rows[clamped].setAttribute('tabindex', '0');
    rows[clamped].focus();
  }

  function openFocusedRow(rows) {
    var idx = getFocusedRowIndex(rows);
    if (idx === -1) return;
    var href = rows[idx].getAttribute('data-href');
    if (href) window.location.href = href;
  }

  // --- Board navigation ---

  function getBoardCards() {
    return Array.from(document.querySelectorAll('#content-area article a'));
  }

  function getFocusedCardIndex(cards) {
    return cards.indexOf(document.activeElement);
  }

  function focusCard(cards, index) {
    if (cards.length === 0) return;
    var clamped = Math.max(0, Math.min(index, cards.length - 1));
    cards[clamped].focus();
  }

  // --- Global keydown handler ---

  document.addEventListener('keydown', function (e) {
    var key = e.key;

    // Always allow Escape to close overlay or blur
    if (key === 'Escape') {
      var dlg = document.getElementById('help-overlay');
      if (dlg && dlg.open) {
        dlg.close();
        return;
      }
      if (document.activeElement && document.activeElement !== document.body) {
        document.activeElement.blur();
      }
      return;
    }

    // Help overlay: ? fires even in inputs so users can always discover shortcuts
    if (key === '?') {
      toggleHelp();
      return;
    }

    // Remaining shortcuts only fire outside of form fields
    if (isTypingTarget(document.activeElement)) return;
    if (e.metaKey || e.ctrlKey || e.altKey) return;

    var path = currentPath();

    if (key === 'n') {
      window.location.href = '/tasks/new';
      return;
    }

    if (key === '/') {
      e.preventDefault();
      var tagInput = document.querySelector('input[name="tag"]');
      if (tagInput) tagInput.focus();
      return;
    }

    // Task list navigation
    if (path === '/tasks' || path.startsWith('/tasks?')) {
      var rows = getListRows();
      if (key === 'j' || key === 'ArrowDown') {
        e.preventDefault();
        var idx = getFocusedRowIndex(rows);
        focusRow(rows, idx === -1 ? 0 : idx + 1);
        return;
      }
      if (key === 'k' || key === 'ArrowUp') {
        e.preventDefault();
        var idx2 = getFocusedRowIndex(rows);
        focusRow(rows, idx2 === -1 ? 0 : idx2 - 1);
        return;
      }
      if (key === 'Enter') {
        openFocusedRow(rows);
        return;
      }
    }

    // Board navigation
    if (path === '/board') {
      var cards = getBoardCards();
      if (key === 'ArrowDown' || key === 'ArrowRight') {
        e.preventDefault();
        var ci = getFocusedCardIndex(cards);
        focusCard(cards, ci === -1 ? 0 : ci + 1);
        return;
      }
      if (key === 'ArrowUp' || key === 'ArrowLeft') {
        e.preventDefault();
        var ci2 = getFocusedCardIndex(cards);
        focusCard(cards, ci2 === -1 ? 0 : ci2 - 1);
        return;
      }
      if (key === 'Enter') {
        var ci3 = getFocusedCardIndex(cards);
        if (ci3 !== -1) cards[ci3].click();
        return;
      }
    }
  });
})();
