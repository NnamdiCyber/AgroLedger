import { Keypair } from "@stellar/stellar-sdk";
import * as dotenv from "dotenv";
import { seedWarehouses, type WarehousesConfig } from "./warehouses";
import { seedLots, type LotsConfig } from "./lots";
import { seedPassports, type PassportsConfig } from "./passports";

dotenv.config();

function getConfig() {
  const network = (process.env.STELLAR_NETWORK || "local") as
    | "local"
    | "testnet";

  const rpcUrl =
    process.env.STELLAR_RPC_URL ||
    (network === "local"
      ? "http://localhost:8000/soroban/rpc"
      : "https://soroban-testnet.stellar.org");

  const networkPassphrase =
    process.env.STELLAR_NETWORK_PASSPHRASE ||
    (network === "local"
      ? "Standalone Network ; June 2018"
      : "Test SDF Network ; September 2015");

  const signerSecret =
    process.env.DEPLOYER_SECRET_KEY || process.env.SEEDER_SECRET_KEY;
  if (!signerSecret) {
    throw new Error(
      "DEPLOYER_SECRET_KEY or SEEDER_SECRET_KEY must be set in .env"
    );
  }
  const signer = Keypair.fromSecret(signerSecret);

  const warehouseOracleId = process.env.WAREHOUSE_ORACLE_CONTRACT_ID;
  const privacyPassportId = process.env.PRIVACY_PASSPORT_CONTRACT_ID;
  const complianceRegistryId = process.env.COMPLIANCE_REGISTRY_CONTRACT_ID;

  if (!warehouseOracleId || !privacyPassportId || !complianceRegistryId) {
    throw new Error(
      "WAREHOUSE_ORACLE_CONTRACT_ID, PRIVACY_PASSPORT_CONTRACT_ID, " +
        "and COMPLIANCE_REGISTRY_CONTRACT_ID must be set in .env"
    );
  }

  return {
    network,
    rpcUrl,
    networkPassphrase,
    signer,
    warehouseOracleId,
    privacyPassportId,
    complianceRegistryId,
  };
}

async function main() {
  console.log("=== AgroLedger Seed Script ===\n");

  const cfg = getConfig();

  const warehouses = await seedWarehouses({
    rpcUrl: cfg.rpcUrl,
    networkPassphrase: cfg.networkPassphrase,
    signer: cfg.signer,
    warehouseOracleId: cfg.warehouseOracleId,
  } satisfies WarehousesConfig);

  const lots = await seedLots({
    rpcUrl: cfg.rpcUrl,
    networkPassphrase: cfg.networkPassphrase,
    signer: cfg.signer,
    warehouseOracleId: cfg.warehouseOracleId,
  } satisfies LotsConfig);

  const passports = await seedPassports({
    rpcUrl: cfg.rpcUrl,
    networkPassphrase: cfg.networkPassphrase,
    signer: cfg.signer,
    privacyPassportId: cfg.privacyPassportId,
    complianceRegistryId: cfg.complianceRegistryId,
  } satisfies PassportsConfig);

  console.log("\n=== Seed Summary ===");
  console.log(`  Warehouses: ${warehouses.length}`);
  console.log(`  Lots:       ${lots.length}`);
  console.log(`  Passports:  ${passports.length}`);
  console.log("\nDone.");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
