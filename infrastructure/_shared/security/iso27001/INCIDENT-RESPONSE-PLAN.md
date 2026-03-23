# Plan de Reponse aux Incidents - KoproGo

## ISO 27001:2022 - A.5.24, A.5.25, A.5.26

### 1. Classification des incidents

| Severite | Description | Temps de reponse | Exemples |
|----------|-------------|-------------------|----------|
| P1 - Critique | Service indisponible ou breach donnees | < 15 min | Fuite GDPR, service down |
| P2 - Majeur | Degradation significative | < 1h | Performance, tentative intrusion |
| P3 - Mineur | Impact limite | < 4h | Bug non-bloquant, alerte monitoring |
| P4 - Information | Pas d'impact | < 24h | Scan detecte, log suspect |

### 2. Workflow de reponse

```
Detection (Alertmanager/ElastAlert2/CrowdSec)
    |
    v
Triage (Classification P1-P4)
    |
    v
Containment (Isoler la menace)
    |
    v
Eradication (Supprimer la cause)
    |
    v
Recovery (Restaurer le service)
    |
    v
Post-mortem (Analyse et amelioration)
```

### 3. Canaux de detection

| Canal | Outil | Type d'incident |
|-------|-------|-----------------|
| Metriques | Alertmanager | Performance, disponibilite |
| Logs | ElastAlert2 | Erreurs, comportement suspect |
| Reseau | Suricata IDS | Intrusion, injection |
| WAF | CrowdSec | Attaques web, bots |
| Brute-force | fail2ban | SSH, API, DB |
| Fichiers | AIDE | Modification non autorisee |
| Rootkit | rkhunter | Malware |
| Audit systeme | Lynis | Vulnerabilites |
| Audit code | cargo audit / npm audit | Dependencies vulnerables |

### 4. Actions par type d'incident

#### Breach donnees (P1)
1. Isoler le systeme compromis
2. Notifier le DPO (GDPR Art. 33 : 72h)
3. Analyser l'etendue de la breach
4. Notifier l'APD (Autorite de Protection des Donnees belge)
5. Notifier les personnes concernees si risque eleve (Art. 34)
6. Rotation de tous les secrets (Vault)
7. Post-mortem

#### Intrusion detectee (P2)
1. CrowdSec/fail2ban : ban IP automatique
2. Analyser les logs Suricata/Filebeat
3. Verifier AIDE (integrity des fichiers)
4. Scanner rkhunter
5. Rotation credentials si necessaire

### 5. GDPR Breach Notification

| Etape | Delai | Responsable |
|-------|-------|-------------|
| Detection | Immediat | Monitoring automatique |
| Evaluation impact | < 24h | DPO + RSSI |
| Notification APD | < 72h | DPO |
| Notification personnes | "Sans retard" si risque eleve | DPO |
| Rapport final | < 30 jours | RSSI |
