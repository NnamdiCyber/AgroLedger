import { nativeToScVal, xdr } from "@stellar/stellar-sdk";
import type { PathResult, PathQuote, TravelRuleData } from "../types";
import { BaseClient } from "./base";

export class CrossBorderRouterClient extends BaseClient {
  async initialize(admin: string, complianceRegistry: string): Promise<string> {
    const { hash } = await this.send("initialize", [admin, complianceRegistry], [
      "address",
      "address",
    ]);
    return hash;
  }

  async route(
    from: string,
    to: string,
    sendAsset: string,
    recvAsset: string,
    amount: number,
    travelRuleData: TravelRuleData,
  ): Promise<PathResult> {
    const travelRuleScVal = xdr.ScVal.scvMap([
      new xdr.ScMapEntry({
        key: nativeToScVal("passport_id", { type: "symbol" }),
        val: nativeToScVal(travelRuleData.passport_id, { type: "u64" }),
      }),
      new xdr.ScMapEntry({
        key: nativeToScVal("jurisdiction", { type: "symbol" }),
        val: nativeToScVal(travelRuleData.jurisdiction, { type: "symbol" }),
      }),
    ]);

    const { result } = await this.send(
      "route",
      [from, to, sendAsset, recvAsset, amount, travelRuleScVal],
      ["address", "address", "address", "address", "i128"],
    );
    return result as PathResult;
  }

  async estimate(
    sendAsset: string,
    recvAsset: string,
    amount: number,
  ): Promise<PathQuote[]> {
    const raw = await this.simulate("estimate", [sendAsset, recvAsset, amount], [
      "address",
      "address",
      "i128",
    ]);
    return raw as PathQuote[];
  }

  async registerAsset(
    admin: string,
    symbol: string,
    contractId: string,
  ): Promise<string> {
    const { hash } = await this.send(
      "register_asset",
      [admin, symbol, contractId],
      ["address", "symbol", "address"],
    );
    return hash;
  }

  async getAsset(symbol: string): Promise<string> {
    return this.simulate("get_asset", [symbol], ["symbol"]) as Promise<string>;
  }
}
