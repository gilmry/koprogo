#!/usr/bin/env python3
"""
Generate complete mirror RST documentation for the entire project.

This script creates a mirrored documentation structure in docs/mirror/
where each source file has a corresponding .rst file WITH FRENCH EXPLANATIONS,
and each directory has an index.rst.

Covers: backend/src, frontend/src, infrastructure/
"""

import os
import re
from pathlib import Path
from typing import List, Tuple, Dict

# Base paths
PROJECT_ROOT = Path(__file__).parent.parent
DOCS_MIRROR = PROJECT_ROOT / "docs" / "mirror"

# Directories to mirror
MIRROR_DIRS = {
    "backend": PROJECT_ROOT / "backend" / "src",
    "frontend": PROJECT_ROOT / "frontend" / "src",
    "infrastructure": PROJECT_ROOT / "infrastructure",
}


def generate_french_explanation(file_path: Path, file_type: str, metadata: dict) -> str:
    """Generate French explanation based on file context."""

    # Get file name and path components
    file_name = file_path.stem
    path_parts = file_path.parts

    # Backend Rust files
    if file_type == "rust":
        # Domain entities
        if "domain/entities" in str(file_path):
            entity_name = file_name.replace("_", " ")
            if "poll" in file_name:
                return f"Entité de domaine représentant un **sondage** pour consultations entre assemblées générales. Implémente la logique métier pour les 4 types de votes (Oui/Non, Choix Multiple, Notation, Texte libre) avec validation des règles métier belges."
            elif "building" in file_name:
                return f"Entité de domaine représentant un **immeuble de copropriété**. Agrégat racine contenant les informations du bâtiment (adresse, nombre de lots, année de construction, informations syndic publiques)."
            elif "unit" in file_name and "owner" not in file_name:
                return f"Entité de domaine représentant un **lot de copropriété** (appartement, cave, parking). Contient les informations du lot (numéro, étage, surface, type)."
            elif "owner" in file_name and "unit" not in file_name:
                return f"Entité de domaine représentant un **copropriétaire**. Contient les données personnelles (GDPR) et coordonnées du propriétaire."
            elif "expense" in file_name:
                return f"Entité de domaine représentant une **charge de copropriété**. Implémente le workflow d'approbation (Draft → PendingApproval → Approved/Rejected) et la gestion TVA belge."
            elif "payment" in file_name and "method" not in file_name:
                return f"Entité de domaine représentant une **transaction de paiement**. Intégration Stripe avec gestion du lifecycle (Pending → Processing → Succeeded/Failed) et support remboursements."
            elif "payment_method" in file_name:
                return f"Entité de domaine représentant un **moyen de paiement** (carte bancaire, SEPA, virement). Stockage sécurisé avec tokens Stripe (PCI-DSS compliant)."
            elif "resolution" in file_name:
                return f"Entité de domaine représentant une **résolution d'assemblée générale**. Implémente les 3 types de majorité belges (Simple, Absolue, Qualifiée) et le système de tantièmes."
            elif "vote" in file_name:
                return f"Entité de domaine représentant un **vote individuel** sur une résolution. Support vote par procuration et traçabilité GDPR complète."
            elif "ticket" in file_name:
                return f"Entité de domaine représentant une **demande d'intervention** (maintenance, dépannage). Workflow de gestion avec priorités et deadlines automatiques selon criticité."
            elif "notification" in file_name:
                return f"Entité de domaine représentant une **notification multi-canal**. Support Email, SMS, Push et In-App avec préférences utilisateur granulaires."
            elif "convocation" in file_name and "recipient" not in file_name:
                return f"Entité de domaine représentant une **convocation d'assemblée générale**. Validation automatique des délais légaux belges (15j ordinaire, 8j extraordinaire)."
            elif "local_exchange" in file_name:
                return f"Entité de domaine représentant un **échange SEL** (Système d'Échange Local). Currency temps (1h = 1 crédit) avec workflow complet (Offered → Requested → InProgress → Completed)."
            elif "achievement" in file_name:
                return f"Entité de domaine représentant un **achievement de gamification**. 8 catégories, 5 tiers (Bronze → Diamond), support achievements secrets et répétables."
            elif "challenge" in file_name:
                return f"Entité de domaine représentant un **challenge de gamification**. Time-bound avec métriques cibles et récompenses en points (Individual/Team/Building)."
            elif "quote" in file_name:
                return f"Entité de domaine représentant un **devis entrepreneur**. Conformité légale belge (3 devis obligatoires >5000€) avec scoring automatique (prix, délai, garantie, réputation)."
            elif "iot_reading" in file_name:
                return f"Entité de domaine représentant une **lecture IoT** (compteur Linky/Ores). Stockage TimescaleDB avec compression et détection anomalies."
            elif "account" in file_name:
                return f"Entité de domaine représentant un **compte comptable PCMN**. Implémentation complète du Plan Comptable Minimum Normalisé belge (AR 12/07/2012) avec hiérarchie 8 classes."
            elif "budget" in file_name:
                return f"Entité de domaine représentant un **budget annuel de copropriété**. Conformité loi belge avec catégories de charges et prévisions."
            elif "meeting" in file_name:
                return f"Entité de domaine représentant une **assemblée générale**. Gestion agenda, procès-verbaux et résolutions votées."
            else:
                return f"Entité de domaine **{entity_name}**. Contient la logique métier pure avec validation des invariants métier dans le constructeur."

        # Domain services
        elif "domain/services" in str(file_path):
            return f"Service de domaine **{file_name.replace('_', ' ')}**. Contient la logique métier complexe nécessitant plusieurs entités (orchestration domaine)."

        # Application DTOs
        elif "application/dto" in str(file_path):
            dto_name = file_name.replace("_dto", "").replace("_", " ")
            return f"Data Transfer Object (DTO) pour **{dto_name}**. Définit les contrats d'API REST (requêtes/réponses) avec validation et sérialisation JSON."

        # Application ports
        elif "application/ports" in str(file_path):
            port_name = file_name.replace("_repository", "").replace("_", " ")
            return f"Port (trait) définissant l'interface **{port_name}**. Abstraction pour l'inversion de dépendance (Hexagonal Architecture), implémentée par la couche Infrastructure."

        # Application use cases
        elif "application/use_cases" in str(file_path):
            usecase_name = file_name.replace("_use_cases", "").replace("_", " ")
            return f"Use Cases pour **{usecase_name}**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine."

        # Infrastructure repositories
        elif "infrastructure/database/repositories" in str(file_path):
            repo_name = file_name.replace("_repository_impl", "").replace("_", " ")
            return f"Implémentation PostgreSQL du repository **{repo_name}**. Adaptateur de la couche Infrastructure implémentant le port défini dans Application."

        # Infrastructure handlers
        elif "infrastructure/web/handlers" in str(file_path):
            handler_name = file_name.replace("_handlers", "").replace("_", " ")
            return f"Handlers HTTP (Actix-web) pour **{handler_name}**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs."

        # Infrastructure external
        elif "infrastructure/external" in str(file_path):
            return f"Client API externe **{file_name.replace('_', ' ')}**. Intégration avec services tiers (Stripe, Linky, etc.) via adaptateur Infrastructure."

        # Main/lib
        elif file_name == "main":
            return "Point d'entrée principal de l'application backend. Configuration du serveur Actix-web, initialisation des repositories, use cases et routes API."
        elif file_name == "lib":
            return "Module racine de la bibliothèque backend. Expose les modules publics (domain, application, infrastructure) et configure les dépendances."

        else:
            return f"Module Rust **{file_name.replace('_', ' ')}**. Fait partie de la couche {get_layer_from_path(file_path)}."

    # Frontend files
    elif file_type == "astro":
        return f"Page Astro **{file_name}**. Génération de page statique (SSG) avec possibilité d'îlots interactifs Svelte. Utilise le layout principal pour le rendu."

    elif file_type == "svelte":
        if "components" in str(file_path):
            return f"Composant Svelte **{file_name}**. Composant interactif réutilisable avec state management, props et événements. Intégré dans les pages Astro via Islands Architecture."
        elif "stores" in str(file_path):
            return f"Store Svelte **{file_name}**. Gestion d'état réactive partagée entre composants (authStore, etc.)."
        else:
            return f"Module Svelte **{file_name}**. Logique frontend interactive avec réactivité Svelte."

    elif file_type == "typescript" or file_type == "javascript":
        if "api" in str(file_path):
            return f"Client API **{file_name}**. Fonctions pour appeler les endpoints backend REST avec gestion des erreurs et authentification."
        elif "utils" in str(file_path) or "lib" in str(file_path):
            return f"Utilitaire **{file_name}**. Fonctions helper réutilisables (formatage, validation, helpers)."
        else:
            return f"Module JavaScript/TypeScript **{file_name}**. Logique métier frontend ou configuration."

    # Infrastructure files
    elif file_type == "yaml":
        if "ansible" in str(file_path):
            if "playbook" in file_name or file_name.endswith(".yml"):
                return f"Playbook Ansible **{file_name}**. Automatisation de déploiement et configuration infrastructure (VPS, sécurité, monitoring)."
            elif "inventory" in file_name:
                return "Inventaire Ansible définissant les serveurs cibles et variables de groupe."
        elif "docker-compose" in file_name:
            return "Configuration Docker Compose pour orchestration des services (PostgreSQL, backend, frontend, monitoring)."
        elif ".github" in str(file_path):
            return f"Workflow GitHub Actions **{file_name}**. Pipeline CI/CD pour tests, build et déploiement automatique."
        else:
            return f"Fichier de configuration YAML **{file_name}**."

    elif file_type == "dockerfile":
        if "backend" in str(file_path):
            return "Dockerfile pour build de l'image Docker backend Rust. Multi-stage build avec optimisations (release, LTO)."
        elif "frontend" in str(file_path):
            return "Dockerfile pour build de l'image Docker frontend Astro. Build statique avec Nginx pour serving."
        else:
            return "Dockerfile pour conteneurisation de service."

    elif file_type == "shell":
        return f"Script shell **{file_name}**. Automatisation de tâches (déploiement, backup, maintenance, etc.)."

    elif file_type == "sql":
        if "migration" in str(file_path):
            return f"Migration SQL **{file_name}**. Migration de schéma PostgreSQL gérée par sqlx (création tables, indexes, contraintes)."
        else:
            return f"Script SQL **{file_name}**. Requêtes ou procédures stockées PostgreSQL."

    # Default fallback
    return f"Fichier **{file_name}** du projet. Type: {file_type}."


def detect_file_type(file_path: Path) -> str:
    """Detect file type from extension."""
    ext = file_path.suffix.lower()

    mapping = {
        ".rs": "rust",
        ".astro": "astro",
        ".svelte": "svelte",
        ".ts": "typescript",
        ".js": "javascript",
        ".yml": "yaml",
        ".yaml": "yaml",
        ".sh": "shell",
        ".sql": "sql",
        ".py": "python",
        ".md": "markdown",
        ".json": "json",
        ".toml": "toml",
    }

    if file_path.name == "Dockerfile":
        return "dockerfile"

    return mapping.get(ext, "unknown")


def extract_rust_metadata(file_path: Path) -> dict:
    """Extract metadata from a Rust source file."""
    metadata = {
        "description": "",
        "structs": [],
        "enums": [],
        "traits": [],
        "functions": [],
        "tests": False,
        "lines": 0,
    }

    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
            lines = content.split('\n')
            metadata["lines"] = len(lines)

            # Extract module-level doc comments
            doc_lines = []
            for line in lines[:20]:
                if line.strip().startswith('//!'):
                    doc_lines.append(line.strip()[3:].strip())
                elif line.strip() and not line.strip().startswith('//'):
                    break

            if doc_lines:
                metadata["description"] = ' '.join(doc_lines)

            # Extract definitions
            metadata["structs"] = re.findall(r'pub\s+struct\s+(\w+)', content)
            metadata["enums"] = re.findall(r'pub\s+enum\s+(\w+)', content)
            metadata["traits"] = re.findall(r'pub\s+trait\s+(\w+)', content)
            metadata["functions"] = re.findall(r'pub\s+(?:async\s+)?fn\s+(\w+)', content)
            metadata["tests"] = '#[cfg(test)]' in content or '#[test]' in content

    except Exception as e:
        print(f"Warning: Could not read {file_path}: {e}")

    return metadata


def extract_generic_metadata(file_path: Path, file_type: str) -> dict:
    """Extract basic metadata from any file."""
    metadata = {"lines": 0, "size": 0}

    try:
        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
            content = f.read()
            metadata["lines"] = len(content.split('\n'))

        metadata["size"] = file_path.stat().st_size
    except Exception as e:
        print(f"Warning: Could not read {file_path}: {e}")

    return metadata


def generate_file_rst(file_path: Path, relative_path: Path, base_name: str) -> str:
    """Generate RST content for any source file with French explanations."""

    file_type = detect_file_type(file_path)

    # Title
    title = f"{relative_path}"
    rst = f"""{'=' * len(title)}
{title}
{'=' * len(title)}

:Fichier: ``{base_name}/{relative_path}``
:Type: {file_type.upper()}
"""

    # Extract metadata based on file type
    if file_type == "rust":
        metadata = extract_rust_metadata(file_path)
        rst += f":Lignes de Code: {metadata['lines']}\n"
        rst += f":Couche: {get_layer_from_path(file_path)}\n"
        rst += f":Tests: {'✅ Oui' if metadata['tests'] else '❌ Non'}\n"

        # French explanation
        french_explanation = generate_french_explanation(file_path, file_type, metadata)
        rst += f"\nÀ quoi sert ce fichier ?\n{'=' * 24}\n\n{french_explanation}\n\n"

        # Public API
        if metadata['structs'] or metadata['enums'] or metadata['traits'] or metadata['functions']:
            rst += """API Publique\n============\n\n"""

            if metadata['structs']:
                rst += """Structures\n----------\n\n"""
                for struct in metadata['structs']:
                    rst += f"- ``{struct}``\n"
                rst += "\n"

            if metadata['enums']:
                rst += """Énumérations\n------------\n\n"""
                for enum in metadata['enums']:
                    rst += f"- ``{enum}``\n"
                rst += "\n"

            if metadata['traits']:
                rst += """Traits\n------\n\n"""
                for trait in metadata['traits']:
                    rst += f"- ``{trait}``\n"
                rst += "\n"

            if metadata['functions']:
                rst += """Fonctions\n---------\n\n"""
                for func in metadata['functions'][:15]:
                    rst += f"- ``{func}()``\n"
                if len(metadata['functions']) > 15:
                    rst += f"\n*... et {len(metadata['functions']) - 15} autres fonctions*\n"
                rst += "\n"

    else:
        # Generic files
        metadata = extract_generic_metadata(file_path, file_type)
        rst += f":Lignes: {metadata['lines']}\n"
        rst += f":Taille: {metadata['size']} octets\n"

        # French explanation
        french_explanation = generate_french_explanation(file_path, file_type, metadata)
        rst += f"\nÀ quoi sert ce fichier ?\n{'=' * 24}\n\n{french_explanation}\n\n"

    # Source reference
    rst += f"""Code Source
===========

Voir: ``{base_name}/{relative_path}``

"""

    # Related documentation
    rst += """Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

"""

    return rst


def get_layer_from_path(path: Path) -> str:
    """Determine the architectural layer from the file path."""
    path_str = str(path)
    if 'domain/' in path_str:
        return 'Domain (Métier)'
    elif 'application/' in path_str:
        return 'Application (Use Cases)'
    elif 'infrastructure/' in path_str:
        return 'Infrastructure (Adaptateurs)'
    elif 'frontend/' in path_str:
        return 'Frontend (UI)'
    else:
        return 'Core'


def generate_index_rst(directory: Path, subdirs: List[str], files: List[str], base_path: Path, base_name: str) -> str:
    """Generate index.rst for a directory."""
    try:
        relative_to_base = directory.relative_to(base_path)
    except ValueError:
        relative_to_base = Path(directory.name)

    title = f"{relative_to_base} - Index"
    rst = f"""{'=' * len(title)}
{title}
{'=' * len(title)}

:Répertoire: ``{base_name}/{relative_to_base}/``
:Couche: {get_layer_from_path(directory)}

Vue d'Ensemble
==============

Ce répertoire contient **{len(files)} fichiers** et **{len(subdirs)} sous-répertoires**.

"""

    if subdirs:
        rst += """Sous-Répertoires
================

.. toctree::
   :maxdepth: 1
   :caption: Navigation

"""
        for subdir in sorted(subdirs):
            rst += f"   {subdir}/index\n"
        rst += "\n"

    if files:
        rst += """Fichiers
========

.. toctree::
   :maxdepth: 1
   :caption: Fichiers

"""
        for file in sorted(files):
            # Remove extension for RST reference
            file_stem = Path(file).stem
            rst += f"   {file_stem}\n"
        rst += "\n"

    return rst


def should_skip_file(file_path: Path) -> bool:
    """Check if file should be skipped."""
    skip_patterns = [
        '.git',
        'node_modules',
        '__pycache__',
        '.pytest_cache',
        'target',
        'dist',
        'build',
        '.next',
        '.DS_Store',
        '.env',
        'package-lock.json',
        'Cargo.lock',
    ]

    return any(pattern in str(file_path) for pattern in skip_patterns)


def process_directory(src_dir: Path, base_path: Path, docs_base: Path, base_name: str):
    """Process a directory and generate mirror documentation."""

    if should_skip_file(src_dir):
        return

    # Get relative path from base
    try:
        relative_path = src_dir.relative_to(base_path)
    except ValueError:
        return

    # Create corresponding docs directory
    docs_dir = docs_base / relative_path
    docs_dir.mkdir(parents=True, exist_ok=True)

    # Lists for index.rst
    subdirs = []
    files = []

    # Process all items
    try:
        items = sorted(src_dir.iterdir())
    except PermissionError:
        return

    for item in items:
        if should_skip_file(item):
            continue

        if item.is_file():
            file_type = detect_file_type(item)

            # Only document known file types
            if file_type != "unknown":
                # Generate .rst file
                rst_content = generate_file_rst(item, relative_path / item.name, base_name)
                rst_file = docs_dir / f"{item.stem}.rst"

                with open(rst_file, 'w', encoding='utf-8') as f:
                    f.write(rst_content)

                files.append(item.name)
                print(f"Généré: {rst_file.relative_to(PROJECT_ROOT)}")

        elif item.is_dir():
            subdirs.append(item.name)
            # Recursively process
            process_directory(item, base_path, docs_base, base_name)

    # Generate index.rst
    if subdirs or files:
        index_content = generate_index_rst(src_dir, subdirs, files, base_path, base_name)
        index_file = docs_dir / "index.rst"

        with open(index_file, 'w', encoding='utf-8') as f:
            f.write(index_content)

        print(f"Index généré: {index_file.relative_to(PROJECT_ROOT)}")


def generate_root_index():
    """Generate root index.rst for complete mirror documentation."""
    title = "Documentation Miroir Complète - KoproGo"
    rst = f"""{'=' * len(title)}
{title}
{'=' * len(title)}

Cette documentation **miroir** du projet KoproGo couvre l'intégralité du code source :

- ✅ **Backend** (Rust + Actix-web) - 321 fichiers
- ✅ **Frontend** (Astro + Svelte + TypeScript)
- ✅ **Infrastructure** (Ansible, Docker, Scripts)

**Principe** : Chaque fichier source a un fichier .rst correspondant avec **explication en français** de son rôle.

Architecture
============

Le projet suit une **architecture hexagonale** (Ports & Adapters) avec **Domain-Driven Design (DDD)**.

.. toctree::
   :maxdepth: 2
   :caption: Documentation par Composant

   backend/index
   frontend/index
   infrastructure/index

Couches Backend
===============

1. **Domain** (Métier)
   - Entités de domaine avec validation métier
   - Services de domaine (logique complexe)
   - Aucune dépendance externe

2. **Application** (Use Cases)
   - Use Cases (orchestration)
   - Ports (interfaces/traits)
   - DTOs (contrats API)

3. **Infrastructure** (Adaptateurs)
   - Repositories PostgreSQL
   - Handlers HTTP (Actix-web)
   - Clients API externes (Stripe, Linky)

Frontend
========

- **Astro** : SSG (Static Site Generation)
- **Svelte** : Composants interactifs (Islands)
- **TypeScript** : Type-safety

Infrastructure
==============

- **Ansible** : Déploiement VPS automatisé
- **Docker** : Conteneurisation services
- **GitHub Actions** : CI/CD pipelines

Liens Rapides
=============

- :doc:`/CLAUDE` - Guide développeur
- :doc:`/ARCHITECTURE` - Architecture détaillée
- :doc:`/NOUVELLES_FONCTIONNALITES_2025` - Features 2025
- :doc:`/IOT_INTEGRATION` - Intégration IoT

"""

    index_file = DOCS_MIRROR / "index.rst"
    with open(index_file, 'w', encoding='utf-8') as f:
        f.write(rst)

    print(f"Index racine généré: {index_file.relative_to(PROJECT_ROOT)}")


def generate_section_index(section_name: str, section_path: Path):
    """Generate section index (backend/frontend/infrastructure)."""

    descriptions = {
        "backend": """
Le backend KoproGo est développé en **Rust** avec le framework **Actix-web**.

**Architecture** : Hexagonale (Ports & Adapters) + DDD (Domain-Driven Design)

**Statistiques** :
- **321 fichiers Rust** (~50,000 lignes de code)
- **51 entités de domaine** avec validation métier
- **46 repositories PostgreSQL**
- **44 handlers HTTP** (73 endpoints API REST)
- **100+ migrations SQL**

**Performance** :
- P99 latency < 5ms
- Throughput > 100k req/s
- Memory < 128MB par instance
""",
        "frontend": """
Le frontend KoproGo utilise **Astro** (SSG) avec **Svelte** pour les composants interactifs.

**Architecture** : Islands Architecture (îlots interactifs dans pages statiques)

**Technologies** :
- **Astro 4.x** : Static Site Generation
- **Svelte 4.x** : Composants réactifs
- **TypeScript** : Type-safety
- **Tailwind CSS** : Styling utility-first

**Structure** :
- ``pages/`` : Pages Astro (SSG)
- ``components/`` : Composants Svelte réutilisables
- ``layouts/`` : Templates de mise en page
- ``stores/`` : State management (authStore, etc.)
""",
        "infrastructure": """
L'infrastructure KoproGo couvre le **déploiement**, **monitoring** et **sécurité**.

**Technologies** :
- **Ansible** : Automatisation déploiement VPS
- **Docker** : Conteneurisation (PostgreSQL, backend, frontend)
- **Prometheus + Grafana** : Monitoring métriques
- **Loki** : Agrégation logs
- **Suricata** : IDS (détection intrusions)
- **CrowdSec** : WAF collaboratif

**Fonctionnalités** :
- ✅ LUKS encryption at rest (AES-XTS-512)
- ✅ GPG encrypted backups (S3 off-site)
- ✅ fail2ban + SSH hardening
- ✅ Kernel hardening (sysctl)
- ✅ Security auditing (Lynis, rkhunter, AIDE)
"""
    }

    title = f"{section_name.capitalize()} - Index"
    rst = f"""{'=' * len(title)}
{title}
{'=' * len(title)}

{descriptions.get(section_name, f"Documentation du module {section_name}.")}

Contenu
=======

.. toctree::
   :maxdepth: 2

   src/index

"""

    section_dir = DOCS_MIRROR / section_name
    section_dir.mkdir(parents=True, exist_ok=True)

    index_file = section_dir / "index.rst"
    with open(index_file, 'w', encoding='utf-8') as f:
        f.write(rst)

    print(f"Index section {section_name} généré: {index_file.relative_to(PROJECT_ROOT)}")


def main():
    """Main entry point."""
    print("=" * 80)
    print("Génération Documentation Miroir Complète - Backend + Frontend + Infrastructure")
    print("=" * 80)
    print()

    # Create base directory
    DOCS_MIRROR.mkdir(parents=True, exist_ok=True)

    # Process each major section
    for section_name, section_path in MIRROR_DIRS.items():
        if not section_path.exists():
            print(f"⚠️  {section_name} non trouvé: {section_path}")
            continue

        print(f"\n📁 Traitement {section_name.upper()}...")
        print(f"   Source: {section_path}")

        docs_base = DOCS_MIRROR / section_name / "src"
        docs_base.mkdir(parents=True, exist_ok=True)

        # Process directory tree
        process_directory(section_path, section_path, docs_base, f"{section_name}/src")

        # Generate section index
        generate_section_index(section_name, section_path)

    # Generate root index
    print("\n📝 Génération index racine...")
    generate_root_index()

    print()
    print("=" * 80)
    print("✅ Documentation miroir complète générée !")
    print("=" * 80)
    print(f"Emplacement: {DOCS_MIRROR}")
    print()
    print("Pour visualiser :")
    print("  cd docs && make html")
    print()


if __name__ == "__main__":
    main()
