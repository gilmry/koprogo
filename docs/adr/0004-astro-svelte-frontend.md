# ADR 0004: Astro + Svelte for Frontend

- **Status**: Accepted
- **Date**: 2025-02-01
- **Track**: Software

## Context

KoproGo targets low carbon footprint (< 0.5g CO2/request) which requires minimal JavaScript in the browser. We need a frontend that:
1. Renders fast with minimal JS
2. Provides interactivity where needed (forms, dashboards)
3. Supports multi-language (FR, NL, EN)
4. SEO-friendly for marketing pages
5. Easy deployment (static hosting)

## Decision

We chose **Astro** as the meta-framework with **Svelte** for interactive components.

**Astro** provides:
- **Zero JS by default**: Only ships JavaScript for interactive components
- **Islands Architecture**: Hydrate only what's needed
- **Framework agnostic**: Can mix Svelte, React, Vue in same project
- **Static Site Generation**: Build to static HTML/CSS/JS

**Svelte** for interactivity:
- **No virtual DOM**: Compiles to vanilla JS, smaller bundles
- **Reactive by default**: Simple syntax without hooks/lifecycle boilerplate
- **Excellent DX**: Less code than React/Vue for same functionality

## Consequences

**Positive**:
- ✅ **Minimal JavaScript**: Astro pages ship ~0-5KB JS vs 100KB+ React SPAs
- ✅ **Fast load times**: Static HTML loads instantly, JS hydrates progressively
- ✅ **Low carbon**: Less data transfer = less energy = lower CO2
- ✅ **SEO-friendly**: Server-rendered HTML crawlable by search engines
- ✅ **Developer experience**: Svelte's syntax is clean and intuitive

**Negative**:
- ⚠️ **Smaller ecosystem**: Fewer Svelte component libraries than React
- ⚠️ **Server rendering**: Complex real-time features need separate architecture

**Measured results**:
- Bundle size: ~15KB JS (gzipped) for interactive pages
- Initial load: < 1s on 3G
- Lighthouse scores: 95+ across all metrics

## Alternatives Considered

1. **React + Next.js**:
   - ✅ Largest ecosystem, most developers
   - ❌ Heavy bundles (100KB+ baseline)
   - ❌ Complex hydration (CLS issues)
   - **Verdict**: Rejected due to JS size

2. **Vue + Nuxt**:
   - ✅ Good DX, lighter than React
   - ❌ Still ships more JS than needed
   - **Verdict**: Close second, Astro+Svelte preferred for performance

3. **HTMX + Go templates**:
   - ✅ Minimal JS, server-driven
   - ❌ Limited interactivity without custom JS
   - **Verdict**: Considered for future admin panel, too limiting for MVP

## Next Steps

- ✅ Implement core pages (login, dashboard, buildings) (**Done**)
- ⏳ Add Storybook for component documentation
- ⏳ Optimize images (WebP, lazy loading)
- ⏳ Evaluate Astro DB for user preferences (Phase 2)

## References

- Astro: https://astro.build
- Svelte: https://svelte.dev
- Islands Architecture: https://jasonformat.com/islands-architecture/
