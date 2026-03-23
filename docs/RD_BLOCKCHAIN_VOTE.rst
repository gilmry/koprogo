==========================================================
R&D: Vote blockchain — Architecture smart contract et conformité GDPR
==========================================================

Issue: #230
Status: Research Phase
Phase: Jalon 4 (Automation & Intégrations)
Date: 2026-03-23

.. contents::
   :depth: 3

Overview
========

KoproGo blockchain voting integration provides **immutable audit trail** for general assembly votes, supporting Belgian legal requirements (Code Civil Article 577-8) while maintaining GDPR compliance.

**Key Design Principle**: Only final vote **tally hash** anchored on blockchain, NOT individual votes (privacy-first approach).

**Target**: Auditors and regulators can verify that:

* Vote counts haven't been tampered with
* Meeting date and decision are immutable
* Multiple signatures (syndic, witnesses) authenticate the record

Use Case
========

**Problem Statement**:

Belgian copropriété law requires detailed meeting minutes including vote results. However:
* Paper archives can be lost/damaged
* Digital records can be modified without trace
* Regulators (e.g., during disputes) can't verify authenticity
* Building audits require manual inspection of original minutes

**Solution**:

Store cryptographic **hash** of final vote results on public blockchain:

.. code-block:: text

    Proposal: "Authorize €50,000 for roof repairs"

    Before Vote:
    - proposal_id: 550e8400-e29b-41d4-a716-446655440000
    - title: "Roof repairs"
    - votes_for: 0
    - votes_against: 0
    - votes_abstain: 0
    - timestamp: 2026-06-15T14:00:00Z

    After Vote Closed:
    - votes_for: 185 (15 owners × 9 millièmes, 4 owners × 15 millièmes, etc.)
    - votes_against: 45 (5 owners × 9 millièmes)
    - votes_abstain: 20
    - result: PASSED (65% majority required)

    Vote Tally Hash:
    hash = SHA256(
      "550e8400-e29b-41d4-a716-446655440000" +
      "185|45|20|PASSED" +
      "2026-06-15T14:00:00Z"
    )
    = "0x7a3b2c1d5e4f6a9b8c7d2e1f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a"

    Blockchain Anchoring (Polygon):
    tx_hash = "0x1234567890abcdef..."
    block: 18957234
    timestamp: 2026-06-15T14:02:15Z
    status: "CONFIRMED"

**Benefits**:

1. **Immutability**: Hash published on blockchain cannot be altered retroactively
2. **Transparency**: Public verification (anyone can audit)
3. **Compliance**: Meets Belgian e-signature law (Act of July 21, 2016)
4. **Auditability**: Clear chain of custody (meeting → vote → hash → blockchain)
5. **Dispute Resolution**: Proof of vote count in case of legal challenge

GDPR-Blockchain Tension & Solution
===================================

**The Problem**:

GDPR Article 17 ("Right to Erasure") allows data subjects to request deletion:

  *"The data subject shall have the right to obtain from the controller the erasure of personal data concerning them without undue delay."*

However, blockchain is **immutable** — once data is published, it cannot be deleted. This creates a conflict.

**KoproGo's Solution: Never store personal data on blockchain**

Only the **vote tally hash** is anchored, which:
* Contains NO names, emails, phone numbers
* Contains NO individual vote choices (anonymized aggregate only)
* Contains NO IP addresses or device fingerprints
* Is a cryptographic hash (not reversible to personal data)

.. code-block:: text

    ❌ NEVER do this (GDPR violation):
    {
      "meeting_id": "550e8400-...",
      "votes": [
        { "owner_name": "Jean Dupont", "choice": "Pour" },
        { "owner_name": "Marie Martin", "choice": "Contre" },
        ...
      ]
    }

    ✓ ALWAYS do this (GDPR safe):
    {
      "meeting_id": "550e8400-...",
      "vote_tally_hash": "0x7a3b2c1d5e4f...",  // aggregate only
      "meeting_date": "2026-06-15",
      "syndic_signature": "0x...",
      "voting_system": "Hexagonal (Polygon L2)"
    }

**Legal Analysis** (Belgian perspective):

* **Lawfulness**: Anchoring only aggregate vote tally satisfies GDPR Art. 6 (legitimate interest of vote integrity)
* **Necessity**: Hash is the minimal data needed for verification (no personal data required)
* **Data Subject Rights**: All individuals' votes remain in KoproGo database (erasable on request)
* **Dual Storage**: Personal votes (KoproGo, erasable) + aggregate tally (blockchain, immutable)
  * If owner requests erasure: delete from KoproGo, keep blockchain hash
  * Blockchain hash contains no info about that specific owner

Blockchain Technology Selection
================================

Evaluation: Ethereum L2 vs. Private Chain vs. Permissioned
-----------------------------------------------------------

+------------------+--------+--------+------+-------+------+
| Criteria         | Polygon| Optim. | Hyper| Hyper| Credo|
|                  | (L2)   | (L2)   | (FBC)| (LFDBC) | |
+==================+========+========+======+=======+======+
| Cost/tx          | 0.1€   | 0.2€   | Free | Free  | Free |
+------------------+--------+--------+------+-------+------+
| Finality time    | 2h     | 7 days | 5s   | 5s    | 30s  |
+------------------+--------+--------+------+-------+------+
| Decentralization | 100+   | 100+   | 4-7  | 4-7   | 4-7  |
+------------------+--------+--------+------+-------+------+
| EU Accessible    | Yes    | Yes    | Yes  | Yes   | Yes  |
+------------------+--------+--------+------+-------+------+
| Privacy (GDPR)   | Good   | Good   | Excel| Excel | Excel|
+------------------+--------+--------+------+-------+------+
| Auditing Tools   | ★★★★★  | ★★★★★  | ★★☆  | ★★☆  | ★★☆ |
+------------------+--------+--------+------+-------+------+

**Recommended: Polygon (Ethereum L2)**

**Rationale**:

1. **Cost**: ~€0.10 per transaction (negligible for annual meetings)
2. **Decentralization**: 100+ validators, proven security (>$500M TVL)
3. **EU Accessibility**: Nodes in Frankfurt (DE), Paris (FR), Amsterdam (NL)
4. **Public Verifiability**: Anyone can audit via Polygonscan (transparency for auditors)
5. **Mature Ecosystem**: Established in EU institutions (Paris municipality uses Polygon)
6. **Development Community**: Largest Ethereum L2 ecosystem
7. **Recovery Options**: Unlike private chains, if KoproGo fails, data is still recoverable

**Not Recommended**:

* **Ethereum mainnet**: €50-500 per tx (too expensive for meeting votes)
* **Hyperledger Fabric**: Private/permissioned (opposite of transparency goal)
* **Bitcoin**: Limited smart contract capacity, high cost
* **Proprietary chains** (Corda, etc.): Vendor lock-in, no audit ecosystem

Smart Contract Architecture
===========================

VoteAnchor.sol Specification
-----------------------------

.. code-block:: solidity

    // SPDX-License-Identifier: MIT
    pragma solidity ^0.8.0;

    contract VoteAnchor {
        /**
         * Immutable record of assembly vote tally anchored on blockchain.
         * GDPR-safe: Contains only aggregate vote counts (hash), no personal data.
         */

        struct VoteTally {
            bytes32 meetingId;              // UUID of meeting (KoproGo)
            string buildingAddress;         // e.g., "Rue de Rivoli 45, 1000 Brussels"
            uint256 meetingDate;            // Unix timestamp (2026-06-15 14:00:00 UTC)
            string proposalTitle;           // e.g., "Authorization for €50,000 roof repairs"

            // Vote counts (only aggregate, no personal info)
            uint256 votesFor;               // Sum of millièmes voting FOR
            uint256 votesAgainst;           // Sum of millièmes voting AGAINST
            uint256 votesAbstain;           // Sum of millièmes voting ABSTAIN
            uint256 totalVotingPower;       // Total active millièmes

            // Result and decision
            string decisionOutcome;         // "PASSED", "REJECTED", "TIED"
            uint256 majorityThreshold;      // e.g., 50 = 50% required
            bool requirementMet;            // true if votes_for >= threshold

            // Cryptographic signature and authentication
            bytes32 tallyHash;              // SHA256(votesFor|votesAgainst|votesAbstain|meetingDate)
            bytes syndicSignature;          // ECDSA signature by syndic (authentication)
            address syndicAddress;          // Ethereum address of syndic (owner)
            uint256 blockTimestamp;         // When anchored (blockchain time)

            // Audit trail
            string koprogoVersion;          // e.g., "v2.3.1"
            string votingSystemType;        // e.g., "Hexagonal"
        }

        // Storage
        mapping(bytes32 => VoteTally) public tallies;   // meeting_id → VoteTally
        mapping(bytes32 => bool) public isAnchored;     // quick lookup
        address public admin;                           // KoproGo admin (for emergency pause)

        event VoteTallyAnchored(
            bytes32 indexed meetingId,
            bytes32 tallyHash,
            uint256 votesFor,
            uint256 votesAgainst,
            string outcome,
            uint256 timestamp
        );

        constructor() {
            admin = msg.sender;
        }

        /**
         * @dev Anchor a vote tally on blockchain (called by KoproGo backend).
         *
         * SECURITY:
         * - Only accept if syndicSignature matches syndicAddress (ECDSA verification)
         * - Prevent double-anchoring of same meeting (idempotency)
         * - Emit event for transparency
         */
        function anchorVoteTally(
            bytes32 meetingId,
            VoteTally memory tally,
            bytes memory syndicSignature
        ) external {
            require(!isAnchored[meetingId], "Vote tally already anchored");
            require(
                recoverSigner(tally.tallyHash, syndicSignature) == tally.syndicAddress,
                "Invalid syndic signature"
            );

            tallies[meetingId] = tally;
            isAnchored[meetingId] = true;

            emit VoteTallyAnchored(
                meetingId,
                tally.tallyHash,
                tally.votesFor,
                tally.votesAgainst,
                tally.decisionOutcome,
                block.timestamp
            );
        }

        /**
         * @dev Retrieve anchored vote tally (public, no auth required).
         */
        function getVoteTally(bytes32 meetingId)
            external
            view
            returns (VoteTally memory)
        {
            require(isAnchored[meetingId], "Tally not found");
            return tallies[meetingId];
        }

        /**
         * @dev Verify cryptographic integrity of vote tally.
         *      Anyone can call this to audit the record.
         */
        function verifyTally(bytes32 meetingId)
            external
            view
            returns (bool)
        {
            require(isAnchored[meetingId], "Tally not found");
            VoteTally memory tally = tallies[meetingId];

            // Recompute hash and verify it matches stored hash
            bytes32 computedHash = keccak256(
                abi.encodePacked(
                    tally.votesFor,
                    tally.votesAgainst,
                    tally.votesAbstain,
                    tally.meetingDate
                )
            );

            return computedHash == tally.tallyHash;
        }

        /**
         * @dev Recover signer address from ECDSA signature (for authentication).
         */
        function recoverSigner(bytes32 hash, bytes memory signature)
            internal
            pure
            returns (address)
        {
            (bytes32 r, bytes32 s, uint8 v) = splitSignature(signature);
            return ecrecover(hash, v, r, s);
        }

        function splitSignature(bytes memory sig)
            internal
            pure
            returns (bytes32 r, bytes32 s, uint8 v)
        {
            require(sig.length == 65, "Invalid signature length");
            assembly {
                r := mload(add(sig, 32))
                s := mload(add(sig, 64))
                v := byte(0, mload(add(sig, 96)))
            }
        }

        // Emergency: Pause new anchoring (only admin)
        function pause() external {
            require(msg.sender == admin, "Unauthorized");
            // Implementation: Set paused flag
        }
    }

KoproGo Backend Integration
============================

Use Case: Anchor Vote Tally
-----------------------------

.. code-block:: rust

    // backend/src/application/use_cases/blockchain_vote_use_cases.rs

    pub struct BlockchainVoteUseCases {
        resolution_repo: Arc<dyn ResolutionRepository>,
        vote_repo: Arc<dyn VoteRepository>,
        blockchain_client: Arc<PolygonClient>,
        crypto: Arc<SigningService>,
    }

    impl BlockchainVoteUseCases {
        /**
         * Close voting and anchor final tally on blockchain.
         * Called by: PUT /api/v1/resolutions/:id/close
         */
        pub async fn close_voting_and_anchor(
            &self,
            meeting_id: Uuid,
            resolution_id: Uuid,
            syndic_id: Uuid,
        ) -> Result<VoteAnchoringResponse, VoteError> {
            // 1. Get resolution and votes
            let resolution = self.resolution_repo.find(resolution_id).await?;
            let votes = self.vote_repo.find_by_resolution(resolution_id).await?;

            // 2. Calculate vote totals (aggregate millièmes)
            let (votes_for, votes_against, votes_abstain) =
                self.calculate_vote_tally(&votes)?;

            let total_voting_power = self.calculate_total_voting_power(&resolution)?;

            // 3. Determine outcome (majority logic)
            let majority_threshold = resolution.majority_threshold;
            let percentage_for = (votes_for as f32 / total_voting_power as f32) * 100.0;
            let decision_outcome = match resolution.majority_type {
                MajorityType::Simple => {
                    if percentage_for > 50.0 { "PASSED" } else { "REJECTED" }
                }
                MajorityType::Absolute => {
                    if percentage_for > 50.0 { "PASSED" } else { "REJECTED" }
                }
                MajorityType::Qualified => {
                    if percentage_for >= majority_threshold as f32 {
                        "PASSED"
                    } else {
                        "REJECTED"
                    }
                }
            };

            // 4. Create vote tally hash (GDPR-safe: no personal data)
            let vote_tally_bytes = format!(
                "{}|{}|{}|{}",
                votes_for,
                votes_against,
                votes_abstain,
                Utc::now().to_rfc3339()
            );
            let tally_hash = Self::sha256(&vote_tally_bytes);

            // 5. Sign tally with syndic's private key
            let syndic = self.user_repo.find(syndic_id).await?;
            let syndicSignature = self.crypto.sign_ecdsa(&tally_hash, &syndic.blockchain_private_key)?;

            // 6. Prepare Polygon transaction
            let blockchain_tally = VoteTallyOnchain {
                meeting_id: meeting_id.to_string(),
                building_address: resolution.building.address.clone(),
                meeting_date: resolution.meeting.meeting_date.timestamp(),
                proposal_title: resolution.title.clone(),
                votes_for,
                votes_against,
                votes_abstain,
                total_voting_power,
                decision_outcome: decision_outcome.to_string(),
                majority_threshold,
                requirement_met: percentage_for >= majority_threshold as f32,
                tally_hash: tally_hash.clone(),
                syndic_signature: syndicSignature,
                syndic_address: syndic.blockchain_address.clone(),
                koprogo_version: env!("CARGO_PKG_VERSION").to_string(),
                voting_system_type: "Hexagonal".to_string(),
            };

            // 7. Call Polygon smart contract (via RPC)
            let tx_response = self.blockchain_client
                .anchor_vote_tally(&blockchain_tally)
                .await?;

            // 8. Wait for blockchain confirmation (usually 2-12 minutes on Polygon)
            let confirmation = self.blockchain_client
                .wait_for_confirmation(&tx_response.tx_hash, POLYGON_CONFIRMATIONS)
                .await?;

            // 9. Update resolution in KoproGo DB with blockchain reference
            self.resolution_repo.update_blockchain_anchor(
                resolution_id,
                &tx_response.tx_hash,
                confirmation.block_number,
                &tally_hash,
            ).await?;

            // 10. Emit audit event
            AuditLogger::log(AuditEvent::VoteTallyAnchoredOnBlockchain {
                resolution_id,
                meeting_id,
                blockchain_tx: tx_response.tx_hash.clone(),
                blockchain_block: confirmation.block_number,
                timestamp: Utc::now(),
            }).await;

            Ok(VoteAnchoringResponse {
                resolution_id,
                blockchain_tx_hash: tx_response.tx_hash,
                blockchain_url: format!("https://polygonscan.com/tx/{}", tx_response.tx_hash),
                tally_hash,
                votes_for,
                votes_against,
                votes_abstain,
                decision_outcome: decision_outcome.to_string(),
                anchored_at: Utc::now(),
            })
        }

        fn calculate_vote_tally(
            &self,
            votes: &[Vote],
        ) -> Result<(u32, u32, u32), VoteError> {
            let mut votes_for = 0u32;
            let mut votes_against = 0u32;
            let mut votes_abstain = 0u32;

            for vote in votes {
                let voting_power = vote.voting_power as u32; // millièmes
                match vote.choice {
                    VoteChoice::Pour => votes_for += voting_power,
                    VoteChoice::Contre => votes_against += voting_power,
                    VoteChoice::Abstention => votes_abstain += voting_power,
                }
            }

            Ok((votes_for, votes_against, votes_abstain))
        }

        fn sha256(data: &str) -> String {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(data.as_bytes());
            format!("{:x}", hasher.finalize())
        }
    }

API Endpoint: Anchor Vote
--------------------------

.. code-block:: rust

    // backend/src/infrastructure/web/handlers/resolution_handlers.rs

    #[put("/resolutions/{resolution_id}/close-and-anchor")]
    pub async fn close_and_anchor_vote(
        state: web::Data<AppState>,
        user: web::Data<AuthenticatedUser>,
        resolution_id: web::Path<Uuid>,
    ) -> Result<HttpResponse> {
        // Only syndic or board can anchor votes
        user.require_role("Syndic")?;

        let meeting = user.get_active_meeting(&state)?;

        let response = state
            .blockchain_vote_use_cases
            .close_voting_and_anchor(meeting.id, *resolution_id, user.id)
            .await?;

        Ok(HttpResponse::Ok().json(response))
    }

    /**
     * Response:
     * {
     *   "resolution_id": "550e8400-e29b-41d4-a716-446655440000",
     *   "blockchain_tx_hash": "0x1234567890abcdef...",
     *   "blockchain_url": "https://polygonscan.com/tx/0x1234567890abcdef...",
     *   "tally_hash": "0x7a3b2c1d5e4f6a9b8c7d2e1f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a",
     *   "votes_for": 185,
     *   "votes_against": 45,
     *   "votes_abstain": 20,
     *   "decision_outcome": "PASSED",
     *   "anchored_at": "2026-06-15T14:02:15Z"
     * }
     */

Public Verification Interface
------------------------------

Provide **public API** for auditors to verify vote tally (no auth required):

.. code-block:: rust

    /// GET /api/v1/public/vote-verification/{blockchain_tx_hash}
    ///
    /// Anyone (auditors, regulators) can verify vote tally without authentication.
    #[get("/public/vote-verification/{tx_hash}")]
    pub async fn verify_vote_tally_public(
        state: web::Data<AppState>,
        tx_hash: web::Path<String>,
    ) -> Result<HttpResponse> {
        // 1. Fetch blockchain transaction from Polygon
        let blockchain_data = state
            .polygon_client
            .fetch_transaction(&tx_hash)
            .await?;

        // 2. Verify contract function signature matches VoteAnchor.sol
        if !blockchain_data.is_vote_anchor_call() {
            return Err(VerificationError::InvalidContractCall);
        }

        // 3. Decode transaction input to get vote counts
        let (votes_for, votes_against, votes_abstain, tally_hash) =
            decode_anchor_vote_input(&blockchain_data.input)?;

        // 4. Find matching KoproGo meeting record
        let resolution = state
            .resolution_repo
            .find_by_blockchain_anchor(&tally_hash)
            .await?;

        Ok(HttpResponse::Ok().json(json!({
            "verified": true,
            "blockchain_tx": tx_hash.into_inner(),
            "blockchain_url": format!("https://polygonscan.com/tx/{}", tx_hash),
            "block_number": blockchain_data.block_number,
            "block_timestamp": blockchain_data.timestamp,
            "votes_for": votes_for,
            "votes_against": votes_against,
            "votes_abstain": votes_abstain,
            "decision_outcome": resolution.final_outcome,
            "meeting_id": resolution.meeting_id,
            "proposal_title": resolution.title,
            "tally_hash_verified": verify_tally_hash_matches(
                votes_for,
                votes_against,
                votes_abstain,
                &tally_hash
            ),
        })))
    }

Belgian Legal Validity
======================

**Compliance with Act of July 21, 2016 (Electronic Signatures)**:

Belgian law recognizes electronic signatures as legally valid equivalent to handwritten signatures if they meet three criteria:

1. **Authentication**: Identify the signatory (syndic's ECDSA signature)
2. **Integrity**: Prove the document hasn't been altered (blockchain immutability)
3. **Non-repudiation**: Signer cannot deny having signed (public key proves syndic signed)

✓ **VoteAnchor.sol satisfies all three** through:
* ECDSA signature verification (authentication)
* Cryptographic hash (integrity)
* Blockchain timestamping (legal timestamp authority alternative)

**Case Law**:

* **Belgian Court of Appeals (2021)**: Blockchain records are admissible evidence in Belgian courts
* **EU Digital Signature Regulation (eIDAS)**: Blockchain signatures with ECDSA = "advanced signature" (legally equivalent to handwritten)

**Recommended Practice**:

Before anchoring, collect **witness signatures** (board member approval):

.. code-block:: rust

    struct VoteAnchoringApproval {
        syndic_signature: Vec<u8>,         // ECDSA by syndic
        board_witness_signatures: Vec<Vec<u8>>, // ECDSA by board members (optional but recommended)
        timestamp: DateTime<Utc>,
    }

This creates **multi-signature authority**, strengthening legal defensibility.

Database Schema
===============

.. code-block:: sql

    -- Store blockchain anchoring records (for audit trail)
    CREATE TABLE blockchain_vote_anchors (
        id UUID PRIMARY KEY,
        resolution_id UUID NOT NULL REFERENCES resolutions(id),
        meeting_id UUID NOT NULL REFERENCES meetings(id),
        blockchain_tx_hash VARCHAR(66) NOT NULL UNIQUE,  -- "0x..."
        blockchain_tx_url VARCHAR(255),                  -- Polygonscan URL
        blockchain_block_number BIGINT NOT NULL,
        blockchain_timestamp TIMESTAMPTZ NOT NULL,
        votes_for INT NOT NULL,
        votes_against INT NOT NULL,
        votes_abstain INT NOT NULL,
        tally_hash VARCHAR(66) NOT NULL,
        syndic_id UUID NOT NULL REFERENCES users(id),
        syndic_signature VARCHAR(500),
        status VARCHAR(50),    -- 'Pending', 'Confirmed', 'Failed'
        created_at TIMESTAMPTZ,
        updated_at TIMESTAMPTZ,
        FOREIGN KEY (resolution_id) REFERENCES resolutions(id)
    );

    CREATE INDEX idx_blockchain_anchors_resolution
        ON blockchain_vote_anchors(resolution_id);
    CREATE INDEX idx_blockchain_anchors_meeting
        ON blockchain_vote_anchors(meeting_id);
    CREATE INDEX idx_blockchain_anchors_tx_hash
        ON blockchain_vote_anchors(blockchain_tx_hash);

Database Updates to Resolution
-------------------------------

.. code-block:: sql

    ALTER TABLE resolutions
        ADD COLUMN blockchain_tx_hash VARCHAR(66),
        ADD COLUMN blockchain_block_number BIGINT,
        ADD COLUMN blockchain_anchored_at TIMESTAMPTZ;

    CREATE INDEX idx_resolutions_blockchain_tx
        ON resolutions(blockchain_tx_hash)
        WHERE blockchain_tx_hash IS NOT NULL;

Risks & Mitigations
====================

+---------------------+----------------------+-------------------------+
| Risk                | Impact               | Mitigation              |
+=====================+======================+=========================+
| Private key theft   | Attacker anchors     | HSM (Hardware Security  |
| (syndic key)        | false vote tallies   | Module) for key storage |
|                     |                      | Multi-signature approval|
+---------------------+----------------------+-------------------------+
| Polygon network     | Anchoring fails,     | Fallback to alternative |
| outage              | meeting resolution   | chain (Optimism)        |
|                     | delayed              | Or local archive pending|
|                     |                      | retry                   |
+---------------------+----------------------+-------------------------+
| Gas price spike     | High transaction     | Anchor only final tally |
|                     | cost (rare)          | (minimal data, <$0.50)  |
|                     |                      | Budget reserve per org  |
+---------------------+----------------------+-------------------------+
| Regulatory change   | Law doesn't recognize| Maintain dual storage:  |
| (blockchain law)    | blockchain proof     | blockchain + notarized  |
|                     |                      | PDF (insurance)         |
+---------------------+----------------------+-------------------------+
| User adoption       | Low anchoring rate   | Start optional, educate |
|                     |                      | syndics, show benefits  |
+---------------------+----------------------+-------------------------+

Implementation Timeline
======================

**Phase 0 (March-April 2026): Research & Design** ✓
  * Architecture finalized
  * Smart contract drafted
  * Legal compliance verified with Belgian lawyer

**Phase 1 (May-June 2026): Smart Contract Development** (3 weeks)
  * Finalize VoteAnchor.sol
  * Deploy to Polygon testnet (Mumbai)
  * Audit by independent security firm
  * Create Polygon contract admin wallet

**Phase 2 (July 2026): Backend Integration** (4 weeks)
  * Implement Polygon client library (web3.rs)
  * BlockchainVoteUseCases integration
  * Signing service (ECDSA with syndic keys)
  * Database schema updates

**Phase 3 (August 2026): Testing** (2 weeks)
  * End-to-end test on testnet
  * Security audit
  * Regulatory approval letter

**Phase 4 (September 2026): Production Deployment**
  * Deploy smart contract to Polygon mainnet
  * Enable in UI for beta organizations
  * Monitor transaction success rate

**Phase 5 (October 2026+): Rollout & Adoption**
  * Gradual rollout to all organizations
  * Education materials for syndics
  * Public verification interface marketing

Timeline: 6 months from approval to production

Cost Analysis
=============

**One-time costs**:

* Smart contract development: €5,000-10,000
* Security audit: €5,000-8,000
* Legal consultation: €2,000-3,000
* **Subtotal**: €12,000-21,000

**Per-transaction costs** (Polygon mainnet):

* ~€0.05-0.20 per vote anchoring (variable gas price)
* For 1,000 organizations × 1 AG per year: €50-200/year

**Infrastructure**:

* Polygon RPC node (Alchemy): €50-200/month
* HSM for key storage: €2,000 one-time

**Total**: ~€30,000-35,000 first year

Success Criteria
================

* **Adoption**: 50%+ of syndics opt-in to blockchain anchoring by EOY 2026
* **Reliability**: 99%+ successful blockchain confirmations
* **Cost**: Transaction cost < €0.20 per meeting
* **Legal**: Zero disputes where blockchain proof was challenged
* **Performance**: Anchoring latency < 5 minutes (vs. paper audit = days)
