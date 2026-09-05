# Drill F3 — restauration et retour arrière

**Joué le 2026-09-04**, sur le VPS ecosolva, contre une base jetable
(`drill_restore`). La production n'a pas été touchée.

F3 réclamait deux drills : rollback de déploiement, et restauration
GPG+S3. Les deux ont été joués. Aucun ne s'est passé comme le runbook le
décrit.

---

## Drill 1 — le rollback de déploiement, joué en conditions réelles

Non simulé : c'est l'incident du 2026-09-03.

Le déploiement de `sha-2770a5c` a échoué (backend *unhealthy*).
`deploy-ecosolva.sh` a fait ce qu'il devait et rebasculé sur
`sha-87d24ac`. **Le rollback a échoué aussi**, et la démo est restée
morte jusqu'à intervention manuelle.

**Cause.** La nouvelle image avait déjà appliqué une partie de ses
migrations. L'ancien binaire voyait `20260903000000` en base sans la
connaître et paniquait sur `VersionMissing`. La base était en avance sur
le binaire.

**Conclusion.** Un rollback qui ne raisonne qu'en images suppose que
l'image détermine l'état. Dès qu'une version migre, c'est faux. Le
script porte pourtant le commentaire « un `up` à moitié appliqué laisse
la production dans un état mixte, ce qui est pire qu'un arrêt franc » :
c'est ce qu'il a produit lui-même.

**Ce qui a débloqué** : destruction et reconstruction de la base, sur
autorisation explicite. 130 migrations rejouées depuis zéro, backend
sain immédiatement.

**Ce qui n'a pas été tenté, et aurait dû l'être** : restaurer une
sauvegarde. Il en existait une du 2026-08-31, à quatre jours. Le
présent drill montre qu'elle se restaure en 13 secondes. Le réflexe a
manqué, pas l'outil.

---

## Drill 2 — la restauration

### Ce que le runbook décrit, et ce qui existe

`docs/RUNBOOK_VPS_PRODUCTION.md` décrit des sauvegardes chiffrées GPG
synchronisées vers S3, par un cron du rôle `backup`.

Sur ecosolva, **rien de tout cela n'est déployé** :

| Élément attendu | État |
|---|---|
| Cron de sauvegarde | absent de `/etc/cron.d` |
| Clé GPG `backup` | aucune clé, ni utilisateur ni root |
| `s3cmd` | binaire absent, pas de `~/.s3cfg` |
| Fichiers `*.sql.gz.gpg` | aucun |

Ce qui existe : des dumps **manuels et ponctuels** dans
`/home/ubuntu/backups/`, non chiffrés, non répliqués hors machine. Le
plus récent datait du 2026-08-31, soit quatre jours avant l'incident.

**Le runbook décrit donc une procédure qui n'est pas en place.** C'est
plus dangereux qu'une absence de procédure : on croit être couvert.

### La restauration elle-même

Restauration de `koprogo-avant-purge-20260831T194258Z.sql.gz` dans une
base neuve :

- **13 secondes**, code de sortie 0, **zéro erreur**
- 95 tables, 114 migrations enregistrées
- 1247 ACP, 1250 immeubles, 11 371 lots, 1846 utilisateurs

La restauration d'un dump non chiffré fonctionne. Le chemin GPG+S3
n'a pas pu être testé : il n'existe pas.

---

## Drill 3 — remonter une sauvegarde à l'état courant

C'est le drill qui compte, et celui qui manquait le 2026-09-03 :
**une sauvegarde d'avant les migrations peut-elle rejoindre l'état
courant ?**

Les 17 migrations postérieures au dump ont été appliquées dans l'ordre.
**Quinze passent. Deux échouent** :

```
20260903020000_owner_contributions_appartiennent_a_lacp
  → Reprise impossible : 13 quote(s)-part(s) sans ACP créancière résoluble

20260903030000_journal_entries_appartiennent_a_lacp
  → Reprise impossible : 5 écriture(s) sans ACP résoluble
```

**C'est la cause de l'incident du 2026-09-03**, restée jusqu'ici
attribuée vaguement à « un conflit avec les données existantes ». Elle
est maintenant nommée.

### Ce que sont ces enregistrements

**Les 13 quotes-parts** ont toutes `unit_id` à `NULL`, et appartiennent
à des copropriétaires qui **ne possèdent aucun lot** — vérifié : zéro
résoluble par le propriétaire, zéro propriétaire multi-lots, treize
propriétaires sans aucun lot.

Une quote-part sans lot est dépourvue de sens juridique : l'Art. 3.86
§ 3 la fait naître de la propriété d'un lot, au prorata de ses
tantièmes. Aucun chemin légitime ne peut leur trouver une ACP.

**Les 9 écritures** (dont 5 irréductibles) n'ont ni dépense ni
quote-part de rattachement, sur 9 organisations distinctes, et portent
18 lignes d'écriture.

### Ce que ça implique

**Les migrations ont raison de refuser.** Elles échouent bruyamment
plutôt que d'affecter une ACP arbitraire. Écrire n'importe quelle ACP
sur une pièce comptable serait exactement le défaut que l'ADR-0045
corrige, en pire : une pièce attribuée à une copropriété qui n'y a
aucun droit.

**Mais aucun déploiement sur données existantes ne passera** tant que
ces 22 enregistrements n'auront pas été traités. La démo n'a fonctionné
que parce que la base a été vidée.

### Remédiation proposée

Ces enregistrements ne sont pas réparables : ils sont sans objet. Une
migration de nettoyage doit les retirer **avant** les deux migrations
bloquantes, en journalisant ce qu'elle supprime.

Ce nettoyage n'a **pas** été écrit ici : supprimer des écritures
comptables, fussent-elles orphelines, n'est pas un acte que l'IA pose
seule. C'est un Tier 1 au sens de `RESPONSABILITE.md`.

---

## Ce que ce drill change pour la 0.1.0

1. **F3 est joué**, et son résultat est négatif sur deux des trois
   volets. Le noter « fait » sans dire cela serait un mensonge.
2. **Le runbook doit être corrigé** : il décrit une procédure de
   sauvegarde absente de la machine.
3. **Une sauvegarde automatisée manque.** Quatre jours d'écart au
   moment de l'incident, sur une machine que la mémoire projet qualifie
   d'« expérimentale mais à traiter comme de la production ».
4. **Le rollback doit connaître les migrations**, ou le runbook doit
   dire franchement qu'un déploiement migrant est irréversible sans
   restauration.
5. **Les 22 enregistrements orphelins** bloquent tout déploiement sur
   données réelles.
