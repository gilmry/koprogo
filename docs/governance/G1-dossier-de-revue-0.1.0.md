# G1 — dossier de revue pour la 0.1.0

**Préparé le 2026-09-04** pour la revue humaine exigée par
`RESPONSABILITE.md`. Ce document ne remplace pas la revue : il lui donne
de quoi porter sur des points précis plutôt que sur une session entière.

Base : `feature/dev` à `31ff3fc8`. Démo en ligne sur `sha-e583b5c`.

---

## Ce que je recommande de regarder, dans cet ordre

### 1. Les choix de droit — priorité haute

Un seul endroit du code traduit une règle juridique en décision de
calcul, et **deux de ses choix ne sont pas dictés par le texte**.

`backend/src/domain/copropriete/procurations.rs`, Art. 3.87 § 7 al. 4 :

- **La répartition de l'écart entre les sens de vote.** Un mandataire
  peut voter « pour » son lot et « contre » celui d'un mandant. Quand il
  est plafonné, la loi ne dit pas d'où l'écart se retranche. Retenu : la
  réduction proportionnelle. Les deux autres lectures — d'abord des
  « pour », d'abord des « contre » — feraient pencher le résultat sans
  fondement, mais ce raisonnement est le mien, pas celui du texte.
- **Le votant unique n'est pas ramené à zéro.** La somme des autres vaut
  alors zéro, et l'application littérale viderait la séance de sens.
  J'ai considéré que le quorum de l'Art. 3.87 § 5 traite ce cas. C'est
  une interprétation.

Le reste de l'article est étayé : le plafonnement plutôt que le refus
est confirmé par la doctrine belge, sources citées dans le RFC-0002.

**Question à trancher** : ces deux lectures vous engagent-elles ?

### 2. Le comportement visible — priorité haute

À voir sur la démo, avec Cowork, sur les données semées :

- Une résolution où un copropriétaire pèse plus que tous les autres.
  Le décompte affiché sera **réduit**, et le résultat peut s'inverser.
  C'est voulu. Est-ce compréhensible pour un syndic sans explication ?
- Le plafond s'applique **même à l'unanimité** : « 0,8 pour, 0 contre »
  là où l'on attendait « 1,0 pour ». Correct, mais déroutant. La trace
  est en base (`resolutions.voix_plafonnees`) ; aucun écran ne
  l'explique encore.
- Les quotes-parts affichent « — » plutôt que `NaN` quand elles
  manquent.

### 3. Ce qui est cassé et que je n'ai pas réparé — priorité haute

| Sujet | Issue | Pourquoi c'est resté |
|---|---|---|
| Identifiants superadmin déductibles du dépôt public | #763 | Le code accepte les variables d'environnement ; il reste à décider quand fermer la porte |
| Repli de `resolve_acp_id` qui fabrique un ACP inexistant | #761 | Cause de 25 échecs. Le supprimer demande de câbler 14 tests unitaires |
| Erreurs HTTP classées par sous-chaînes | #762 | Un message français ressortait en 500. Rustiné ; le fond demande une erreur typée |
| Aucune sauvegarde automatisée sur ecosolva | drill F3 | Le runbook en décrit une qui n'existe pas |
| 11 harnais BDD ne s'exécutent nulle part | #540 | Exigent le socket Docker ; `ci.yml` exclut la branche |

### 4. Ce que la session a produit — pour contexte, pas pour relecture

45 régressions résorbées, mesurées sur 62 binaires exécutés contre une
base réelle. Trajectoire : 72 échecs → 31, tous environnementaux.
Neuf commits déployés.

---

## Ce que je crois honnête de dire sur ma propre fiabilité

Cette revue n'est pas une formalité. Dans cette seule session :

- **J'ai créé les 45 régressions** que j'ai ensuite résorbées, en
  rendant obligatoire la résolution de l'ACP sans exécuter les harnais.
- **Mon outillage m'a menti deux fois.** L'étape `svelte-check` de ma CI
  locale testait `grep -q "COMPLETED"`, une chaîne toujours présente :
  elle n'a jamais rien vérifié et a masqué six erreurs de type, dont un
  formulaire qui corrompait la nature des lots. Et ma machine était plus
  permissive que la CI à cause d'un fichier généré que je possédais.
- **J'ai pris un silence pour un succès.** Treize binaires BDD étaient
  comptés « sans échec » alors qu'ils n'avaient jamais démarré, rejetés
  par un drapeau incompatible.
- **J'ai mis la démo à terre** en déployant des migrations dont le
  rollback ne pouvait pas revenir, sans avoir vérifié qu'une sauvegarde
  récente existait. Elle existait.

Chacune de ces erreurs a été trouvée par une vérification **écrite
autrement** que la première : le barrage CI contre ma CI locale, une
exécution réelle contre une compilation. Aucune ne l'a été par ma
vigilance seule.

C'est l'argument le plus solide en faveur de G1, et il vaut mieux qu'il
vienne de moi.

---

## Décisions attendues de la revue

1. **GO / NO-GO** sur le tag `v0.1.0`.
2. Les deux lectures de l'Art. 3.87 § 7 (§ 1 ci-dessus).
3. #763 : fermer la porte du superadmin avant ou après le tag.
4. Le sort des 40 enregistrements en quarantaine, si le déploiement se
   fait un jour sur données réelles.
5. Le barrage à 11 minutes : soutenable, ou à relâcher.
