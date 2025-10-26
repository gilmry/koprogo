# Guide de Test Manuel - Phase 1

## Prérequis

1. Backend démarré : `cd backend && cargo run` OU `docker compose up`
2. Frontend démarré : `cd frontend && npm run dev` OU via Docker
3. Base de données PostgreSQL active

## URLs de Test

- **Frontend** : http://localhost:3000
- **Backend API** : http://localhost:8080/api/v1
- **Via Traefik (Docker)** : http://localhost

---

## 1. Test d'Inscription (Nouveau Compte)

### Étapes :
1. Ouvrir http://localhost:3000/register
2. Remplir le formulaire :
   - **Prénom** : Jean
   - **Nom** : Dupont
   - **Email** : jean.dupont@test.com
   - **Mot de passe** : password123
   - **Confirmer mot de passe** : password123
   - **Type de compte** : Copropriétaire

### Résultats Attendus :
✅ Formulaire de validation affiche les erreurs en temps réel
✅ Si mot de passe < 6 caractères → Erreur "Au moins 6 caractères"
✅ Si mots de passe différents → Erreur "Les mots de passe ne correspondent pas"
✅ Si email invalide → Erreur "Format d'email invalide"
✅ Si prénom/nom < 2 caractères → Erreur appropriée
✅ Après soumission réussie :
   - Toast vert "Compte créé avec succès!"
   - Redirection vers `/owner` (pour role Owner)
   - localStorage contient :
     - `koprogo_user`
     - `koprogo_token`
     - `koprogo_refresh_token`

### Vérification dans DevTools (F12) :
```javascript
// Console
localStorage.getItem('koprogo_token')  // JWT token visible
localStorage.getItem('koprogo_refresh_token')  // Refresh token visible
localStorage.getItem('koprogo_user')  // User object visible
```

---

## 2. Test de Connexion

### Étapes :
1. Se déconnecter (bouton dans navigation)
2. Ouvrir http://localhost:3000/login
3. Se connecter avec **SuperAdmin** :
   - Email : `admin@koprogo.com`
   - Mot de passe : `admin123`

### Résultats Attendus :
✅ Connexion réussie
✅ Redirection vers `/admin`
✅ Navigation affiche le menu SuperAdmin
✅ localStorage mis à jour avec nouveau token

### Autres Comptes de Test (après seed demo) :
- **Syndic** : `syndic@grandplace.be / syndic123`
- **Comptable** : `comptable@grandplace.be / comptable123`
- **Propriétaire** : `proprietaire1@grandplace.be / owner123`

---

## 3. Test de Gestion de Session

### Test A : Token Refresh Automatique

#### Étapes :
1. Se connecter comme SuperAdmin
2. Ouvrir DevTools → Network tab
3. Attendre **10 minutes** (ou modifier `TOKEN_REFRESH_INTERVAL` dans `auth.ts` à 30 secondes pour tester plus vite)
4. Observer le Network tab

#### Résultats Attendus :
✅ Requête POST vers `/api/v1/auth/refresh` automatiquement
✅ Nouveau token reçu et stocké
✅ Aucune redirection vers `/login`
✅ localStorage mis à jour avec nouveaux tokens

---

### Test B : Validation de Session au Chargement

#### Étapes :
1. Se connecter
2. Naviguer vers `/admin`
3. Rafraîchir la page (F5)

#### Résultats Attendus :
✅ Page se charge sans redirection
✅ Requête GET `/api/v1/auth/me` visible dans Network tab
✅ Utilisateur reste connecté

---

### Test C : Token Expiré / Invalide

#### Étapes :
1. Se connecter
2. Dans DevTools Console, exécuter :
```javascript
localStorage.setItem('koprogo_token', 'invalid_token')
```
3. Rafraîchir la page

#### Résultats Attendus :
✅ Tentative d'appel `/auth/me` échoue (401)
✅ Tentative d'appel `/auth/refresh` avec refresh token
✅ Si refresh échoue → Déconnexion automatique
✅ Redirection vers `/login`
✅ localStorage vidé

---

### Test D : Suppression Manuelle du Token

#### Étapes :
1. Se connecter
2. Dans Console :
```javascript
localStorage.removeItem('koprogo_token')
```
3. Tenter de naviguer vers `/admin`

#### Résultats Attendus :
✅ Tentative de refresh avec refresh_token
✅ Si succès → Accès autorisé avec nouveau token
✅ Si échec → Redirection vers `/login`

---

## 4. Test des Composants UI Réutilisables

### Test Modal
1. Aller sur `/admin/organizations`
2. Cliquer "➕ Nouvelle organisation" (bouton existe mais modal pas encore câblée)
3. **Pour tester manuellement** : Créer une page de test ou utiliser RegisterForm comme exemple

#### Fonctionnalités à Vérifier :
✅ Modal s'ouvre avec overlay sombre
✅ ESC ferme la modal
✅ Clic sur overlay ferme la modal
✅ Bouton X ferme la modal
✅ Scroll fonctionne si contenu long

---

### Test Formulaires (FormInput, FormSelect)
1. Aller sur `/register`

#### Fonctionnalités à Vérifier :
✅ Labels affichés correctement
✅ Champ requis affiche une astérisque rouge (*)
✅ Erreurs affichées en rouge sous le champ
✅ Hints affichés en gris sous le champ (ex: "Au moins 6 caractères")
✅ Focus change la bordure en primary-500
✅ États disabled fonctionnent (grisé, curseur not-allowed)

---

### Test Boutons (Button Component)
Tester dans `/register` et `/login`

#### Fonctionnalités à Vérifier :
✅ Variantes de couleur : primary (vert), secondary (gris), danger (rouge)
✅ Tailles : sm, md, lg
✅ Loading state → Spinner + texte "Connexion..." / "Création..."
✅ Disabled state → Opacité 50%, curseur not-allowed
✅ fullWidth → Bouton prend toute la largeur

---

### Test Toasts (Notifications)
1. Se connecter avec mauvais mot de passe

#### Résultats Attendus :
✅ Toast rouge apparaît en haut à droite
✅ Message : "Email ou mot de passe incorrect"
✅ Icône d'erreur visible
✅ Bouton X pour fermer
✅ Auto-dismiss après 7 secondes (error) ou 5 secondes (success)

2. S'inscrire avec succès

#### Résultats Attendus :
✅ Toast vert "Compte créé avec succès!"
✅ Auto-dismiss après 5 secondes

---

### Test ConfirmDialog
**Note** : Pas encore utilisé dans l'UI actuelle, sera utilisé pour les actions de suppression

#### Fonctionnalités Prévues :
- Modal de confirmation avec variante danger (rouge) ou primary
- Texte personnalisable
- Boutons "Confirmer" / "Annuler"
- Loading state sur le bouton de confirmation

---

## 5. Test des Routes Protégées

### Test A : Accès Sans Authentification

#### Étapes :
1. Se déconnecter
2. Vider localStorage
3. Tenter d'accéder directement à :
   - http://localhost:3000/admin
   - http://localhost:3000/syndic
   - http://localhost:3000/buildings

#### Résultats Attendus :
✅ Redirection automatique vers `/login`
✅ Pages ne chargent pas de contenu sensible

---

### Test B : Accès Basé sur le Rôle

#### Étapes :
1. Se connecter comme **Owner** (proprietaire1@grandplace.be)
2. Tenter d'accéder à `/admin`

#### Résultats Attendus :
⚠️ **Note** : Protection par rôle pas encore implémentée (Phase 2)
🔜 À terme devrait rediriger ou afficher "Accès refusé"

---

## 6. Test de l'Intégration Backend

### Test API Auth

#### Register :
```bash
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "password123",
    "first_name": "Test",
    "last_name": "User",
    "role": "owner"
  }'
```

**Attendu** : Status 201, retourne `{token, refresh_token, user}`

---

#### Login :
```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@koprogo.com",
    "password": "admin123"
  }'
```

**Attendu** : Status 200, retourne `{token, refresh_token, user}`

---

#### Refresh Token :
```bash
TOKEN="<votre_refresh_token>"
curl -X POST http://localhost:8080/api/v1/auth/refresh \
  -H "Content-Type: application/json" \
  -d "{\"refresh_token\": \"$TOKEN\"}"
```

**Attendu** : Status 200, nouveaux `{token, refresh_token, user}`

---

#### Get Current User :
```bash
JWT="<votre_access_token>"
curl http://localhost:8080/api/v1/auth/me \
  -H "Authorization: Bearer $JWT"
```

**Attendu** : Status 200, retourne user object

---

## 7. Test de Déconnexion

### Étapes :
1. Se connecter
2. Cliquer sur "Se déconnecter" dans la navigation

### Résultats Attendus :
✅ Redirection vers `/login`
✅ localStorage vidé (user, token, refresh_token supprimés)
✅ Menu navigation mis à jour (mode déconnecté)
✅ Impossible d'accéder aux routes protégées

---

## 8. Checklist Complète Phase 1

### Authentification
- [x] Page d'inscription fonctionnelle
- [x] Page de connexion fonctionnelle
- [x] Déconnexion fonctionne
- [x] Tokens stockés dans localStorage
- [x] Refresh tokens stockés

### Gestion de Session
- [x] Auto-refresh tous les 10 minutes
- [x] Validation session au chargement (GET /auth/me)
- [x] Redirection automatique si session invalide
- [x] SessionManager intégré dans Layout

### Composants UI
- [x] Modal réutilisable
- [x] FormInput avec validation
- [x] FormSelect
- [x] FormTextarea
- [x] Button avec loading/disabled states
- [x] ConfirmDialog
- [x] Toast system global
- [x] ToastContainer

### Types & Stores
- [x] Types Organization, Meeting, Document ajoutés
- [x] Toast store fonctionnel
- [x] Auth store étendu (refreshAccessToken, validateSession)

### Intégration
- [x] Layout mis à jour (ToastContainer, SessionManager)
- [x] LoginForm utilise refresh_token
- [x] RegisterForm utilise refresh_token
- [x] Toutes les pages se chargent sans erreur

---

## Problèmes Connus & Limitations

1. **Protection par Rôle** : Pas encore implémentée (prévu Phase 2)
2. **Token Expiration UI** : Pas de notification visible quand token expire
3. **Remember Me** : Checkbox UI seulement, pas fonctionnel
4. **Password Reset** : Pas encore implémenté
5. **Email Verification** : Pas prévu pour le moment
6. **Multi-Device Logout** : Endpoint backend existe mais pas d'UI

---

## Prochaines Étapes (Phase 2+)

1. **Admin CRUD** : Organizations & Users management
2. **Buildings CRUD** : Detail, Edit, Delete
3. **Units/Owners CRUD** : Create, Edit, Assign
4. **Documents Upload** : Drag & drop, multipart form-data
5. **PCN Reports** : Generation & export
6. **Offline Sync** : Integration syncService
7. **Tests Automatisés** : Unit, Integration, E2E

---

**Date** : 2025-10-26
**Version** : Phase 1 Complete
**Status** : ✅ Prêt pour test manuel
