import { Keypair, SorobanRpc } from "@stellar/stellar-sdk";
import type { NetworkType } from "./types";
import { PrivacyPassportClient } from "./clients/privacyPassport";
import { ComplianceRegistryClient } from "./clients/complianceRegistry";
import { CropTokenClient } from "./clients/cropToken";
import { CollateralVaultClient } from "./clients/collateralVault";
import { CrossBorderRouterClient } from "./clients/crossBorderRouter";
import { CommodityAmmClient } from "./clients/commodityAmm";
import { HarvestVaultClient } from "./clients/harvestVault";
import { ForwardHedgeClient } from "./clients/forwardHedge";
import { WarehouseOracleClient } from "./clients/warehouseOracle";

export interface AgroLedgerConfig {
  network: NetworkType;
  signer: Keypair;
  rpcUrl?: string;
}

export class AgroLedger {
  public readonly network: NetworkType;
  public readonly signer: Keypair;
  public readonly rpc: SorobanRpc.Server;

  public privacyPassport!: PrivacyPassportClient;
  public complianceRegistry!: ComplianceRegistryClient;
  public cropToken!: CropTokenClient;
  public collateralVault!: CollateralVaultClient;
  public crossBorderRouter!: CrossBorderRouterClient;
  public commodityAmm!: CommodityAmmClient;
  public harvestVault!: HarvestVaultClient;
  public forwardHedge!: ForwardHedgeClient;
  public warehouseOracle!: WarehouseOracleClient;

  constructor(config: AgroLedgerConfig) {
    this.network = config.network;
    this.signer = config.signer;

    const rpcUrl =
      config.rpcUrl ??
      (config.network === "local"
        ? "http://localhost:8000/soroban/rpc"
        : config.network === "testnet"
          ? "https://soroban-testnet.stellar.org"
          : "https://soroban.stellar.org");

    const allowHttp = config.network === "local";
    this.rpc = allowHttp
      ? new SorobanRpc.Server(rpcUrl, { allowHttp: true })
      : new SorobanRpc.Server(rpcUrl);
  }

  connectPrivacyPassport(contractId: string): PrivacyPassportClient {
    this.privacyPassport = new PrivacyPassportClient(
      contractId,
      this.rpc,
      this.signer,
    );
    return this.privacyPassport;
  }

  connectComplianceRegistry(contractId: string): ComplianceRegistryClient {
    this.complianceRegistry = new ComplianceRegistryClient(
      contractId,
      this.rpc,
      this.signer,
    );
    return this.complianceRegistry;
  }

  connectCropToken(contractId: string): CropTokenClient {
    this.cropToken = new CropTokenClient(contractId, this.rpc, this.signer);
    return this.cropToken;
  }

  connectCollateralVault(contractId: string): CollateralVaultClient {
    this.collateralVault = new CollateralVaultClient(
      contractId,
      this.rpc,
      this.signer,
    );
    return this.collateralVault;
  }

  connectCrossBorderRouter(contractId: string): CrossBorderRouterClient {
    this.crossBorderRouter = new CrossBorderRouterClient(
      contractId,
      this.rpc,
      this.signer,
    );
    return this.crossBorderRouter;
  }

  connectCommodityAmm(contractId: string): CommodityAmmClient {
    this.commodityAmm = new CommodityAmmClient(
      contractId,
      this.rpc,
      this.signer,
    );
    return this.commodityAmm;
  }

  connectHarvestVault(contractId: string): HarvestVaultClient {
    this.harvestVault = new HarvestVaultClient(
      contractId,
      this.rpc,
      this.signer,
    );
    return this.harvestVault;
  }

  connectForwardHedge(contractId: string): ForwardHedgeClient {
    this.forwardHedge = new ForwardHedgeClient(
      contractId,
      this.rpc,
      this.signer,
    );
    return this.forwardHedge;
  }

  connectWarehouseOracle(contractId: string): WarehouseOracleClient {
    this.warehouseOracle = new WarehouseOracleClient(
      contractId,
      this.rpc,
      this.signer,
    );
    return this.warehouseOracle;
  }
}

export * from "./types";
export * from "./clients/base";
export * from "./clients/privacyPassport";
export * from "./clients/complianceRegistry";
export * from "./clients/cropToken";
export * from "./clients/collateralVault";
export * from "./clients/crossBorderRouter";
export * from "./clients/commodityAmm";
export * from "./clients/harvestVault";
export * from "./clients/forwardHedge";
export * from "./clients/warehouseOracle";
