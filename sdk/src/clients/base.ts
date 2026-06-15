import {
  Keypair,
  SorobanRpc,
  TransactionBuilder,
  Networks,
  nativeToScVal,
  scValToNative,
  Address,
  Contract,
  xdr,
  ScInt,
} from "@stellar/stellar-sdk";

type ScVal = xdr.ScVal;

export type ScValArg =
  | string
  | number
  | bigint
  | boolean
  | Buffer
  | Uint8Array
  | ScVal
  | ScValArg[]
  | { [key: string]: ScValArg };

export interface SendResult {
  hash: string;
  result?: unknown;
}

export class BaseClient {
  protected contract: Contract;
  protected rpc: SorobanRpc.Server;
  protected signer: Keypair;
  protected networkPassphrase: string;
  public readonly contractId: string;

  constructor(contractId: string, rpc: SorobanRpc.Server, signer: Keypair) {
    this.contract = new Contract(contractId);
    this.contractId = contractId;
    this.rpc = rpc;
    this.signer = signer;
    this.networkPassphrase = Networks.TESTNET;
  }

  toScVal(val: unknown, typeHint?: string): ScVal {
    if (val instanceof xdr.ScVal) return val;
    if (typeHint === "address")
      return Address.fromString(val as string).toScVal();
    if (typeHint === "symbol")
      return nativeToScVal(val as string, { type: "symbol" });
    if (typeHint === "u64")
      return new ScInt(val as number | bigint, { type: "u64" }).toScVal();
    if (typeHint === "i128")
      return nativeToScVal(val as number | bigint, { type: "i128" });
    if (typeHint === "bool") return xdr.ScVal.scvBool(val as boolean);
    if (typeHint === "bytes")
      return xdr.ScVal.scvBytes(val as Buffer);
    if (typeHint === "void") return xdr.ScVal.scvVoid();
    if (typeHint === "addressVec") {
      const arr = val as string[];
      return xdr.ScVal.scvVec(arr.map((v) => Address.fromString(v).toScVal()));
    }
    if (typeHint === "symbolVec") {
      const arr = val as string[];
      return xdr.ScVal.scvVec(
        arr.map((v) => nativeToScVal(v, { type: "symbol" }) as ScVal),
      );
    }
    return nativeToScVal(val);
  }

  protected toScVals(args: ScValArg[], types: string[] = []): ScVal[] {
    return args.map((arg, i) => this.toScVal(arg, types[i]));
  }

  async simulate(
    method: string,
    args: ScValArg[] = [],
    types: string[] = [],
  ): Promise<unknown> {
    const source = await this.rpc.getAccount(this.signer.publicKey());
    const tx = new TransactionBuilder(source, {
      fee: "100",
      networkPassphrase: this.networkPassphrase,
    })
      .addOperation(this.contract.call(method, ...this.toScVals(args, types)))
      .setTimeout(30)
      .build();
    const result = await this.rpc.simulateTransaction(tx);
    if (SorobanRpc.Api.isSimulationSuccess(result)) {
      return scValToNative(result.result!.retval);
    }
    const errorInfo = SorobanRpc.Api.isSimulationError(result)
      ? result.error
      : JSON.stringify(result);
    throw new Error(`Simulation failed for ${method}: ${errorInfo}`);
  }

  async send(
    method: string,
    args: ScValArg[] = [],
    types: string[] = [],
  ): Promise<SendResult> {
    const source = await this.rpc.getAccount(this.signer.publicKey());
    const tx = new TransactionBuilder(source, {
      fee: "100",
      networkPassphrase: this.networkPassphrase,
    })
      .addOperation(this.contract.call(method, ...this.toScVals(args, types)))
      .setTimeout(30)
      .build();

    let result: unknown | undefined;
    try {
      const sim = await this.rpc.simulateTransaction(tx);
      if (SorobanRpc.Api.isSimulationSuccess(sim)) {
        result = scValToNative(sim.result!.retval);
      }
    } catch {
      // simulation may fail for auth reasons, that's ok
    }

    const prepared = await this.rpc.prepareTransaction(tx);
    prepared.sign(this.signer);
    const sendResult = await this.rpc.sendTransaction(prepared);
    return { hash: sendResult.hash, result };
  }
}
