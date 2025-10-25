#!/bin/bash
# Synchronise la structure docs/ avec le projet réel
# Usage: .claude/scripts/sync-docs-structure.sh

set -e

echo "🔄 Synchronizing docs/ structure with real project..."

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$PROJECT_ROOT"

# Fonction pour créer un fichier RST miroir
create_rst_mirror() {
    local rs_file=$1
    local rst_file=$2
    local title=$3

    mkdir -p "$(dirname "$rst_file")"

    if [ ! -f "$rst_file" ]; then
        echo "  📝 Creating $rst_file"
        cat > "$rst_file" <<EOF
$title
$(printf '=%.0s' $(seq 1 ${#title}))

Auto-generated documentation mirror for \`\`$rs_file\`\`.

.. note::
   This file mirrors the structure of the Rust source code.
   For detailed API documentation, see the Rust API docs.

Overview
--------

[TODO: Add overview of this module]

Source
------

**Location**: \`\`$rs_file\`\`

**Last synced**: $(date +"%Y-%m-%d %H:%M:%S")
EOF
    else
        # Update "Last synced" timestamp
        sed -i "s/\*\*Last synced\*\*: .*/\*\*Last synced\*\*: $(date +"%Y-%m-%d %H:%M:%S")/" "$rst_file"
    fi
}

# Fonction pour convertir snake_case en Title Case
to_title_case() {
    echo "$1" | sed 's/_/ /g' | sed 's/\b\(.\)/\u\1/g'
}

# Synchroniser les entités Domain
echo "📦 Syncing domain entities..."
for entity_file in backend/src/domain/entities/*.rs; do
    [ "$(basename "$entity_file")" = "mod.rs" ] && continue

    entity_name=$(basename "$entity_file" .rs)
    rst_file="docs/backend/src/domain/entities/$entity_name.rst"
    title="$(to_title_case "$entity_name") Entity"

    create_rst_mirror "$entity_file" "$rst_file" "$title"
done

# Synchroniser les services Domain
echo "🔧 Syncing domain services..."
mkdir -p docs/backend/src/domain/services
for service_file in backend/src/domain/services/*.rs; do
    [ "$(basename "$service_file")" = "mod.rs" ] && continue

    service_name=$(basename "$service_file" .rs)
    rst_file="docs/backend/src/domain/services/$service_name.rst"
    title="$(to_title_case "$service_name") Service"

    create_rst_mirror "$service_file" "$rst_file" "$title"
done

# Synchroniser les Use Cases
echo "⚙️  Syncing application use cases..."
mkdir -p docs/backend/src/application/use_cases
for uc_file in backend/src/application/use_cases/*.rs; do
    [ "$(basename "$uc_file")" = "mod.rs" ] && continue

    uc_name=$(basename "$uc_file" .rs)
    rst_file="docs/backend/src/application/use_cases/$uc_name.rst"
    title="$(to_title_case "$uc_name")"

    create_rst_mirror "$uc_file" "$rst_file" "$title"
done

# Synchroniser les Ports
echo "🔌 Syncing application ports..."
mkdir -p docs/backend/src/application/ports
for port_file in backend/src/application/ports/*.rs; do
    [ "$(basename "$port_file")" = "mod.rs" ] && continue

    port_name=$(basename "$port_file" .rs)
    rst_file="docs/backend/src/application/ports/$port_name.rst"
    title="$(to_title_case "$port_name")"

    create_rst_mirror "$port_file" "$rst_file" "$title"
done

# Synchroniser les DTOs
echo "📋 Syncing application DTOs..."
mkdir -p docs/backend/src/application/dto
for dto_file in backend/src/application/dto/*.rs; do
    [ "$(basename "$dto_file")" = "mod.rs" ] && continue

    dto_name=$(basename "$dto_file" .rs)
    rst_file="docs/backend/src/application/dto/$dto_name.rst"
    title="$(to_title_case "$dto_name")"

    create_rst_mirror "$dto_file" "$rst_file" "$title"
done

# Synchroniser les Repositories
echo "🗄️  Syncing infrastructure repositories..."
mkdir -p docs/backend/src/infrastructure/database/repositories
for repo_file in backend/src/infrastructure/database/repositories/*.rs; do
    [ "$(basename "$repo_file")" = "mod.rs" ] && continue

    repo_name=$(basename "$repo_file" .rs)
    rst_file="docs/backend/src/infrastructure/database/repositories/$repo_name.rst"
    title="$(to_title_case "$repo_name")"

    create_rst_mirror "$repo_file" "$rst_file" "$title"
done

# Synchroniser les Handlers
echo "🌐 Syncing infrastructure web handlers..."
mkdir -p docs/backend/src/infrastructure/web/handlers
for handler_file in backend/src/infrastructure/web/handlers/*.rs; do
    [ "$(basename "$handler_file")" = "mod.rs" ] && continue

    handler_name=$(basename "$handler_file" .rs)
    rst_file="docs/backend/src/infrastructure/web/handlers/$handler_name.rst"
    title="$(to_title_case "$handler_name")"

    create_rst_mirror "$handler_file" "$rst_file" "$title"
done

# Générer le fichier PROJECT_STRUCTURE.md
echo "📊 Generating PROJECT_STRUCTURE.md..."
cat > docs/PROJECT_STRUCTURE.md <<EOF
# KoproGo Project Structure

**Last updated**: $(date +"%Y-%m-%d %H:%M:%S")

This document provides an overview of the KoproGo project structure, automatically generated
from the actual codebase.

## Root Structure

\`\`\`
$(tree -L 2 -I 'target|node_modules|.git|_build|.venv|__pycache__|.pytest_cache' . | head -50)
...
\`\`\`

## Backend Structure (Hexagonal Architecture)

### Domain Layer

The core business logic with no external dependencies.

\`\`\`
$(tree backend/src/domain -I 'target')
\`\`\`

**Entities**: $(ls backend/src/domain/entities/*.rs 2>/dev/null | grep -v mod.rs | wc -l) entities
**Services**: $(ls backend/src/domain/services/*.rs 2>/dev/null | grep -v mod.rs | wc -l) domain services

### Application Layer

Use cases and port definitions (interfaces).

\`\`\`
$(tree backend/src/application -I 'target')
\`\`\`

**Use Cases**: $(ls backend/src/application/use_cases/*.rs 2>/dev/null | grep -v mod.rs | wc -l) use cases
**Ports**: $(ls backend/src/application/ports/*.rs 2>/dev/null | grep -v mod.rs | wc -l) ports
**DTOs**: $(ls backend/src/application/dto/*.rs 2>/dev/null | grep -v mod.rs | wc -l) DTOs

### Infrastructure Layer

Adapters implementing the ports.

\`\`\`
$(tree backend/src/infrastructure -I 'target' -L 3)
\`\`\`

**Repositories**: $(find backend/src/infrastructure/database/repositories -name '*.rs' 2>/dev/null | grep -v mod.rs | wc -l) repository implementations
**Handlers**: $(find backend/src/infrastructure/web/handlers -name '*.rs' 2>/dev/null | grep -v mod.rs | wc -l) HTTP handlers

## Frontend Structure

\`\`\`
$(tree frontend/src -L 3 -I 'node_modules')
\`\`\`

## Tests Structure

\`\`\`
$(tree backend/tests -L 2 -I 'target')
\`\`\`

## Documentation Structure

\`\`\`
$(tree docs -L 2 -I '_build|.venv|__pycache__')
\`\`\`

---

*This file is automatically generated by \`.claude/scripts/sync-docs-structure.sh\`*
EOF

echo ""
echo "✅ Documentation structure synchronized!"
echo ""
echo "Summary:"
echo "  • Domain entities: $(ls backend/src/domain/entities/*.rs 2>/dev/null | grep -v mod.rs | wc -l)"
echo "  • Domain services: $(ls backend/src/domain/services/*.rs 2>/dev/null | grep -v mod.rs | wc -l)"
echo "  • Use cases: $(ls backend/src/application/use_cases/*.rs 2>/dev/null | grep -v mod.rs | wc -l)"
echo "  • Ports: $(ls backend/src/application/ports/*.rs 2>/dev/null | grep -v mod.rs | wc -l)"
echo "  • DTOs: $(ls backend/src/application/dto/*.rs 2>/dev/null | grep -v mod.rs | wc -l)"
echo "  • Repositories: $(find backend/src/infrastructure/database/repositories -name '*.rs' 2>/dev/null | grep -v mod.rs | wc -l)"
echo "  • Handlers: $(find backend/src/infrastructure/web/handlers -name '*.rs' 2>/dev/null | grep -v mod.rs | wc -l)"
echo ""
echo "📝 Next steps:"
echo "  1. Review generated RST files in docs/backend/src/"
echo "  2. Fill in TODO sections with actual documentation"
echo "  3. Rebuild Sphinx docs: make docs-sphinx"
echo "  4. Commit changes"
