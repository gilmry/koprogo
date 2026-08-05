========================================================================================
Issue #353: R&D: Crowdlending ACP — financement participatif pour travaux de copropriété
========================================================================================

:State: **OPEN**
:Milestone: Jalon 4: Automation & Intégrations 📅
:Labels: enhancement,R&D
:Assignees: Unassigned
:Created: 2026-03-28
:Updated: 2026-03-28
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/353>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Concept
   
   Permettre a une ACP (personne morale, Art. 3.86 CC) d'emprunter pour financer des travaux via une plateforme de crowdlending regulee, avec possibilite pour les coproprietaires eux-memes de participer comme preteurs.
   
   ## Cas d'usage
   
   Jeanne (82 ans, pension 1.050 EUR) ne peut pas payer 2.000 EUR d'appel de fonds pour la facade. Au lieu d'un appel de fonds massif, l'ACP emprunte 200.000 EUR via crowdlending sur 7 ans. Les charges mensuelles augmentent legerement (remboursement lisse) au lieu d'un choc unique. Philippe (investisseur, 18%) peut preter a l'ACP a 5% — meilleur rendement que son livret, et ca preserve la valeur de ses appartements.
   
   ## Recherche effectuee
   
   ### Plateformes belges (FSMA)
   - **Ecco Nova** (ecconova.com) — Specialisee renovation energetique, la plus pertinente. Pas d'API publique, partenariat B2B necessaire.
   - **Look&Fin** (lookandfin.com) — Immobilier belge. Pas d'API publique.
   - **Beebonds** (beebonds.com) — Immobilier + energie. Pas d'API publique.
   
   ### Plateforme EU avec API
   - **October** (october.eu) — API documentee (October Connect). Prets 30k+ EUR, 3-9%, 6 mois a 7 ans.
   
   ### Cadre reglementaire
   - **ECSP** (EU 2020/1503) : Reglement europeen crowdfunding, max 5M EUR/projet/12 mois
   - **FSMA** : Autorite belge, registre des plateformes autorisees
   - **L'ACP peut emprunter** : Personne morale, decision en AG a majorite qualifiee (2/3 ou 4/5 selon les travaux)
   - **Conflit d'interet** : Coproprietaires preteurs doivent s'abstenir sur les votes lies au pret
   
   ### Programmes publics belges
   - Pret Vert Bruxellois (0-2%), Ecopack/Renopack (Wallonie), MijnVerbouwLening (Flandre)
   - Renowatt (agregation coproprietes en Wallonie)
   - Tiers-investissement : financement par economies d'energie
   
   ## Integration KoproGo
   
   1. KoproGo prepare le dossier (donnees financieres, decisions AG, audit energetique)
   2. Export structure (PDF/JSON) ou API vers plateforme partenaire
   3. Suivi du pret dans le module comptable (ecritures PCMN)
   4. Simulation d'impact sur les charges mensuelles avant vote AG
   5. Vote AG a majorite qualifiee pour autoriser l'emprunt
   
   ## Questions ouvertes
   
   - Partenariat Ecco Nova ou October ?
   - Attention reglementaire si KoproGo facilite l'acces au crowdlending (ECSP marketing rules)
   - Structure juridique : pret a l'ACP vs pret aux coproprietaires individuels
   - Garanties : les parties communes ne peuvent pas facilement etre hypothequees
   
   ## Timeline
   
   Jalon 4+ (pas avant production stable). R&D exploratoire pour le moment.

.. raw:: html

   </div>

