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
    btn.innerHTML = currentTheme === 'dark' ? '&#9728; Light mode' : '&#9790; Dark mode';
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
      // Close the settings menu after toggling
      var menu = document.getElementById('settings-menu');
      var gear = document.getElementById('settings-gear');
      if (menu) menu.setAttribute('hidden', '');
      if (gear) gear.setAttribute('aria-expanded', 'false');
    });
  }

  // --- Settings gear dropdown ---

  function initSettingsDropdown() {
    var gear = document.getElementById('settings-gear');
    var menu = document.getElementById('settings-menu');
    if (!gear || !menu) return;

    gear.addEventListener('click', function (e) {
      e.stopPropagation();
      var isOpen = !menu.hasAttribute('hidden');
      if (isOpen) {
        menu.setAttribute('hidden', '');
        gear.setAttribute('aria-expanded', 'false');
      } else {
        menu.removeAttribute('hidden');
        gear.setAttribute('aria-expanded', 'true');
      }
    });

    // Close when clicking outside
    document.addEventListener('click', function (e) {
      if (!e.target.closest('#settings-dropdown')) {
        menu.setAttribute('hidden', '');
        gear.setAttribute('aria-expanded', 'false');
      }
    });
  }

  // --- Nav active tab highlight ---

  function initNavActive() {
    var path = window.location.pathname;
    // Normalise trailing slash: /tasks/ -> /tasks
    if (path.length > 1 && path.endsWith('/')) {
      path = path.slice(0, -1);
    }
    var navMap = {
      'nav-issues': '/tasks',
      'nav-board': '/board',
      'nav-epics': '/epics',
    };
    Object.keys(navMap).forEach(function (id) {
      var el = document.getElementById(id);
      if (!el) return;
      var base = navMap[id];
      var isActive = path === base || path.startsWith(base + '/') || path.startsWith(base + '?');
      if (isActive) {
        el.classList.add('nav-active');
      } else {
        el.classList.remove('nav-active');
      }
    });
  }

  // Run after DOM is ready (script is deferred)
  document.addEventListener('DOMContentLoaded', function () {
    initThemeToggle();
    initSettingsDropdown();
    initNavActive();
  });

  // Re-run nav active highlight after HTMX navigates (hx-push-url updates location).
  // htmx:afterSettle is the most reliable: fires after content is swapped AND
  // window.location.pathname is already updated by hx-push-url.
  document.addEventListener('htmx:afterSettle', initNavActive);
  // Keep pushUrl/replaceUrl as belt-and-suspenders for immediate URL feedback
  document.addEventListener('htmx:pushUrl', initNavActive);
  document.addEventListener('htmx:replaceUrl', initNavActive);

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

  // --- Toast notifications ---

  /**
   * Show a transient toast notification.
   *
   * @param {string} message - The text to display.
   * @param {string} [type='error'] - One of 'error', 'success', or 'info'.
   * @param {number} [duration=4000] - Milliseconds before auto-dismiss.
   */
  function showToast(message, type, duration) {
    type = type || 'error';
    duration = duration !== undefined ? duration : 4000;

    var container = document.getElementById('toast-container');
    if (!container) return;

    var toast = document.createElement('div');
    toast.className = 'toast toast-' + type;
    toast.textContent = message;

    container.appendChild(toast);

    // Auto-hide after duration: add the hiding class, then remove from DOM
    var hideTimer = setTimeout(function () {
      toast.classList.add('toast-hiding');
      // Remove after CSS transition completes (400 ms)
      setTimeout(function () {
        if (toast.parentNode) toast.parentNode.removeChild(toast);
      }, 450);
    }, duration);

    // Clicking dismisses immediately
    toast.addEventListener('click', function () {
      clearTimeout(hideTimer);
      toast.classList.add('toast-hiding');
      setTimeout(function () {
        if (toast.parentNode) toast.parentNode.removeChild(toast);
      }, 450);
    });
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

  // Pause HTMX polling swaps while inline editing or dragging is active
  document.addEventListener('htmx:beforeSwap', function (e) {
    // Guard board-columns polling swaps when dragging
    if (e.detail.target && e.detail.target.id === 'board-columns') {
      if (document.querySelector('.board-card.dragging')) {
        e.detail.shouldSwap = false;
        return;
      }
    }
    // Guard content-area polling swaps (epics list, epic detail) when inline editing
    if (e.detail.target && e.detail.target.id === 'content-area') {
      var editing = document.querySelector('[data-editable].editing');
      if (editing) {
        e.detail.shouldSwap = false;
        return;
      }
    }
    // Guard tbody polling swaps (task list) when inline editing
    if (e.detail.target && e.detail.target.tagName === 'TBODY') {
      var editing = document.querySelector('[data-editable].editing');
      if (editing) {
        e.detail.shouldSwap = false;
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

  // --- Tag multi-select dropdown ---

  function initTagMultiSelect() {
    var wrapper = document.getElementById('tag-multiselect');
    if (!wrapper) return;

    var trigger = document.getElementById('tag-multiselect-trigger');
    var dropdown = document.getElementById('tag-multiselect-dropdown');
    var pillsContainer = document.getElementById('tag-multiselect-pills');
    var placeholder = document.getElementById('tag-multiselect-placeholder');
    var hiddenInput = document.getElementById('tag-hidden-input');

    if (!trigger || !dropdown || !hiddenInput) return;

    // Collect currently selected tags from hidden input
    function getSelectedTags() {
      var val = hiddenInput.value;
      if (!val) return [];
      return val.split(',').map(function (t) { return t.trim(); }).filter(Boolean);
    }

    // Update the hidden input and fire change to trigger HTMX
    function setSelectedTags(tags) {
      hiddenInput.value = tags.join(',');
      hiddenInput.dispatchEvent(new Event('change', { bubbles: true }));
    }

    // Rebuild the pills display in the trigger area
    function renderPills(tags) {
      // Remove existing dynamic pills (keep static server-rendered ones cleared first)
      Array.from(pillsContainer.querySelectorAll('.filter-tag-pill')).forEach(function (p) {
        p.remove();
      });
      tags.forEach(function (tag) {
        var pill = document.createElement('span');
        pill.className = 'filter-tag-pill';
        pill.setAttribute('data-tag', tag);
        pill.innerHTML =
          '<span class="filter-tag-pill-text">' + escapeHtml(tag) + '</span>' +
          '<button class="filter-tag-pill-remove" type="button" aria-label="Remove ' + escapeHtml(tag) + ' filter" data-remove-tag="' + escapeHtml(tag) + '">&times;</button>';
        pillsContainer.appendChild(pill);
      });
      // Show/hide placeholder
      if (placeholder) {
        placeholder.style.display = tags.length > 0 ? 'none' : '';
      }
    }

    // Update the checkmarks and selected class in the dropdown options
    function syncDropdownOptions(tags) {
      Array.from(dropdown.querySelectorAll('.tag-dropdown-option')).forEach(function (li) {
        var tag = li.getAttribute('data-tag');
        var isSelected = tags.indexOf(tag) !== -1;
        li.classList.toggle('selected', isSelected);
        li.setAttribute('aria-selected', isSelected ? 'true' : 'false');
        // Update or add checkmark
        var check = li.querySelector('.tag-check');
        if (isSelected) {
          if (!check) {
            check = document.createElement('span');
            check.className = 'tag-check';
            check.textContent = '\u2713';
            li.appendChild(check);
          }
        } else {
          if (check) check.remove();
        }
      });
    }

    function escapeHtml(str) {
      var d = document.createElement('div');
      d.textContent = str;
      return d.innerHTML;
    }

    function openDropdown() {
      dropdown.removeAttribute('hidden');
      trigger.setAttribute('aria-expanded', 'true');
    }

    function closeDropdown() {
      dropdown.setAttribute('hidden', '');
      trigger.setAttribute('aria-expanded', 'false');
    }

    function toggleTag(tag) {
      var tags = getSelectedTags();
      var idx = tags.indexOf(tag);
      if (idx === -1) {
        tags.push(tag);
      } else {
        tags.splice(idx, 1);
      }
      setSelectedTags(tags);
      renderPills(tags);
      syncDropdownOptions(tags);
    }

    // Toggle dropdown open/close on trigger click
    trigger.addEventListener('click', function (e) {
      // Don't open if clicking a remove-pill button
      if (e.target.closest('.filter-tag-pill-remove')) return;
      if (dropdown.hasAttribute('hidden')) {
        openDropdown();
      } else {
        closeDropdown();
      }
    });

    // Keyboard: open dropdown on Enter/Space/ArrowDown while trigger focused
    trigger.addEventListener('keydown', function (e) {
      if (e.key === 'Enter' || e.key === ' ' || e.key === 'ArrowDown') {
        e.preventDefault();
        if (dropdown.hasAttribute('hidden')) {
          openDropdown();
        }
      } else if (e.key === 'Escape') {
        closeDropdown();
        trigger.focus();
      }
    });

    // Click on dropdown option toggles that tag
    dropdown.addEventListener('mousedown', function (e) {
      // mousedown fires before blur on trigger; prevent blur from closing dropdown
      e.preventDefault();
    });

    dropdown.addEventListener('click', function (e) {
      var option = e.target.closest('.tag-dropdown-option');
      if (!option) return;
      var tag = option.getAttribute('data-tag');
      if (tag) toggleTag(tag);
    });

    // Click on remove button inside a pill (event bubbles from pillsContainer)
    wrapper.addEventListener('click', function (e) {
      var btn = e.target.closest('.filter-tag-pill-remove');
      if (!btn) return;
      e.stopPropagation();
      var tag = btn.getAttribute('data-remove-tag');
      if (tag) toggleTag(tag);
    });

    // Close dropdown when clicking outside
    document.addEventListener('click', function (e) {
      if (!wrapper.contains(e.target)) {
        closeDropdown();
      }
    });

    // On init: sync state from hidden input (handles server-rendered selections)
    var initialTags = getSelectedTags();
    renderPills(initialTags);
    syncDropdownOptions(initialTags);
  }

  // Initialize on DOMContentLoaded and after HTMX settles (full content swaps)
  document.addEventListener('DOMContentLoaded', initTagMultiSelect);
  document.addEventListener('htmx:afterSettle', function (e) {
    // Re-init only when the content-area or a parent was swapped (not tbody polling)
    var target = e.detail.target;
    if (
      target &&
      (target.id === 'content-area' ||
        target.id === 'main' ||
        (target.querySelector && target.querySelector('#tag-multiselect')))
    ) {
      initTagMultiSelect();
    }
  });

  // --- Filter multi-select widget ---

  /**
   * Initialize a filter multi-select widget.
   *
   * @param {HTMLElement} container - The `.filter-multiselect` element.
   *
   * The widget reads initial selection from the hidden input value (comma-separated).
   * On toggle, it updates the hidden input and dispatches a `change` event with
   * `bubbles: true` so HTMX's `change from:#<id>` trigger fires.
   */
  function initFilterMultiSelect(container) {
    if (!container || container._msInitialized) return;
    container._msInitialized = true;

    var trigger = container.querySelector('.filter-multiselect-trigger');
    var dropdown = container.querySelector('.filter-multiselect-dropdown');
    var pillsEl = container.querySelector('.filter-multiselect-pills');
    var placeholder = container.querySelector('.filter-multiselect-placeholder');
    var hiddenInput = container.querySelector('input[type="hidden"]');
    var items = Array.from(container.querySelectorAll('.filter-multiselect-dropdown li[role="option"]'));

    if (!trigger || !dropdown || !pillsEl || !placeholder || !hiddenInput) return;

    // --- State ---

    // Parse current hidden input value into a set of selected values
    function getSelected() {
      var val = hiddenInput.value;
      if (!val) return [];
      return val.split(',').map(function (s) { return s.trim(); }).filter(Boolean);
    }

    var selected = getSelected();

    // --- Rendering ---

    function render() {
      // Update aria-selected on each option
      items.forEach(function (li) {
        var v = li.getAttribute('data-value');
        li.setAttribute('aria-selected', selected.indexOf(v) !== -1 ? 'true' : 'false');
      });

      // Rebuild pills
      pillsEl.innerHTML = '';
      selected.forEach(function (value) {
        var item = items.find(function (li) { return li.getAttribute('data-value') === value; });
        if (!item) return;
        var label = item.getAttribute('data-label') || value;
        var badgeClass = item.getAttribute('data-badge-class') || '';

        var pill = document.createElement('span');
        pill.className = 'filter-multiselect-pill badge ' + badgeClass;

        var text = document.createElement('span');
        text.textContent = label;

        var removeBtn = document.createElement('button');
        removeBtn.className = 'filter-multiselect-pill-remove';
        removeBtn.type = 'button';
        removeBtn.setAttribute('aria-label', 'Remove ' + label + ' filter');
        removeBtn.textContent = '\u00d7'; // ×

        removeBtn.addEventListener('click', function (e) {
          e.stopPropagation();
          toggleValue(value);
        });

        pill.appendChild(text);
        pill.appendChild(removeBtn);
        pillsEl.appendChild(pill);
      });

      // Show/hide placeholder
      placeholder.style.display = selected.length === 0 ? '' : 'none';

      // Update hidden input and notify HTMX
      var newVal = selected.join(',');
      if (hiddenInput.value !== newVal) {
        hiddenInput.value = newVal;
        hiddenInput.dispatchEvent(new Event('change', { bubbles: true }));
      }
    }

    // --- Toggle a value in/out of selected ---

    function toggleValue(value) {
      var idx = selected.indexOf(value);
      if (idx === -1) {
        selected.push(value);
      } else {
        selected.splice(idx, 1);
      }
      render();
    }

    // --- Dropdown open/close ---

    function openDropdown() {
      dropdown.removeAttribute('hidden');
      trigger.setAttribute('aria-expanded', 'true');
    }

    function closeDropdown() {
      dropdown.setAttribute('hidden', '');
      trigger.setAttribute('aria-expanded', 'false');
    }

    function isOpen() {
      return !dropdown.hasAttribute('hidden');
    }

    // Trigger click: toggle dropdown
    trigger.addEventListener('click', function (e) {
      e.stopPropagation();
      if (isOpen()) {
        closeDropdown();
      } else {
        openDropdown();
      }
    });

    // Keyboard: Enter/Space on trigger opens dropdown
    trigger.addEventListener('keydown', function (e) {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        if (isOpen()) {
          closeDropdown();
        } else {
          openDropdown();
        }
      } else if (e.key === 'Escape') {
        closeDropdown();
      }
    });

    // Dropdown item clicks: use mousedown + preventDefault to avoid blur-before-click
    dropdown.addEventListener('mousedown', function (e) {
      e.preventDefault(); // prevent trigger blur before click fires
    });

    items.forEach(function (li) {
      li.addEventListener('click', function (e) {
        e.stopPropagation();
        var value = li.getAttribute('data-value');
        if (value) toggleValue(value);
      });
    });

    // Close when clicking outside
    document.addEventListener('click', function (e) {
      if (!container.contains(e.target)) {
        closeDropdown();
      }
    });

    // Initial render (reflects pre-selected values from URL params)
    render();
  }

  function initAllFilterMultiSelects() {
    var containers = document.querySelectorAll('.filter-multiselect');
    containers.forEach(function (c) {
      // Reset init flag on re-init so we recreate state from current DOM/URL
      c._msInitialized = false;
      initFilterMultiSelect(c);
    });
  }

  document.addEventListener('DOMContentLoaded', initAllFilterMultiSelects);
  document.addEventListener('htmx:afterSettle', function (e) {
    var target = e.detail.target;
    if (
      target &&
      (target.id === 'content-area' ||
        target.id === 'main' ||
        (target.querySelector && target.querySelector('.filter-multiselect')))
    ) {
      initAllFilterMultiSelects();
    }
  });

  // --- Inline editing ---

  // Track elements currently being edited to avoid double-saves.
  // WeakMap<HTMLElement, { saved: boolean, original: string }>
  var editingState = new WeakMap();

  // Build a status badge HTML string for re-rendering after save
  function statusBadgeHtml(status) {
    var icons = { open: '○', in_progress: '◐', done: '✓', blocked: '⊘' };
    var labels = { open: 'Open', in_progress: 'In Progress', done: 'Done', blocked: 'Blocked' };
    var icon = icons[status] || '';
    var label = labels[status] || status.replace('_', ' ');
    return '<span class="badge status-' + status + '">' + icon + ' ' + label + '</span>';
  }

  // Build a priority badge HTML string for re-rendering after save
  function priorityBadgeHtml(priority) {
    var icons = { 1: '▲', 2: '▬', 3: '▽', 4: '·' };
    var icon = icons[priority] || '';
    return '<span class="badge priority-' + priority + '">' + icon + ' P' + priority + '</span>';
  }

  // Build tag pill HTML for a single tag
  function tagPillHtml(tag) {
    return '<span class="tag-pill">' + tag + '</span>';
  }

  // Determine what HTML to show in the element after a successful save
  function renderSavedValue(field, value) {
    if (field === 'status') {
      return statusBadgeHtml(value);
    }
    if (field === 'priority') {
      return priorityBadgeHtml(value);
    }
    if (field === 'tags') {
      var tagList = Array.isArray(value) ? value : [value];
      return tagList.map(tagPillHtml).join(' ');
    }
    // For plain text fields: escape HTML entities
    var div = document.createElement('div');
    div.textContent = value;
    return div.innerHTML;
  }

  // Finish editing: restore original content and remove editing class
  function cancelEdit(el) {
    var state = editingState.get(el);
    if (!state) return;
    editingState.delete(el);
    el.classList.remove('editing');
    el.innerHTML = state.original;
  }

  // Show a brief error flash on the element
  function flashError(el) {
    el.classList.add('edit-error');
    setTimeout(function () {
      el.classList.remove('edit-error');
    }, 2000);
  }

  // Commit an edit: PATCH the server and update the DOM on success
  function commitEdit(el, field, rawValue) {
    var state = editingState.get(el);
    if (!state || state.saved) return;
    state.saved = true;

    var taskId = el.getAttribute('data-task-id');
    if (!taskId) {
      cancelEdit(el);
      return;
    }

    // Build the PATCH payload
    var payload = {};
    if (field === 'priority') {
      payload[field] = parseInt(rawValue, 10);
    } else if (field === 'tags') {
      // Split comma-separated string into trimmed array, drop empty strings
      payload[field] = rawValue
        .split(',')
        .map(function (t) { return t.trim(); })
        .filter(function (t) { return t.length > 0; });
    } else {
      payload[field] = rawValue;
    }

    fetch('/api/tasks/' + taskId, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    })
      .then(function (r) {
        if (!r.ok) throw new Error('HTTP ' + r.status);
        return r.json();
      })
      .then(function () {
        // Successful save — render the new value
        editingState.delete(el);
        el.classList.remove('editing');
        el.innerHTML = renderSavedValue(field, payload[field]);
      })
      .catch(function () {
        // Failed — revert to original, flash error, and show toast
        editingState.delete(el);
        el.classList.remove('editing');
        el.innerHTML = state.original;
        flashError(el);
        showToast('Failed to save — change not applied', 'error');
      });
  }

  // Create the appropriate input element for the given field
  function createInput(field, currentText) {
    var input;

    if (field === 'status') {
      input = document.createElement('select');
      input.className = 'inline-edit-select';
      var statusLabels = { open: '○ Open', in_progress: '◐ In Progress', done: '✓ Done', blocked: '⊘ Blocked' };
      ['open', 'in_progress', 'done', 'blocked'].forEach(function (opt) {
        var o = document.createElement('option');
        o.value = opt;
        o.textContent = statusLabels[opt] || opt.replace('_', ' ');
        // Strip leading icon character (non-ASCII) then spaces before matching
        var normalised = currentText.replace(/^[^\w]+/, '').trim().replace(/\s+/g, '_').toLowerCase();
        if (normalised === opt) {
          o.selected = true;
        }
        input.appendChild(o);
      });
    } else if (field === 'priority') {
      input = document.createElement('select');
      input.className = 'inline-edit-select';
      var priorityLabels = { 1: '▲ P1', 2: '▬ P2', 3: '▽ P3', 4: '· P4' };
      [1, 2, 3, 4].forEach(function (p) {
        var o = document.createElement('option');
        o.value = String(p);
        o.textContent = priorityLabels[p] || 'P' + p;
        // currentText might be "▲ P1", "P1", or "1" — strip non-digits
        var numText = currentText.replace(/[^0-9]/g, '');
        if (numText === String(p)) o.selected = true;
        input.appendChild(o);
      });
    } else if (field === 'description') {
      input = document.createElement('textarea');
      input.className = 'inline-edit-textarea';
      input.value = currentText;
      // Auto-size based on content
      input.rows = Math.max(3, (currentText.match(/\n/g) || []).length + 2);
    } else {
      // title, assignee, tags — plain text input
      input = document.createElement('input');
      input.type = 'text';
      input.className = 'inline-edit-input';
      input.value = currentText;
    }

    return input;
  }

  // Extract the "current value" from an editable element's inner text/content.
  // For badge/pill elements we parse the text content; for plain text we use textContent.
  function extractCurrentText(el, field) {
    if (field === 'tags') {
      // Tags are rendered as multiple .tag-pill spans — collect their text
      var pills = el.querySelectorAll('.tag-pill');
      if (pills.length > 0) {
        return Array.from(pills).map(function (p) { return p.textContent.trim(); }).join(', ');
      }
    }
    // For all other fields: use trimmed textContent (works for badges too)
    return el.textContent.trim();
  }

  // Create save (✓) and cancel (✗) action buttons for text-mode edits
  function createActionButtons(el, field, input) {
    var actions = document.createElement('span');
    actions.className = 'inline-edit-actions';

    var saveBtn = document.createElement('button');
    saveBtn.type = 'button';
    saveBtn.className = 'inline-edit-btn inline-edit-btn-save';
    saveBtn.setAttribute('aria-label', 'Save');
    saveBtn.textContent = '\u2713'; // ✓

    var cancelBtn = document.createElement('button');
    cancelBtn.type = 'button';
    cancelBtn.className = 'inline-edit-btn inline-edit-btn-cancel';
    cancelBtn.setAttribute('aria-label', 'Cancel');
    cancelBtn.textContent = '\u2715'; // ✕

    // mousedown: prevent blur from firing before the click completes
    saveBtn.addEventListener('mousedown', function (e) { e.preventDefault(); });
    cancelBtn.addEventListener('mousedown', function (e) { e.preventDefault(); });

    saveBtn.addEventListener('click', function () {
      commitEdit(el, field, input.value);
    });

    cancelBtn.addEventListener('click', function () {
      cancelEdit(el);
    });

    actions.appendChild(saveBtn);
    actions.appendChild(cancelBtn);
    return actions;
  }

  // Begin editing an element
  function beginEdit(el) {
    // Already editing?
    if (editingState.has(el)) return;

    var field = el.getAttribute('data-field');
    if (!field) return;

    var originalHtml = el.innerHTML;
    var currentText = extractCurrentText(el, field);

    editingState.set(el, { saved: false, original: originalHtml });
    el.classList.add('editing');

    var input = createInput(field, currentText);

    // For select elements: insert directly (no wrapper/buttons), save on change
    if (field === 'status' || field === 'priority') {
      el.innerHTML = '';
      el.appendChild(input);
      input.focus();
      input.addEventListener('change', function () {
        commitEdit(el, field, input.value);
      });
      // Escape: cancel edit (preventDefault stops native <dialog> close)
      input.addEventListener('keydown', function (e) {
        if (e.key === 'Escape') {
          e.preventDefault();
          e.stopPropagation();
          cancelEdit(el);
        }
      });
      return;
    }

    // For text inputs and textareas: wrap in flex row with save/cancel buttons
    var wrapper = document.createElement('span');
    wrapper.className = 'inline-edit-wrapper';
    wrapper.appendChild(input);

    // Textareas span the full width — buttons go below rather than inline
    if (field !== 'description') {
      wrapper.appendChild(createActionButtons(el, field, input));
    }

    el.innerHTML = '';
    el.appendChild(wrapper);

    // For description textarea, add a block-level actions row below
    if (field === 'description') {
      var blockActions = document.createElement('div');
      blockActions.className = 'inline-edit-actions';
      blockActions.style.marginTop = '0.35em';

      var saveBtn2 = document.createElement('button');
      saveBtn2.type = 'button';
      saveBtn2.className = 'inline-edit-btn inline-edit-btn-save';
      saveBtn2.setAttribute('aria-label', 'Save');
      saveBtn2.textContent = '\u2713';

      var cancelBtn2 = document.createElement('button');
      cancelBtn2.type = 'button';
      cancelBtn2.className = 'inline-edit-btn inline-edit-btn-cancel';
      cancelBtn2.setAttribute('aria-label', 'Cancel');
      cancelBtn2.textContent = '\u2715';

      saveBtn2.addEventListener('mousedown', function (e) { e.preventDefault(); });
      cancelBtn2.addEventListener('mousedown', function (e) { e.preventDefault(); });
      saveBtn2.addEventListener('click', function () { commitEdit(el, field, input.value); });
      cancelBtn2.addEventListener('click', function () { cancelEdit(el); });

      blockActions.appendChild(saveBtn2);
      blockActions.appendChild(cancelBtn2);
      el.appendChild(blockActions);
    }

    // Focus and select text
    input.focus();
    if (input.select) {
      input.select();
    }

    // For text inputs and textareas: save on Enter (text only), Escape cancels
    input.addEventListener('keydown', function (e) {
      if (e.key === 'Escape') {
        e.preventDefault();
        e.stopPropagation();
        cancelEdit(el);
        return;
      }
      // Enter saves for single-line inputs; Shift+Enter in textarea is a newline
      if (e.key === 'Enter' && field !== 'description') {
        e.preventDefault();
        commitEdit(el, field, input.value);
      }
    });

    // Save on blur (handles click-away), but only if not clicking an action button
    input.addEventListener('blur', function (e) {
      // relatedTarget is the element receiving focus — skip blur-save if it's one of our buttons
      if (e.relatedTarget && e.relatedTarget.closest('.inline-edit-actions')) return;
      var state = editingState.get(el);
      if (state && !state.saved) {
        commitEdit(el, field, input.value);
      }
    });
  }

  // Event delegation: clicks on [data-editable] elements begin editing
  document.addEventListener('click', function (e) {
    var el = e.target.closest('[data-editable]');
    if (!el) return;
    // Don't start a new edit if we clicked inside an already-active input or action button
    if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA' || e.target.tagName === 'SELECT') return;
    if (e.target.closest('.inline-edit-actions')) return;
    beginEdit(el);
  });

  // --- Board drag-and-drop ---

  // Track the card being dragged and its original column for revert on failure.
  var dragState = null;

  // Find the nearest .board-column ancestor of an element (or null).
  function getBoardColumn(el) {
    return el ? el.closest('.board-column') : null;
  }

  // dragstart: capture source info and add .dragging class
  document.addEventListener('dragstart', function (e) {
    var card = e.target.closest('.board-card[data-task-id]');
    if (!card) return;

    var sourceColumn = getBoardColumn(card);
    if (!sourceColumn) return;

    dragState = {
      card: card,
      taskId: card.getAttribute('data-task-id'),
      sourceColumn: sourceColumn,
      sourceStatus: sourceColumn.getAttribute('data-status'),
    };

    card.classList.add('dragging');
    // Store task ID in dataTransfer so it works across iframes/tabs if needed
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('text/plain', dragState.taskId);
  });

  // dragover: allow drop and highlight the target column
  document.addEventListener('dragover', function (e) {
    if (!dragState) return;
    var col = getBoardColumn(e.target);
    if (!col) return;
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    // Remove drag-over from all columns, add to current target
    document.querySelectorAll('.board-column.drag-over').forEach(function (c) {
      if (c !== col) c.classList.remove('drag-over');
    });
    col.classList.add('drag-over');
  });

  // dragleave: remove highlight when leaving a column
  document.addEventListener('dragleave', function (e) {
    if (!dragState) return;
    var col = getBoardColumn(e.target);
    if (!col) return;
    // Only remove if we've actually left the column (relatedTarget is outside it)
    if (!col.contains(e.relatedTarget)) {
      col.classList.remove('drag-over');
    }
  });

  // dragend: always clean up .dragging and any leftover .drag-over classes
  document.addEventListener('dragend', function (e) {
    if (!dragState) return;
    dragState.card.classList.remove('dragging');
    document.querySelectorAll('.board-column.drag-over').forEach(function (c) {
      c.classList.remove('drag-over');
    });
    // dragState is cleared in drop handler or here if drop didn't fire
    dragState = null;
  });

  // drop: move the card and PATCH the API
  document.addEventListener('drop', function (e) {
    if (!dragState) return;
    e.preventDefault();

    var targetColumn = getBoardColumn(e.target);
    if (!targetColumn) {
      // Dropped outside a column — clean up and bail
      dragState.card.classList.remove('dragging');
      document.querySelectorAll('.board-column.drag-over').forEach(function (c) {
        c.classList.remove('drag-over');
      });
      dragState = null;
      return;
    }

    targetColumn.classList.remove('drag-over');

    var targetStatus = targetColumn.getAttribute('data-status');
    var card = dragState.card;
    var taskId = dragState.taskId;
    var sourceColumn = dragState.sourceColumn;
    // Clear dragState before async work to allow new drags
    dragState = null;

    card.classList.remove('dragging');

    // No-op: dropped on the same column
    if (targetStatus === sourceColumn.getAttribute('data-status')) {
      return;
    }

    // Optimistic UI: move card to target column immediately
    targetColumn.appendChild(card);

    // PATCH the API to persist the status change
    fetch('/api/tasks/' + taskId, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ status: targetStatus }),
    })
      .then(function (r) {
        if (!r.ok) throw new Error('HTTP ' + r.status);
        // Success — card is already in the right column
      })
      .catch(function () {
        // Failure — revert card to its original column, flash error, and show toast
        sourceColumn.appendChild(card);
        card.classList.add('drag-error');
        setTimeout(function () {
          card.classList.remove('drag-error');
        }, 700);
        showToast('Failed to move task — status not updated', 'error');
      });
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
      // Open the tag multi-select dropdown; fall back to focusing the trigger
      var trigger = document.getElementById('tag-multiselect-trigger');
      if (trigger) {
        trigger.focus();
        trigger.click();
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
