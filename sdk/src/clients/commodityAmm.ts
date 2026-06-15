import type { PoolInfo } from "../types";
import { BaseClient } from "./base";

export class CommodityAmmClient extends BaseClient {
  async initialize(
    admin: string,
    cropToken: string,
    usdcToken: string,
  ): Promise<string> {
    const { hash } = await this.send(
      "initialize",
      [admin, cropToken, usdcToken],
      ["address", "address", "address"],
    );
    return hash;
  }

  async createPool(admin: string, commodity: string): Promise<string> {
    const { hash } = await this.send("create_pool", [admin, commodity], [
      "address",
      "symbol",
    ]);
    return hash;
  }

  async swap(
    user: string,
    commodity: string,
    amountIn: number,
    minAmountOut: number,
    sellCrop: boolean,
  ): Promise<number> {
    const { result } = await this.send(
      "swap",
      [user, commodity, amountIn, minAmountOut, sellCrop],
      ["address", "symbol", "i128", "i128", "bool"],
    );
    return result as number;
  }

  async addLiquidity(
    user: string,
    commodity: string,
    amountCrop: number,
    amountUsdc: number,
  ): Promise<[number, number, number]> {
    const { result } = await this.send(
      "add_liquidity",
      [user, commodity, amountCrop, amountUsdc],
      ["address", "symbol", "i128", "i128"],
    );
    return result as [number, number, number];
  }

  async removeLiquidity(
    user: string,
    commodity: string,
    lpTokens: number,
    minCrop: number,
    minUsdc: number,
  ): Promise<[number, number]> {
    const { result } = await this.send(
      "remove_liquidity",
      [user, commodity, lpTokens, minCrop, minUsdc],
      ["address", "symbol", "i128", "i128", "i128"],
    );
    return result as [number, number];
  }

  async getPool(commodity: string): Promise<PoolInfo> {
    const raw = await this.simulate("get_pool", [commodity], ["symbol"]);
    return raw as PoolInfo;
  }

  async getLpBalance(user: string, commodity: string): Promise<number> {
    return this.simulate("get_lp_balance", [user, commodity], [
      "address",
      "symbol",
    ]) as Promise<number>;
  }
}
