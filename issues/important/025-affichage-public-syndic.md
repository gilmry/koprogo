# Issue #025 - Affichage Public Informations Syndic

**Priorité**: 🟡 HIGH
**Estimation**: 3-4 heures
**Labels**: `enhancement`, `frontend`, `legal-compliance`

---

## 📋 Description

Créer une **page publique** (non authentifiée) affichant les informations obligatoires du syndic, conformément à la loi belge. L'affichage public des coordonnées syndic est une **obligation légale**.

**Contexte légal** : Le syndic doit afficher ses coordonnées de manière visible et accessible à tous les copropriétaires et visiteurs.

---

## 🎯 Objectifs

- [ ] Page publique `/syndic/:building_slug`
- [ ] Affichage nom, adresse, téléphone, email syndic
- [ ] Horaires permanence
- [ ] Dates prochaines AG
- [ ] Règlement de copropriété (PDF téléchargeable)
- [ ] Pas d'authentification requise

---

## 📐 Implémentation

### Route Publique

```
GET /public/buildings/:slug/syndic
```

### Données Affichées

- Nom syndic
- Adresse bureau
- Téléphone / Email
- Horaires permanence
- Prochaine AG (date/lieu)
- Lien règlement copropriété PDF
- Lien contact urgence

### SEO

- Meta tags pour référencement
- Schema.org markup (Organization)

---

## ✅ Critères

- [ ] Accessible sans login
- [ ] Responsive mobile
- [ ] WCAG AA accessible
- [ ] SEO optimized

---

**Créé le** : 2025-11-01
**Milestone** : v1.0 - Legal Compliance
