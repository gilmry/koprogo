# Issue #027 - Accessibilité WCAG 2.1 Niveau AA

**Priorité**: 🟡 HIGH
**Estimation**: 8-10 heures
**Labels**: `enhancement`, `frontend`, `accessibility`, `a11y`, `compliance`

---

## 📋 Description

Implémenter la conformité **WCAG 2.1 niveau AA** pour rendre KoproGo accessible aux personnes en situation de handicap (déficience visuelle, motrice, auditive, cognitive).

**Contexte légal** : Bien que non strictement obligatoire pour logiciels privés en Belgique, l'accessibilité devient un standard de qualité et évite discrimination.

---

## 🎯 Objectifs WCAG 2.1 AA

### 1. Perceptible

- [ ] **Contraste** : Ratio minimum 4.5:1 (texte normal), 3:1 (texte large)
- [ ] **Images** : Alt text pour toutes images informatives
- [ ] **Multimédia** : Sous-titres vidéos (si applicable)
- [ ] **Adaptable** : Responsive jusqu'à 320px largeur

### 2. Utilisable

- [ ] **Clavier** : Navigation complète au clavier (Tab, Enter, Espace, Échap)
- [ ] **Focus visible** : Indicateur focus clair (outline 2px minimum)
- [ ] **Temps** : Pas de limite temps stricte, ou extensible
- [ ] **Épilepsie** : Pas de flash >3 fois/seconde

### 3. Compréhensible

- [ ] **Langue** : `lang="fr"` sur `<html>`, `lang="en"` si contenu anglais
- [ ] **Prévisible** : Navigation cohérente sur toutes pages
- [ ] **Saisie** : Labels explicites, messages d'erreur clairs
- [ ] **Aide** : Tooltips contextuels

### 4. Robuste

- [ ] **HTML valide** : Validation W3C
- [ ] **ARIA** : Attributs ARIA corrects (roles, labels, states)
- [ ] **Lecteurs d'écran** : Tests NVDA (Windows), VoiceOver (Mac)

---

## 📐 Implémentation

### Audit Automatisé

```bash
# Lighthouse CI
npm run lighthouse -- --only-categories=accessibility

# axe-core
npm install -D @axe-core/cli
npx axe http://localhost:3000 --tags wcag2a,wcag2aa
```

### Composants Accessibles

**Fichier** : `frontend/src/components/ui/AccessibleButton.svelte`

```svelte
<script>
  export let label;
  export let ariaLabel = label;
  export let disabled = false;
</script>

<button
  class="btn"
  aria-label={ariaLabel}
  {disabled}
  on:click
>
  {label}
</button>

<style>
  .btn:focus {
    outline: 2px solid #4A90E2;
    outline-offset: 2px;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
```

### Mode Contraste Élevé

```css
/* frontend/src/styles/high-contrast.css */
@media (prefers-contrast: high) {
  :root {
    --color-bg: #000;
    --color-text: #FFF;
    --color-primary: #FFD700;
  }
}
```

### Navigation Clavier

```svelte
<!-- Skip to main content link -->
<a href="#main-content" class="skip-link">
  Aller au contenu principal
</a>

<main id="main-content">
  <!-- Content -->
</main>

<style>
  .skip-link {
    position: absolute;
    top: -40px;
    left: 0;
    background: #000;
    color: #fff;
    padding: 8px;
    z-index: 100;
  }

  .skip-link:focus {
    top: 0;
  }
</style>
```

---

## ✅ Checklist Conformité

### Niveau A (Minimum)

- [ ] 1.1.1 : Contenu non textuel (alt)
- [ ] 1.3.1 : Info et relations (structure sémantique)
- [ ] 2.1.1 : Clavier (toutes fonctions accessibles)
- [ ] 2.4.1 : Contourner blocs (skip links)
- [ ] 3.1.1 : Langue de la page
- [ ] 4.1.1 : Analyse syntaxique (HTML valide)
- [ ] 4.1.2 : Nom, rôle, valeur (ARIA)

### Niveau AA (Cible)

- [ ] 1.4.3 : Contraste minimum (4.5:1)
- [ ] 1.4.5 : Texte sous forme d'image (éviter)
- [ ] 2.4.5 : Accès multiples (plusieurs chemins navigation)
- [ ] 2.4.6 : En-têtes et étiquettes (descriptifs)
- [ ] 2.4.7 : Visibilité du focus
- [ ] 3.2.3 : Navigation cohérente
- [ ] 3.2.4 : Identification cohérente
- [ ] 3.3.3 : Suggestion d'erreur
- [ ] 3.3.4 : Prévention erreurs (confirmation actions importantes)

---

## 🧪 Tests Accessibilité

### Tests Automatisés

```javascript
// frontend/tests/a11y/accessibility.spec.ts
import { test, expect } from '@playwright/test';
import { injectAxe, checkA11y } from 'axe-playwright';

test.describe('Accessibility', () => {
  test('homepage should not have any automatically detectable accessibility issues', async ({ page }) => {
    await page.goto('http://localhost:3000');
    await injectAxe(page);
    await checkA11y(page, null, {
      detailedReport: true,
      detailedReportOptions: { html: true }
    });
  });

  test('buildings page should be keyboard navigable', async ({ page }) => {
    await page.goto('http://localhost:3000/buildings');
    await page.keyboard.press('Tab');
    const focused = await page.evaluate(() => document.activeElement?.tagName);
    expect(focused).not.toBe('BODY'); // Focus doit se déplacer
  });
});
```

### Tests Manuels

- [ ] Navigation complète au clavier (sans souris)
- [ ] Test lecteur d'écran NVDA (Windows)
- [ ] Test VoiceOver (Mac)
- [ ] Zoom 200% sans perte fonctionnalité
- [ ] Mode contraste élevé Windows/Mac

---

## 📊 Outils de Validation

- **Lighthouse** : Score >90 (accessibilité)
- **axe DevTools** : 0 violations critiques
- **WAVE** : 0 erreurs, warnings acceptables documentés
- **Color Contrast Analyzer** : Tous contrastes validés

---

## 🔗 Ressources

- WCAG 2.1 Guidelines : https://www.w3.org/WAI/WCAG21/quickref/
- axe-core : https://www.deque.com/axe/
- ARIA Authoring Practices : https://www.w3.org/WAI/ARIA/apg/

---

## 🚀 Checklist

- [ ] 1. Audit initial Lighthouse + axe
- [ ] 2. Corriger violations critiques
- [ ] 3. Ajouter skip links
- [ ] 4. Valider contrastes couleurs
- [ ] 5. Ajouter ARIA labels
- [ ] 6. Tests clavier complets
- [ ] 7. Tests lecteurs d'écran
- [ ] 8. Mode contraste élevé
- [ ] 9. Documentation accessibilité
- [ ] 10. Commit : `feat: implement WCAG 2.1 AA accessibility compliance`

---

**Créé le** : 2025-11-01
**Milestone** : v1.2 - Quality & Compliance
**Impact** : HIGH - Inclusion, qualité, compliance
