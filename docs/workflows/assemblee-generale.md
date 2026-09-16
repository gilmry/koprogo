---
workflow: "Le cycle de vie d'une assemblée générale"
statut: brouillon
date: 2026-09-16
version: "0.1"
superviseur: null
signature_humaine: null
issue_cadre: 810
issue_parentes: [780, 784]
videos: []
---

> Ce document décrit un **workflow**, pas un rôle : le parcours qui traverse
> syndic → copropriétaire → syndic sur le cycle de vie d'une assemblée
> générale (AG). Les parcours par rôle (capacité C8.1) vivent dans
> `docs/personas/` ; celui-ci vit à part parce qu'aucun des six documents de
> rôle ne peut montrer le **relais** — le moment où l'un attend l'autre, où
> une pièce change de main. `docs/personas/README.md` renvoyait justement ce
> tiroir à la story qui en aurait besoin : c'est #810.
>
> Ne jamais remplir `superviseur` ni `signature_humaine` en tant qu'agent —
> ce sont des champs Tier 1 (`.claude/rules/CRITICAL.md` §11), un humain les
> valide.

## Pourquoi ce document existe

C'est le parcours central du produit — et le seul qui ne va pas à son terme
aujourd'hui. Il porte le risque juridique du produit : trois défauts trouvés
en recette (décompte par têtes au lieu des tantièmes, plafonnement invisible
avant clôture, égalité transformée en majorité par arrondi) y étaient logés,
et chacun aurait fait consigner une décision annulable.

Un test par rôle ne le couvre pas. Il faut alterner les sessions — le
syndic convoque, le copropriétaire reçoit et vote, le syndic clôt — pour
prouver qu'une convocation arrive vraiment à son destinataire. C'est l'objet
du test e2e associé :
[`frontend/tests/e2e/scenarios/assemblee-generale-cycle-de-vie.scenario.ts`](../../frontend/tests/e2e/scenarios/assemblee-generale-cycle-de-vie.scenario.ts).

## Méthode

Chaque ligne du tableau cite l'article, le module du domaine qui le porte
(jamais le nombre recopié — cf.
[`backend/src/domain/copropriete/registre_legal.rs`](../../backend/src/domain/copropriete/registre_legal.rs),
le registre exécutable RFC-0002), le test qui l'atteste, et l'ancre
`data-testid` quand une UI existe. Une étape marquée 🔴 est nommée avec son
issue, jamais lissée — c'est la règle que ce document doit respecter en
premier : « il ne peut pas se taire ».

## Le fil, étape par étape

| # | Rôle | Action | Article | Statut | Module / test | `data-testid` |
|---|---|---|---|---|---|---|
| 1 | Syndic | Crée l'AG, à une date qui laisse le délai légal | Art. 3.87 §2/§3 | ✅ Fonctionnel | `meeting.rs::Meeting::new` + `delai_de_convocation.rs::evaluer` (`meeting_dto.rs:77-95`) — `delai_de_convocation::tests::happy_une_assemblee_dans_un_mois_est_tenable` | `input-meeting-date`, `meeting-date-delai-tenable` / `meeting-date-delai-trop-court` |
| 2 | Syndic | Établit l'ordre du jour | Art. 3.87 §2 (ordre du jour) | ⚠️ Fonctionnel, API seulement | `meeting_use_cases.rs::add_agenda_item` — `resolution_use_cases::tests::security_vote_refuse_sur_resolution_hors_ordre_du_jour` | `meeting-info-agenda` (lecture seule ; **aucun formulaire de création** — `MeetingCreateModal.svelte` et `MeetingDetail.svelte` n'exposent aucun champ agenda, confirmé par grep exhaustif du 2026-09-16 — aucune issue connue, à ouvrir) |
| 3 | Syndic | Convoque, au moins quinze jours avant | Art. 3.87 §3 | ✅ Fonctionnel | `convocation.rs::Convocation::new` (refuse si délai dépassé) — `delai_de_convocation::tests::edge_la_borne_des_quinze_jours_est_tenable` | `convocation-detail-btn-send`, `convocation-detail-legal-deadline` |
| 4 | Copropriétaire | Reçoit la convocation, la lit | Art. 3.87 §3 al. 3 | 🔴 Règle non appliquée — #784 | `envoi_convocation.rs::envoi_regulier` — `envoi_convocation::tests::security_un_courriel_sans_accord_rend_la_convocation_irreguliere` | `owner-quick-meetings` (l'AG est visible ; l'accord écrit préalable, lui, n'est jamais vérifié — détail ci-dessous) |
| 5 | Copropriétaire | Donne procuration s'il ne peut venir | Art. 3.87 §7 | ✅ Fonctionnel (fusionné avec l'étape 7 dans l'UI, voir note) | `procurations.rs::verifier_procurations`, appelé par `resolution_use_cases.rs:493` — `procurations::tests::negative_quatre_procurations_au_dessus_de_dix_pourcents_sont_refusees` | `vote-proxy-input` |
| 6 | Syndic | Constate le quorum à l'ouverture | Art. 3.87 §5 | ✅ Fonctionnel | `meeting.rs` (quorum double : têtes + quotités) — `ag_session::tests::test_quotas_alone_do_not_carry_the_quorum` | `quorum-panel`, `quorum-present-input`, `quorum-total-input`, `quorum-validate-btn`, `quorum-validated-badge` |
| 7 | Copropriétaire | Vote, ou son mandataire vote pour lui | Art. 3.87 §6 | ✅ Fonctionnel, avec réserve — #850 | `vote.rs::Vote::new`, `resolution_use_cases.rs::cast_vote` — **non couvert** : aucun test n'atteste que `voting_power` égale la quote-part du copropriétaire (`registre_legal.rs:255-262`) | `vote-btn-pour`, `vote-voting-power`, `vote-proxy-input`, `resolution-vote-submit-button` |
| 8 | Syndic | Clôt le vote ; le plafonnement s'applique | Art. 3.87 §7 al. 4 | ✅ Fonctionnel (corrigé recette 5, 2026-09-06) | `resolution_use_cases.rs::close_voting` (appelle `verifier_procurations`) | `vote-close-btn` |
| 9 | Système | Proclame le résultat selon la majorité requise | Art. 3.88 §1er | ✅ Fonctionnel, avec réserve | `resolution.rs::Resolution::calculate_result` (absolue/2-3/4-5/unanimité) — `resolution::tests::test_calculate_result_*`. **Réserve** : `majorites.rs` (garde-fou plus fin, unanimité forcée pour une modification de quotes-parts quel que soit le `majority_required` saisi) est orphelin — même motif que #784, aucune issue connue | badge `Adoptée`/`Rejetée` sur `resolution-item` |
| 10 | Syndic | Rédige le procès-verbal | Art. 3.87 §10 | 🔴 Verrou circulaire — aucune issue connue, détail ci-dessous | `signatures_pv.rs::pv_valablement_signe` (orphelin) ; `meeting_use_cases.rs::attach_minutes` (inatteignable) | aucune — confirmé absent par `docs/personas/syndic.md` (2026-09-13) |
| 11 | Syndic | Clôt l'assemblée | Art. 3.87 §4 | 🔴 Bloqué par le même verrou que l'étape 10 | `meeting.rs::assert_can_complete` — `MinutesDraftMissing` | `meeting-complete-btn` (visible, **désactivé**), `meeting-complete-button-disabled-reason` |
| 12 | Syndic | Transmet le PV au registre, sous trente jours | Art. 3.87 §12 | 🔴 Inatteignable (dépend de 10-11) | `consignation_pv.rs::echeance`/`EtatConsignation` (orphelin, seul `DELAI_JOURS` est lu, pour un rappel de tableau de bord) — `consignation_pv::tests::security_un_seul_destinataire_oublie_suffit_a_faire_defaut` | rappel `stats_repository_impl.rs:504-539` (« PV à transmettre sous N jours ») — pas d'action |

## Ce que la règle non appliquée doit devenir visible (étape 4, #784)

`domain/copropriete/envoi_convocation.rs` porte l'Art. 3.87 § 3 al. 3 —
l'accord individuel, explicite et écrit, préalable à l'envoi d'une
convocation par courriel — avec son test
`security_un_courriel_sans_accord_rend_la_convocation_irreguliere`. Grep
exhaustif du 2026-09-16 : **ce module n'est appelé par personne** hors
`mod.rs` (déclaration), ses propres tests, et `registre_legal.rs` (qui ne
fait que le CITER comme métadonnée de conformité, pas l'invoquer).

`convocation_use_cases.rs::send_convocation` (L165-266) envoie effectivement
les convocations — il boucle sur les destinataires, crée un
`ConvocationRecipient`, appelle `mark_email_sent()` — **sans jamais
construire de `ModeDenvoi` ni d'`AccordAutreMoyen`**. Il n'existe même pas de
champ « mode d'envoi » ou « accord du destinataire » sur `Convocation` ni
sur `ConvocationRecipient` : rien ne permettrait de brancher la règle sans
d'abord étendre ces deux entités.

**Verdict, tel que ce document doit le rendre visible plutôt que le taire** :
la règle est écrite, testée, et n'a aucun effet sur une convocation envoyée
aujourd'hui. Une convocation par courriel sans accord préalable est
irrégulière au sens de la loi, et rien dans le produit ne l'empêche ni ne le
signale. Suivi par #784.

## Le verrou circulaire des étapes 10-12 — constat nouveau, aucune issue trouvée

Ceci n'est **pas** l'un des trois verrous connus de #780 : c'est un constat
distinct, fait en préparant ce document, et vérifié directement dans le
code (pas dans un résumé d'outil) :

- `assert_can_complete` (`meeting.rs:264-325`) refuse de clôturer une AG tant
  que `checklist.minutes_draft_exists` est faux — cette checklist vient de
  `minutes_document_id IS NOT NULL`
  (`meeting_completion_checker_impl.rs:96`).
- Le seul point d'entrée qui écrit `minutes_document_id` est
  `set_minutes_sent` (`meeting.rs:478-486`), appelé par
  `meeting_use_cases.rs::attach_minutes` — et `set_minutes_sent` **exige que
  le statut soit déjà `Completed`** (`meeting.rs:479-481`).
- Donc : on ne peut pas clôturer sans PV, et on ne peut pas attacher de PV
  avant clôture. Grep exhaustif : aucune troisième voie n'écrit
  `minutes_document_id` (le lien générique `Document::link_to_meeting`
  renseigne `related_meeting_id`, un champ distinct, sans effet sur la
  checklist).
- Ce verrou **est actif dans la version qui tourne réellement** :
  `meeting_use_cases.rs:309-315` documente explicitement « Gate Art. 3.87
  §3-5 CC — branché en prod via `with_completion_checker` », et
  `main.rs:279-287` est le seul point qui appelle `with_completion_checker`.
- Il est **invisible en CI** parce que le harnais d'intégration Rust
  (`backend/tests/common`) ne câble jamais `with_completion_checker` : le
  test `test_attach_minutes_to_completed_meeting`
  (`backend/tests/e2e_second_convocation.rs:216-264`) complète une réunion
  puis attache un PV, et passe — parce qu'il emprunte le chemin **legacy**
  (`meeting.complete(attendees_count)`, sans passer par
  `assert_can_complete`), pas le chemin réellement servi par le serveur.
  C'est exactement ce que #810 est censé exposer : chaque brique testée
  séparément, la couture jamais rejouée.
- Côté UI, `MeetingDetail.svelte` le rend honnête : `meeting-complete-btn`
  est **désactivé** (pas juste refusé après clic) tant que la checklist a
  un invariant manquant, et affiche
  `meeting-complete-button-disabled-reason`. Le syndic voit qu'il ne peut
  pas conclure — il ne peut simplement rien faire pour en sortir.
- `docs/personas/syndic.md` (2026-09-13) l'avait déjà repéré indirectement :
  « La clôture de l'assemblée et la publication du procès-verbal ne sont pas
  incluses [...] : aucune ancre `data-testid` dédiée n'a été trouvée [...]
  pour cette étape précise. » Ce document explique maintenant pourquoi : il
  n'y a pas d'ancre parce qu'il n'y a rien à ancrer — l'action n'existe pas.

**Recommandation** : ouvrir une issue dédiée (ce constat n'a pas encore de
numéro). Le correctif le plus direct serait un `minutes_draft_exists` porté
par un document *brouillon* distinct du PV final signé — c'est-à-dire
séparer « un brouillon existe » (précondition de clôture) de « le PV final
est transmis » (`attach_minutes`, postcondition de clôture), qui sont
aujourd'hui la même colonne.

## #780 — état réel au 2026-09-16, à corriger dans la prochaine relecture de la story

La story de ce ticket (partie basse) affirme : « Deux verrous restent
(#780) : aucun avertissement à la création si la date rend la convocation
impossible, et le report qui, bien que fonctionnel, ne rattrape pas une
assemblée déjà trop proche. » Vérification dans le code au 2026-09-16 :

| Verrou (#780) | État réel |
|---|---|
| Destinataires de convocation non constituables par l'UI | **Levé** (#784) — le serveur les déduit des copropriétaires actifs de l'immeuble (`convocation_use_cases.rs:209-226`) |
| Aucun avertissement à la création si la date rend la convocation impossible | **Levé le 2026-09-06** — `delai_de_convocation.rs`, câblé dans `MeetingResponse::from` et visible dès la saisie (`meeting-date-delai-trop-court`/`tenable`) |
| Le report (reschedule) ne rattrape pas une AG déjà trop proche | **Toujours ouvert** — `Meeting::reschedule` (`meeting.rs:339-349`) et `meeting_use_cases.rs::reschedule_meeting` (L353-368) ne rappellent jamais `delai_de_convocation::evaluer` ; rien n'empêche ni n'avertit lors d'un report vers une date déjà trop proche. La seule revalidation a lieu *a posteriori*, au prochain `GET`, jamais pendant l'appel. |

Autrement dit : au moment où ce document est écrit, **un seul** des verrous
cités par la story reste ouvert (le report), pas deux — la levée du
deuxième (avertissement à la création) date du 2026-09-06 et n'a
apparemment pas encore été répercutée dans le texte de la story. Ce document
ne corrige pas la story ; il documente l'état réel du code, comme le
critère de fin l'exige, et signale l'écart pour que le PO tranche.

Le report n'est pas l'une des douze étapes du tableau ci-dessus (ce n'est
pas une étape du chemin nominal, c'est une reprise sur incident), donc il
n'y figure pas en ligne propre — mais il reste tracé ici parce que le
critère de fin de #810 demande explicitement de cerner son périmètre.

## Comptes de recette

Uniquement les personas fictifs de `docs/specs/00-personas-et-seed.rst`
(« Résidence du Parc Royal »), désignés par leur **nom**, jamais par une
adresse email recopiée ici. Aucune donnée réelle de copropriété — la vidéo
qui accompagne ce document est un document public.

- **François**, syndic — crée l'AG, convoque, constate le quorum, clôt le
  vote, tente de clôturer l'assemblée.
- **Alice**, copropriétaire (présidente du conseil de copropriété dans le
  seed) — reçoit/consulte l'AG, vote (avec procuration démontrée pour un
  second copropriétaire absent).

## Le test e2e multi-session

[`frontend/tests/e2e/scenarios/assemblee-generale-cycle-de-vie.scenario.ts`](../../frontend/tests/e2e/scenarios/assemblee-generale-cycle-de-vie.scenario.ts)
joue les étapes 1, 3, 4, 6, 7 (avec procuration), 8, 9 au navigateur, avec
cinq bascules de session (`humanLogin` syndic → owner → syndic → owner →
syndic — jamais un seul login pour tout, règle 9 de `.claude/rules/CRITICAL.md`).
L'étape 2 (ordre du jour) est amorcée par requête directe, commentée comme
telle : aucune UI n'existe pour la jouer autrement. Les étapes 10-12 sont
jouées jusqu'au constat du blocage : le test clique jusqu'à
`meeting-complete-btn`, l'observe désactivé, et affirme la présence de
`meeting-complete-button-disabled-reason` — la preuve, filmée, que le cycle
ne va pas à son terme, plutôt qu'un contournement pour que la vidéo soit
jolie.

La vidéo est produite par le projet Playwright `scenarios`
(`frontend/playwright.config.ts`, `video: { mode: "on" }`), sur les comptes
de recette ci-dessus, sans donnée réelle. Elle atterrit dans
`frontend/test-results/` ; sa publication vers `docs/_static/videos/` (via
`make docs-sync-videos`) est un geste humain, pas déclenché par ce document.
