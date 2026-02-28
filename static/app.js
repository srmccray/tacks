// Tacks keyboard shortcuts
// Global shortcuts fire only when focus is not inside an input, textarea, or select.

(function () {
  'use strict';

  // --- Theme toggle ---

  function getEffectiveTheme() {
    var explicit = document.documentElement.getAttribute('data-theme');
    if (explicit) return explicit;
    // Fall back to system preference
    return window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches
      ? 'dark'
      : 'light';
  }

  function updateToggleButton(btn, currentTheme) {
    // Use innerHTML so HTML entities (sun/moon characters) render correctly
    btn.innerHTML = currentTheme === 'dark' ? '&#9728; Light' : '&#9790; Dark';
  }

  function initThemeToggle() {
    var btn = document.getElementById('theme-toggle');
    if (!btn) return;

    updateToggleButton(btn, getEffectiveTheme());

    btn.addEventListener('click', function () {
      var current = getEffectiveTheme();
      var next = current === 'dark' ? 'light' : 'dark';
      document.documentElement.setAttribute('data-theme', next);
      localStorage.setItem('theme', next);
      updateToggleButton(btn, next);
    });
  }

  // Run after DOM is ready (script is deferred)
  document.addEventListener('DOMContentLoaded', initThemeToggle);

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
    if (!href) return;
    var dlg = document.getElementById('task-modal');
    if (dlg) {
      htmx.ajax('GET', href, { target: '#task-modal', swap: 'innerHTML' });
    } else {
      window.location.href = href;
    }
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

  // --- Filter form: strip empty params before submit ---

  // HTMX requests: remove empty-valued params before the request fires
  document.addEventListener('htmx:configRequest', function (e) {
    var params = e.detail.parameters;
    Object.keys(params).forEach(function (key) {
      if (params[key] === '') delete params[key];
    });
  });

  // Plain form submits (no-JS fallback): disable empty inputs
  document.addEventListener('submit', function (e) {
    var form = e.target;
    if (form.tagName !== 'FORM' || form.method !== 'get') return;
    Array.from(form.elements).forEach(function (el) {
      if (el.name && el.value === '') el.disabled = true;
    });
    setTimeout(function () {
      Array.from(form.elements).forEach(function (el) { el.disabled = false; });
    }, 0);
  });

  // --- Task modal ---

  // Open the modal after HTMX swaps content into it
  document.addEventListener('htmx:afterSwap', function (e) {
    if (e.detail.target.id === 'task-modal') {
      var dlg = document.getElementById('task-modal');
      if (dlg && !dlg.open) {
        dlg.showModal();
      }
    }
  });

  // Delegate close-button clicks inside the task modal
  document.addEventListener('click', function (e) {
    if (e.target.closest('#task-modal [aria-label="Close"]')) {
      var dlg = document.getElementById('task-modal');
      if (dlg) dlg.close();
    }
  });

  // Close task modal when user clicks the backdrop
  document.addEventListener('click', function (e) {
    var dlg = document.getElementById('task-modal');
    if (dlg && dlg.open && e.target === dlg) {
      dlg.close();
    }
  });

  // --- Tag autocomplete ---

  // Cached tag list — fetched once, reused across re-inits (survives HTMX swaps)
  var cachedTags = null;

  function fetchTags(cb) {
    if (cachedTags !== null) {
      cb(cachedTags);
      return;
    }
    fetch('/api/tags')
      .then(function (r) { return r.json(); })
      .then(function (tags) {
        cachedTags = tags;
        cb(tags);
      })
      .catch(function () { cb([]); });
  }

  function initTagAutocomplete() {
    var wrapper = document.querySelector('.tag-autocomplete');
    if (!wrapper) return;

    var textInput = wrapper.querySelector('.tag-text-input');
    var hiddenInput = wrapper.querySelector('input[name="tag"]');
    var pill = wrapper.querySelector('.filter-tag-pill');
    var pillText = wrapper.querySelector('.filter-tag-pill-text');
    var pillRemove = wrapper.querySelector('.filter-tag-pill-remove');
    var suggList = wrapper.querySelector('.tag-suggestions');

    if (!textInput || !hiddenInput || !pill || !suggList) return;

    var activeIdx = -1;

    function getSuggestions() {
      return Array.from(suggList.querySelectorAll('li'));
    }

    function closeSuggestions() {
      suggList.setAttribute('hidden', '');
      suggList.innerHTML = '';
      activeIdx = -1;
    }

    function showSuggestions(matches) {
      suggList.innerHTML = '';
      activeIdx = -1;
      if (matches.length === 0) {
        closeSuggestions();
        return;
      }
      matches.forEach(function (tag) {
        var li = document.createElement('li');
        li.textContent = tag;
        li.addEventListener('mousedown', function (e) {
          // mousedown fires before blur; prevent blur from closing the list
          e.preventDefault();
          selectTag(tag);
        });
        suggList.appendChild(li);
      });
      suggList.removeAttribute('hidden');
    }

    function highlightActive(items) {
      items.forEach(function (li, i) {
        if (i === activeIdx) {
          li.classList.add('active');
        } else {
          li.classList.remove('active');
        }
      });
    }

    function selectTag(tag) {
      // Show the pill
      if (pillText) pillText.textContent = tag;
      pill.style.display = '';
      // Hide the text input
      textInput.style.display = 'none';
      textInput.value = '';
      // Set hidden input and fire change to trigger HTMX
      hiddenInput.value = tag;
      hiddenInput.dispatchEvent(new Event('change', { bubbles: true }));
      closeSuggestions();
    }

    function clearTag() {
      pill.style.display = 'none';
      if (pillText) pillText.textContent = '';
      textInput.style.display = '';
      textInput.value = '';
      hiddenInput.value = '';
      hiddenInput.dispatchEvent(new Event('change', { bubbles: true }));
      closeSuggestions();
    }

    // Clicking the wrapper focuses the text input
    wrapper.addEventListener('click', function (e) {
      if (!e.target.closest('.filter-tag-pill')) {
        textInput.focus();
      }
    });

    // Remove pill button
    pillRemove.addEventListener('click', function (e) {
      e.preventDefault();
      clearTag();
      textInput.focus();
    });

    // Text input: filter suggestions on input
    textInput.addEventListener('input', function () {
      var query = textInput.value.trim().toLowerCase();
      if (query.length === 0) {
        closeSuggestions();
        return;
      }
      fetchTags(function (tags) {
        var matches = tags.filter(function (t) {
          return t.toLowerCase().includes(query);
        });
        showSuggestions(matches);
      });
    });

    // Pre-fetch on focus so suggestions are ready
    textInput.addEventListener('focus', function () {
      fetchTags(function () {}); // warm the cache silently
    });

    // Keyboard navigation in the suggestions list
    textInput.addEventListener('keydown', function (e) {
      var items = getSuggestions();
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        if (items.length === 0 && textInput.value.trim()) {
          // Try to show suggestions if not visible
          fetchTags(function (tags) {
            var q = textInput.value.trim().toLowerCase();
            showSuggestions(tags.filter(function (t) { return t.toLowerCase().includes(q); }));
          });
          return;
        }
        activeIdx = Math.min(activeIdx + 1, items.length - 1);
        highlightActive(items);
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        activeIdx = Math.max(activeIdx - 1, -1);
        highlightActive(items);
      } else if (e.key === 'Tab' || e.key === 'Enter') {
        if (items.length > 0) {
          e.preventDefault();
          var chosen = activeIdx >= 0 ? items[activeIdx] : items[0];
          selectTag(chosen.textContent);
        }
      } else if (e.key === 'Escape') {
        closeSuggestions();
        textInput.blur();
      }
    });

    // Close suggestions when clicking outside
    document.addEventListener('click', function (e) {
      if (!wrapper.contains(e.target)) {
        closeSuggestions();
      }
    });
  }

  // Initialize on DOMContentLoaded and after HTMX settles (full content swaps)
  document.addEventListener('DOMContentLoaded', initTagAutocomplete);
  document.addEventListener('htmx:afterSettle', function (e) {
    // Re-init only when the content-area or a parent was swapped (not tbody polling)
    var target = e.detail.target;
    if (
      target &&
      (target.id === 'content-area' ||
        target.id === 'main' ||
        target.querySelector && target.querySelector('.tag-autocomplete'))
    ) {
      initTagAutocomplete();
    }
  });

  // --- Global keydown handler ---

  document.addEventListener('keydown', function (e) {
    var key = e.key;

    // Always allow Escape to close overlay or blur
    if (key === 'Escape') {
      var taskModal = document.getElementById('task-modal');
      if (taskModal && taskModal.open) {
        taskModal.close();
        return;
      }
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
      // Focus the visible tag text input (autocomplete); fall back to hidden input name="tag"
      var tagInput = document.querySelector('.tag-text-input') || document.querySelector('input[name="tag"]');
      if (tagInput) {
        tagInput.style.display = '';
        tagInput.focus();
      }
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
