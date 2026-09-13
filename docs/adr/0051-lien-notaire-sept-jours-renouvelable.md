# ADR 0051 : le lien du notaire vaut sept jours, renouvelable par le syndic

- **Status**: Accepted (tranché par @gilmry le 2026-09-12)
- **Date**: 2026-09-12
- **Track**: Security / Legal-compliance
- **Authors**: @gilmry (décision) + Claude Opus 5 (rédaction)
- **Related**: #855, [ADR 0048](0048-identite-notaire-et-routes-publiques.md),
  `backend/src/domain/copropriete/releve_notaire.rs`

## Contexte

[ADR 0048](0048-identite-notaire-et-routes-publiques.md) a tranché la
**destination** : `GET /etats-dates/reference/{reference_number}` cesse d'être
publique. Un état daté porte les dettes d'un copropriétaire nommé ; le rendre
lisible à quiconque connaît une référence revenait à publier une situation
financière individuelle derrière un identifiant devinable, transmissible et
jamais révocable. La référence ne porte que 32 bits d'aléa.

Le 2026-09-12, le mécanisme a été arrêté : **lien signé à durée limitée, émis
par le syndic, révocable, journalisé**. Restait la modalité : quelle durée, et
combien de lectures.

C'est la **dernière** route sans identité du produit (`garde_identite_absente` :
30 → 1).

## Décision

**Le lien vaut sept jours calendaires, autorise plusieurs lectures pendant cette
fenêtre, et le syndic peut le renouveler explicitement.** Chaque consultation et
chaque renouvellement sont journalisés.

## Pourquoi

**Sept jours, parce que la fenêtre d'exposition est le seul paramètre qui
protège vraiment.** Le lien circule par courriel, il atterrit dans des dossiers
de vente et des greffes ; ce qui borne le dégât, c'est sa péremption. Sept jours
couvrent une instruction courante sans laisser un lien vivant un mois dans une
boîte aux lettres.

**Plusieurs lectures, parce que l'usage unique se paie en support.** Un notaire
rouvre un dossier, rafraîchit une page, transmet à un clerc. Un lien qui meurt à
la première lecture produit des appels au syndic, et un journal d'accès qui
compte les réémissions au lieu des consultations — soit exactement l'information
qu'on voulait tracer.

**Le renouvellement est un acte du syndic, pas un automatisme.** Une vente qui
dépasse sept jours est normale ; c'est au syndic de le constater, et cet acte
laisse une trace.

### Une correction de ma recommandation

J'avais recommandé trente jours, pour aligner la durée du lien sur
`DELAI_JOURS = 30` de `releve_notaire.rs` (Art. 3.89 § 5, 5°), au motif qu'une
seule durée serait plus simple à expliquer et à tester.

**L'argument était faux, et il aurait produit une confusion durable.** Le délai
de trente jours est une **échéance légale** : le temps dont le syndic dispose
pour *fournir* un relevé. La validité du lien est un **paramètre de sécurité** :
le temps pendant lequel un secret reste opposable. Les faire coïncider n'aurait
pas unifié deux durées, cela aurait amarré un TTL d'accès à un délai que le
législateur peut changer — et rendu impossible de resserrer l'un sans toucher à
l'autre.

Le domaine porte d'ailleurs déjà deux durées légales distinctes (quinze jours
au § 1er, trente au § 5) : « une seule durée » n'était pas l'état des lieux.

## Conséquences

### À construire

- Un jeton signé porteur de `{ etat_date_id, emis_par, expire_le }`, vérifié à
  chaque lecture, **révocable** avant terme par le syndic.
- Un journal : émission, chaque consultation, renouvellement, révocation.
- La route `GET /etats-dates/reference/{reference_number}` passe sous garde
  d'identité — `garde_identite_absente` tombe à 0.
- Renouvellement : action explicite du syndic, qui repart pour sept jours.

### La question laissée ouverte

**Quel `subject_user_id` pour un notaire sans compte ?** Le champ est
aujourd'hui obligatoire dans le journal d'audit. Trois pistes — un utilisateur
technique « porteur de lien », un sujet nullable assorti du `token_id`, ou le
rôle notaire d'[ADR 0048](0048-identite-notaire-et-routes-publiques.md) si le
notaire a fini par recevoir une identité. Cette ADR ne la tranche pas : elle
dépend de l'ordre dans lequel #855 et le rôle notaire seront construits.

### Les quatre classes de tests

`@happy` lecture dans la fenêtre · `@edge` bornes de l'expiration, lecture à
J+7 et J+8, renouvellement en chaîne · `@security` jeton forgé, jeton d'un autre
état daté, jeton révoqué, rejeu après expiration · `@negative` référence
inconnue, jeton malformé — erreur typée, jamais de panic.
