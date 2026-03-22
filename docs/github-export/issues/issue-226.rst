================================================================================
Issue #226: R&D: Assistant IA Syndic - Architecture LLM, RAG et conformité GDPR
================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: priority:medium,gdpr proptech:ai,R&D
:Assignees: Unassigned
:Created: 2026-03-07
:Updated: 2026-03-07
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/226>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   L'issue #94 prévoit des fonctionnalités IA. Cette R&D couvre l'architecture
   de l'assistant IA syndic pour répondre aux questions juridiques et administratives.
   
   **Issue liée**: #94
   
   ## Objectifs de la R&D
   
   1. **Sélection du LLM** :
      - Claude API (Anthropic) : qualité, coût, API stable
      - GPT-4 (OpenAI) : écosystème mature, function calling
      - LLaMA/Mistral local (Ollama) : souveraineté des données, GDPR-friendly
      - OVH AI Endpoints : hébergement européen, conformité RGPD
   
   2. **RAG (Retrieval-Augmented Generation)** :
      - Corpus : loi belge copropriété (Code Civil art. 577), règlements types
      - Vector DB : pgvector (PostgreSQL natif) vs. Qdrant vs. ChromaDB
      - Chunking strategy : par article de loi, par FAQ, par jurisprudence
      - Embedding model : sentence-transformers vs. OpenAI embeddings
   
   3. **Use cases prioritaires** :
      - FAQ juridique (« Quorum pour travaux urgents ? »)
      - Rédaction assistée de PV d'AG
      - Classification automatique de tickets (catégorie + priorité)
      - Anomalie detection sur les charges (factures inhabituelles)
   
   4. **GDPR Article 22** :
      - Pas de décision automatisée sans intervention humaine
      - Explication des résultats (« Pourquoi cette classification ? »)
      - Droit d'opposition au traitement IA
      - Anonymisation des données d'entraînement
   
   ## Points de décision
   
   - [ ] LLM provider (API externe vs. local)
   - [ ] Vector DB (pgvector intégré vs. service dédié)
   - [ ] Coût opérationnel mensuel estimé (€/1000 requêtes)
   - [ ] Stratégie de fallback si LLM indisponible
   - [ ] Consentement utilisateur obligatoire pour features IA
   
   ## Estimation
   
   15-20h

.. raw:: html

   </div>

