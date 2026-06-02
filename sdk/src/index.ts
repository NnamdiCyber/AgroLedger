import { Horizon, Keypair, SorobanRpc } from "@stellar/stellar-sdk";

export interface AgroLedgerConfig {
  network: "local" | "testnet" | "mainnet";
  signer: Keypair;
}

export class AgroLedger {
  public readonly network: string;
  public readonly signer: Keypair;
  public readonly rpc: SorobanRpc.Server;

  constructor(config: AgroLedgerConfig) {
    this.network = config.network;
    this.signer = config.signer;

    const rpcUrl =
      config.network === "local"
        ? "http://localhost:8000/soroban/rpc"
        : config.network === "testnet"
          ? "https://soroban-testnet.stellar.org"
          : "https://soroban.stellar.org";

    this.rpc = new SorobanRpc.Server(rpcUrl);
  }
}
