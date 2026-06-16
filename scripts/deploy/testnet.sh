#!/usr/bin/env bash
set -euo pipefail

# AgroLedger Testnet Deploy (Shell Version)
# Usage: ./scripts/deploy/testnet.sh
# Prerequisites: soroban-cli, funded testnet account, .env with DEPLOYER_SECRET_KEY

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$SCRIPT_DIR/../.."
WASM_DIR="$ROOT/target/wasm32-unknown-unknown/release"
RPC_URL="https://soroban-testnet.stellar.org"
NETWORK_PASSPHRASE="Test SDF Network ; September 2015"

# Load .env if present
if [ -f "$ROOT/.env" ]; then
  export $(grep -v '^\s*#' "$ROOT/.env" | grep -v '^\s*$' | xargs)
fi

echo "=== AgroLedger Testnet Deploy ==="
echo ""

# Check prerequisites
if ! command -v soroban &>/dev/null; then
  echo "soroban CLI is required. Install with: cargo install soroban-cli"
  exit 1
fi

if [ -z "${DEPLOYER_SECRET_KEY:-}" ]; then
  read -rp "Enter deployer secret key: " DEPLOYER_SECRET_KEY
fi
if [ -z "${ADMIN_ADDRESS:-}" ]; then
  read -rp "Enter admin public address: " ADMIN_ADDRESS
fi
if [ -z "${USDC_CONTRACT_ID:-}" ]; then
  read -rp "Enter USDC contract ID (testnet default: CCW67TSZV3SSS2HXMBQ5JFGCKJNXKZM7UQUWUZPUTHXSTZLEO7SJMI75): " USDC_CONTRACT_ID
fi

echo ""
echo "Building contracts..."
cargo build --workspace --target wasm32-unknown-unknown --release --manifest-path "$ROOT/Cargo.toml"

CONTRACTS=(
  "privacy_passport"
  "compliance_registry"
  "warehouse_oracle"
  "crop_token"
  "collateral_vault"
  "cross_border_router"
  "commodity_amm"
  "harvest_vault"
  "forward_hedge"
)

declare -A IDS

deploy() {
  local name="$1"
  local wasm="$WASM_DIR/${name}.wasm"
  if [ ! -f "$wasm" ]; then
    echo "ERROR: $wasm not found" >&2
    exit 1
  fi
  echo "--- Deploying $name ---"
  local id
  id=$(soroban contract deploy \
    --wasm "$wasm" \
    --source "$DEPLOYER_SECRET_KEY" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE")
  IDS[$name]="$id"
  echo "  $name: $id"
}

invoke() {
  local contract="$1"
  local fn="$2"
  shift 2
  local id="${IDS[$contract]}"
  echo "  Calling ${contract}.${fn}()"
  soroban contract invoke \
    --id "$id" \
    --source "$DEPLOYER_SECRET_KEY" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    --fn "$fn" "$@" 2>/dev/null || echo "  (warning: ${fn} may have failed)"
}

for c in "${CONTRACTS[@]}"; do
  deploy "$c"
done

echo ""
echo "=== Initializing contracts ==="

invoke "privacy_passport" "initialize" "--arg" "$ADMIN_ADDRESS"
invoke "compliance_registry" "initialize" "--arg" "$ADMIN_ADDRESS" "--arg" "${IDS[privacy_passport]}"
invoke "warehouse_oracle" "initialize" "--arg" "$ADMIN_ADDRESS" "--arg" "$ADMIN_ADDRESS" "--arg" '{"inspectors":[],"threshold":0}'
invoke "crop_token" "initialize" "--arg" "$ADMIN_ADDRESS" "--arg" "${IDS[warehouse_oracle]}" "--arg" "${IDS[compliance_registry]}"
invoke "collateral_vault" "initialize" "--arg" "$ADMIN_ADDRESS" "--arg" "${IDS[compliance_registry]}" "--arg" "$USDC_CONTRACT_ID" "--arg" "${IDS[warehouse_oracle]}"
invoke "cross_border_router" "initialize" "--arg" "$ADMIN_ADDRESS" "--arg" "${IDS[compliance_registry]}"
invoke "commodity_amm" "initialize" "--arg" "$ADMIN_ADDRESS" "--arg" "${IDS[crop_token]}" "--arg" "$USDC_CONTRACT_ID"
invoke "harvest_vault" "initialize" "--arg" "$ADMIN_ADDRESS" "--arg" "${IDS[crop_token]}" "--arg" "${IDS[commodity_amm]}" "--arg" "$USDC_CONTRACT_ID"
invoke "forward_hedge" "initialize" "--arg" "$ADMIN_ADDRESS" "--arg" "${IDS[crop_token]}" "--arg" "${IDS[collateral_vault]}"

# Write .env
cat > "$ROOT/.env" << ENVEOF
# ── Stellar Network (Testnet) ───────────────────────────────────────────
STELLAR_NETWORK=testnet
STELLAR_RPC_URL=$RPC_URL
STELLAR_HORIZON_URL=https://horizon-testnet.stellar.org
STELLAR_NETWORK_PASSPHRASE="$NETWORK_PASSPHRASE"

# ── Deployer / Admin Keys ──────────────────────────────────────────────────
DEPLOYER_SECRET_KEY=$DEPLOYER_SECRET_KEY
ADMIN_ADDRESS=$ADMIN_ADDRESS

# ── Deployed Contract IDs ──────────────────────────────────────────────────
CROP_TOKEN_CONTRACT_ID=${IDS[crop_token]}
COLLATERAL_VAULT_CONTRACT_ID=${IDS[collateral_vault]}
COMPLIANCE_REGISTRY_CONTRACT_ID=${IDS[compliance_registry]}
CROSS_BORDER_ROUTER_CONTRACT_ID=${IDS[cross_border_router]}
WAREHOUSE_ORACLE_CONTRACT_ID=${IDS[warehouse_oracle]}
HARVEST_VAULT_CONTRACT_ID=${IDS[harvest_vault]}
COMMODITY_AMM_CONTRACT_ID=${IDS[commodity_amm]}
FORWARD_HEDGE_CONTRACT_ID=${IDS[forward_hedge]}
PRIVACY_PASSPORT_CONTRACT_ID=${IDS[privacy_passport]}

# ── Stablecoin Asset IDs ──────────────────────────────────────────────────
USDC_CONTRACT_ID=$USDC_CONTRACT_ID
CNGN_CONTRACT_ID=
CXOF_CONTRACT_ID=
CGHS_CONTRACT_ID=
CKES_CONTRACT_ID=

# ── Oracle Sidecar ──────────────────────────────────────────────────────────
ORACLE_SIGNING_SECRET=
ORACLE_MULTISIG_THRESHOLD=3
AFEX_API_KEY=
AFEX_API_URL=https://api.afexnigeria.com/v1
GCX_API_KEY=
CME_API_KEY=
PRICE_FEED_INTERVAL_SECONDS=300

# ── Compliance / KYC ──────────────────────────────────────────────────────
KYC_PROVIDER=reclaim
RECLAIM_APP_ID=
RECLAIM_APP_SECRET=
VERITE_ISSUER_DID=
FATF_TRAVEL_RULE_THRESHOLD_USD=10000

# ── Indexer ──────────────────────────────────────────────────────────────
DATABASE_URL=postgresql://agroledger:password@localhost:5432/agroledger
INDEXER_START_LEDGER=0

# ── USSD Gateway ──────────────────────────────────────────────────────────
AFRICASTALKING_API_KEY=
AFRICASTALKING_USERNAME=sandbox
USSD_SERVICE_CODE=*384#

# ── App ────────────────────────────────────────────────────────────────
PORT=3000
LOG_LEVEL=info
ENVEOF

echo ""
echo ".env written."
echo ""
echo "=== Deploy complete! ==="
echo "Run seed scripts to populate test data."
