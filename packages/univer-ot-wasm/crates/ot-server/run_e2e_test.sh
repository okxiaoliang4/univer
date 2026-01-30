#!/bin/bash

# E2E Changeset Test Runner with Memory Monitoring
# This script runs the e2e changeset tests against a running ot-server instance

set -e

# Default configuration
OT_SERVER_URL="${OT_SERVER_URL:-http://localhost:3000}"
OT_SOCKETIO_URL="${OT_SOCKETIO_URL:-http://localhost:3000}"
CELL_COUNT="${CELL_COUNT:-20000}"
MUTATION_COUNT="${MUTATION_COUNT:-3}"
TEST_NAME="${TEST_NAME:-test_e2e_changeset_with_memory_monitoring}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔═══════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  OT Server E2E Changeset Test Runner                  ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════╝${NC}"
echo ""

# Print configuration
echo -e "${YELLOW}📋 Test Configuration:${NC}"
echo -e "   Server URL:        ${GREEN}${OT_SERVER_URL}${NC}"
echo -e "   Socket.IO URL:     ${GREEN}${OT_SOCKETIO_URL}${NC}"
echo -e "   Cells per mutation:${GREEN}${CELL_COUNT}${NC}"
echo -e "   Mutation count:    ${GREEN}${MUTATION_COUNT}${NC}"
echo -e "   Test name:         ${GREEN}${TEST_NAME}${NC}"
echo ""

# Check if server is running
echo -e "${YELLOW}🔍 Checking if OT server is running...${NC}"
if curl -s -f "${OT_SERVER_URL}/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Server is running${NC}"
else
    echo -e "${RED}❌ Server is not responding at ${OT_SERVER_URL}${NC}"
    echo -e "${YELLOW}Please start the ot-server first:${NC}"
    echo -e "   cd packages/univer-ot-wasm/crates/ot-server"
    echo -e "   cargo run --release"
    exit 1
fi

echo ""
echo -e "${YELLOW}🚀 Running tests...${NC}"
echo ""

# Export environment variables for the test
export OT_SERVER_URL
export OT_SOCKETIO_URL
export CELL_COUNT
export MUTATION_COUNT

# Run the test
if cargo test --test e2e_changeset "${TEST_NAME}" -- --ignored --nocapture; then
    echo ""
    echo -e "${GREEN}╔═══════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║  Test Passed Successfully! ✅                         ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════╝${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}╔═══════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║  Test Failed! ❌                                      ║${NC}"
    echo -e "${RED}╚═══════════════════════════════════════════════════════╝${NC}"
    exit 1
fi
