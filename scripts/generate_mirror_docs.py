#!/usr/bin/env python3
"""
Generate mirror RST documentation for the entire backend/src codebase.

This script creates a mirrored documentation structure in docs/backend-mirror/
where each .rs file has a corresponding .rst file, and each directory has an index.rst.
"""

import os
import re
from pathlib import Path
from typing import List, Tuple

# Base paths
PROJECT_ROOT = Path(__file__).parent.parent
BACKEND_SRC = PROJECT_ROOT / "backend" / "src"
DOCS_MIRROR = PROJECT_ROOT / "docs" / "backend-mirror"


def extract_file_metadata(file_path: Path) -> dict:
    """Extract metadata from a Rust source file."""
    metadata = {
        "description": "",
        "structs": [],
        "enums": [],
        "traits": [],
        "functions": [],
        "pub_items": [],
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
            for line in lines[:20]:  # Check first 20 lines
                if line.strip().startswith('//!'):
                    doc_lines.append(line.strip()[3:].strip())
                elif line.strip() and not line.strip().startswith('//'):
                    break

            if doc_lines:
                metadata["description"] = ' '.join(doc_lines)

            # Extract struct definitions
            metadata["structs"] = re.findall(r'pub\s+struct\s+(\w+)', content)

            # Extract enum definitions
            metadata["enums"] = re.findall(r'pub\s+enum\s+(\w+)', content)

            # Extract trait definitions
            metadata["traits"] = re.findall(r'pub\s+trait\s+(\w+)', content)

            # Extract public function definitions
            metadata["functions"] = re.findall(r'pub\s+(?:async\s+)?fn\s+(\w+)', content)

            # Check for tests
            metadata["tests"] = '#[cfg(test)]' in content or '#[test]' in content

    except Exception as e:
        print(f"Warning: Could not read {file_path}: {e}")

    return metadata


def generate_file_rst(rust_file: Path, relative_path: Path) -> str:
    """Generate RST content for a Rust source file."""
    metadata = extract_file_metadata(rust_file)

    # Title (underlined with =)
    title = f"{relative_path}"
    rst = f"""{'=' * len(title)}
{title}
{'=' * len(title)}

:File: ``{relative_path}``
:Lines of Code: {metadata['lines']}
:Layer: {get_layer_from_path(relative_path)}
:Has Tests: {'✅ Yes' if metadata['tests'] else '❌ No'}

"""

    # Description
    if metadata['description']:
        rst += f"""Description
===========

{metadata['description']}

"""

    # Public API
    if metadata['structs'] or metadata['enums'] or metadata['traits'] or metadata['functions']:
        rst += """Public API
==========

"""

    if metadata['structs']:
        rst += """Structures
----------

"""
        for struct in metadata['structs']:
            rst += f"- ``{struct}``\n"
        rst += "\n"

    if metadata['enums']:
        rst += """Enumerations
------------

"""
        for enum in metadata['enums']:
            rst += f"- ``{enum}``\n"
        rst += "\n"

    if metadata['traits']:
        rst += """Traits
------

"""
        for trait in metadata['traits']:
            rst += f"- ``{trait}``\n"
        rst += "\n"

    if metadata['functions']:
        rst += """Functions
---------

"""
        for func in metadata['functions'][:10]:  # Limit to first 10
            rst += f"- ``{func}()``\n"
        if len(metadata['functions']) > 10:
            rst += f"\n*... and {len(metadata['functions']) - 10} more functions*\n"
        rst += "\n"

    # Source reference
    rst += f"""Source Code
===========

See: ``backend/src/{relative_path}``

"""

    # Related documentation
    rst += """Related Documentation
=====================

.. seealso::

   - :doc:`/ARCHITECTURE`
   - :doc:`/CLAUDE`

"""

    return rst


def get_layer_from_path(path: Path) -> str:
    """Determine the architectural layer from the file path."""
    path_str = str(path)
    if 'domain/' in path_str:
        return 'Domain'
    elif 'application/' in path_str:
        return 'Application'
    elif 'infrastructure/' in path_str:
        return 'Infrastructure'
    else:
        return 'Core'


def generate_index_rst(directory: Path, subdirs: List[str], files: List[str]) -> str:
    """Generate index.rst for a directory."""
    dir_name = directory.name
    relative_to_src = directory.relative_to(BACKEND_SRC)

    title = f"{relative_to_src} - Index"
    rst = f"""{'=' * len(title)}
{title}
{'=' * len(title)}

:Path: ``backend/src/{relative_to_src}/``
:Layer: {get_layer_from_path(relative_to_src)}

Overview
========

This directory contains {len(files)} source files and {len(subdirs)} subdirectories.

"""

    if subdirs:
        rst += """Subdirectories
==============

.. toctree::
   :maxdepth: 1
   :caption: Subdirectories

"""
        for subdir in sorted(subdirs):
            rst += f"   {subdir}/index\n"
        rst += "\n"

    if files:
        rst += """Source Files
============

.. toctree::
   :maxdepth: 1
   :caption: Files

"""
        for file in sorted(files):
            # Remove .rs extension for RST reference
            file_stem = file.replace('.rs', '')
            rst += f"   {file_stem}\n"
        rst += "\n"

    return rst


def process_directory(src_dir: Path):
    """Process a directory and generate mirror documentation."""
    # Get relative path from backend/src
    try:
        relative_path = src_dir.relative_to(BACKEND_SRC)
    except ValueError:
        print(f"Skipping {src_dir} (not under backend/src)")
        return

    # Create corresponding docs directory
    docs_dir = DOCS_MIRROR / "src" / relative_path
    docs_dir.mkdir(parents=True, exist_ok=True)

    # Lists for index.rst
    subdirs = []
    files = []

    # Process all items in this directory
    for item in sorted(src_dir.iterdir()):
        if item.is_file() and item.suffix == '.rs':
            # Generate .rst file
            rst_content = generate_file_rst(item, relative_path / item.name)
            rst_file = docs_dir / f"{item.stem}.rst"

            with open(rst_file, 'w', encoding='utf-8') as f:
                f.write(rst_content)

            files.append(item.name)
            print(f"Generated: {rst_file.relative_to(PROJECT_ROOT)}")

        elif item.is_dir() and not item.name.startswith('.'):
            subdirs.append(item.name)
            # Recursively process subdirectory
            process_directory(item)

    # Generate index.rst for this directory
    if subdirs or files:
        index_content = generate_index_rst(src_dir, subdirs, files)
        index_file = docs_dir / "index.rst"

        with open(index_file, 'w', encoding='utf-8') as f:
            f.write(index_content)

        print(f"Generated index: {index_file.relative_to(PROJECT_ROOT)}")


def generate_root_index():
    """Generate root index.rst for backend-mirror documentation."""
    title = "Backend Source Code Documentation (Mirror)"
    rst = f"""{'=' * len(title)}
{title}
{'=' * len(title)}

This documentation mirrors the entire backend source code structure,
providing detailed documentation for each file.

**Total Files**: 321 Rust source files

**Architecture**: Hexagonal (Ports & Adapters) + Domain-Driven Design (DDD)

Directory Structure
===================

.. toctree::
   :maxdepth: 2
   :caption: Source Code Mirror

   src/index

Layers
======

The backend follows a strict 3-layer architecture:

1. **Domain Layer** (``domain/``)
   - Pure business logic
   - No external dependencies
   - Entities with invariant validation
   - Domain services

2. **Application Layer** (``application/``)
   - Use cases (orchestration logic)
   - Ports (trait definitions)
   - DTOs (data transfer objects)
   - Services (application-level orchestration)

3. **Infrastructure Layer** (``infrastructure/``)
   - Database repositories (PostgreSQL)
   - Web handlers (Actix-web)
   - External integrations (Stripe, Linky, etc.)
   - Storage adapters

Quick Links
===========

- :doc:`/ARCHITECTURE` - Architecture documentation
- :doc:`/CLAUDE` - Project overview and commands
- :doc:`/NOUVELLES_FONCTIONNALITES_2025` - 2025 features documentation

"""

    index_file = DOCS_MIRROR / "index.rst"
    with open(index_file, 'w', encoding='utf-8') as f:
        f.write(rst)

    print(f"Generated root index: {index_file.relative_to(PROJECT_ROOT)}")


def main():
    """Main entry point."""
    print("=" * 80)
    print("Generating Mirror Documentation for Backend Source Code")
    print("=" * 80)
    print(f"Source: {BACKEND_SRC}")
    print(f"Target: {DOCS_MIRROR}")
    print()

    # Create base directory
    DOCS_MIRROR.mkdir(parents=True, exist_ok=True)

    # Process entire backend/src directory
    print("Processing directories...")
    process_directory(BACKEND_SRC)

    # Generate root index
    print("\nGenerating root index...")
    generate_root_index()

    print()
    print("=" * 80)
    print("Mirror documentation generation complete!")
    print("=" * 80)
    print(f"Generated files in: {DOCS_MIRROR}")
    print()
    print("To view the documentation:")
    print("  cd docs && make html && open _build/html/backend-mirror/index.html")
    print()


if __name__ == "__main__":
    main()
