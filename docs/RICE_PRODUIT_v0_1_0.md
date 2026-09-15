# RICE produit — v0.1.0, à Reach et Impact substitués

*Généré par `scripts/rice-produit.py`. Ne pas éditer à la main.*

> **Deux facteurs sur quatre sont des substituts.** KoproGo n'a aucun
> utilisateur : la v0.1.0 n'est pas sortie. `Reach` est donc le nombre
> de chantiers qu'une issue débloque, et `Impact` son palier légal.
> Ce ne sont pas Reach et Impact au sens de RICE, et les appeler ainsi
> sans le dire reviendrait à habiller de l'intuition en arithmétique.
>
> À la bêta fermée, les signaux RACE existeront et ce script devra
> être **réécrit**, pas ajusté.

**Confiance** : 0 passe(s) réellement mesurée(s) au 
registre CSI. Tant que ce nombre est bas, le classement vaut comme
**ordre**, pas comme mesure.

**L'ADR 0049 tient** : tout reste dans la 0.1.0. Un score bas dit
« en dernier », jamais « hors release ».

## Là où RICE contredit MoSCoW

Ces lignes ne tranchent rien : elles demandent un arbitrage. Un
classement qui confirmerait MoSCoW partout n'apprendrait rien.

**Classées `Could`, et pourtant dans les quinze premières** —
elles débloquent beaucoup, ou portent un article :

- **#805** (rang 1, score 2.4) — débloque 11 chantiers, palier 3 : cite Art. 3.89
- **#585** (rang 5, score 0.7) — débloque 6 chantiers, palier 2 : touche « acp »
- **#587** (rang 9, score 0.4) — débloque 2 chantiers, palier 2 : touche « acp »
- **#816** (rang 10, score 0.4) — débloque 1 chantiers, palier 3 : cite Art. 3.90
- **#806** (rang 13, score 0.3) — débloque 1 chantiers, palier 3 : cite Art. 3.89
- **#810** (rang 14, score 0.3) — débloque 1 chantiers, palier 3 : cite Art. 3.87

**Classées `Must`, et dans la moitié basse** — à vérifier : un
`Must` qui ne débloque rien et ne porte aucun article est
peut-être un `Should` qui s'ignore :

- **#847** (rang 48, score 0.15) — cite Art. 3.89
- **#848** (rang 49, score 0.15) — cite Art. 3.87
- **#882** (rang 51, score 0.15) — cite Art. 3.89
- **#581** (rang 53, score 0.133) — touche « acp »
- **#696** (rang 57, score 0.133) — touche « périmètre »
- **#841** (rang 62, score 0.133) — touche « périmètre »
- **#842** (rang 63, score 0.133) — touche « rgpd »
- **#880** (rang 66, score 0.133) — aucun article, aucun marqueur d'irréversibilité
- … et 6 autres

## Le classement

| # | Score | Cap. | MoSCoW | Reach* | Impact* | Conf. | Effort | Pourquoi ce palier |
|---|---:|---|---|---:|---:|---:|---:|---|
| #805 | **2.4** | C8.1 | Could | 11 | 3 | 0.2 | 3 | cite Art. 3.89 |
| #803 | **1.2** | C5.2 | Must | 11 | 2 | 0.2 | 4 | touche « périmètre » |
| #802 | **1.0** | C5.2 | Must | 9 | 2 | 0.2 | 4 | touche « périmètre » |
| #780 | **0.9** | C1.1 | Must | 5 | 3 | 0.2 | 4 | cite Art. 3.87 |
| #585 | **0.7** | C3.3 | Could | 6 | 2 | 0.2 | 4 | touche « acp » |
| #872 | **0.6** | C7.1 | Must | 11 | 1 | 0.2 | 4 | aucun article, aucun marqueur d'irréversibilité |
| #797 | **0.5** | C5.1 | Should | 9 | 1 | 0.2 | 4 | aucun article, aucun marqueur d'irréversibilité |
| #579 | **0.45** | C1.2 | Should | 2 | 3 | 0.2 | 4 | étiquette legal-compliance, sans article cité |
| #587 | **0.4** | C3.2 | Could | 2 | 2 | 0.2 | 3 | touche « acp » |
| #816 | **0.4** | C8.1 | Could | 1 | 3 | 0.2 | 3 | cite Art. 3.90 |
| #835 | **0.4** | C4.4 | Should | 3 | 2 | 0.2 | 4 | touche « périmètre » |
| #576 | **0.3** | C1.1 | Must | 1 | 3 | 0.2 | 4 | cite Art. 3.87 |
| #806 | **0.3** | C8.1 | Could | 1 | 3 | 0.2 | 4 | cite Art. 3.89 |
| #810 | **0.3** | C8.2 | Could | 1 | 3 | 0.2 | 4 | cite Art. 3.87 |
| #850 | **0.3** | C1.1 | Must | 1 | 3 | 0.2 | 4 | cite Art. 3.87 |
| #855 | **0.3** | C1.5 | Should | 1 | 3 | 0.2 | 4 | cite Art. 3.94 |
| #873 | **0.3** | C7.3 | Must | 2 | 1 | 0.2 | 2 | aucun article, aucun marqueur d'irréversibilité |
| #881 | **0.3** | C1.3 | Must | 0 | 3 | 0.2 | 2 | cite Art. 3.89 |
| #586 | **0.267** | C3.3 | Could | 1 | 2 | 0.2 | 3 | touche « acp » |
| #808 | **0.267** | C8.1 | Could | 1 | 2 | 0.2 | 3 | touche « périmètre » |
| #809 | **0.267** | C8.1 | Could | 1 | 2 | 0.2 | 3 | touche « acp » |
| #811 | **0.267** | C8.2 | Could | 1 | 2 | 0.2 | 3 | touche « acp » |
| #812 | **0.267** | C8.2 | Could | 1 | 2 | 0.2 | 3 | touche « acp » |
| #815 | **0.267** | C8.1 | Could | 1 | 2 | 0.2 | 3 | touche « acp » |
| #865 | **0.267** | C6.1 | Should | 1 | 2 | 0.2 | 3 | touche « acp » |
| #582 | **0.2** | C1.4 | Could | 0 | 3 | 0.2 | 3 | cite Art. 3.87 |
| #589 | **0.2** | C3.2 | Could | 0 | 2 | 0.2 | 2 | touche « acp » |
| #595 | **0.2** | C8.4 | Could | 0 | 2 | 0.2 | 2 | touche « acp » |
| #694 | **0.2** | C4.2 | Must | 1 | 2 | 0.2 | 4 | touche « scoping » |
| #762 | **0.2** | C4.5 | Should | 1 | 2 | 0.2 | 4 | touche « acp » |
| #798 | **0.2** | C4.2 | Must | 1 | 2 | 0.2 | 4 | touche « périmètre » |
| #807 | **0.2** | C8.1 | Could | 1 | 2 | 0.2 | 4 | touche « acp » |
| #822 | **0.2** | C5.3 | Could | 0 | 3 | 0.2 | 3 | cite Art. 3.89 |
| #846 | **0.2** | C1.3 | Must | 0 | 3 | 0.2 | 3 | cite Art. 3.87 |
| #852 | **0.2** | C2.1 | Should | 0 | 2 | 0.2 | 2 | touche « acp » |
| #854 | **0.2** | C8.4 | Could | 0 | 3 | 0.2 | 3 | cite Art. 3.89 |
| #856 | **0.2** | C10.1 | Must | 0 | 2 | 0.2 | 2 | touche « acp » |
| #868 | **0.2** | C4.2 | Must | 0 | 2 | 0.2 | 2 | touche « migration » |
| #555 | **0.15** | C4.5 | Should | 0 | 3 | 0.2 | 4 | cite Art. 3.87 |
| #556 | **0.15** | C5.3 | Could | 0 | 3 | 0.2 | 4 | cite Art. 3.84 |
| #577 | **0.15** | C1.1 | Must | 0 | 3 | 0.2 | 4 | cite Art. 3.87 |
| #578 | **0.15** | C1.2 | Should | 0 | 3 | 0.2 | 4 | étiquette legal-compliance, sans article cité |
| #583 | **0.15** | C1.4 | Could | 0 | 3 | 0.2 | 4 | cite Art. 3.88 |
| #635 | **0.15** | C2.3 | Could | 0 | 3 | 0.2 | 4 | cite Art. 3.86 |
| #820 | **0.15** | C5.3 | Could | 0 | 3 | 0.2 | 4 | cite Art. 3.90 |
| #824 | **0.15** | C5.3 | Could | 0 | 3 | 0.2 | 4 | cite Art. 3.87 |
| #826 | **0.15** | C5.3 | Could | 0 | 3 | 0.2 | 4 | cite Art. 3.87 |
| #847 | **0.15** | C1.3 | Must | 0 | 3 | 0.2 | 4 | cite Art. 3.89 |
| #848 | **0.15** | C1.1 | Must | 0 | 3 | 0.2 | 4 | cite Art. 3.87 |
| #867 | **0.15** | C2.2 | Should | 0 | 3 | 0.2 | 4 | cite Art. 3.86 |
| #882 | **0.15** | C4.1 | Must | 0 | 3 | 0.2 | 4 | cite Art. 3.89 |
| #466 | **0.133** | C9.2 | Could | 0 | 2 | 0.2 | 3 | touche « identité » |
| #581 | **0.133** | C1.1 | Must | 0 | 2 | 0.2 | 3 | touche « acp » |
| #588 | **0.133** | C3.2 | Could | 0 | 2 | 0.2 | 3 | touche « acp » |
| #590 | **0.133** | C3.3 | Could | 0 | 2 | 0.2 | 3 | touche « acp » |
| #592 | **0.133** | C6.1 | Should | 0 | 2 | 0.2 | 3 | touche « acp » |
| #696 | **0.133** | C7.1 | Must | 0 | 2 | 0.2 | 3 | touche « périmètre » |
| #781 | **0.133** | C3.1 | Should | 0 | 2 | 0.2 | 3 | touche « acp » |
| #817 | **0.133** | C8.2 | Could | 1 | 1 | 0.2 | 3 | aucun article, aucun marqueur d'irréversibilité |
| #821 | **0.133** | C5.3 | Could | 0 | 2 | 0.2 | 3 | touche « acp » |
| #827 | **0.133** | C5.3 | Could | 0 | 2 | 0.2 | 3 | touche « périmètre » |
| #841 | **0.133** | C4.2 | Must | 0 | 2 | 0.2 | 3 | touche « périmètre » |
| #842 | **0.133** | C4.3 | Must | 0 | 2 | 0.2 | 3 | touche « rgpd » |
| #866 | **0.133** | C6.2 | Should | 0 | 2 | 0.2 | 3 | touche « acp » |
| #869 | **0.133** | C6.2 | Should | 1 | 1 | 0.2 | 3 | aucun article, aucun marqueur d'irréversibilité |
| #880 | **0.133** | C7.1 | Must | 1 | 1 | 0.2 | 3 | aucun article, aucun marqueur d'irréversibilité |
| #354 | **0.1** | C9.2 | Could | 1 | 1 | 0.2 | 4 | aucun article, aucun marqueur d'irréversibilité |
| #425 | **0.1** | C9.4 | Could | 0 | 2 | 0.2 | 4 | touche « migration » |
| #427 | **0.1** | C7.2 | Should | 0 | 2 | 0.2 | 4 | touche « périmètre » |
| #429 | **0.1** | C9.4 | Could | 0 | 2 | 0.2 | 4 | touche « migration » |
| #515 | **0.1** | C9.1 | Should | 1 | 1 | 0.2 | 4 | aucun article, aucun marqueur d'irréversibilité |
| #591 | **0.1** | C3.3 | Could | 0 | 2 | 0.2 | 4 | touche « acp » |
| #779 | **0.1** | C3.1 | Should | 0 | 2 | 0.2 | 4 | touche « périmètre » |
| #818 | **0.1** | C5.3 | Could | 0 | 2 | 0.2 | 4 | touche « périmètre » |
| #823 | **0.1** | C5.3 | Could | 0 | 2 | 0.2 | 4 | touche « périmètre » |
| #825 | **0.1** | C5.3 | Could | 0 | 2 | 0.2 | 4 | touche « périmètre » |
| #834 | **0.1** | C5.1 | Should | 0 | 2 | 0.2 | 4 | touche « acp » |
| #845 | **0.1** | C4.1 | Must | 0 | 2 | 0.2 | 4 | touche « identité » |
| #864 | **0.1** | C4.1 | Must | 0 | 2 | 0.2 | 4 | touche « identité » |
| #870 | **0.1** | C7.1 | Must | 0 | 1 | 0.2 | 2 | aucun article, aucun marqueur d'irréversibilité |
| #876 | **0.1** | C7.3 | Must | 1 | 1 | 0.2 | 4 | aucun article, aucun marqueur d'irréversibilité |
| #877 | **0.1** | C7.1 | Must | 0 | 1 | 0.2 | 2 | aucun article, aucun marqueur d'irréversibilité |
| #432 | **0.067** | C9.3 | Should | 0 | 1 | 0.2 | 3 | aucun article, aucun marqueur d'irréversibilité |
| #453 | **0.067** | C9.1 | Should | 0 | 1 | 0.2 | 3 | aucun article, aucun marqueur d'irréversibilité |
| #731 | **0.067** | C9.1 | Should | 0 | 1 | 0.2 | 3 | aucun article, aucun marqueur d'irréversibilité |
| #874 | **0.067** | C7.3 | Must | 0 | 1 | 0.2 | 3 | aucun article, aucun marqueur d'irréversibilité |
| #355 | **0.05** | C9.2 | Could | 0 | 1 | 0.2 | 4 | aucun article, aucun marqueur d'irréversibilité |
| #718 | **0.05** | C9.1 | Should | 0 | 1 | 0.2 | 4 | aucun article, aucun marqueur d'irréversibilité |
| #813 | **0.05** | C8.3 | Could | 0 | 1 | 0.2 | 4 | aucun article, aucun marqueur d'irréversibilité |

`Reach*` = chantiers débloqués, transitivement. `Impact*` = palier :
**3** invariant légal — un article du Code civil · 
**2** irréversibilité du modèle de données · 
**1** mécanisme de mise en œuvre · 

---

*Dérivé du Manifeste Maury (CC BY-SA 4.0). Skill `adoption` de la
Méthode Foyer, à Reach et Impact substitués faute d'usage réel.*
