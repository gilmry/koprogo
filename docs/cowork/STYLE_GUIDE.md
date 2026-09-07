# Conventions de travail avec un agent

Livrable du point 1.8 de **#426**. Ces conventions ne sont pas inventées : elles
sont relevées dans le dépôt tel qu'il est, sur les commits, les tests et les
issues qui ont tenu.

---

## 1. La langue

**Le français**, partout : commentaires, messages de commit, corps d'issue,
messages d'erreur destinés à un humain, noms de tests métier.

Ce n'est pas une préférence esthétique. Le domaine est juridique et belge : un
test qui s'appelle `le_syndic_ne_peut_pas_etre_mandataire` se relit contre
l'Art. 3.87 § 7 ; `test_proxy_validation` ne se relit contre rien.

Les identifiants techniques restent dans la langue de leur écosystème —
`data-testid`, noms de champs d'API, variantes d'énumération sérialisées.

**Le piège mesuré** : les messages du domaine sont en français, ceux des
couches techniques en anglais, et un gestionnaire qui ne cherchait que
« not found » rendait **500** sur un « introuvable ». Voir
`classification_erreurs.rs` (#762).

## 2. Les messages de commit

Une ligne de titre à l'impératif, disant **ce que le commit change pour le
produit** — pas la catégorie du changement :

```
Rend le lot obligatoire dans le formulaire de quote-part (#832, #780)
Supprime le repli qui fabriquait un ACP inexistant (#761)
Fait décrire à .claude/README.md le dépôt qui existe (#426)
```

Le corps répond à trois questions, dans cet ordre :

1. **Qu'est-ce qui n'allait pas**, avec la mesure — pas « c'était cassé » mais
   « 52 fois `not found`, 6 fois `introuvable` » ;
2. **pourquoi personne ne l'avait vu** — c'est souvent le plus instructif ;
3. **ce que la correction ne fait pas**, quand elle est partielle.

Citer l'article de loi, l'ADR ou l'issue qui fonde une décision. Une règle dont
on ignore le fondement finit par céder devant une maquette qui semble plus
jolie.

## 3. Les commentaires

Ils disent **pourquoi**, jamais ce que le code fait déjà lire. Le format qui a
tenu :

```
── Ce que faisait ce code ──
── Pourquoi personne ne l'a vu ──
── Ce qu'il fait maintenant ──
```

Un commentaire qui pose une règle sans la vérifier est un vœu. Quand c'est
possible, le transformer en `assert!` — c'est ce qui a été fait pour
l'identifiant de test de #777, dont le commentaire demandait « un identifiant
qui ne contienne ni 400 ni 1000 » sans que rien ne le contrôle.

## 4. Les tests

**Un test qui ne peut pas échouer ne prouve rien.** Trois règles en découlent.

**Vérifier par témoin.** On réintroduit le défaut, on s'assure que le test
échoue, on le retire. Six gardes ont été trouvées inopérantes le 2026-09-07 ;
aucune n'avait subi cette épreuve.

**Écrire le cas négatif.** Un test qui vérifie un refus passerait aussi avec une
implémentation qui refuse toujours. Il en faut donc deux : celui qui refuse, et
celui qui accepte.

**Monter le composant.** Un contrôle de types trouve un champ conforme ; seul le
montage distingue « la donnée est servie » de « l'écran la montre ».

Les cliquets partent du chiffre **mesuré après une première baisse** : posé sans
baisse, un cliquet n'est qu'une constatation. Et le chiffre vient d'une
exécution, pas d'un `grep` — la constante de #762 valait 114 par estimation et
118 en réalité.

## 5. Les issues

Décrire, mesurer, distinguer :

- **ce qui est vérifié** de ce qui est supposé, explicitement ;
- **ce qui relève d'un arbitrage produit** de ce qui se corrige en codant ;
- **le critère de fin**, et s'il n'est pas rempli, laisser l'issue ouverte
  plutôt que de fermer sur une preuve d'une autre nature.

Corriger publiquement une affirmation fausse dès qu'elle est identifiée. Une
chronologie erronée a coûté une enquête entière sur #828.

## 6. Ce qu'il ne faut pas faire

- Fermer une issue sur une lecture du code plutôt que sur une exécution.
- Supprimer une assertion qui gêne. Si la règle change, elle change dans le
  code source de la règle, avec le test mis à jour et l'issue citée.
- Ajouter une ligne à une liste d'exceptions sans la justification que cette
  liste exige.
- Traiter un `continue-on-error` comme une garantie d'exécution : il protège un
  step contre son propre échec, il ne le rend pas atteignable.
