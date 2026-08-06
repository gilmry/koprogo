========================================================================================================================
Issue #636: [SECURITY] Rust Security Audit rouge — 2 vulns high (lopdf RUSTSEC-2026-0187, quinn-proto RUSTSEC-2026-0185)
========================================================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: security
:Assignees: Unassigned
:Created: 2026-06-27
:Updated: 2026-06-27
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/636>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Constat
   
   Le job CI **Security Audit → Rust Security Audit** (`cargo audit`, 691 crates) échoue avec **2 vulnérabilités high** sur `feature/dev` (constaté commit 006030e, déjà rouge sur les commits précédents). Distinct de #634 (qui concerne le sous-job *NPM* Security Audit + le build frontend).
   
   ## Vulnérabilités
   
   | ID | Crate | Titre | Sévérité | Solution |
   |---|---|---|---|---|
   | [RUSTSEC-2026-0187](https://rustsec.org/advisories/RUSTSEC-2026-0187) | `lopdf` | Stack overflow via PDF profondément imbriqués | **7.5 (high)** | Upgrade `>= 0.42.0` |
   | [RUSTSEC-2026-0185](https://rustsec.org/advisories/RUSTSEC-2026-0185) | `quinn-proto` | Remote memory exhaustion (réassemblage de flux out-of-order non borné) | **7.5 (high)** | Upgrade `>= 0.11.15` |
   | [RUSTSEC-2026-0173](https://rustsec.org/advisories/RUSTSEC-2026-0173) | `proc-macro-error2` | Unmaintained | warning (toléré) | — |
   
   ## Cause
   
   - `lopdf` : dépendance directe (génération PDF — convocations / PV / état daté). Bump mineur/majeur à vérifier (API).
   - `quinn-proto` : dépendance **transitive** (QUIC, probablement via reqwest/HTTP3 ou un client). `cargo update -p quinn-proto --precise 0.11.15` peut suffire.
   
   ## Recette proposée
   - [ ] `lopdf` → `>= 0.42.0` (vérifier l'API PDF utilisée ; tests génération PDF verts).
   - [ ] `quinn-proto` → `>= 0.11.15` (`cargo update -p quinn-proto`, ou bump du parent).
   - [ ] `cargo audit` vert (hors warning `proc-macro-error2` toléré via `.cargo/audit.toml` si déjà configuré).
   - [ ] (optionnel) remplacer `proc-macro-error2` (unmaintained) à terme.
   
   ## Critères d'acceptation
   - [ ] Job `Rust Security Audit` vert sur `feature/dev`.
   - [ ] Pas de régression (build + tests backend verts).
   
   Note : v0.1.0 pre-release, rien en prod — pas une crise, mais à corriger avant go-live. Refs : CI run Security Audit ; voisin #634 (NPM audit / frontend).

.. raw:: html

   </div>

