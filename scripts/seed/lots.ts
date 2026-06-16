import { Keypair, SorobanRpc } from "@stellar/stellar-sdk";
import { WarehouseOracleClient } from "../../sdk/src/clients/warehouseOracle";

export interface LotsConfig {
  rpcUrl: string;
  networkPassphrase: string;
  signer: Keypair;
  warehouseOracleId: string;
}

export interface SeededLot {
  warehouseId: string;
  lotId: string;
  commodity: string;
  quantityKg: number;
}

const SEED_LOTS = [
  { warehouseId: "WH001", lotId: "LOT-MAIZE-001", commodity: "MAIZE", quantityKg: 50_000 },
  { warehouseId: "WH001", lotId: "LOT-MAIZE-002", commodity: "MAIZE", quantityKg: 25_000 },
  { warehouseId: "WH002", lotId: "LOT-SOYA-001", commodity: "SOYA", quantityKg: 10_000 },
  { warehouseId: "WH002", lotId: "LOT-SOYA-002", commodity: "SOYA", quantityKg: 8_000 },
  { warehouseId: "WH003", lotId: "LOT-COCOA-001", commodity: "COCOA", quantityKg: 5_000 },
  { warehouseId: "WH003", lotId: "LOT-RICE-001", commodity: "RICE", quantityKg: 20_000 },
];

const PRICE_FEEDS = [
  { commodity: "MAIZE", priceUsdc: 250_000_000, timestamp: Math.floor(Date.now() / 1000) },
  { commodity: "SOYA", priceUsdc: 450_000_000, timestamp: Math.floor(Date.now() / 1000) },
  { commodity: "COCOA", priceUsdc: 2_500_000_000, timestamp: Math.floor(Date.now() / 1000) },
  { commodity: "RICE", priceUsdc: 600_000_000, timestamp: Math.floor(Date.now() / 1000) },
  { commodity: "COTTON", priceUsdc: 1_200_000_000, timestamp: Math.floor(Date.now() / 1000) },
  { commodity: "COFFEE", priceUsdc: 3_000_000_000, timestamp: Math.floor(Date.now() / 1000) },
];

export async function seedLots(config: LotsConfig): Promise<SeededLot[]> {
  const server = new SorobanRpc.Server(config.rpcUrl, { allowHttp: true });
  const oracle = new WarehouseOracleClient(
    config.warehouseOracleId,
    server,
    config.signer
  );

  console.log("[lots] Pushing price feeds...");
  for (const feed of PRICE_FEEDS) {
    try {
      await oracle.pushPrice(feed.commodity, feed.priceUsdc, feed.timestamp);
      console.log(`[lots]   Price set: ${feed.commodity} = $${(feed.priceUsdc / 1e7).toFixed(2)}`);
    } catch (e: any) {
      console.log(`[lots]   Price already set or failed: ${feed.commodity} — ${e.message}`);
    }
  }

  console.log("[lots] Submitting lots...");
  const seeded: SeededLot[] = [];

  for (const lot of SEED_LOTS) {
    try {
      await oracle.submitLot(
        lot.warehouseId,
        lot.lotId,
        lot.commodity,
        lot.quantityKg,
        []
      );
      console.log(
        `[lots]   Submitted ${lot.lotId}: ${lot.quantityKg.toLocaleString()} kg ${lot.commodity} @ ${lot.warehouseId}`
      );
      seeded.push({ ...lot });
    } catch (e: any) {
      console.log(`[lots]   Failed to submit ${lot.lotId}: ${e.message}`);
    }
  }

  console.log(`[lots] Seeded ${seeded.length} lots and ${PRICE_FEEDS.length} price feeds`);
  return seeded;
}

export function getLotSeedData() {
  return { lots: SEED_LOTS, priceFeeds: PRICE_FEEDS };
}
