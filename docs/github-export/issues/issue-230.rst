==================================================================================
Issue #230: R&D: Vote blockchain - Architecture smart contract et conformité GDPR
==================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: priority:low,proptech:blockchain R&D
:Assignees: Unassigned
:Created: 2026-03-07
:Updated: 2026-03-07
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/230>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   L'issue #111 prévoit un système de vote blockchain. Cette R&D couvre les
   choix architecturaux fondamentaux et la tension blockchain/GDPR.
   
   **Issue liée**: #111
   
   ## Objectifs de la R&D
   
   1. **Sélection blockchain** :
      - Polygon (L2 Ethereum, faible coût, EVM-compatible)
      - Solana (haute performance, Rust natif - aligné avec le stack)
      - Avalanche (subnets, adapté aux cas d'usage entreprise)
      - Hyperledger Fabric (privé, permissioned, GDPR-friendly)
      - Blockchain privée custom (contrôle total, complexité élevée)
   
   2. **Smart contract design** :
      - Vote storage : on-chain (hash) vs. hybrid (données off-chain, preuve on-chain)
      - Anonymisation : zero-knowledge proofs pour confidentialité du vote
      - Gestion des mandats (procurations) on-chain
      - Résultat : calcul on-chain vs. off-chain avec preuve
   
   3. **Tension GDPR vs. Immutabilité** :
      - Droit à l'effacement (Art. 17) vs. blockchain immutable
      - Solution : stocker uniquement des hashes, pas de données personnelles
      - Consentement explicite pour utilisation blockchain
      - Juridiction : nœuds dans l'UE uniquement
   
   4. **UX Wallet** :
      - MetaMask (standard mais complexe pour non-techniciens)
      - Social login wallet (Web3Auth, Privy)
      - Custodial wallet (simple mais centralisé)
      - Hardware wallet (Ledger) pour syndics
   
   5. **Coûts** :
      - Gas fees par vote (Polygon : ~0.01€, Ethereum L1 : ~5-50€)
      - Infrastructure nœud (archival node vs. light node)
      - Audit sécurité smart contract (Trail of Bits : ~100k€)
   
   ## Points de décision
   
   - [ ] Public vs. private blockchain
   - [ ] On-chain data model (hash-only vs. full data)
   - [ ] Wallet UX strategy
   - [ ] Coût total estimé vs. bénéfice (confiance, transparence)
   - [ ] Faut-il vraiment une blockchain ? (alternative : merkle tree + signature)
   
   ## Estimation
   
   20-25h (étude approfondie)

.. raw:: html

   </div>

