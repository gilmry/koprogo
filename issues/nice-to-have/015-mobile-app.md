# Issue #015 - Application Mobile Native

**Priorité**: 🟢 NICE-TO-HAVE
**Estimation**: 30-40 heures
**Labels**: `enhancement`, `mobile`, `react-native`, `flutter`

---

## 📋 Description

Développer une application mobile native iOS/Android pour copropriétaires et syndics. Complément de la PWA avec fonctionnalités natives.

---

## 🎯 Objectifs

- [ ] App iOS (App Store)
- [ ] App Android (Play Store)
- [ ] Authentification biométrique (Face ID, Touch ID)
- [ ] Notifications push natives
- [ ] Scanner QR codes factures
- [ ] Photos haute résolution (tickets, dégradations)
- [ ] Mode hors ligne complet
- [ ] Partage de localisation (intervention technicien)

---

## 📐 Stack Technique

### Option 1 : React Native (Recommandé)
**Avantages** :
- Code partagé iOS/Android (90%)
- Écosystème riche
- Performance proche native
- Team déjà familière avec TS/React

**Libs clés** :
- `react-navigation` - Navigation
- `react-native-biometrics` - Auth biométrique
- `react-native-camera` - Appareil photo
- `react-native-qrcode-scanner` - QR codes
- `@react-native-async-storage` - Storage local
- `@notifee/react-native` - Notifications

### Option 2 : Flutter
**Avantages** :
- Performance supérieure
- UI widgets riches
- Hot reload excellent

**Inconvénient** : Nouvelle stack (Dart)

---

## 📱 Écrans Principaux

### Copropriétaire
1. **Dashboard** : Prochaines échéances, notifications
2. **Mes Charges** : Appels de fonds, historique paiements
3. **Documents** : Accès PV AG, règlements
4. **Tickets** : Créer incident avec photo
5. **Assemblées** : Ordre du jour, votes
6. **Profil** : Infos personnelles, préférences

### Syndic
1. **Multi-Copro Dashboard** : Tous immeubles
2. **Tickets** : Liste incidents à traiter
3. **Paiements** : Encaissements du jour
4. **Scanner Facture** : OCR instantané
5. **Notifications** : Alertes temps réel

---

## 📝 User Stories

```gherkin
En tant que copropriétaire
Je veux payer mes charges depuis mon iPhone
Afin de gagner du temps

Scénario: Paiement mobile
  Étant donné que j'ai une charge impayée
  Quand j'ouvre l'app
  Et je clique sur "Payer 450€"
  Et je m'authentifie avec Face ID
  Alors le paiement est traité
  Et je reçois notification de confirmation
```

```gherkin
En tant que syndic
Je veux scanner une facture papier
Afin de l'enregistrer rapidement

Scénario: Scan facture
  Quand j'ouvre l'appareil photo
  Et je scanne une facture
  Alors l'OCR extrait les données
  Et je valide en 1 clic
```

---

## 🔧 Fonctionnalités Natives

### 1. Authentification Biométrique

```typescript
import ReactNativeBiometrics from 'react-native-biometrics';

const authenticate = async () => {
  const { success } = await ReactNativeBiometrics.simplePrompt({
    promptMessage: 'Confirmez votre identité',
  });

  if (success) {
    // Proceed with payment
  }
};
```

### 2. Scanner QR Code

```typescript
import QRCodeScanner from 'react-native-qrcode-scanner';

<QRCodeScanner
  onRead={({ data }) => {
    // Parse QR code (expense reference)
    fetchExpenseDetails(data);
  }}
  topContent={<Text>Scannez le QR code de votre facture</Text>}
/>
```

### 3. Notifications Push

```typescript
import notifee from '@notifee/react-native';

await notifee.displayNotification({
  title: 'Nouvelle Assemblée Générale',
  body: 'AG planifiée le 15/12/2025 à 14h',
  android: {
    channelId: 'meetings',
    pressAction: {
      id: 'open-meeting',
    },
  },
  ios: {
    sound: 'default',
  },
});
```

### 4. Appareil Photo Haute Résolution

```typescript
import { launchCamera } from 'react-native-image-picker';

const takePhoto = async () => {
  const result = await launchCamera({
    mediaType: 'photo',
    quality: 1, // Max quality
    includeBase64: true,
  });

  if (result.assets) {
    uploadTicketPhoto(result.assets[0].base64);
  }
};
```

---

## 🗂️ Structure Projet

```
koprogo-mobile/
├── android/
├── ios/
├── src/
│   ├── screens/
│   │   ├── Dashboard.tsx
│   │   ├── Charges.tsx
│   │   ├── Tickets.tsx
│   │   ├── Documents.tsx
│   ├── components/
│   │   ├── PaymentButton.tsx
│   │   ├── BiometricAuth.tsx
│   │   ├── QRScanner.tsx
│   ├── services/
│   │   ├── api.ts (fetch backend)
│   │   ├── storage.ts (AsyncStorage)
│   │   ├── notifications.ts
│   ├── navigation/
│   │   └── RootNavigator.tsx
│   └── App.tsx
├── package.json
└── app.json
```

---

## ✅ Critères d'Acceptation

### Fonctionnel
- [ ] Auth biométrique fonctionne iOS + Android
- [ ] Push notifications reçues
- [ ] Scanner QR codes précis
- [ ] Photos haute résolution uploadées
- [ ] Mode offline avec sync auto
- [ ] Deep linking (ouvrir AG depuis notif)

### Performance
- [ ] Démarrage app < 2s
- [ ] Navigation fluide 60 FPS
- [ ] Consommation batterie raisonnable

### Store
- [ ] Soumis App Store (iOS)
- [ ] Soumis Play Store (Android)
- [ ] Screenshots + description marketing

---

## 🧪 Tests

```typescript
// Jest + React Native Testing Library
import { render, fireEvent } from '@testing-library/react-native';

test('payment button triggers biometric', async () => {
  const { getByText } = render(<PaymentButton amount={450} />);

  fireEvent.press(getByText('Payer 450€'));

  // Mock biometric prompt
  expect(ReactNativeBiometrics.simplePrompt).toHaveBeenCalled();
});
```

---

## 🚀 Checklist

- [ ] Init projet React Native
- [ ] Setup navigation (React Navigation)
- [ ] Écrans principaux (Dashboard, Charges, Tickets)
- [ ] Intégration API backend
- [ ] Auth biométrique
- [ ] Scanner QR code
- [ ] Appareil photo
- [ ] Push notifications
- [ ] Offline sync (AsyncStorage)
- [ ] Tests E2E (Detox)
- [ ] Build Android APK
- [ ] Build iOS IPA
- [ ] Soumission stores
- [ ] Documentation

---

## 📦 Publication

### App Store (iOS)
- Apple Developer Account : 99$/an
- Review time : 1-3 jours
- App Store Connect setup

### Play Store (Android)
- Google Play Developer : 25$ one-time
- Review time : < 24h
- Play Console setup

---

## 📊 Métriques de Succès

- Downloads : > 1000 dans 3 mois
- Retention D7 : > 40%
- Rating stores : > 4.5/5
- Crash-free rate : > 99%

---

**Créé le** : 2025-10-23
**Budget** : ~150€ (comptes développeurs)
**Maintenance** : 5-10h/mois (updates OS)
