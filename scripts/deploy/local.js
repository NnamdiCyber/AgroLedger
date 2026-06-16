#!/usr/bin/env node
const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");
const readline = require("readline");

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
});

function prompt(q) {
  return new Promise((r) => rl.question(q, r));
}

const ROOT = path.resolve(__dirname, "../..");
const WASM_DIR = path.join(
  ROOT,
  "target/wasm32-unknown-unknown/release"
);

const CONTRACTS = [
  "privacy_passport",
  "compliance_registry",
  "warehouse_oracle",
  "crop_token",
  "collateral_vault",
  "cross_border_router",
  "commodity_amm",
  "harvest_vault",
  "forward_hedge",
];

const CONTRACT_IDS = {};

function run(cmd, opts = {}) {
  console.log(`> ${cmd}`);
  return execSync(cmd, { stdio: "inherit", ...opts });
}

function read(cmd, opts = {}) {
  return execSync(cmd, { encoding: "utf8", ...opts }).toString().trim();
}

async function ensureNetwork() {
  try {
    const res = read(
      `curl -s -o /dev/null -w "%{http_code}" http://localhost:8000/soroban/rpc 2>/dev/null || echo "000"`
    );
    if (res !== "200") {
      console.log(
        "Local Stellar Quickstart node not detected on http://localhost:8000"
      );
      console.log("Start it with:");
      console.log(
        "  docker run --rm -it -p 8000:8000 stellar/quickstart:soroban-dev@sha256:latest --local --enable-soroban-rpc"
      );
      process.exit(1);
    }
  } catch {
    console.log("Could not connect to local Stellar node. Is Docker running?");
    process.exit(1);
  }
}

async function buildContracts() {
  console.log("\n=== Building contracts ===");
  run("cargo build --workspace --target wasm32-unknown-unknown --release", {
    cwd: ROOT,
  });
}

async function deployContract(name) {
  const wasm = path.join(WASM_DIR, `${name}.wasm`);
  if (!fs.existsSync(wasm)) {
    throw new Error(`WASM file not found: ${wasm}`);
  }
  console.log(`\n--- Deploying ${name} ---`);
  const contractId = read(
    `soroban contract deploy ` +
      `--wasm "${wasm}" ` +
      `--source ${process.env.DEPLOYER_SECRET_KEY} ` +
      `--rpc-url http://localhost:8000/soroban/rpc ` +
      `--network-passphrase "Standalone Network ; June 2018" ` +
      `--network local`
  );
  CONTRACT_IDS[name] = contractId;
  console.log(`  ${name}: ${contractId}`);
  return contractId;
}

async function invokeContract(contractName, fn, args = []) {
  const id = CONTRACT_IDS[contractName];
  if (!id) throw new Error(`${contractName} not deployed yet`);
  const argStr = args.map((a) => `--arg '${a}'`).join(" ");
  console.log(`  Calling ${contractName}.${fn}()`);
  try {
    read(
      `soroban contract invoke ` +
        `--id ${id} ` +
        `--source ${process.env.DEPLOYER_SECRET_KEY} ` +
        `--rpc-url http://localhost:8000/soroban/rpc ` +
        `--network-passphrase "Standalone Network ; June 2018" ` +
        `--fn ${fn} ${argStr}`
    );
  } catch (e) {
    console.error(`  Warning: ${contractName}.${fn}() failed: ${e.message}`);
  }
}

async function initializeContracts(admin, usdcContractId) {
  console.log("\n=== Initializing contracts ===");

  await invokeContract("privacy_passport", "initialize", [admin]);
  await invokeContract("compliance_registry", "initialize", [
    admin,
    CONTRACT_IDS["privacy_passport"],
  ]);
  const oraclePubkey = admin;
  const inspectors = JSON.stringify({
    inspectors: [],
    threshold: 0,
  });
  await invokeContract("warehouse_oracle", "initialize", [
    admin,
    oraclePubkey,
    inspectors,
  ]);
  await invokeContract("crop_token", "initialize", [
    admin,
    CONTRACT_IDS["warehouse_oracle"],
    CONTRACT_IDS["compliance_registry"],
  ]);
  await invokeContract("collateral_vault", "initialize", [
    admin,
    CONTRACT_IDS["compliance_registry"],
    usdcContractId,
    CONTRACT_IDS["warehouse_oracle"],
  ]);
  await invokeContract("cross_border_router", "initialize", [
    admin,
    CONTRACT_IDS["compliance_registry"],
  ]);
  await invokeContract("commodity_amm", "initialize", [
    admin,
    CONTRACT_IDS["crop_token"],
    usdcContractId,
  ]);
  await invokeContract("harvest_vault", "initialize", [
    admin,
    CONTRACT_IDS["crop_token"],
    CONTRACT_IDS["commodity_amm"],
    usdcContractId,
  ]);
  await invokeContract("forward_hedge", "initialize", [
    admin,
    CONTRACT_IDS["crop_token"],
    CONTRACT_IDS["collateral_vault"],
  ]);
}

function writeEnv(admin, usdcContractId) {
  const envPath = path.join(ROOT, ".env");
  const content = `# ── Stellar Network (Local) ─────────────────────────────────────────────
STELLAR_NETWORK=local
STELLAR_RPC_URL=http://localhost:8000/soroban/rpc
STELLAR_HORIZON_URL=http://localhost:8000
STELLAR_NETWORK_PASSPHRASE="Standalone Network ; June 2018"

# ── Deployer / Admin Keys ──────────────────────────────────────────────────
DEPLOYER_SECRET_KEY=${process.env.DEPLOYER_SECRET_KEY || ""}
ADMIN_ADDRESS=${admin}

# ── Deployed Contract IDs ──────────────────────────────────────────────────
CROP_TOKEN_CONTRACT_ID=${CONTRACT_IDS["crop_token"] || ""}
COLLATERAL_VAULT_CONTRACT_ID=${CONTRACT_IDS["collateral_vault"] || ""}
COMPLIANCE_REGISTRY_CONTRACT_ID=${CONTRACT_IDS["compliance_registry"] || ""}
CROSS_BORDER_ROUTER_CONTRACT_ID=${CONTRACT_IDS["cross_border_router"] || ""}
WAREHOUSE_ORACLE_CONTRACT_ID=${CONTRACT_IDS["warehouse_oracle"] || ""}
HARVEST_VAULT_CONTRACT_ID=${CONTRACT_IDS["harvest_vault"] || ""}
COMMODITY_AMM_CONTRACT_ID=${CONTRACT_IDS["commodity_amm"] || ""}
FORWARD_HEDGE_CONTRACT_ID=${CONTRACT_IDS["forward_hedge"] || ""}
PRIVACY_PASSPORT_CONTRACT_ID=${CONTRACT_IDS["privacy_passport"] || ""}

# ── Stablecoin Asset IDs ──────────────────────────────────────────────────
USDC_CONTRACT_ID=${usdcContractId}
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
`;
  fs.writeFileSync(envPath, content);
  console.log(`\n.env written to ${envPath}`);
}

async function main() {
  console.log("=== AgroLedger Local Deploy ===\n");

  const deployerSecret =
    process.env.DEPLOYER_SECRET_KEY ||
    (await prompt("Enter deployer secret key (or set DEPLOYER_SECRET_KEY): "));
  if (!deployerSecret || deployerSecret.startsWith("S...")) {
    console.log("A funded Stellar secret key is required.");
    process.exit(1);
  }
  process.env.DEPLOYER_SECRET_KEY = deployerSecret;

  const admin =
    process.env.ADMIN_ADDRESS ||
    (await prompt("Enter admin public address (or set ADMIN_ADDRESS): "));
  if (!admin) {
    console.log("Admin address is required.");
    process.exit(1);
  }

  const usdcContractId =
    process.env.USDC_CONTRACT_ID ||
    (await prompt("Enter USDC contract ID (or set USDC_CONTRACT_ID): "));
  if (!usdcContractId) {
    console.log("USDC contract ID is required (e.g. C... on testnet).");
    process.exit(1);
  }

  await ensureNetwork();
  await buildContracts();

  for (const name of CONTRACTS) {
    await deployContract(name);
  }

  await initializeContracts(admin, usdcContractId);
  writeEnv(admin, usdcContractId);

  console.log("\n=== Deploy complete! ===");
  console.log("Run `npm run seed:local` to seed test data.");
  rl.close();
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
