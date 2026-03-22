====================================================
Issue #262: MCP: Indexation base légale docs/legal/
====================================================

:State: **OPEN**
:Milestone: Jalon 4: Automation & Intégrations 📅
:Labels: documentation,enhancement track:mcp,release:0.2.0
:Assignees: Unassigned
:Created: 2026-03-10
:Updated: 2026-03-15
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/262>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Description
   
   Indexer la base légale `docs/legal/` pour qu'elle soit exploitable par les outils MCP (legal_search, majority_calculator, alertes_list).
   
   ## Contenu à indexer
   
   ```
   docs/legal/
   ├── base_legale.rst                         ← Vue d'ensemble (NOUVEAU)
   ├── README.rst                              ← Navigation et conventions (NOUVEAU)
   ├── syndic/
   │   ├── index.rst
   │   ├── mandat.rst                          (M01-M07)
   │   ├── missions_legales.rst                (L01-L18)
   │   ├── deontologie_generale.rst            (G01-G09)
   │   ├── deontologie_specifique.rst          (D01-D10)
   │   └── travaux.rst                         (T01-T05)
   ├── coproprietaire/
   │   ├── index.rst
   │   └── droits_obligations.rst              (CP01-CP10)
   ├── locataire/
   │   ├── index.rst
   │   └── droits_obligations.rst              (LO01-LO04)
   ├── commissaire/
   │   ├── index.rst
   │   └── droits_obligations.rst              (CO01-CO05)
   ├── conseil-copropriete/
   │   ├── index.rst
   │   └── droits_obligations.rst              (CC01-CC05)
   ├── acp/
   │   ├── index.rst
   │   └── personnalite_juridique.rst          (ACP01-ACP04)
   ├── notaire/
   │   ├── index.rst
   │   └── transmission_lot.rst                (N01-N05)
   ├── assemblee-generale/
   │   ├── index.rst
   │   └── sequence_odj.rst                    (AG01-AG14)
   ├── achat-groupe-energie/
   │   └── module-specification.rst            (spécification fonctionnelle)
   ├── audit_conformite.rst                    (existant)
   ├── copropriete_art_3_84_3_92.rst           (existant)
   ├── matrice_conformite.rst                  (existant)
   ├── pcmn_ar_12_07_2012.rst                  (existant)
   ├── rgpd_conformite.rst                     (existant)
   └── risques_juridiques.rst                  (existant)
   ```
   
   ## État actuel
   
   - [x] ✅ Fichiers `docs/legal/` créés et structurés (commit `64c35f2`)
   - [x] ✅ Codes de règles assignés par domaine (M, L, D, G, T, AG, CP, LO, CO, CC, ACP, N)
   - [x] ✅ Index Sphinx (`index.rst`) restructuré avec toctree par rôle
   - [x] ✅ Convention de nommage documentée dans `README.rst`
   - [ ] Parser les fichiers RST : extraire codes de règle, articles de loi, rôles
   - [ ] Construire un index en mémoire (HashMap code → règle)
   - [ ] Implémenter la recherche full-text (par mot-clé, par rôle, par catégorie)
   - [ ] Mapper les types de décision aux majorités (pour majority_calculator)
   - [ ] Hot-reload si les fichiers changent (optionnel, phase ultérieure)
   - [ ] Tests : vérifier que tous les codes de règles sont indexés
   
   ## Convention de nommage
   
   | Préfixe | Domaine | Fichier source |
   |---------|---------|----------------|
   | M | Mandat syndic | `syndic/mandat.rst` |
   | L | Missions légales syndic | `syndic/missions_legales.rst` |
   | D | Déontologie spécifique syndic | `syndic/deontologie_specifique.rst` |
   | G | Déontologie générale | `syndic/deontologie_generale.rst` |
   | T | Travaux | `syndic/travaux.rst` |
   | AG | Assemblée générale | `assemblee-generale/sequence_odj.rst` |
   | CP | Copropriétaire | `coproprietaire/droits_obligations.rst` |
   | LO | Locataire/occupant | `locataire/droits_obligations.rst` |
   | CO | Commissaire aux comptes | `commissaire/droits_obligations.rst` |
   | CC | Conseil de copropriété | `conseil-copropriete/droits_obligations.rst` |
   | ACP | Association des copropriétaires | `acp/personnalite_juridique.rst` |
   | N | Notaire / transmission de lot | `notaire/transmission_lot.rst` |
   | F | Finance / comptabilité | (à créer) |
   
   ## Dépendances
   
   - Requis par #254 (legal_search + majority_calculator)
   - ✅ Fichiers `docs/legal/` créés (commit `64c35f2`)

.. raw:: html

   </div>

