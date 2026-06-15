import { BaseClient } from "./base";

export class ComplianceRegistryClient extends BaseClient {
  async initialize(admin: string, privacyPassport: string): Promise<string> {
    const { hash } = await this.send("initialize", [admin, privacyPassport], [
      "address",
      "address",
    ]);
    return hash;
  }

  async addJurisdiction(code: string): Promise<string> {
    const { hash } = await this.send("add_jurisdiction", [code], ["symbol"]);
    return hash;
  }

  async removeJurisdiction(code: string): Promise<string> {
    const { hash } = await this.send("remove_jurisdiction", [code], ["symbol"]);
    return hash;
  }

  async isAllowed(jurisdiction: string): Promise<boolean> {
    return this.simulate("is_allowed", [jurisdiction], [
      "symbol",
    ]) as Promise<boolean>;
  }

  async verify(passportId: number, jurisdiction: string): Promise<boolean> {
    return this.simulate("verify", [passportId, jurisdiction], [
      "u64",
      "symbol",
    ]) as Promise<boolean>;
  }

  async validateTravelRule(
    amount: number,
    jurisdiction: string,
  ): Promise<boolean> {
    return this.simulate("validate_travel_rule", [amount, jurisdiction], [
      "i128",
      "symbol",
    ]) as Promise<boolean>;
  }
}
