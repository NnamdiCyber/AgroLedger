import { BaseClient } from "./base";

export class HarvestVaultClient extends BaseClient {
  async initialize(
    admin: string,
    cropToken: string,
    commodityAmm: string,
    usdcToken: string,
  ): Promise<string> {
    const { hash } = await this.send(
      "initialize",
      [admin, cropToken, commodityAmm, usdcToken],
      ["address", "address", "address", "address"],
    );
    return hash;
  }

  async deposit(user: string, amount: number): Promise<number> {
    const { result } = await this.send("deposit", [user, amount], [
      "address",
      "i128",
    ]);
    return result as number;
  }

  async withdraw(
    user: string,
    hctAmount: number,
  ): Promise<[number, number]> {
    const { result } = await this.send("withdraw", [user, hctAmount], [
      "address",
      "i128",
    ]);
    return result as [number, number];
  }

  async accrueYield(admin: string): Promise<number> {
    const { result } = await this.send("accrue_yield", [admin], ["address"]);
    return result as number;
  }

  async getApy(): Promise<number> {
    return this.simulate("get_apy") as Promise<number>;
  }

  async rebalance(admin: string): Promise<string> {
    const { hash } = await this.send("rebalance", [admin], ["address"]);
    return hash;
  }

  async getHctBalance(user: string): Promise<number> {
    return this.simulate("get_hct_balance", [user], ["address"]) as Promise<number>;
  }

  async getTotalCropDeposited(): Promise<number> {
    return this.simulate("get_total_crop_deposited") as Promise<number>;
  }

  async getTotalHctSupply(): Promise<number> {
    return this.simulate("get_total_hct_supply") as Promise<number>;
  }

  async getTotalYield(): Promise<number> {
    return this.simulate("get_total_yield") as Promise<number>;
  }
}
