import type { LotMeta } from "../types";
import { BaseClient } from "./base";

export class CropTokenClient extends BaseClient {
  async initialize(
    admin: string,
    warehouseOracle: string,
    complianceRegistry: string,
  ): Promise<string> {
    const { hash } = await this.send(
      "initialize",
      [admin, warehouseOracle, complianceRegistry],
      ["address", "address", "address"],
    );
    return hash;
  }

  async issue(
    warehouseId: string,
    lotId: string,
    commodity: string,
    quantityKg: number,
    oracleSig: string,
  ): Promise<string> {
    const sigBuf = Buffer.from(
      oracleSig.startsWith("0x") ? oracleSig.slice(2) : oracleSig,
      "hex",
    );
    const { hash } = await this.send(
      "issue",
      [warehouseId, lotId, commodity, quantityKg, sigBuf],
      ["symbol", "symbol", "symbol", "u64", "bytes"],
    );
    return hash;
  }

  async transfer(from: string, to: string, amount: number): Promise<string> {
    const { hash } = await this.send("transfer", [from, to, amount], [
      "address",
      "address",
      "i128",
    ]);
    return hash;
  }

  async burn(lotId: string): Promise<string> {
    const { hash } = await this.send("burn", [lotId], ["symbol"]);
    return hash;
  }

  async getLotMetadata(lotId: string): Promise<LotMeta> {
    const raw = await this.simulate("get_lot_metadata", [lotId], ["symbol"]);
    return raw as LotMeta;
  }

  async balance(id: string): Promise<number> {
    return this.simulate("balance", [id], ["address"]) as Promise<number>;
  }

  async linkPassport(
    address: string,
    passportId: number,
    jurisdiction: string,
  ): Promise<string> {
    const { hash } = await this.send(
      "link_passport",
      [address, passportId, jurisdiction],
      ["address", "u64", "symbol"],
    );
    return hash;
  }
}
