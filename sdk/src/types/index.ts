export interface PassportState {
  nullifier_hash: string;
  jurisdiction: string;
  active: boolean;
  registered_at: number;
}

export interface LotState {
  warehouse_id: string;
  lot_id: string;
  commodity: string;
  quantity_kg: number;
  approved: boolean;
  approved_at: number;
}

export interface InspectorSet {
  inspectors: string[];
  threshold: number;
}

export interface PriceData {
  price: number;
  timestamp: number;
}

export interface LotMeta {
  warehouse_id: string;
  lot_id: string;
  commodity: string;
  quantity_kg: number;
  oracle_attestation: string;
  expiry: number;
  price: number;
}

export interface VaultState {
  owner: string;
  crop_token: string;
  collateral_amount: number;
  debt_amount: number;
  commodity: string;
  opened_at: number;
}

export interface PathResult {
  from: string;
  to: string;
  send_asset: string;
  recv_asset: string;
  amount_sent: number;
  amount_received: number;
  fee: number;
}

export interface PathQuote {
  send_asset: string;
  recv_asset: string;
  amount_out: number;
  fee: number;
}

export interface TravelRuleData {
  passport_id: number;
  jurisdiction: string;
}

export interface PoolInfo {
  commodity: string;
  reserve_crop: number;
  reserve_usdc: number;
  total_lp_supply: number;
  created_at: number;
}

export interface HedgeState {
  buyer: string;
  farmer: string;
  commodity: string;
  quantity: number;
  commitment: string;
  expiry: number;
  status: string;
  placed_at: number;
}

export type JurisdictionSymbol = "NG" | "US" | "GH" | "KE" | "ZA" | "CI" | "SN" | string;

export type NetworkType = "local" | "testnet" | "mainnet";

export type SettlementType = "Physical" | "Cash";

export type HedgeStatus = "Placed" | "Accepted" | "SettledPhysical" | "SettledCash" | "Cancelled";
