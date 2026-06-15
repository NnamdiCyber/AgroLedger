import type { VaultState } from "../types";
import { BaseClient } from "./base";

export class CollateralVaultClient extends BaseClient {
  async initialize(
    admin: string,
    complianceRegistry: string,
    usdcToken: string,
    warehouseOracle: string,
  ): Promise<string> {
    const { hash } = await this.send(
      "initialize",
      [admin, complianceRegistry, usdcToken, warehouseOracle],
      ["address", "address", "address", "address"],
    );
    return hash;
  }

  async open(
    user: string,
    cropToken: string,
    commodity: string,
    passportId: number,
    jurisdiction: string,
    collateralAmount: number,
    borrowAmountUsdc: number,
  ): Promise<number> {
    const { result } = await this.send(
      "open",
      [user, cropToken, commodity, passportId, jurisdiction, collateralAmount, borrowAmountUsdc],
      ["address", "address", "symbol", "u64", "symbol", "i128", "i128"],
    );
    return result as number;
  }

  async repay(
    user: string,
    vaultId: number,
    amount: number,
  ): Promise<string> {
    const { hash } = await this.send("repay", [user, vaultId, amount], [
      "address",
      "u64",
      "i128",
    ]);
    return hash;
  }

  async liquidate(liquidator: string, vaultId: number): Promise<string> {
    const { hash } = await this.send("liquidate", [liquidator, vaultId], [
      "address",
      "u64",
    ]);
    return hash;
  }

  async getVault(vaultId: number): Promise<VaultState> {
    const raw = await this.simulate("get_vault", [vaultId], ["u64"]);
    return raw as VaultState;
  }
}
