#!/bin/bash
# Validation complète du seed contre le schéma de base de données

echo "=== VALIDATION DU SEED CONTRE LE SCHÉMA ==="
echo ""

# Extraire toutes les tables du seed
echo "1. Tables utilisées dans le seed:"
grep -o "INSERT INTO [a-z_]*" src/infrastructure/database/seed.rs | sort -u
echo ""

# Vérifier chaque INSERT
echo "2. Vérification des colonnes pour chaque table:"
echo ""

# USERS
echo "--- TABLE: users ---"
echo "Migration:"
grep -A 15 "CREATE TABLE users" migrations/*.sql | grep -v "^--" | head -20
echo ""
echo "Seed:"
grep "INSERT INTO users" src/infrastructure/database/seed.rs | head -2
echo ""

# BUILDINGS  
echo "--- TABLE: buildings ---"
echo "Migration:"
grep -A 15 "CREATE TABLE buildings" migrations/*.sql | grep -v "^--" | head -20
echo ""
echo "Seed:"
grep "INSERT INTO buildings" src/infrastructure/database/seed.rs | head -2
echo ""

# OWNERS
echo "--- TABLE: owners ---"
echo "Migration (original):"
cat migrations/20240101000002_create_owners.sql
echo ""
echo "Migration (organization_id added):"
grep -A 2 "ALTER TABLE owners" migrations/*.sql 2>/dev/null || echo "Checking..."
echo ""
echo "Seed:"
grep "INSERT INTO owners" src/infrastructure/database/seed.rs | head -2
echo ""

# UNITS
echo "--- TABLE: units ---"
echo "Migration:"
grep -A 20 "CREATE TABLE units" migrations/*.sql | grep -v "^--" | head -25
echo ""
echo "Seed:"
grep "INSERT INTO units" src/infrastructure/database/seed.rs | head -2
echo ""

# EXPENSES
echo "--- TABLE: expenses ---"
echo "Migration:"
grep -A 20 "CREATE TABLE expenses" migrations/*.sql | grep -v "^--" | head -25
echo ""
echo "Seed:"
grep "INSERT INTO expenses" src/infrastructure/database/seed.rs | head -2
echo ""
