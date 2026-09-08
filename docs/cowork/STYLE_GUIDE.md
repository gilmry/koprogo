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

## 3. Les ancres `data-testid`

La convention n'est pas inventée : elle est **relevée** sur les 964 ancres du
contrat figé (`data-testid.contrat.json`).

### La forme

```
<domaine>-<objet>-<rôle>
```

Trois segments dans 516 cas sur 964, deux dans 210, quatre dans 190. Casse
kebab stricte — minuscules, chiffres, tirets — dans 961 cas sur 964.

```
building-acp-select        notices-create-btn        login-email
gdpr-erase-confirm-modal   owner-units               ticket-status-filter
```

Les trois exceptions (`mandate-error-scopeId`, `mandate-error-validUntil`,
`tech-spec-create-error-requiredSignatures`) reprennent un nom de champ d'API
en casse chameau. C'est un choix défendable — l'ancre désigne l'erreur d'un
champ précis — mais il ne doit pas se répandre : la garde les liste nommément.

### Le rôle, en dernier segment

Les plus fréquents, relevés :

```
99 -button    81 -input    65 -list      63 -btn     39 -select
32 -row       26 -loading  26 -error     24 -form    16 -submit
16 -empty     13 -cancel   13 -filter    13 -detail
```

**`-button` et `-btn` sont deux orthographes du même rôle.** C'est une dette,
pas un choix : elle oblige quiconque cherche un bouton à essayer les deux.
`-button` l'emporte au nombre, et c'est lui qu'on écrit désormais. Un cliquet
borne `-btn` à 63 et l'empêche de croître.

### Ce qu'une ancre doit désigner

**Un écran, pas un composant.** Une ancre posée sur le composant qui porte le
bon NOM mais que l'écran testé ne monte pas fait paraître l'écran couvert : le
décompte est bon, le test échoue, et on cherche la panne du côté du rendu.
C'est ce qui a fait échouer le portique de caractérisation quarante fois de
suite (#832) — `owner-units` vivait sur la liste repliée côté syndic, pas sur
la page du copropriétaire.

**Le conteneur, pas la branche peuplée.** Une ancre placée dans un `{:else}`
ne mesure pas l'écran, elle mesure les données : `buildings-list` était à
l'intérieur de « il y a des immeubles », donc un syndic sans immeuble ne la
rendait jamais.

**Un nom par écran.** Une même ancre sur deux composants rend
`getByTestId` ambigu. Quinze doublons subsistent, bornés par
`garde-ancres-ambigues`, et certains sont légitimes — `loading-spinner`
désigne la même chose partout.

### Pourquoi ancrer plutôt que nommer

Une assertion peut exiger une **ancre**, une **valeur interpolée** ou une
**structure**. Jamais une **formulation**. Sur un produit traduit en quatre
langues, chercher un bouton par son libellé est un pari sur la langue résolue :
`getByRole("button", { name: "Générer le rapport" })` ne trouve rien dès que
l'écran rend « Generate report », le clic ne part pas, et c'est le
`waitForResponse` d'à côté qui expire — un symptôme qui ne dit rien de sa
cause.


## 4. Les commentaires

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

## 5. Les tests

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

## 6. Les issues

Décrire, mesurer, distinguer :

- **ce qui est vérifié** de ce qui est supposé, explicitement ;
- **ce qui relève d'un arbitrage produit** de ce qui se corrige en codant ;
- **le critère de fin**, et s'il n'est pas rempli, laisser l'issue ouverte
  plutôt que de fermer sur une preuve d'une autre nature.

Corriger publiquement une affirmation fausse dès qu'elle est identifiée. Une
chronologie erronée a coûté une enquête entière sur #828.

## 7. Ce qu'il ne faut pas faire

- Fermer une issue sur une lecture du code plutôt que sur une exécution.
- Supprimer une assertion qui gêne. Si la règle change, elle change dans le
  code source de la règle, avec le test mis à jour et l'issue citée.
- Ajouter une ligne à une liste d'exceptions sans la justification que cette
  liste exige.
- Traiter un `continue-on-error` comme une garantie d'exécution : il protège un
  step contre son propre échec, il ne le rend pas atteignable.
