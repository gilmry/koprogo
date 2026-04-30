---
persona: support-agent
created: 2026-04-29
last_updated: 2026-04-29
---

# Mémoire persistante — `support-agent`

## Sources de connaissance principales

Par ordre de priorité de recherche pour répondre aux Q&A :

1. **`Maury/README.md`** + `Maury/Méthode Maury.md` — méthode + positionnement (entrée canonique tout agent vierge).
2. **`Maury/CHANGELOG.md`** — versioning méthode et évolutions Phase H.
3. **`CLAUDE.md`** (210 lignes après trim 2026-04-29) — quick reference projet.
4. **`.claude/AGENT_GUARDRAILS.md`** — détail garde-fous IA (L1+L2+L3+L4 + Tier 1/2).
5. **`.claude/rules/CRITICAL.md`** — top-11 règles non négociables.
6. **`docs/`** — docs domaine projet (PCMN, GDPR, Convocations, Notifications, etc.).
7. **GH Issues fermées** + **GH Discussions historiques** — knowledge accumulé.

## Catégories GH Discussions actives (2026-04-29)

| Catégorie | Slug | Usage |
|---|---|---|
| 🏗️ Architecture & Technique | `-️-architecture-technique` | Débats RFC pré-décision |
| ⚙️ Process | `-️process` | Améliorations méthodo, propositions cross-sprints |
| 📋 Decisions Log | `-decisions-log` | Récap mensuel décisions importantes (cron par documentation-writer) |
| Q&A | `q-a` | Questions agent→humain ou agent→agent |
| Show and tell | `show-and-tell` | Démos d'agents, patterns, ADRs intéressants |
| 🔄 Retrospective Themes | `retrospective-themes` | Thèmes émergents inter-sprints |

## Questions Q&A archivées (à enrichir au fur et à mesure)

| Date | Topic | Source matched | Outcome |
|---|---|---|---|
| (vide — premier sprint) | | | |

## Patterns de questions récurrentes (≥ 3 occurrences)

(vide — à détecter au fil du temps)

## RFCs FAQ proposées par moi

(vide — proposera quand pattern récurrent détecté)

## Conventions de réponse acceptées

- Réponse directe en 1-2 phrases en haut.
- Citation source en quote bloc avec lien `file:line`.
- "À noter" si nuances.
- "Si ta question concerne plutôt X, voir Y" si la question semble mal cadrée.
- Signature : `🤖 support-agent (Claude)`.
- Si pas de réponse trouvée : dire honnêtement + escalader au persona compétent + ouvrir RFC FAQ si récurrent.

## Mapping topic → persona expert

| Topic | Expert à tagger |
|---|---|
| Rust idioms / patterns | `rust-expert` |
| Astro / Svelte 5 / runes / a11y | `astro-svelte-expert` |
| CI/CD / GitHub Actions / GitOps | `devops-engineer` |
| IaC Terraform / Ansible | `platform-engineer` |
| Incident / SLO / observability | `sre-platform` |
| Sécurité (XSS, RBAC, secrets) | `security-officer` |
| Architecture cross-cutting | `code-reviewer` (review) ou `togaf-chief-architect` (design) |
| Méthode Maury, processus | `maury-mary/john/winston/bob` selon phase |
| Métriques / DORA / SLO trends | `csi-analyst` |
| Décision business / roadmap / pricing | `human-supervisor` (gilmry) |

## Lessons learned

- (à enrichir après chaque cycle Q&A)

## Liens

- [`.claude/agents/support-agent.md`](support-agent.md)
- Issues : [#428](https://github.com/gilmry/koprogo/issues/428), [#429](https://github.com/gilmry/koprogo/issues/429)
