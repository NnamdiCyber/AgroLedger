import { nativeToScVal, Address, xdr } from "@stellar/stellar-sdk";
import type { PriceData, InspectorSet } from "../types";
import { BaseClient } from "./base";

export class WarehouseOracleClient extends BaseClient {
  async initialize(
    admin: string,
    oraclePubkey: string,
    inspectors: InspectorSet,
  ): Promise<string> {
    const inspectorScVals = inspectors.inspectors.map((addr) =>
      Address.fromString(addr).toScVal(),
    );
    const inspectorSetScVal = xdr.ScVal.scvMap([
      new xdr.ScMapEntry({
        key: nativeToScVal("inspectors", { type: "symbol" }),
        val: xdr.ScVal.scvVec(inspectorScVals),
      }),
      new xdr.ScMapEntry({
        key: nativeToScVal("threshold", { type: "symbol" }),
        val: xdr.ScVal.scvU32(inspectors.threshold),
      }),
    ]);

    const { hash } = await this.send(
      "initialize",
      [admin, oraclePubkey, inspectorSetScVal],
      ["address", "address"],
    );
    return hash;
  }

  async submitLot(
    warehouseId: string,
    lotId: string,
    commodity: string,
    quantityKg: number,
    inspectorSigs: string[],
  ): Promise<number> {
    const { result } = await this.send(
      "submit_lot",
      [warehouseId, lotId, commodity, quantityKg, inspectorSigs],
      ["symbol", "symbol", "symbol", "u64", "addressVec"],
    );
    return result as number;
  }

  async pushPrice(
    commodity: string,
    priceUsdc: number,
    timestamp: number,
  ): Promise<string> {
    const { hash } = await this.send(
      "push_price",
      [commodity, priceUsdc, timestamp],
      ["symbol", "u64", "u64"],
    );
    return hash;
  }

  async getPrice(commodity: string): Promise<PriceData> {
    const raw = await this.simulate("get_price", [commodity], ["symbol"]);
    return raw as PriceData;
  }

  async verifyLot(warehouseId: string, lotId: string): Promise<boolean> {
    return this.simulate("verify_lot", [warehouseId, lotId], [
      "symbol",
      "symbol",
    ]) as Promise<boolean>;
  }
}
