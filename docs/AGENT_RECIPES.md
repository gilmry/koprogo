# Recettes d'agents

Livrable du point 1.8 de **#426**. Ce document dit ce que le dépôt offre pour
travailler avec un agent : les agents disponibles, les hooks qui les
contraignent, et les scripts d'appui.

Il existe parce que `.claude/README.md` décrivait un répertoire `templates/`
inexistant tout en taisant seize agents et huit hooks. Une configuration qu'on
ne peut pas trouver ne sert à personne.

---

## 1. Les huit hooks, et ce qu'ils empêchent vraiment

Un hook qui « avertit » et un hook qui « bloque » ne se remplacent pas. La
colonne compte autant que le nom.

| Hook | Moment | Bloque ? |
|---|---|---|
| `pretool-deny-prod-action.sh` | avant l'outil | **oui** — commandes Bash dangereuses |
| `pretool-deny-secret-write.sh` | avant l'outil | **oui** — écriture dans un fichier porteur de secret |
| `pretool-warn-sensitive-edit.sh` | avant l'outil | non — avertit sur une modification sensible |
| `posttool-format.sh` | après l'outil | non — formate le fichier édité |
| `posttool-warn-unwrap.sh` | après l'outil | non — signale un `unwrap` introduit |
| `stop-leak-scan.sh` | à l'arrêt | **oui** — secrets dans les changements en attente |
| `session-start.sh` | au démarrage | non — vérifie les dépendances, alerte sur branche protégée |
| `userprompt-inject-rules.sh` | à chaque message | non — injecte les règles critiques dans le contexte |

**Trois bloquent, cinq informent.** Confondre les deux catégories, c'est croire
protégé ce qui est seulement commenté — le défaut le plus fréquent de ce
dépôt, retrouvé six fois le 2026-09-07 dans les gardes de CI.

## 2. Les seize agents

`.claude/agents/` contient huit agents, chacun avec son fichier de mémoire :

| Agent | Domaine |
|---|---|
| `rust-expert` | backend, domaine, hexagonal |
| `astro-svelte-expert` | frontend, Svelte 5, Astro |
| `code-reviewer` | revue |
| `devops-engineer` | CI, conteneurs |
| `platform-engineer` | plateforme |
| `sre-platform` | exploitation, incidents |
| `csi-analyst` | amélioration continue |
| `support-agent` | support utilisateur |

Le fichier `*.memory.md` de chacun conserve ce qu'il a appris. Le lire avant de
lancer l'agent évite de refaire une enquête déjà menée.

## 3. Les scripts d'appui

`.claude/scripts/` — sept scripts, dont le rôle n'est pas devinable au nom :

- `sync-playwright-videos.sh` et `copy-videos.sh` — récupèrent les
  enregistrements de la documentation vivante ;
- `generate-video-rst.py` — en produit les pages ;
- `slow-down-tests.sh` / `restore-test-speed.sh` — ralentissent les parcours
  pour que les vidéos soient lisibles par un humain, puis rétablissent ;
- `sync-docs-structure.sh` — aligne l'arborescence documentaire.

## 4. Les gardes du dépôt, et ce qu'elles gardent

Ce sont des tests, pas des hooks : ils s'exécutent en CI et échouent.

| Garde | Ce qu'elle refuse |
|---|---|
| `garde-data-testid` | la disparition d'un identifiant de test (#802) |
| `garde-couverture-testid` | l'aggravation de la dette d'ancrage (#803) |
| `garde-messages-en-dur` | un message de toast non traduit (#833) |
| `garde-libelles-en-dur` | l'aggravation des libellés non traduits (#834) |
| `garde-roles` | un rôle serveur sans menu ni registre (#814) |
| `garde-roles-non-travestis` | un rôle remplacé par un autre (#836) |
| `garde-classes-tailwind` | une classe assemblée à l'exécution (#788) |
| `garde-secrets-console` | un secret écrit dans la console |
| `garde_ecriture`, `garde_lecture` | l'aggravation de la dette d'identité (#772) |
| `garde_perimetre_wbs` | une issue du périmètre absente du WBS |
| `garde_classement_par_souschaines` | l'aggravation du tri d'erreurs par sous-chaîne (#762) |
| `check-no-f64-money.sh` | un montant en `f64`, dans `src/` **et** dans `tests/` (#443) |

**La plupart sont des cliquets** : ils bornent une dette sans exiger sa
disparition immédiate. Celui qui touche un écran le laisse mieux ancré et mieux
traduit qu'il ne l'a trouvé.

## 5. La règle qui vaut plus que les autres

> Jouer les suites, adapter les spécifications, ajouter les tests manquants
> **dans le même commit que le changement** — pas en passe de nettoyage après
> coup.

Et sa conséquence directe : **vérifier une garde neuve par témoin.** On
réintroduit le défaut, on s'assure que le test échoue, on le retire. Sans ce
geste, on ne sait pas si la garde garde.

Six gardes ont été trouvées inopérantes le 2026-09-07 — l'une affichait sans
bloquer, une autre comptait la mauvaise porte, une troisième éprouvait un
chemin que la production n'emprunte pas, une quatrième regardait le mauvais
répertoire. Aucune n'avait été vérifiée par témoin.
