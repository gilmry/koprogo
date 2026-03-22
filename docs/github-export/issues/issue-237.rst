=============================================================================================================
Issue #237: R&D: AG en visioconférence - Quorum, convocation et validité (Art. 3.87 §1er Code civil belge)
=============================================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: R&D
:Assignees: Unassigned
:Created: 2026-03-07
:Updated: 2026-03-07
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/237>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte légal
   
   Depuis la loi du 20 décembre 2020, l'article 3.87, §1er du Code civil belge prévoit que chaque propriétaire d'un lot participe, physiquement ou **si la convocation le prévoit**, à distance, à ses délibérations.
   
   ## Concepts clés à implémenter
   
   ### 1. Droit de convocation vs Quorum de validité
   
   **Deux notions distinctes à bien séparer dans le domaine :**
   
   - **Droit de convocation (seuil de 1/5)** : Un ou plusieurs copropriétaires possédant au moins 1/5ème (20%) des quotes-parts dans les parties communes peuvent convoquer une AG spéciale. Ce seuil suffit pour *convoquer*, mais pas pour *tenir* valablement une AG.
   
   - **Double quorum de validité** : Les délibérations sont valides si et seulement si :
     - 50% des copropriétaires sont présents ou représentés **ET**
     - Ces copropriétaires possèdent au moins 50% des quotes-parts dans les parties communes
     - C'est un **double quorum** : nombre de personnes + quotités
   
   ### 2. Visioconférence et quorum
   
   **Point clé** : La loi ne fait aucune distinction entre présence physique et présence à distance pour le calcul du quorum. Un copropriétaire en visio est considéré comme "présent" au même titre qu'un copropriétaire physiquement là.
   
   **Exigences techniques** :
   - La solution technique doit permettre le débat, l'interaction et le vote
   - L'audioconférence suffit (pas nécessairement vidéo)
   - La **convocation doit explicitement prévoir** la participation à distance (condition légale)
   
   ### 3. Mécanisme de la deuxième AG sans quorum
   
   Si le quorum n'est pas atteint à la première AG :
   - Une nouvelle AG doit être convoquée dans les **15 jours**
   - Cette seconde AG peut valablement siéger **sans tenir compte du double quorum**
   - Les décisions sont prises à la majorité prévue pour chaque type de résolution
   
   ## Impact sur les modules existants
   
   ### Module Convocations (Issue #88)
   - [ ] Ajouter champ `allows_remote_participation: bool` dans `Convocation`
   - [ ] Validation domaine : si `allows_remote_participation = true`, la convocation doit mentionner la solution technique
   - [ ] Support du lien de connexion visio dans le template email/PDF
   
   ### Module Resolutions/Voting (Issue #46)
   - [ ] Tracker le mode de présence par participant : `Physique | Visio | Audio | Procuration`
   - [ ] Le calcul du quorum doit inclure les participants distants
   - [ ] Distinction entre "quorum de présence" et "quorum de quotités"
   - [ ] Support du mode "2ème AG sans quorum"
   
   ### Module Meetings
   - [ ] Champ `is_second_convocation: bool` (2ème AG après quorum non atteint)
   - [ ] Si 2ème AG : bypass du double quorum
   - [ ] Délai de 15 jours max entre 1ère et 2ème AG (validation domaine)
   - [ ] Champ `remote_participation_url: Option<String>` (lien visio)
   
   ### Nouvelle entité `AttendanceRecord`
   ```rust
   struct AttendanceRecord {
       meeting_id: Uuid,
       owner_id: Uuid,
       attendance_mode: AttendanceMode, // Physical, Video, Audio, Proxy
       proxy_owner_id: Option<Uuid>,
       voting_power: f64, // millièmes
       connected_at: Option<DateTime>,
       disconnected_at: Option<DateTime>,
   }
   ```
   
   ### Calcul du quorum (domain service)
   ```rust
   fn check_quorum(meeting: &Meeting, attendees: &[AttendanceRecord]) -> QuorumResult {
       let total_owners = meeting.total_eligible_owners;
       let total_quotites = 1000; // millièmes
       
       let present_count = attendees.len(); // physique + visio + audio
       let present_quotites: f64 = attendees.iter().map(|a| a.voting_power).sum();
       
       if meeting.is_second_convocation {
           return QuorumResult::Valid; // Pas de quorum requis pour 2ème AG
       }
       
       QuorumResult {
           owners_present: present_count,
           owners_required: total_owners / 2 + 1,
           quotites_present: present_quotites,
           quotites_required: 500.0, // 50% des millièmes
           is_valid: present_count >= total_owners / 2 + 1 
                     && present_quotites >= 500.0,
       }
   }
   ```
   
   ## Références légales
   
   - **Art. 3.87, §1er Code civil belge** : Participation à distance aux délibérations
   - **Loi du 20 décembre 2020** : Introduction permanente de la participation à distance
   - **Art. 577-6 ancien Code civil** : Double quorum de validité
   - **Art. 577-6, §5** : Mécanisme de la 2ème AG sans quorum
   
   ## Priorité
   
   **Moyenne** - Fonctionnalité importante pour la conformité légale complète du module AG, mais les bases (convocations, votes, résolutions) sont déjà en place.
   
   ## Jalon cible
   
   **Jalon 2** (Conformité Légale Belge) - Complète le module AG existant
   
   ## Labels
   R&D, legal-compliance, meetings, belgian-law

.. raw:: html

   </div>

