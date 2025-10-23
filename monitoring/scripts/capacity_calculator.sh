#!/bin/bash
################################################################################
# KoproGo Capacity Calculator
# Estimates how many copropriétés can be hosted on current VPS
################################################################################

set -euo pipefail

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Database connection
DB_URL="${DATABASE_URL:-postgresql://koprogo:koprogo123@localhost:5432/koprogo_db}"

echo -e "${BLUE}========================================"
echo "KoproGo Capacity Calculator"
echo "=======================================${NC}"
echo ""

# Check DB connection
if ! psql "$DB_URL" -c "SELECT 1;" &>/dev/null; then
    echo -e "${YELLOW}WARNING: Cannot connect to database${NC}"
    echo "Using theoretical estimates..."
    echo ""

    # Theoretical estimates
    echo -e "${GREEN}=== Theoretical Capacity ===${NC}"
    echo "Assumptions:"
    echo "  - Average copropriété: 10 lots, 8 owners, 50 expenses/year"
    echo "  - Data per copropriété/year: ~40 KB"
    echo ""
    echo "VPS: Hetzner CPX11 (2GB RAM, 40GB disk)"
    echo "  Available disk for data: 30 GB"
    echo "  30 GB / 40 KB = 750,000 copropriété-years"
    echo ""
    echo "Practical estimates:"
    echo "  - 500-1,000 small copropriétés (5-10 lots): COMFORTABLE"
    echo "  - 100-500 medium copropriétés (20-30 lots): GOOD"
    echo "  - 50-100 large copropriétés (50+ lots): OK"
    echo ""
    exit 0
fi

# Get actual data from database
echo -e "${GREEN}=== Current Database Statistics ===${NC}"

BUILDINGS=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM buildings;" | xargs)
UNITS=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM units;" | xargs)
OWNERS=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM owners;" | xargs)
EXPENSES=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM expenses;" | xargs)
MEETINGS=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM meetings;" | xargs)
DOCUMENTS=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM documents;" | xargs)

DB_SIZE_BYTES=$(psql "$DB_URL" -t -c "SELECT pg_database_size('koprogo_db');" | xargs)
DB_SIZE_PRETTY=$(psql "$DB_URL" -t -c "SELECT pg_size_pretty(pg_database_size('koprogo_db'));" | xargs)

echo "Copropriétés: $BUILDINGS"
echo "Units/Lots: $UNITS"
echo "Owners: $OWNERS"
echo "Expenses: $EXPENSES"
echo "Meetings: $MEETINGS"
echo "Documents: $DOCUMENTS"
echo ""
echo "Database size: $DB_SIZE_PRETTY"
echo ""

# Calculate averages
if [ "$BUILDINGS" -gt 0 ]; then
    AVG_UNITS=$(echo "scale=2; $UNITS / $BUILDINGS" | bc)
    AVG_OWNERS=$(echo "scale=2; $OWNERS / $BUILDINGS" | bc)
    AVG_EXPENSES=$(echo "scale=2; $EXPENSES / $BUILDINGS" | bc)
    BYTES_PER_BUILDING=$(echo "scale=0; $DB_SIZE_BYTES / $BUILDINGS" | bc)
    KB_PER_BUILDING=$(echo "scale=2; $BYTES_PER_BUILDING / 1024" | bc)

    echo -e "${GREEN}=== Averages per Copropriété ===${NC}"
    echo "Units/lots: $AVG_UNITS"
    echo "Owners: $AVG_OWNERS"
    echo "Expenses: $AVG_EXPENSES"
    echo "Data size: ${KB_PER_BUILDING} KB"
    echo ""
fi

# Capacity estimation
echo -e "${GREEN}=== Capacity Estimation ===${NC}"

AVAILABLE_DISK_GB=30
AVAILABLE_DISK_BYTES=$(echo "$AVAILABLE_DISK_GB * 1024 * 1024 * 1024" | bc)

if [ "$BUILDINGS" -gt 0 ] && [ "$DB_SIZE_BYTES" -gt 0 ]; then
    ESTIMATED_MAX=$(echo "scale=0; $AVAILABLE_DISK_BYTES / ($DB_SIZE_BYTES / $BUILDINGS)" | bc)

    echo "Available disk space: ${AVAILABLE_DISK_GB}GB"
    echo "Current growth rate: ${KB_PER_BUILDING}KB per copropriété"
    echo ""
    echo "Estimated maximum copropriétés: $ESTIMATED_MAX"

    # Current usage percentage
    CURRENT_PCT=$(echo "scale=2; ($BUILDINGS * 100) / $ESTIMATED_MAX" | bc)
    echo "Current capacity usage: ${CURRENT_PCT}%"
    echo ""

    # Upgrade recommendations
    echo -e "${GREEN}=== Upgrade Recommendations ===${NC}"

    if [ "$BUILDINGS" -lt 100 ]; then
        echo "Current tier: Hetzner CPX11 (4.15€/month) - OPTIMAL"
        echo "Upgrade to CPX21 at: ~100 copropriétés"
    elif [ "$BUILDINGS" -lt 500 ]; then
        echo -e "${YELLOW}Consider upgrade: Hetzner CPX21 (8.25€/month)${NC}"
        echo "  - 4GB RAM, 80GB disk"
        echo "  - Capacity: ~500-1000 copropriétés"
    elif [ "$BUILDINGS" -lt 2000 ]; then
        echo -e "${YELLOW}Recommended: Hetzner CPX31 (16.50€/month)${NC}"
        echo "  - 8GB RAM, 160GB disk"
        echo "  - Capacity: ~2000-5000 copropriétés"
    else
        echo -e "${YELLOW}Time for dedicated infrastructure!${NC}"
        echo "  - Separate database server"
        echo "  - Load balancer + multiple app servers"
        echo "  - Estimated cost: 50-100€/month"
    fi
else
    echo "No data in database yet. Run seeder or add data to get accurate estimates."
    echo ""
    echo "Theoretical capacity: 500-1,000 small copropriétés"
fi

echo ""

# RAM estimation
echo -e "${GREEN}=== Memory Capacity ===${NC}"
TOTAL_RAM_MB=2048
SYSTEM_RAM_MB=200
POSTGRES_RAM_MB=768
BACKEND_RAM_MB=300
AVAILABLE_RAM_MB=$(echo "$TOTAL_RAM_MB - $SYSTEM_RAM_MB - $POSTGRES_RAM_MB - $BACKEND_RAM_MB" | bc)

echo "Total RAM: ${TOTAL_RAM_MB}MB"
echo "Estimated usage:"
echo "  - System: ${SYSTEM_RAM_MB}MB"
echo "  - PostgreSQL: ${POSTGRES_RAM_MB}MB"
echo "  - Backend (Rust): ${BACKEND_RAM_MB}MB"
echo "  - Available: ${AVAILABLE_RAM_MB}MB"
echo ""

if [ "$AVAILABLE_RAM_MB" -lt 200 ]; then
    echo -e "${YELLOW}WARNING: Low RAM headroom. Consider upgrade.${NC}"
elif [ "$AVAILABLE_RAM_MB" -lt 500 ]; then
    echo "RAM headroom: OK"
else
    echo "RAM headroom: COMFORTABLE"
fi

echo ""
echo -e "${BLUE}=======================================${NC}"
