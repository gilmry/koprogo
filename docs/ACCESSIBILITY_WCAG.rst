=============================================
WCAG 2.1 Level AA Accessibility (Issue #93)
=============================================

Overview
========

KoproGo implements WCAG 2.1 Level AA accessibility compliance across its
frontend application. This ensures the platform is usable by people with
disabilities and complies with the **European Accessibility Act 2025**.

Belgian Context
===============

- **European Accessibility Act (EAA)**: Directive 2019/882, applicable from June 2025
- **Belgian transposition**: Loi du 13/04/2023 relative aux exigences d'accessibilite
- **Target**: WCAG 2.1 Level AA for all public-facing web interfaces

Implementation Status
=====================

Semantic HTML & Landmarks
-------------------------

+------------------------+------------------------------------------+
| Feature                | Implementation                           |
+========================+==========================================+
| ``lang`` attribute     | ``<html lang="fr">`` (Belgian French)    |
+------------------------+------------------------------------------+
| Skip navigation        | Skip link ``#main-content`` in Layout    |
+------------------------+------------------------------------------+
| Main landmark          | ``<main id="main-content" role="main">`` |
+------------------------+------------------------------------------+
| Footer landmark        | ``<footer role="contentinfo">``          |
+------------------------+------------------------------------------+
| Navigation landmark    | ``<aside role="navigation">``            |
+------------------------+------------------------------------------+
| Screen reader live     | ``#sr-announcer-polite`` and              |
| regions                | ``#sr-announcer-assertive``              |
+------------------------+------------------------------------------+

Keyboard Navigation
-------------------

- **Tab navigation**: All interactive elements reachable via Tab/Shift+Tab
- **Skip link**: "Passer au contenu principal" visible on focus
- **Escape key**: Closes modals and mobile drawer
- **Focus trap**: Modal dialogs trap focus (Tab cycles within dialog)
- **Focus management**: Mobile drawer saves/restores focus on open/close
- **Active page indicator**: ``aria-current="page"`` on active nav links

Forms
-----

- **Labels**: All inputs have explicit ``<label for="...">`` associations
- **Required fields**: Visual ``*`` indicator + screen reader ``(obligatoire)`` text
- **Error messages**: ``role="alert"`` and ``aria-invalid="true"`` on invalid inputs
- **Error association**: ``aria-describedby`` links inputs to error/hint messages
- **Autocomplete**: Login form uses ``autocomplete="email"`` and ``autocomplete="current-password"``

Color Contrast
--------------

- **Text contrast**: 4.5:1 minimum ratio (normal text on white/gray backgrounds)
- **Large text contrast**: 3:1 minimum ratio (headings, large buttons)
- **Focus indicators**: ``focus:ring-2 focus:ring-offset-2`` provides visible ring
- **Error states**: Red borders + text meet contrast requirements

Components
----------

**Accessible Button** (``AccessibleButton.svelte``):

- ``aria-label``, ``aria-pressed``, ``aria-expanded``, ``aria-busy``
- Loading state with ``sr-only`` "Chargement..." text
- Keyboard event forwarding (click, keydown, keyup, focus, blur)

**Accessible Modal** (``AccessibleModal.svelte``):

- ``role="dialog"`` and ``aria-modal="true"``
- ``aria-labelledby`` linked to modal title
- Focus trap via ``trapFocus()`` utility
- Focus save/restore via ``FocusManager``
- Screen reader announcement on open/close
- Escape key to dismiss

**Form Components** (``FormInput.svelte``, ``FormSelect.svelte``):

- Explicit ``<label for="id">`` association
- ``aria-invalid="true"`` when validation error present
- ``aria-describedby`` linking to error or hint paragraph
- ``role="alert"`` on error messages for live announcement

Accessibility Utilities
=======================

The ``src/lib/accessibility.ts`` module provides:

- ``trapFocus(container)``: Traps Tab/Shift+Tab within a container element
- ``getFocusableElements(container)``: Returns all focusable elements
- ``announce(message, priority)``: Screen reader announcements via ARIA live regions
- ``meetsContrastRatio(fg, bg, isLargeText)``: WCAG AA contrast validation
- ``getContrastRatio(color1, color2)``: Calculate contrast ratio
- ``handleListKeyboard(event, items, index, onSelect)``: Arrow/Home/End list navigation
- ``generateId(prefix)``: Unique ID generation for ARIA associations
- ``FocusManager``: Save and restore focus for dialogs

Automated Testing
=================

**axe-core integration** via ``@axe-core/playwright``::

    cd frontend
    npx playwright test Accessibility.spec.ts

Tests cover:

- WCAG 2.1 AA rule violations (comprehensive axe-core scan)
- Heading hierarchy verification
- Form label associations
- Keyboard navigation flow
- Skip navigation functionality
- Language attribute validation
- ARIA landmark structure
- Color contrast compliance
- Focus indicator visibility

Developer Guidelines
====================

When adding new UI components:

1. **Use semantic HTML**: Prefer ``<button>`` over ``<div onclick>``, ``<nav>`` over ``<div>``
2. **Add labels**: Every interactive element needs visible text or ``aria-label``
3. **Handle keyboard**: All click handlers should also work with Enter/Space
4. **Manage focus**: Dialogs must trap focus; restore focus when dismissed
5. **Announce changes**: Use ``announce()`` for dynamic content updates
6. **Test contrast**: Use ``meetsContrastRatio()`` for custom colors
7. **Use existing components**: Prefer ``AccessibleButton``, ``AccessibleModal``, ``FormInput``
8. **Add ``aria-current="page"``** on active navigation links
9. **Add ``role="alert"``** on error messages that appear dynamically
10. **Mark decorative elements** with ``aria-hidden="true"`` (icons, separators)

File Locations
==============

- Layout: ``frontend/src/layouts/Layout.astro``
- Accessibility library: ``frontend/src/lib/accessibility.ts``
- Accessible Button: ``frontend/src/components/ui/AccessibleButton.svelte``
- Accessible Modal: ``frontend/src/components/ui/AccessibleModal.svelte``
- Form Input: ``frontend/src/components/ui/FormInput.svelte``
- Form Select: ``frontend/src/components/ui/FormSelect.svelte``
- Navigation: ``frontend/src/components/Navigation.svelte``
- E2E Tests: ``frontend/tests/e2e/Accessibility.spec.ts``
