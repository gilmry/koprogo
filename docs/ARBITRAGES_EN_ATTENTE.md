# Arbitrages en attente — ce qui bloque la clôture du WBS v0.1.0

État au 2026-09-08. **70 issues ouvertes** en `release:0.1.0`.

Ce document existe parce que six décisions dispersées dans autant d'issues
bloquent, à elles seules, trois tracks entiers. Les regrouper permet d'y
répondre en une passe.

Aucune ne se tranche en écrivant du code. Chacune porte ma recommandation, mais
**la décision t'appartient** : elles engagent le produit, pas l'implémentation.

---

## 1. Le porteur du périmètre communautaire — #779

**La question.** Les six modules communautaires — SEL, annonces, compétences,
partage d'objets, réservations, gamification — sont servis par immeuble ou par
organisation. Le frontend les appelle par ACP. Qui est le bon porteur ?

Ce n'est pas une faute de frappe : le frontend a été écrit dans le monde
d'après l'ADR-0045, le backend sert encore celui d'avant.

**Ce qui en dépend.** 111 points d'entrée, dont 75 absents du contrat OpenAPI.
Les décrire avant de trancher serait du travail à refaire.

**Ma recommandation, module par module :**

| Module | Porteur | Pourquoi |
|---|---|---|
| Annonces | **ACP** | un avis du syndic concerne la copropriété, pas un bâtiment |
| SEL, compétences, partage | **immeuble** | l'entraide se joue entre voisins de palier, pas entre bâtiments d'un même cabinet |
| Réservations | **immeuble** | on réserve une salle commune, qui est dans un bâtiment précis |
| Gamification | **ACP** | servie par organisation aujourd'hui, ce qui est un troisième porteur et probablement le mauvais depuis le recentrage |

**La vraie question sous-jacente** : une ACP peut compter plusieurs immeubles.
Un prêt de perceuse se partage-t-il entre immeubles d'une même copropriété ?

---

## 2. Le syndic agissant pour le compte de l'ACP — #781, #588

**La question.** Trois modules résolvent l'auteur d'une action via une fiche de
copropriétaire, et refusent sinon. Un syndic n'en a pas.

**Ce qui est déjà tranché par les faits.** Le refus est légitime pour les
compétences et le partage d'objets : offrir une compétence ou prêter une
perceuse engage une personne nommée, pas la copropriété. La création d'annonce
par le syndic fonctionne, et c'est cohérent — un avis émane de la copropriété.

**Ce qui reste à trancher.** Les réservations. Réserver la salle commune pour
une assemblée est un acte de gestion, pas un échange entre voisins.

**Ma recommandation :** oui, le syndic doit pouvoir réserver au nom de l'ACP,
avec une trace explicite de qui a réservé pour qui. Non pour les compétences et
le partage. Et un bandeau sur ces pages disant qu'il consulte sans participer —
aujourd'hui il voit tout et ne peut rien, ce qui se lit comme une panne.

---

## 3. Les dix-neuf routes sans identité — #845

**La question.** Trente routes ne vérifiaient aucune identité ; onze sont
corrigées. Sur les dix-neuf restantes, **certaines sont peut-être publiques à
dessein**, et je ne veux pas le supposer.

| Route | Hypothèse à confirmer |
|---|---|
| `POST /energy-campaigns/{id}/join-as-individual` et les 3 routes de consentement qui suivent | une campagne énergétique s'ouvre-t-elle à des particuliers hors copropriété ? |
| `GET /etats-dates/reference/{reference_number}` | un notaire consulte-t-il un état daté par sa référence, sans compte ? |
| `PUT /convocation-recipients/{id}/email-opened` | pixel de suivi d'ouverture, appelé par un client de messagerie qui ne porte aucun jeton |

**Ma recommandation :** les trois hypothèses me paraissent plausibles et
défendables. Si tu les confirmes, ces six routes rejoignent la liste
`PUBLIQUES` de la garde **avec leur raison écrite** — une liste sans
justification se remplit toute seule. Les treize autres n'ont pas de raison
apparente d'être publiques et je les garderai.

**Priorité si tu ne dois en trancher qu'une** : le pixel de suivi. C'est un
`PUT` non authentifié, donc une écriture.

---

## 4. Le périmètre d'immeuble à travers les navigations — #841

**La question.** Le périmètre est nul au premier rendu de chaque page, parce
que le frontend est une application Astro multi-page et que le store est un
`$state` de module.

**Ce que j'ai fait, et défait.** J'ai implémenté la réhydratation depuis
`?buildingId=`, validée par le serveur. **Elle casse trois specs Playwright**
— `networkidle` qui n'arrive jamais, signe d'une activité réseau qui ne
s'arrête pas. Je l'ai débranchée plutôt que de laisser une régression sur la
branche qui alimente la démo. La fonction et ses tests restent.

**Ce qui reste à décider.** Faut-il un lien profond, un défaut serveur (le
dernier immeuble consulté, ou l'unique immeuble quand il n'y en a qu'un), ou
les deux ? Le commentaire du store annonce les deux depuis l'origine.

**Ma recommandation :** les deux, mais **le défaut serveur d'abord**. Il couvre
le cas courant sans paramètre d'URL, et il ne peut pas boucler.

---

## 5. Deux issues qui contredisent leur propre étiquette — #635, #694

- **#635** porte « (v0.2.0) » dans son titre et l'étiquette `release:0.1.0`.
- **#694** écrit « Non bloquant pour v0.1.0 (bêta fermée) » dans son corps, et
  demande par ailleurs un brief Maury signé avant tout code.

Les deux pèsent sur le décompte, donc sur la date du tag.

**Ma recommandation :** sortir les deux du périmètre 0.1.0. #694 dit elle-même
qu'elle n'est pertinente que « si un cabinet pilote gère plusieurs ACPs avec du
staff dédié par ACP » — ce n'est pas la bêta fermée.

---

## 6. Les onze tables muettes — #846

**La question.** Onze tables existent en base et ne sont lues par aucun code,
dont `meeting_proxy_mandates` et sa vue `proxy_mandate_stats`, qui se déclare
outil de vérification des procurations et renvoie toujours zéro ligne.

**Ma recommandation :** supprimer par migration celles dont la fonctionnalité
n'est pas prévue, documenter les autres comme réservées avec l'issue qui les
activera. Le pire est l'état actuel : présentes, commentées comme actives, et
muettes.

**Je ne supprime rien sans ton accord** : une migration de suppression est
irréversible en production, et deux de ces tables portent des données
potentielles (`api_key_usage`, `mqtt_messages`) qu'il faudrait vérifier vides.

**Le cas urgent est `proxy_mandate_stats`** : une vue qui se déclare outil de
conformité et rend toujours zéro doit être branchée ou disparaître.

---

## Ce qui ne dépend d'aucun arbitrage

**G1 et G2 restent hors de portée quoi qu'il arrive.** La revue humaine avec GO
signé et la pose du tag `v0.1.0` sont Tier 1 par
`docs/governance/RESPONSABILITE.md`. Le WBS le dit lui-même : « G1 devient le
vrai jalon ».

**Le Track D attend le Track U.** Filmer la documentation vivante avant la
refonte produirait une documentation périmée le jour de sa livraison. Neuf
maquettes séparent U de son terme.
