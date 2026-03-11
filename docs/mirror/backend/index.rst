===============
Backend - Index
===============


Le backend KoproGo est développé en **Rust** avec le framework **Actix-web**.

**Architecture** : Hexagonale (Ports & Adapters) + DDD (Domain-Driven Design)

**Statistiques** :
- **321 fichiers Rust** (~50,000 lignes de code)
- **51 entités de domaine** avec validation métier
- **46 repositories PostgreSQL**
- **44 handlers HTTP** (73 endpoints API REST)
- **100+ migrations SQL**

**Performance** :
- P99 latency < 5ms
- Throughput > 100k req/s
- Memory < 128MB par instance


Contenu
=======

.. toctree::
   :maxdepth: 2

   src/index

