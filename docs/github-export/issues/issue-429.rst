========================================================================================================================
Issue #429: Méta — Operations runtime: deploy IaC en prod + agents DevOps/SRE/Support/CSI (Tier 1 humain / Tier 2 logué)
========================================================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: documentation,track:infrastructure priority:critical,security governance
:Assignees: Unassigned
:Created: 2026-04-29
:Updated: 2026-04-29
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/429>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte — la dimension runtime manquante
   
   Les issues #425 (garde-fous build-time), #426 (cleanup docs), #427 (validation discipline pre-release), #428 (simulation organisation produit) couvrent **build → release**. Mais elles ne couvrent pas **runtime** :
   - Comment déployer l'IaC en production de manière sûre ?
   - Comment un agent IA participe à la supervision de la plateforme et à l'incident response sans casser la prod ?
   
   Cette issue acte la **stratégie ops runtime** : agents DevOps/SRE en **lecture seule**, validation humaine **obligatoire** (message OU workflow_dispatch) pour toute mutation.
   
   **Important : v0.1.0 n'est pas en production**, aucun système live. Cette issue prépare les recettes **avant** la première mise en ligne, pas une remédiation incidente. L'urgence est stratégique.
   
   ---
   
   ## 1. Principe directeur — modèle deux-tiers d'autorisation
   
   > **Agent IA = diagnostic + proposal. Humain = approval + execution. Mutation jamais autonome.**
   
   Toute activité d'agent en runtime tombe dans **un de deux tiers**, jamais dans la zone grise :
   
   ### Tier 1 — Dangereux : validation humaine obligatoire
   - Toute mutation prod : `terraform apply`, `helm upgrade`, `kubectl mutate`, `argocd sync`, `velero restore`.
   - Création de documentation publique-facing.
   - Envoi d'emails, messages Slack/Telegram à des personnes externes.
   - Fermeture d'issues, suppression de branches, opérations git destructives.
   - Bumps de versions Cargo/npm critiques.
   - Approbation : message reply OR workflow_dispatch GH OR environment GH approval (cf. §5).
   
   ### Tier 2 — Autorisé non-supervisé : tracé dans un rapport d'activité
   - Lecture de logs / métriques / configurations.
   - Recherche dans la documentation interne et retour d'extraits.
   - Diagnostic + plan proposal (publié en GH issue, pas exécuté).
   - Commentaires d'agents sur issues/PRs (avec attribution explicite `🤖 [persona]`).
   - Mise à jour de status de tâches internes (label issue, milestone tracking).
   - Génération de rapports automatisés (velocity, token budget, ADR digest, WBS).
   
   Tout Tier 2 est **logué dans un rapport d'activité** (`docs/agent-activity/YYYY-MM-DD-<persona>.md`) — généré quotidien par `documentation-writer`, contenant : actions taken, rationale, outcome, lien GH issue/PR concerné. Le superviseur humain peut auditer rétrospectivement.
   
   ### Tableau capacités
   
   | Capacité | Tier | Agent IA | Humain |
   |---|---|---|---|
   | Lire logs / métriques / kubectl get | T2 | ✓ logué | ✓ |
   | Diagnostiquer cause racine | T2 | ✓ logué | ✓ |
   | Proposer plan d'action (PR, commande, plan) | T2 | ✓ logué | ✓ |
   | Répondre à question Q&A depuis docs existantes | T2 | ✓ logué | ✓ |
   | Créer nouvelle documentation | T1 | ✗ propose only | ✓ |
   | Envoyer email / message externe | T1 | ✗ | ✓ |
   | Approuver une action prod | T1 | ✗ | ✓ |
   | Exécuter `terraform apply` / `helm upgrade` / `kubectl mutate` / `argocd sync` | T1 | ✗ (deny #425) | ✓ |
   | Rédiger postmortem (draft) | T2 | ✓ logué | révision |
   | Signer postmortem | T1 | ✗ | ✓ |
   
   **Conséquence** : les credentials que reçoit l'agent en runtime sont **strictement read-only**. ServiceAccount k8s avec RBAC `get/list/watch` uniquement. Token Prometheus read. Token Loki read. ArgoCD `application:get` only.
   
   ---
   
   ## 2. Architecture en 7 couches
   
   ```
   ┌────────────────────────────────────────────────────────────────────────┐
   │  L1 — OBSERVATION (read-only credentials)                              │
   │  Prometheus query API • Loki query • kubectl get/describe/logs •      │
   │  argocd app get • terraform show • Sentry • Grafana                    │
   └────────────────────────────────┬───────────────────────────────────────┘
                                    │
                                    ▼
   ┌────────────────────────────────────────────────────────────────────────┐
   │  L2 — TRIGGERING (3 modes au choix)                                   │
   │  ① Alertmanager webhook → endpoint receiver → repository_dispatch GH  │
   │  ② Loki recording rule (anomaly) → cron worker → repository_dispatch  │
   │  ③ workflow_dispatch manuel par humain (gh workflow run incident.yml) │
   │  ④ Validation par message (Slack/Telegram bot reply → dispatch)        │
   └────────────────────────────────┬───────────────────────────────────────┘
                                    │
                                    ▼
   ┌────────────────────────────────────────────────────────────────────────┐
   │  L3 — HUMAN GATE (avant que l'agent agisse)                            │
   │  Notification au superviseur (Slack DM, email, mention GH issue)      │
   │  Humain décide : investigate / ignore / escalate                       │
   │  Si investigate → trigger workflow Claude (modes ③/④ ci-dessus)       │
   └────────────────────────────────┬───────────────────────────────────────┘
                                    │
                                    ▼
   ┌────────────────────────────────────────────────────────────────────────┐
   │  L4 — CLAUDE DIAGNOSTIC (agents `sre-platform` / `devops-engineer`)   │
   │  Workflow GH lance Claude Code (remote agent) ou Claude API           │
   │  Inputs : alert payload + recent logs window + recent metrics +       │
   │           recent argocd sync history + linked issues + runbook         │
   │  Outputs : root cause hypotheses + severity + blast radius            │
   └────────────────────────────────┬───────────────────────────────────────┘
                                    │
                                    ▼
   ┌────────────────────────────────────────────────────────────────────────┐
   │  L5 — ACTION PROPOSAL (NEVER execute, only propose)                    │
   │  Pour code  : agent ouvre une PR avec le fix proposé                   │
   │  Pour IaC   : agent génère terraform plan / helm template diff         │
   │  Pour cluster : agent draft kubectl/argocd command (avec risques)     │
   │  Tout est posté en GH issue/PR + tag du superviseur humain             │
   └────────────────────────────────┬───────────────────────────────────────┘
                                    │
                                    ▼
   ┌────────────────────────────────────────────────────────────────────────┐
   │  L6 — HUMAN APPROVAL + EXECUTION                                       │
   │  Humain review le plan agent                                           │
   │  Si OK :                                                               │
   │    code → merge PR (CI deploy via GitOps existant)                     │
   │    IaC  → workflow_dispatch `terraform-apply-production` avec approver │
   │    cluster → humain exécute commande OU workflow audité avec approver │
   │  Tout laisse trace : PR, workflow run log, kubectl audit log           │
   └────────────────────────────────┬───────────────────────────────────────┘
                                    │
                                    ▼
   ┌────────────────────────────────────────────────────────────────────────┐
   │  L7 — POSTMORTEM                                                       │
   │  Agent rédige draft `docs/postmortem/YYYY-MM-DD-<incident>.md`        │
   │  Sections : timeline, impact, what happened, action items, lessons     │
   │  Humain review + signe + archive                                      │
   │  Lessons → potentielle RFC évolution Maury (Phase H, cf. #428 §3bis)   │
   └────────────────────────────────────────────────────────────────────────┘
   ```
   
   ---
   
   ## 3. Personas DevOps/SRE/Platform/Support (additions à #428 §6)
   
   5 personas explicites dans le cluster "cross-cutting" (tous Tier 2 en lecture, Tier 1 en mutation) :
   
   | Persona | Rôle | Activité cadencée | Modes de déclenchement |
   |---|---|---|---|
   | `devops-engineer` | CI/CD pipelines, GitOps configs (ArgoCD applications), release workflows GH Actions, image policies (no `:latest`, digest pinning). Crée les PRs Helm/Kustomize values. | weekly : revue ArgoCD applications drift. par release : auto-doc refresh PR (#428 §8bis). | dispatch, routine cron |
   | `sre-platform` | On-call rotation simulée, incident response, observability owner. Reçoit alertmanager webhooks via repository_dispatch. Diagnostic + plan proposal. Tient SLO/SLI + CSI report. | daily : check alertmanager state, activity report. weekly : SLO review. monthly : CSI report. par incident : `incident-investigate`. | webhook alertmanager, dispatch, routine |
   | `platform-engineer` | IaC (terraform modules, ansible roles), bumps providers/versions, gère le tfstate (S3 + lock + KMS). Crée PRs IaC avec `terraform plan` en commentaire. | monthly : audit drift IaC. par PI : roadmap IaC (capacités, refactor modules). | dispatch, routine, claude code |
   | **`support-agent`** | **Q&A, retrieval doc, création doc si manquant.** Répond aux questions agents et humains. Search dans CLAUDE.md, docs/, Maury/, ADRs/RFCs, postmortems, GH Discussions. Si manquant → propose nouveau doc (T1 = humain valide). Détecte questions récurrentes → propose RFC FAQ. | continu : Q&A en GH Discussions Q&A category. weekly : digest questions/réponses. monthly : metrics Q&A coverage. | cowork (chat direct), dispatch (webhook chat platform), claude code (interactive), routine (digest) |
   | **`csi-analyst`** | **ITIL Continual Service Improvement.** Aggrège métriques (SLO, MTTR, MTBF, error budget burn, satisfaction support), détecte tendances, propose initiatives d'amélioration. | monthly : `csi-report-YYYY-MM.md`. quarterly : présenté en Portfolio Sync (#428). | routine cron monthly, dispatch sur alerte SLO breach |
   
   Ces personas participent aux cérémonies #428 §7 (daily standup, sprint reviews, retrospectives, ART sync). Leur voix porte sur la **fiabilité plateforme** + **support utilisateur** + **amélioration continue** — ils contre-balancent l'agent `dev` qui pousse pour la vélocité features.
   
   Memory files à créer : `.claude/agents/{devops-engineer,sre-platform,platform-engineer,support-agent,csi-analyst}.memory.md`.
   
   ### Modes de déclenchement par persona (récap)
   
   | Mode | Quand | Cas d'usage |
   |---|---|---|
   | **cowork (chat direct)** | Humain pose une question dans Claude.ai cowork | `support-agent` répond en live |
   | **dispatch (workflow_dispatch GH)** | Humain trigger workflow manuellement | Tous personas, action ponctuelle |
   | **dispatch (webhook external)** | Plateforme externe envoie repository_dispatch | `sre-platform` sur alertmanager, `support-agent` sur question Slack |
   | **routine (cron schedule)** | Cadence définie | Reports périodiques (CSI, activity, velocity, etc.) |
   | **claude code (interactive)** | Humain en session Claude Code IDE | Tous personas, mode dev |
   
   ---
   
   ## 4. Déploiement IaC en production — workflow concret
   
   ### État actuel à corriger (cf. audit #425)
   - ArgoCD `autoSync: true` + `prune: true` même en prod → un push Git = déploiement immédiat sans validation humaine.
   - Terraform state sans lock ni encrypt.
   - Aucun gate humain entre PR mergé et apply prod.
   
   ### État cible
   
   **1. Branche `release/vX.Y.Z`** ouverte → CI :
   - `terraform fmt -check` + `terraform validate`
   - `tfsec` + `checkov` (security scan)
   - `terraform plan` → output posté en commentaire de PR
   - `helm template` + `kube-linter` sur charts modifiés
   - `dependency-review-action` sur PR
   
   **2. PR review** :
   - Reviewer humain examine le `terraform plan` posté en commentaire.
   - Si delta inattendu (drift) → bloque merge.
   - Approbation = au moins 2 reviewers (CODEOWNERS sur `infrastructure/`).
   
   **3. Merge** → **ne déclenche RIEN automatiquement en prod**.
   
   **4. Apply prod** = workflow_dispatch manuel `terraform-apply-production.yml` :
   - Inputs : `pr_number` ou `commit_sha`
   - GH Environment "production" avec `required_reviewers: 2` + `wait_timer: 5min` (cool-off).
   - Workflow lance `terraform apply -auto-approve` UNIQUEMENT après que les reviewers ont approuvé l'environment dans la GH UI.
   - Plan output réaffiché → humain confirme → apply effectif.
   - Logs complets, audit trail dans GH Actions run.
   
   **5. ArgoCD `syncPolicy`** :
   - `automated.prune: false` en production (manual sync only).
   - `automated.selfHeal: false` en production.
   - Sync prod = via UI ArgoCD avec MFA OU workflow `argocd-sync-production.yml` avec approver.
   - Sync dev/staging peut rester `automated: true` (tolérance pour itération).
   
   **6. Velero / backups** :
   - Backup `cron` actif en prod, mais `velero restore` jamais autonome.
   - Restore = workflow_dispatch `velero-restore.yml` avec approver + checklist runbook.
   
   ---
   
   ## 5. Runtime monitoring + reactive agent — workflow concret
   
   ### Stack existante (déjà en place)
   - **Prometheus** + **Grafana** + **Loki** + **Alertmanager** (cf. CLAUDE.md "Monitoring Endpoints").
   - **Suricata** IDS + **CrowdSec** WAF + **fail2ban**.
   - **AIDE** file integrity, **rkhunter**, **Lynis**.
   
   ### Pipeline réactif
   ```
   Alertmanager fire alert (severity=critical, namespace=koprogo-prod)
      ↓ webhook receiver (POST https://api.github.com/repos/.../dispatches)
      ↓ repository_dispatch type "alertmanager"
      ↓
   GH Action `incident-investigate.yml` triggered
      ↓ pulls alert payload from event_payload
      ↓ pulls last 30 min logs from Loki (read-only token)
      ↓ pulls last 30 min metrics from Prometheus
      ↓ pulls recent argocd sync history
      ↓ pulls linked issues / runbook for this alert name
      ↓ invokes Claude (Opus 4.7) with all context
      ↓
   Claude agent `sre-platform`:
      ↓ analyses, hypothèses, severity assessment
      ↓ rédige diagnostic Markdown
      ↓ propose plan d'action(s) avec risques
      ↓ ouvre GH issue `incident-YYYY-MM-DD-<alertname>`
      ↓ tag @gilmry (superviseur humain)
      ↓ poste lien Slack/Telegram
      ↓
   Humain reçoit notif → ouvre l'issue → review diagnostic
      ↓ approuve plan OU corrige OU ignore
      ↓ Si approuvé → exécute (PR merge, workflow_dispatch, kubectl direct avec MFA)
      ↓
   Post-incident :
      ↓ agent draft postmortem dans la même issue
      ↓ humain review + signe
      ↓ archive `docs/postmortem/YYYY-MM-DD-<alertname>.md`
      ↓ lessons → RFC si pattern récurrent (Maury Phase H)
   ```
   
   ### Modes de validation humaine (4 modes au choix)
   
   | Mode | Comment | Cas d'usage |
   |---|---|---|
   | **Workflow_dispatch GH UI** | Humain ouvre Actions → run workflow → approuve l'environment | Action planifiée, pas urgente |
   | **Workflow_dispatch CLI** | `gh workflow run incident-investigate.yml -f alert_id=...` | Humain en terminal |
   | **Validation par message Slack/Telegram** | Bot relaie alerte → humain répond `@bot investigate` → bot trigger dispatch | Mobilité, urgence |
   | **Approval GH Environment** | Workflow déjà déclenché attend approval dans la GH UI ; humain clique "Approve" | Cool-off period intégrée |
   
   Tous laissent trace audit dans GH (workflow run + issue + qui a approuvé quoi quand).
   
   ---
   
   ## 6. SLO/SLI à définir (par `sre-platform`)
   
   Avant la mise en prod, définir :
   - **Availability SLO** : 99.5 % (initial conservative).
   - **Latency SLO** : P99 < 500ms pour les endpoints critiques (CLAUDE.md mentionne P99 < 5ms cible — irréaliste, à corriger).
   - **Error rate SLO** : < 0.5 % 5xx sur 1h.
   - **Backup RPO/RTO** : RPO ≤ 24h, RTO ≤ 4h (à valider via test restore).
   
   Alertes alertmanager basées sur ces SLO. SRE-platform tient le SLO board (Grafana).
   
   ---
   
   ## 6bis. ITIL CSI — boucle d'amélioration continue (`csi-analyst`)
   
   Les métriques collectées (SLO breach, MTTR, MTBF, error budget burn, support satisfaction, deploy frequency, change failure rate) alimentent une **boucle ITIL CSI** (Continual Service Improvement) — Phase 5 du cycle ITIL.
   
   ### Pipeline CSI (mensuel + trimestriel)
   
   ```
   Sources de données :
     ├── Prometheus / Grafana       → SLO trend, latency P99, error rate
     ├── Alertmanager history       → MTTR (mean time to resolve), MTBF
     ├── GH issues `incident-*`     → incidents count, root causes distribution
     ├── GH PRs                     → deploy frequency, change failure rate (DORA)
     ├── docs/postmortem/*          → action items follow-up status
     ├── GH Discussions Q&A         → support volume, question types, satisfaction
     └── GH labels/milestones       → velocity, sprint completion rate
          │
          ▼
   csi-analyst agent (cron monthly):
     ├── Aggregate
     ├── Detect trends (improving/degrading/stable)
     ├── Compare to SLO targets + DORA quartiles (Elite/High/Medium/Low)
     ├── Identify top-3 improvement opportunities
     └── Generate `docs/csi/csi-report-YYYY-MM.md`
          │
          ▼
   Trimestriel : présenté en Portfolio Sync (#428 §7)
     ├── Improvement initiatives → tickets GH labels `csi:improvement`
     ├── RFCs proposés pour évolutions process / Maury method (Phase H)
     └── ADRs si décision technique / archi nécessaire
   ```
   
   ### Métriques CSI clés à tracker (DORA + SRE)
   
   | Métrique | Source | Cible Elite | Cible Medium |
   |---|---|---|---|
   | **Deploy frequency** | GH PRs mergées + workflow runs | multiple/jour | weekly |
   | **Lead time for change** | issue creation → PR merge | < 1 jour | 1 sem - 1 mois |
   | **Mean time to restore (MTTR)** | incident open → resolve | < 1h | < 1 jour |
   | **Change failure rate** | incidents post-deploy / total deploys | < 5 % | 16-30 % |
   | **SLO availability** | Prometheus uptime probe | ≥ 99.9 % | ≥ 99 % |
   | **SLO latency P99** | Prometheus histogram | < 500ms | < 1s |
   | **Support resolution time** | GH Discussion Q&A response time | < 4h | < 24h |
   | **Doc coverage Q&A** | % questions ayant doc existante (no creation needed) | ≥ 80 % | ≥ 50 % |
   
   ### CSI report structure (template)
   ```yaml
   ---
   period: 2026-04
   owner: csi-analyst (Claude)
   reviewer: <human-supervisor>
   status: draft | reviewed | published
   ---
   
   ## Executive summary
   [3 lignes : trend global, top achievement, top concern]
   
   ## Métriques DORA + SLO
   [tableau métriques + variation vs mois précédent]
   
   ## Top 3 improvement opportunities
   1. [opportunity] — proposed action — RFC link
   2. ...
   3. ...
   
   ## Lessons from incidents this period
   [liens postmortems + thèmes communs]
   
   ## Lessons from support questions
   [questions récurrentes → RFCs FAQ proposées]
   
   ## Maury method evolution candidates (Phase H)
   [patterns observés qui pourraient déclencher une RFC v1.X]
   ```
   
   ---
   
   ## 6ter. Support agent — Q&A + doc retrieval/creation (`support-agent`)
   
   ### Workflow Q&A
   
   ```
   Question reçue (GH Discussion Q&A | Slack message via webhook | cowork chat | claude code interactive)
        │
        ▼
   support-agent search :
     ├── CLAUDE.md
     ├── docs/ (incluant Maury/, ADRs, RFCs)
     ├── postmortems
     ├── GH issues fermées (label question)
     └── GH Discussions historiques
        │
        ▼
   Match trouvé ? ──── Oui ──→ Réponse avec citation + lien (Tier 2, logué)
        │
        Non
        │
        ▼
   Question récurrente ? (≥ 3 fois ce mois)
        │
        ├── Oui → Propose RFC FAQ entry (T1 humain valide)
        │
        └── Non → Cherche personne compétente (PO, dev, SRE selon topic)
                 → Tag dans GH Discussion + draft réponse
                 → Humain compétent finalise (T1)
                 → Réponse archivée pour future search
   ```
   
   ### Création de documentation (Tier 1 strict)
   
   L'agent `support-agent` peut **proposer** un nouveau doc, jamais le créer directement :
   - Détecte que la question n'a pas de réponse + récurrente.
   - Draft un fichier `docs/faq/<topic>.md` ou `docs/runbook/<topic>.md`.
   - Ouvre une PR.
   - Humain reviews + approuve + merge.
   
   **Anti-pattern** : agent qui crée des fichiers doc à la volée sans review → drift, contradictions, pollution. Toujours PR-based.
   
   ### Cas d'usage support pour `koprogo`
   
   - Q&A interne (entre agents) : *"Comment je structure une PRD selon Maury v1.1 ?"* → retourne template + Maury/Méthode_Maury_v1.1.md.
   - Q&A humain → agent : *"Quel est l'état des secrets en helm values ?"* → retourne audit findings + lien #425.
   - Q&A externe (futurs utilisateurs) : *"Comment configurer mon copropriété ?"* → docs/getting-started.md (à créer).
   - Doc creation triggered : 5e fois qu'un agent demande "comment trigger un sprint review" → propose `docs/runbook/sprint-review.md`.
   
   ---
   
   ## 6quater. Rapport d'activité agent (Tier 2 audit)
   
   Toute activité Tier 2 d'agent est **logée**. Le `documentation-writer` agrège quotidiennement :
   
   ```
   docs/agent-activity/YYYY-MM-DD-<persona>.md
   
   ---
   date: 2026-04-29
   persona: support-agent
   activity_count: 47
   tier_breakdown:
     tier_2_logged: 47
     tier_1_proposed: 3 (PRs awaiting human review)
   ---
   
   ## Activity log
   
   ### 09:14 — Q&A from gilmry
   Question: "Quelle taxonomie de tests j'utilise pour FR-007 ?"
   Source matched: docs/maury/.../prd.md, .claude/rules/CRITICAL.md
   Response: 4 catégories obligatoires (@happy/@edge/@security/@negative), cf. #427 §A.3
   Outcome: ✓ closed by gilmry
   
   ### 10:22 — Q&A from dev-team-A
   Question: "Où est le hook RED-first ?"
   Source matched: AGENT_GUARDRAILS.md §L2 (planned for #427 partie A)
   Response: pas encore implémenté, prévu sprint S2
   Outcome: ✓ closed
   
   ### 14:55 — Recurring question detected
   Pattern: "Comment je gate une migration SQL ?" (5e occurrence ce mois)
   Action: proposed RFC FAQ → PR #520 [awaiting review]
   
   ### ... (47 total)
   
   ## Anomalies / blockers
   [questions sans réponse satisfaisante → escalade humaine]
   ```
   
   Le superviseur humain peut **auditer rétroactivement** sans avoir à suivre en temps réel chaque action.
   
   ---
   
   ## 7. Critères d'acceptation
   
   ### Couches L1-L7 implémentées
   - [ ] ServiceAccount k8s `koprogo-agent-readonly` avec RBAC `get/list/watch` + RoleBinding scopé namespace prod.
   - [ ] Token Prometheus read-only + token Loki read-only stockés en GH Secrets.
   - [ ] Webhook receiver alertmanager → repository_dispatch GH actif.
   - [ ] Workflow `incident-investigate.yml` qui invoque Claude avec contexte complet.
   - [ ] Workflow `terraform-apply-production.yml` avec environment "production" + required reviewers + audit log.
   - [ ] Workflow `argocd-sync-production.yml` similaire.
   - [ ] Workflow `velero-restore.yml` similaire.
   - [ ] ArgoCD `syncPolicy.automated: false` sur applications prod.
   - [ ] Template issue `incident-template.md` (sections : alert, diagnostic, plan, approval, postmortem).
   - [ ] Template issue `postmortem-template.md`.
   
   ### Personas
   - [ ] `.claude/agents/devops-engineer.md` + `.memory.md`.
   - [ ] `.claude/agents/sre-platform.md` + `.memory.md`.
   - [ ] `.claude/agents/platform-engineer.md` + `.memory.md`.
   - [ ] `.claude/agents/support-agent.md` + `.memory.md`.
   - [ ] `.claude/agents/csi-analyst.md` + `.memory.md`.
   - [ ] Mention des 5 personas dans `docs/SIMULATION_MANIFEST.md`.
   
   ### Tier model + activity reports
   - [ ] Doc explicite Tier 1 vs Tier 2 dans `.claude/AGENT_GUARDRAILS.md` (à amender).
   - [ ] `documentation-writer` génère quotidien `docs/agent-activity/YYYY-MM-DD-<persona>.md` à partir des events GH + logs agent.
   - [ ] Hook `Stop` étendu pour ajouter le résumé d'activité du tour à `docs/agent-activity/`.
   - [ ] Audit retro (manuel) : choisir 5 entrées au hasard de l'activity report et vérifier que c'était bien Tier 2 (pas de mutation cachée).
   
   ### Validation pilote
   - [ ] Un incident pilote simulé : alerte synthétique fire → workflow trigger → Claude diagnose → issue créée → humain approuve → exécution + audit trail.
   - [ ] Un déploiement IaC pilote : PR sur `infrastructure/` → plan en commentaire → review → merge → workflow_dispatch apply → environment approval → terraform apply → audit log complet.
   - [ ] Un Q&A pilote : humain pose 10 questions au `support-agent` (mix de questions ayant doc existante + sans) ; mesure : recall (% trouvées avec doc) + précision (% bonnes réponses) + propositions de RFC FAQ pour les manquantes.
   - [ ] Un CSI report pilote : `csi-analyst` génère `docs/csi/csi-report-2026-MM.md` avec métriques DORA + SLO + improvement opportunities.
   
   ### SLO + CSI
   - [ ] SLO formalisés dans `docs/safe/slo.md` (par `sre-platform`).
   - [ ] Alertmanager rules basées sur ces SLO.
   - [ ] Pipeline CSI mensuel actif (cron + report dans `docs/csi/`).
   - [ ] Top-3 improvement opportunities du premier CSI report → issues GH `csi:improvement` créées.
   
   ---
   
   ## 8. Sprints (proposition)
   
   | Sprint | Livrables | Durée |
   |---|---|---|
   | **S1 — Read-only credentials + 5 personas** | RBAC k8s + tokens read + 5 personas matérialisés (devops/sre/platform/support/csi) avec system prompts + memory files | 2 sem |
   | **S2 — Tier model + activity reports** | Doc Tier 1/2 dans AGENT_GUARDRAILS.md, `documentation-writer` cron daily activity report, audit retro pilote | 1 sem |
   | **S3 — Workflow dispatch IaC apply + ArgoCD manual sync** | terraform-apply-production.yml + argocd-sync-production.yml + environments + required reviewers | 2 sem |
   | **S4 — Reactive incident workflow** | webhook alertmanager → repository_dispatch → incident-investigate.yml + Claude integration | 3 sem |
   | **S5 — Support agent (Q&A + retrieval + RFC FAQ proposal)** | `support-agent` actif sur GH Discussions Q&A + Slack webhook ; templates FAQ/runbook | 2 sem |
   | **S6 — CSI pipeline + SLO** | SLO doc, alertmanager rules, `csi-analyst` cron monthly, dashboard métriques DORA | 2 sem |
   | **S7 — Pilotes complets** | incident simulé + IaC apply + Q&A pilote + CSI pilote ; audit traces | 2 sem |
   
   Total ~14 semaines pour couverture runtime ops complète (deploy + monitoring + support + CSI).
   
   ---
   
   ## 9. Lien avec autres issues
   
   - **#425** (garde-fous build-time) — la deny list `terraform apply`, `helm upgrade`, `kubectl mutate`, `argocd sync` couvre la couche L6 côté agent. Cette issue ajoute le côté humain (workflows + environments) + le pattern Tier 1/Tier 2 + activity reports.
   - **#426** (cleanup docs) — manifeste public mentionnera le mode ops + 5 personas runtime + le `support-agent` qui peut répondre aux questions.
   - **#427** (validation pre-release) — la Cowork release report est prerequis avant promotion staging→prod ; cette issue prend le relais après prod (incident response, CSI).
   - **#428** (simulation org) — ajoute 5 personas runtime au cluster cross-cutting §6 (devops-engineer, sre-platform, platform-engineer, support-agent, csi-analyst). Ajoute 4 modes de déclenchement (cowork/dispatch/routine/claude code). Ajoute la rythme CSI au calendrier des cérémonies (#428 §7) : monthly CSI report, trimestriel CSI review en Portfolio Sync.
   - **Maury v1.1** — la couche garde-fous v1.1 inclut désormais explicitement la dimension runtime ops + Tier model + activity reports (mise à jour CHANGELOG à faire en S1).
   - **CRITICAL.md** — à étendre avec règle #11 "Tier 1 toujours validé humain ; Tier 2 toujours logué".
   
   ---
   
   ## 10. Risques
   
   - **Cost Claude API** : si chaque alertmanager fire déclenche un workflow Claude, le coût peut exploser. Mitigation : budget cap par jour, debounce (1 incident/alertname/heure), filtrage côté receiver pour skip alerts non-critiques.
   - **Faux positifs alertmanager** : trop de runbook automation → fatigue humaine. Mitigation : tuning des seuils + SLO-based alerts plutôt que threshold-based.
   - **Token compromis** : un token Prometheus read leaked = accès aux métriques (PII potentielle). Mitigation : rotation regulière, scoping ServiceAccount minimal.
   - **Agent qui tente de bypasser** : un agent malicieux pourrait essayer de générer un PR qui modifie les permissions. Mitigation : CODEOWNERS sur `.github/`, `.claude/settings.json`, infrastructure/, plus reviews humaines obligatoires.
   
   ---
   
   🤖 Issue générée par Claude Opus 4.7 — couverture runtime ops manquante : déploiement IaC sécurisé + agents DevOps/SRE réactifs + gates humains stricts. Complète l'apex #428 sur la dimension run.

.. raw:: html

   </div>

