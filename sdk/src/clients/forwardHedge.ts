import type { HedgeState } from "../types";
import { BaseClient } from "./base";

export class ForwardHedgeClient extends BaseClient {
  async initialize(
    admin: string,
    cropToken: string,
    collateralVault: string,
  ): Promise<string> {
    const { hash } = await this.send(
      "initialize",
      [admin, cropToken, collateralVault],
      ["address", "address", "address"],
    );
    return hash;
  }

  async placeHedge(
    buyer: string,
    commodity: string,
    quantity: number,
    commitment: string,
    expiry: number,
  ): Promise<number> {
    const commitmentBuf = Buffer.from(
      commitment.startsWith("0x") ? commitment.slice(2) : commitment,
      "hex",
    );
    const { result } = await this.send(
      "place_hedge",
      [buyer, commodity, quantity, commitmentBuf, expiry],
      ["address", "symbol", "i128", "bytes", "u64"],
    );
    return result as number;
  }

  async acceptHedge(hedgeId: number, farmer: string): Promise<string> {
    const { hash } = await this.send("accept_hedge", [hedgeId, farmer], [
      "u64",
      "address",
    ]);
    return hash;
  }

  async reveal(
    hedgeId: number,
    price: number,
    salt: number,
  ): Promise<string> {
    const { hash } = await this.send("reveal", [hedgeId, price, salt], [
      "u64",
      "i128",
      "i128",
    ]);
    return hash;
  }

  async settle(
    hedgeId: number,
    settlementType: string,
    caller: string,
  ): Promise<string> {
    const { hash } = await this.send(
      "settle",
      [hedgeId, settlementType, caller],
      ["u64", "symbol", "address"],
    );
    return hash;
  }

  async cancel(hedgeId: number, caller: string): Promise<string> {
    const { hash } = await this.send("cancel", [hedgeId, caller], [
      "u64",
      "address",
    ]);
    return hash;
  }

  async getHedge(hedgeId: number): Promise<HedgeState> {
    const raw = await this.simulate("get_hedge", [hedgeId], ["u64"]);
    return raw as HedgeState;
  }

  async getRevealedPrice(hedgeId: number): Promise<number> {
    return this.simulate("get_revealed_price", [hedgeId], [
      "u64",
    ]) as Promise<number>;
  }
}
