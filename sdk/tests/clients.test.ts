import { Keypair, StrKey } from "@stellar/stellar-sdk";
import { AgroLedger } from "../src";

function makeContractId(): string {
  const buf = Buffer.alloc(32);
  for (let i = 0; i < 32; i++) buf[i] = Math.floor(Math.random() * 256);
  return StrKey.encodeContract(buf);
}

describe("AgroLedger SDK", () => {
  let agro: AgroLedger;
  let signer: Keypair;
  const testContractId = makeContractId();

  beforeAll(() => {
    signer = Keypair.random();
    agro = new AgroLedger({
      network: "local",
      signer,
      rpcUrl: "http://localhost:8000/soroban/rpc",
    });
  });

  it("creates an AgroLedger instance with local network", () => {
    expect(agro).toBeInstanceOf(AgroLedger);
    expect(agro.network).toBe("local");
    expect(agro.signer).toBe(signer);
  });

  it("connects to PrivacyPassport contract", () => {
    const client = agro.connectPrivacyPassport(testContractId);
    expect(client).toBeDefined();
    expect(client.contractId).toBe(testContractId);
  });

  it("connects to ComplianceRegistry contract", () => {
    const client = agro.connectComplianceRegistry(testContractId);
    expect(client).toBeDefined();
    expect(client.contractId).toBe(testContractId);
  });

  it("connects to CropToken contract", () => {
    const client = agro.connectCropToken(testContractId);
    expect(client).toBeDefined();
    expect(client.contractId).toBe(testContractId);
  });

  it("connects to CollateralVault contract", () => {
    const client = agro.connectCollateralVault(testContractId);
    expect(client).toBeDefined();
    expect(client.contractId).toBe(testContractId);
  });

  it("connects to CrossBorderRouter contract", () => {
    const client = agro.connectCrossBorderRouter(testContractId);
    expect(client).toBeDefined();
    expect(client.contractId).toBe(testContractId);
  });

  it("connects to CommodityAmm contract", () => {
    const client = agro.connectCommodityAmm(testContractId);
    expect(client).toBeDefined();
    expect(client.contractId).toBe(testContractId);
  });

  it("connects to HarvestVault contract", () => {
    const client = agro.connectHarvestVault(testContractId);
    expect(client).toBeDefined();
    expect(client.contractId).toBe(testContractId);
  });

  it("connects to ForwardHedge contract", () => {
    const client = agro.connectForwardHedge(testContractId);
    expect(client).toBeDefined();
    expect(client.contractId).toBe(testContractId);
  });

  it("connects to WarehouseOracle contract", () => {
    const client = agro.connectWarehouseOracle(testContractId);
    expect(client).toBeDefined();
    expect(client.contractId).toBe(testContractId);
  });

  it("uses testnet network configuration", () => {
    const testnetAgro = new AgroLedger({
      network: "testnet",
      signer,
    });
    expect(testnetAgro.network).toBe("testnet");
  });

  it("uses mainnet network configuration", () => {
    const mainnetAgro = new AgroLedger({
      network: "mainnet",
      signer,
    });
    expect(mainnetAgro.network).toBe("mainnet");
  });
});

describe("BaseClient type conversion", () => {
  let agro: AgroLedger;
  let signer: Keypair;

  beforeAll(() => {
    signer = Keypair.random();
    agro = new AgroLedger({
      network: "local",
      signer,
      rpcUrl: "http://localhost:8000/soroban/rpc",
    });
  });

  it("converts address types correctly", () => {
    const client = agro.connectPrivacyPassport(makeContractId());
    const scVal = client.toScVal(signer.publicKey(), "address");
    expect(scVal).toBeDefined();
    expect(typeof scVal.switch).toBe("function");
  });

  it("converts symbol types correctly", () => {
    const client = agro.connectPrivacyPassport(makeContractId());
    const scVal = client.toScVal("NG", "symbol");
    expect(scVal).toBeDefined();
    expect(scVal.switch().name).toBe("scvSymbol");
  });

  it("converts u64 types correctly", () => {
    const client = agro.connectPrivacyPassport(makeContractId());
    const scVal = client.toScVal(42, "u64");
    expect(scVal).toBeDefined();
    expect(scVal.switch().name).toBe("scvU64");
  });

  it("converts i128 types correctly", () => {
    const client = agro.connectPrivacyPassport(makeContractId());
    const scVal = client.toScVal(100000, "i128");
    expect(scVal).toBeDefined();
    expect(scVal.switch().name).toBe("scvI128");
  });

  it("converts bytes types correctly", () => {
    const client = agro.connectPrivacyPassport(makeContractId());
    const buf = Buffer.alloc(32);
    buf[0] = 0x01;
    const scVal = client.toScVal(buf, "bytes");
    expect(scVal).toBeDefined();
    expect(scVal.switch().name).toBe("scvBytes");
  });

  it("converts addressVec types correctly", () => {
    const client = agro.connectPrivacyPassport(makeContractId());
    const scVal = client.toScVal(
      [signer.publicKey()],
      "addressVec",
    );
    expect(scVal).toBeDefined();
    expect(scVal.switch().name).toBe("scvVec");
  });

  it("converts symbolVec types correctly", () => {
    const client = agro.connectPrivacyPassport(makeContractId());
    const scVal = client.toScVal(["NG", "US"], "symbolVec");
    expect(scVal).toBeDefined();
    expect(scVal.switch().name).toBe("scvVec");
  });
});
