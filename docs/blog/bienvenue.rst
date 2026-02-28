.. post:: 2026-02-28
   :tags: annonce, communaute, conformite
   :category: Annonces
   :author: KoproGo Team

Bienvenue sur le blog KoproGo — Premieres fondations
======================================================

Nous inaugurons aujourd'hui le blog officiel du projet KoproGo, en meme temps
qu'une etape importante : la publication de notre **audit de conformite juridique**.

Ou en est KoproGo ?
---------------------

KoproGo est en **version 0.1.0** — les fondations techniques. Le projet comprend :

- **350+ endpoints REST API** couvrant 53 entites de domaine
- **Architecture hexagonale** (Ports & Adapters) avec Domain-Driven Design en Rust
- **Frontend** Astro + Svelte avec 80 pages
- **Comptabilite belge (PCMN)** : ~90 comptes pre-seedes, partie double, rapports financiers
- **RGPD** : Articles 15-21 implementes (acces, rectification, effacement, limitation, opposition)
- **Infrastructure** : LUKS, backups chiffres, Suricata IDS, fail2ban, 2FA TOTP

Pourquoi un audit de conformite ?
-----------------------------------

Avant de pretendre etre pret pour la production, nous avons confronte notre code
aux **textes de loi officiels belges** : Code Civil (Art. 3.84-3.94), AR 12/07/2012
(PCMN), et RGPD.

Resultat : **score global de 65%**. Des erreurs factuelles ont ete trouvees et corrigees :

1. **Delai de convocation** : le code utilisait 8 jours pour les AG extraordinaires.
   La loi (Art. 3.87 §3) impose **15 jours pour tous les types**.
2. **Taux d'interet de retard** : le code utilisait 8%. Le taux legal civil 2026
   est de **4.5%** (Moniteur belge).
3. **Etat date** : le seuil de retard etait de 10 jours. L'Art. 3.94 impose **15 jours**.
4. **3 devis obligatoires** : presente comme une obligation legale. C'est en fait
   une **bonne pratique professionnelle** — aucun article de loi ne l'impose.

Des lacunes critiques restent a corriger avant la mise en production (quorum AG,
limitation des procurations, lien agenda-resolutions). Le plan de remediation
est detaille dans la `section juridique de la documentation <../legal/index.html>`_.

Transparence sur les tests
----------------------------

Nous avons ecrit **752 scenarios BDD** (Gherkin) et **7 specs Playwright** pour
couvrir l'ensemble des fonctionnalites. Cependant, ces tests n'ont **pas tous ete
valides en environnement d'integration complet**. Des corrections seront publiees
au fur et a mesure des executions CI.

La montee en version (au-dela de 0.1.0) n'interviendra qu'apres que la conformite
juridique belge aura ete verifiee et validee.

La suite
---------

Ce blog suivra l'evolution du projet :

- **Annonces** : nouvelles fonctionnalites, jalons atteints
- **Articles techniques** : architecture, retours d'experience
- **Conformite** : avancees vers la conformite juridique complete
- **Communaute** : contributions, evenements

**Liens utiles** :

- Code source : `GitHub <https://github.com/gilmry/koprogo>`_
- Documentation : `doc.koprogo.com <https://doc.koprogo.com>`_
- Conformite juridique : `Section legale <../legal/index.html>`_
- Discussions : `GitHub Discussions <https://github.com/gilmry/koprogo/discussions>`_
