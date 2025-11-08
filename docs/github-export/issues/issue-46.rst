=======================================================================
Issue #46: feat: Implement meeting voting system (Résolutions & votes)
=======================================================================

:State: **OPEN**
:Milestone: Phase 2: K3s + Automation
:Labels: phase:vps,track:software priority:high
:Assignees: Unassigned
:Created: 2025-10-27
:Updated: 2025-11-08
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/46>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Context
   
   **Meeting management:** ✅ **80% implemented**
   - Meeting entity (AGO/AGE types, statuses, agenda)
   - 8 API endpoints (create, update, complete, cancel, etc.)
   - Frontend meeting list and management
   
   **Missing:** ❌ **Voting/Resolution system**
   - Votes (Pour/Contre/Abstention)
   - Résolutions with vote tracking
   - Quorum calculation
   - Majority rules (simple, absolute, qualified)
   - Vote results and minutes generation
   
   Belgian copropriété law requires tracking:
   - Resolutions discussed
   - Vote results per resolution
   - Owner attendance and voting power (tantièmes/millièmes)
   - Quorum validation
   
   ## Objective
   
   Implement complete voting system for general assemblies (AG).
   
   ## Domain Model
   
   ### New Entities
   
   **1. Resolution** (`backend/src/domain/entities/resolution.rs`)
   
   ```rust
   pub struct Resolution {
       pub id: Uuid,
       pub meeting_id: Uuid,
       pub title: String,
       pub description: String,
       pub resolution_type: ResolutionType,  // Ordinary, Extraordinary
       pub majority_required: MajorityType,  // Simple, Absolute, Qualified(f64)
       pub vote_count_pour: i32,
       pub vote_count_contre: i32,
       pub vote_count_abstention: i32,
       pub total_voting_power_pour: f64,      // Sum of tantièmes "pour"
       pub total_voting_power_contre: f64,
       pub total_voting_power_abstention: f64,
       pub status: ResolutionStatus,          // Pending, Adopted, Rejected
       pub created_at: DateTime<Utc>,
       pub voted_at: Option<DateTime<Utc>>,
   }
   
   pub enum ResolutionType {
       Ordinary,       // Majority of votes present
       Extraordinary,  // Qualified majority (e.g., 2/3, 3/4)
   }
   
   pub enum MajorityType {
       Simple,            // 50% + 1 of votes cast
       Absolute,          // 50% + 1 of all votes (including absent)
       Qualified(f64),    // Custom threshold (e.g., 0.67 for 2/3)
   }
   
   pub enum ResolutionStatus {
       Pending,     // Not yet voted
       Adopted,     // Vote passed
       Rejected,    // Vote failed
   }
   ```
   
   **2. Vote** (`backend/src/domain/entities/vote.rs`)
   
   ```rust
   pub struct Vote {
       pub id: Uuid,
       pub resolution_id: Uuid,
       pub owner_id: Uuid,
       pub unit_id: Uuid,
       pub vote_choice: VoteChoice,
       pub voting_power: f64,    // Tantièmes/millièmes for this unit
       pub voted_at: DateTime<Utc>,
       pub proxy_owner_id: Option<Uuid>,  // If voting by proxy
   }
   
   pub enum VoteChoice {
       Pour,          // For
       Contre,        // Against
       Abstention,    // Abstain
   }
   ```
   
   ### Business Rules
   
   **Quorum validation:**
   ```rust
   impl Meeting {
       pub fn has_quorum(&self, total_units: i32, present_units: i32) -> bool {
           let attendance_rate = present_units as f64 / total_units as f64;
           match self.meeting_type {
               MeetingType::Ordinary => attendance_rate >= 0.5,      // 50%
               MeetingType::Extraordinary => attendance_rate >= 0.67, // 2/3
           }
       }
   }
   ```
   
   **Resolution validation:**
   ```rust
   impl Resolution {
       pub fn calculate_result(&self, total_voting_power: f64) -> ResolutionStatus {
           match self.majority_required {
               MajorityType::Simple => {
                   // Majority of votes cast
                   if self.vote_count_pour > self.vote_count_contre + self.vote_count_abstention {
                       ResolutionStatus::Adopted
                   } else {
                       ResolutionStatus::Rejected
                   }
               }
               MajorityType::Absolute => {
                   // Majority of all possible votes
                   if self.total_voting_power_pour > total_voting_power / 2.0 {
                       ResolutionStatus::Adopted
                   } else {
                       ResolutionStatus::Rejected
                   }
               }
               MajorityType::Qualified(threshold) => {
                   // Custom threshold (e.g., 2/3)
                   let pour_ratio = self.total_voting_power_pour / total_voting_power;
                   if pour_ratio >= threshold {
                       ResolutionStatus::Adopted
                   } else {
                       ResolutionStatus::Rejected
                   }
               }
           }
       }
   }
   ```
   
   ## Database Schema
   
   **New tables:**
   
   ```sql
   -- Resolutions
   CREATE TABLE resolutions (
       id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       meeting_id UUID NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
       title VARCHAR(255) NOT NULL,
       description TEXT,
       resolution_type VARCHAR(50) NOT NULL, -- 'Ordinary', 'Extraordinary'
       majority_required VARCHAR(50) NOT NULL, -- 'Simple', 'Absolute', 'Qualified:0.67'
       vote_count_pour INT DEFAULT 0,
       vote_count_contre INT DEFAULT 0,
       vote_count_abstention INT DEFAULT 0,
       total_voting_power_pour DECIMAL(10,4) DEFAULT 0,
       total_voting_power_contre DECIMAL(10,4) DEFAULT 0,
       total_voting_power_abstention DECIMAL(10,4) DEFAULT 0,
       status VARCHAR(50) DEFAULT 'Pending',
       created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
       voted_at TIMESTAMP
   );
   
   -- Votes
   CREATE TABLE votes (
       id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       resolution_id UUID NOT NULL REFERENCES resolutions(id) ON DELETE CASCADE,
       owner_id UUID NOT NULL REFERENCES owners(id),
       unit_id UUID NOT NULL REFERENCES units(id),
       vote_choice VARCHAR(50) NOT NULL, -- 'Pour', 'Contre', 'Abstention'
       voting_power DECIMAL(10,4) NOT NULL,
       proxy_owner_id UUID REFERENCES owners(id),
       voted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
       UNIQUE(resolution_id, unit_id)  -- One vote per unit per resolution
   );
   
   CREATE INDEX idx_resolutions_meeting ON resolutions(meeting_id);
   CREATE INDEX idx_votes_resolution ON votes(resolution_id);
   CREATE INDEX idx_votes_owner ON votes(owner_id);
   ```
   
   ## API Endpoints (New)
   
   ### Resolutions
   
   - `POST /api/v1/meetings/:id/resolutions` - Create resolution
   - `GET /api/v1/meetings/:id/resolutions` - List meeting resolutions
   - `GET /api/v1/resolutions/:id` - Get resolution details
   - `PUT /api/v1/resolutions/:id` - Update resolution
   - `DELETE /api/v1/resolutions/:id` - Delete resolution
   
   ### Votes
   
   - `POST /api/v1/resolutions/:id/vote` - Cast vote
   - `GET /api/v1/resolutions/:id/votes` - Get all votes for resolution
   - `PUT /api/v1/resolutions/:id/close` - Close voting & calculate result
   - `GET /api/v1/meetings/:id/vote-summary` - Get all resolution results for meeting
   
   ## Frontend Components
   
   ### 1. ResolutionList.svelte
   
   Display resolutions for a meeting with vote counts:
   ```svelte
   <script lang="ts">
     export let meetingId: string;
     
     let resolutions: Resolution[] = [];
     
     async function loadResolutions() {
       const response = await fetch(`/api/v1/meetings/${meetingId}/resolutions`);
       resolutions = await response.json();
     }
   </script>
   
   <div class="resolutions">
     {#each resolutions as resolution}
       <div class="resolution-card">
         <h3>{resolution.title}</h3>
         <p>{resolution.description}</p>
         
         <div class="vote-summary">
           <span class="pour">Pour: {resolution.vote_count_pour} ({resolution.total_voting_power_pour}%)</span>
           <span class="contre">Contre: {resolution.vote_count_contre}</span>
           <span class="abstention">Abstention: {resolution.vote_count_abstention}</span>
         </div>
         
         <div class="status">
           {#if resolution.status === 'Adopted'}
             <span class="badge success">✓ Adoptée</span>
           {:else if resolution.status === 'Rejected'}
             <span class="badge danger">✗ Rejetée</span>
           {:else}
             <span class="badge warning">En attente</span>
           {/if}
         </div>
       </div>
     {/each}
   </div>
   ```
   
   ### 2. VotingModal.svelte
   
   Modal for casting votes:
   ```svelte
   <script lang="ts">
     export let resolution: Resolution;
     export let ownerUnits: Unit[];
     
     let selectedChoice: VoteChoice = 'Pour';
     
     async function castVote(unitId: string) {
       await fetch(`/api/v1/resolutions/${resolution.id}/vote`, {
         method: 'POST',
         body: JSON.stringify({
           unit_id: unitId,
           vote_choice: selectedChoice
         })
       });
     }
   </script>
   
   <Modal>
     <h2>Vote: {resolution.title}</h2>
     
     {#each ownerUnits as unit}
       <div class="vote-form">
         <span>Lot {unit.unit_number} ({unit.quota} millièmes)</span>
         
         <div class="vote-buttons">
           <button on:click={() => castVote(unit.id, 'Pour')}>Pour</button>
           <button on:click={() => castVote(unit.id, 'Contre')}>Contre</button>
           <button on:click={() => castVote(unit.id, 'Abstention')}>Abstention</button>
         </div>
       </div>
     {/each}
   </Modal>
   ```
   
   ### 3. VoteResults.svelte
   
   Display final vote results with charts:
   ```svelte
   <script lang="ts">
     export let resolution: Resolution;
     
     const total = resolution.vote_count_pour + resolution.vote_count_contre + resolution.vote_count_abstention;
     const pourPercent = (resolution.vote_count_pour / total) * 100;
     const contrePercent = (resolution.vote_count_contre / total) * 100;
   </script>
   
   <div class="vote-results">
     <div class="progress-bar">
       <div class="pour" style="width: {pourPercent}%">{pourPercent.toFixed(1)}%</div>
       <div class="contre" style="width: {contrePercent}%">{contrePercent.toFixed(1)}%</div>
     </div>
     
     <table>
       <tr><td>Pour</td><td>{resolution.vote_count_pour}</td><td>{resolution.total_voting_power_pour} millièmes</td></tr>
       <tr><td>Contre</td><td>{resolution.vote_count_contre}</td><td>{resolution.total_voting_power_contre} millièmes</td></tr>
       <tr><td>Abstention</td><td>{resolution.vote_count_abstention}</td><td>{resolution.total_voting_power_abstention} millièmes</td></tr>
     </table>
   </div>
   ```
   
   ## PDF Generation Integration
   
   Update meeting minutes PDF to include vote results:
   ```rust
   // In pcn_exporter.rs or new meeting_minutes_exporter.rs
   pub fn generate_meeting_minutes_pdf(meeting: &Meeting, resolutions: Vec<Resolution>) -> Vec<u8> {
       // Add resolution results to PDF
       for resolution in resolutions {
           doc.add_page(...);
           doc.add_text(format!("Résolution: {}", resolution.title));
           doc.add_text(format!("Pour: {} votes ({} millièmes)", 
                               resolution.vote_count_pour, 
                               resolution.total_voting_power_pour));
           doc.add_text(format!("Résultat: {}", resolution.status));
       }
   }
   ```
   
   ## Testing
   
   - [ ] Create resolution
   - [ ] Cast vote (Pour/Contre/Abstention)
   - [ ] Calculate quorum
   - [ ] Calculate vote results (Simple/Absolute/Qualified majority)
   - [ ] Close voting and finalize status
   - [ ] Generate PDF with vote results
   - [ ] Proxy voting
   - [ ] Prevent duplicate votes per unit
   
   ## Acceptance Criteria
   
   - [ ] Resolution and Vote entities implemented
   - [ ] Database migrations complete
   - [ ] 8 new API endpoints functional
   - [ ] Quorum calculation correct
   - [ ] Majority calculation (3 types) correct
   - [ ] Frontend voting UI complete
   - [ ] Vote results display complete
   - [ ] PDF generation includes vote results
   - [ ] Tests passing
   - [ ] Documentation updated
   
   ## Effort Estimate
   
   **Medium** (2-3 days)
   - Day 1: Domain entities + database + repositories
   - Day 2: Use cases + API endpoints + business logic
   - Day 3: Frontend components + PDF integration + testing
   
   ## Related
   
   - Enhances: Meeting management (Issue in roadmap)
   - Supports: PDF generation (procès-verbaux)
   - Complies with: Belgian copropriété law
   
   ## References
   
   - Belgian copropriété law: https://www.notaire.be/
   - Voting systems: https://fr.wikipedia.org/wiki/Syst%C3%A8me_de_vote

.. raw:: html

   </div>

