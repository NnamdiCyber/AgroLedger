import { BaseClient } from "./base";

export class PrivacyPassportClient extends BaseClient {
  async initialize(admin: string): Promise<string> {
    const { hash } = await this.send("initialize", [admin], ["address"]);
    return hash;
  }

  async register(
    nullifierHash: string,
    credentialProof: string,
    jurisdiction: string,
  ): Promise<number> {
    const nullifierBuf = Buffer.from(
      nullifierHash.startsWith("0x") ? nullifierHash.slice(2) : nullifierHash,
      "hex",
    );
    const proofBuf = Buffer.from(
      credentialProof.startsWith("0x")
        ? credentialProof.slice(2)
        : credentialProof,
      "hex",
    );
    const { result } = await this.send("register", [
      nullifierBuf,
      proofBuf,
      jurisdiction,
    ], ["bytes", "bytes", "symbol"]);
    return result as number;
  }

  async verify(
    passportId: number,
    requiredJurisdiction: string,
  ): Promise<boolean> {
    return this.simulate("verify", [passportId, requiredJurisdiction], [
      "u64",
      "symbol",
    ]) as Promise<boolean>;
  }

  async revoke(passportId: number): Promise<string> {
    const { hash } = await this.send("revoke", [passportId], ["u64"]);
    return hash;
  }
}
