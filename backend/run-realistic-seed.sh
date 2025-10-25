#!/bin/bash
################################################################################
# Helper script to run the realistic seed
# Generates ~190 total entities: 3 orgs, 23 buildings, 190 units, ~127 owners, ~60 expenses
################################################################################

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}=========================================${NC}"
echo -e "${BLUE}🌱 KoproGo Realistic Seed Generator${NC}"
echo -e "${BLUE}=========================================${NC}"
echo ""
echo "This will generate realistic test data for a 1 vCPU / 2GB RAM server:"
echo ""
echo "  📊 Target data volume:"
echo "    - 3 organizations (small, medium, large)"
echo "    - 23 buildings total"
echo "    - ~190 units total"
echo "    - ~127 owners total"
echo "    - ~60 expenses total"
echo ""
echo -e "${YELLOW}⚠️  WARNING: This will DELETE all existing demo data!${NC}"
echo ""
read -p "Continue? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Cancelled."
    exit 0
fi

echo ""
echo -e "${GREEN}Building seed binary...${NC}"
SQLX_OFFLINE=true cargo build --bin seed_realistic --release

echo ""
echo -e "${GREEN}Running seed...${NC}"
cargo run --bin seed_realistic --release

echo ""
echo -e "${GREEN}✅ Seed complete!${NC}"
echo ""
echo "You can now test with the credentials:"
echo "  Small org:  admin@small.be / admin123"
echo "  Medium org: admin@medium.be / admin123"
echo "  Large org:  admin@large.be / admin123"
echo ""
echo "Run load tests with:"
echo "  cd ../load-tests"
echo "  export BASE_URL=https://api2.koprogo.com"
echo "  ./scripts/realistic-load.sh"
