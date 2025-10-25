#!/usr/bin/env python3
"""
Script pour configurer automatiquement le DNS OVH via l'API
Crée ou met à jour un enregistrement A pour pointer vers le VPS
"""

import sys
import os

try:
    import ovh
except ImportError:
    print("ERROR: Module 'ovh' non installé")
    print("Installation: pip3 install ovh")
    sys.exit(1)

def extract_zone_and_subdomain(domain):
    """
    Extrait la zone DNS et le sous-domaine depuis un FQDN

    Exemples:
    - koprogo.com -> zone: koprogo.com, subdomain: ''
    - api.koprogo.com -> zone: koprogo.com, subdomain: 'api'
    - qa.koprogo.com -> zone: koprogo.com, subdomain: 'qa'
    - api.qa.koprogo.com -> zone: koprogo.com, subdomain: 'api.qa'

    Note: Pour les sous-domaines multi-niveaux comme api.qa.koprogo.com,
    on assume que la zone racine est toujours les 2 derniers segments (koprogo.com),
    et tout le reste devient le sous-domaine (api.qa).
    """
    parts = domain.split('.')

    if len(parts) < 2:
        raise ValueError(f"Invalid domain format: {domain}")

    # La zone DNS est TOUJOURS les 2 derniers segments (ex: koprogo.com)
    # Tout ce qui précède devient le sous-domaine
    zone = '.'.join(parts[-2:])
    subdomain = '.'.join(parts[:-2]) if len(parts) > 2 else ''

    return zone, subdomain

def configure_dns(domain, target_ip, ovh_endpoint, app_key, app_secret, consumer_key):
    """Configure le DNS OVH pour pointer vers l'IP cible"""

    # Créer le client OVH
    try:
        client = ovh.Client(
            endpoint=ovh_endpoint,
            application_key=app_key,
            application_secret=app_secret,
            consumer_key=consumer_key,
        )
    except Exception as e:
        print(f"ERROR: Impossible de créer le client OVH: {e}")
        return False

    # Extraire zone et sous-domaine
    try:
        zone, subdomain = extract_zone_and_subdomain(domain)
    except ValueError as e:
        print(f"ERROR: {e}")
        return False

    print(f"Configuration DNS:")
    print(f"  Domain complet: {domain}")
    print(f"  Zone DNS: {zone}")
    print(f"  Sous-domaine: {subdomain or '@ (racine)'}")
    print(f"  IP cible: {target_ip}")
    print()

    try:
        # Vérifier que la zone existe
        try:
            zone_info = client.get(f'/domain/zone/{zone}')
            print(f"✓ Zone DNS trouvée: {zone}")
        except ovh.exceptions.ResourceNotFoundError:
            print(f"ERROR: Zone DNS '{zone}' non trouvée dans votre compte OVH")
            print(f"Vérifiez que le domaine est bien géré chez OVH")
            return False

        # Chercher les enregistrements A existants pour ce sous-domaine
        print(f"\n🔍 Recherche d'enregistrements A existants...")

        search_params = {
            'fieldType': 'A',
        }

        # Si subdomain est vide, on cherche les records sans subdomain (racine)
        if subdomain:
            search_params['subDomain'] = subdomain

        existing_records = client.get(
            f'/domain/zone/{zone}/record',
            **search_params
        )

        if existing_records:
            print(f"✓ {len(existing_records)} enregistrement(s) A existant(s) trouvé(s)")

            # Mettre à jour le premier enregistrement trouvé
            record_id = existing_records[0]

            # Récupérer les détails de l'enregistrement
            record_details = client.get(f'/domain/zone/{zone}/record/{record_id}')
            old_ip = record_details.get('target', 'N/A')

            print(f"  ID: {record_id}")
            print(f"  Ancienne IP: {old_ip}")
            print(f"  Nouvelle IP: {target_ip}")

            if old_ip == target_ip:
                print(f"\n✓ L'enregistrement pointe déjà vers {target_ip}")
                print(f"  Aucune modification nécessaire")
            else:
                # Mettre à jour l'enregistrement
                print(f"\n📝 Mise à jour de l'enregistrement...")
                client.put(
                    f'/domain/zone/{zone}/record/{record_id}',
                    target=target_ip,
                    ttl=60  # TTL de 60 secondes pour propagation rapide
                )
                print(f"✓ Enregistrement mis à jour")
        else:
            print(f"ℹ  Aucun enregistrement A existant")
            print(f"\n📝 Création d'un nouvel enregistrement A...")

            # Créer un nouvel enregistrement
            new_record = client.post(
                f'/domain/zone/{zone}/record',
                fieldType='A',
                subDomain=subdomain if subdomain else None,
                target=target_ip,
                ttl=60
            )

            print(f"✓ Nouvel enregistrement créé (ID: {new_record['id']})")

        # Rafraîchir la zone DNS
        print(f"\n🔄 Rafraîchissement de la zone DNS...")
        client.post(f'/domain/zone/{zone}/refresh')
        print(f"✓ Zone DNS rafraîchie")

        print(f"\n{'='*60}")
        print(f"✅ Configuration DNS réussie!")
        print(f"{'='*60}")
        print(f"\nLe domaine {domain} pointe maintenant vers {target_ip}")
        print(f"\n⏱️  Propagation DNS:")
        print(f"  - Peut prendre 1 à 60 minutes")
        print(f"  - Vérifier: dig {domain}")
        print(f"  - Ou: nslookup {domain}")
        print()

        return True

    except ovh.exceptions.APIError as e:
        print(f"\nERROR API OVH: {e}")
        return False
    except Exception as e:
        print(f"\nERROR: {e}")
        import traceback
        traceback.print_exc()
        return False

def main():
    """Point d'entrée principal"""

    # Récupérer les paramètres depuis les variables d'environnement
    domain = os.environ.get('DOMAIN')
    target_ip = os.environ.get('TARGET_IP')
    ovh_endpoint = os.environ.get('OVH_ENDPOINT', 'ovh-eu')
    app_key = os.environ.get('OVH_APPLICATION_KEY')
    app_secret = os.environ.get('OVH_APPLICATION_SECRET')
    consumer_key = os.environ.get('OVH_CONSUMER_KEY')

    # Vérifier que tous les paramètres requis sont présents
    missing_params = []

    if not domain:
        missing_params.append('DOMAIN')
    if not target_ip:
        missing_params.append('TARGET_IP')
    if not app_key:
        missing_params.append('OVH_APPLICATION_KEY')
    if not app_secret:
        missing_params.append('OVH_APPLICATION_SECRET')
    if not consumer_key:
        missing_params.append('OVH_CONSUMER_KEY')

    if missing_params:
        print("ERROR: Variables d'environnement manquantes:")
        for param in missing_params:
            print(f"  - {param}")
        print("\nUsage:")
        print("  export DOMAIN='example.com'")
        print("  export TARGET_IP='1.2.3.4'")
        print("  export OVH_ENDPOINT='ovh-eu'")
        print("  export OVH_APPLICATION_KEY='your_key'")
        print("  export OVH_APPLICATION_SECRET='your_secret'")
        print("  export OVH_CONSUMER_KEY='your_consumer_key'")
        print("  python3 configure-ovh-dns.py")
        sys.exit(1)

    print("╔════════════════════════════════════════════════════════════╗")
    print("║     Configuration DNS OVH via API                          ║")
    print("╚════════════════════════════════════════════════════════════╝")
    print()

    # Configurer le DNS
    success = configure_dns(
        domain=domain,
        target_ip=target_ip,
        ovh_endpoint=ovh_endpoint,
        app_key=app_key,
        app_secret=app_secret,
        consumer_key=consumer_key
    )

    sys.exit(0 if success else 1)

if __name__ == '__main__':
    main()
