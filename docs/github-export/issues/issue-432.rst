===========================================================================================
Issue #432: Security — 14 dependabot vulnerabilities sur main (5 high / 3 moderate / 6 low)
===========================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: dependencies,priority:high security
:Assignees: Unassigned
:Created: 2026-04-30
:Updated: 2026-04-30
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/432>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Source
   
   `git push origin feature/dev` 2026-04-30 a déclenché un message remote :
   
   ```
   remote: GitHub found 14 vulnerabilities on gilmry/koprogo's default branch
   remote: (5 high, 3 moderate, 6 low). To find out more, visit:
   remote: https://github.com/gilmry/koprogo/security/dependabot
   ```
   
   Vulnérabilités sur `main` (default branch). Pas spécifique à AUTH-001 (#430/#431) — pré-existantes.
   
   ## À faire
   
   - Persona `security-officer` (Tier 2) : ouvrir le dashboard dependabot, lister les 14 CVE, classer par sévérité+exploitabilité+effort fix.
   - Confronter avec les 28 branches `dependabot/*` zombies signalées dans l'audit #425 — y a-t-il des PRs déjà ouvertes qui adressent ces vulnérabilités et n'attendent que merge ?
   - Décider stratégie : auto-merge dependabot patch+minor, manuel pour major.
   
   ## Lien
   
   - Audit initial : #425 (mentionne déjà l'absence de discipline dependabot)
   - Reportage : 5 high → traiter cette sprint si exploitable, sinon W19+
   - Dashboard : https://github.com/gilmry/koprogo/security/dependabot
   
   🤖 Issue créée par `rust-expert` (Claude) sur observation push remote — escalade à `security-officer` per matrice personas (#428).

.. raw:: html

   </div>

