import { Keypair, SorobanRpc } from "@stellar/stellar-sdk";
import { AgroLedger } from "../../sdk/src";

export interface PassportsConfig {
  rpcUrl: string;
  networkPassphrase: string;
  signer: Keypair;
  privacyPassportId: string;
  complianceRegistryId: string;
  skipCompliance?: boolean;
}

export interface SeededPassport {
  passportId: number;
  owner: string;
  jurisdiction: string;
}

const TEST_CREDENTIALS = [
  {
    label: "Alice (Nigerian Farmer)",
    ownerKey: Keypair.random(),
    jurisdiction: "NG",
  },
  {
    label: "Bob (Nigerian Buyer)",
    ownerKey: Keypair.random(),
    jurisdiction: "NG",
  },
  {
    label: "Charles (Ghanaian Trader)",
    ownerKey: Keypair.random(),
    jurisdiction: "GH",
  },
  {
    label: "Diana (Compliance Officer)",
    ownerKey: Keypair.random(),
    jurisdiction: "NG",
  },
];

export async function seedPassports(
  config: PassportsConfig
): Promise<SeededPassport[]> {
  const agro = new AgroLedger({
    network: config.rpcUrl.includes("localhost") ? "local" : "testnet",
    signer: config.signer,
    rpcUrl: config.rpcUrl,
  });

  const passport = agro.connectPrivacyPassport(config.privacyPassportId);
  const compliance = agro.connectComplianceRegistry(config.complianceRegistryId);

  console.log("[passports] Adding jurisdictions to allowlist...");
  const jurisdictions = [...new Set(TEST_CREDENTIALS.map((c) => c.jurisdiction))];
  for (const j of jurisdictions) {
    try {
      await compliance.addJurisdiction(j);
      console.log(`[passports]   Jurisdiction ${j} added to allowlist`);
    } catch (e: any) {
      console.log(`[passports]   Jurisdiction ${j} already allowed or failed: ${e.message}`);
    }
  }

  console.log("[passports] Registering KYC credentials...");
  const seeded: SeededPassport[] = [];

  for (const cred of TEST_CREDENTIALS) {
    const nullifierHash =
      "0x" +
      Array.from({ length: 32 }, () =>
        Math.floor(Math.random() * 256)
          .toString(16)
          .padStart(2, "0")
      ).join("");
    const proof =
      "0x" +
      Array.from({ length: 32 }, () =>
        Math.floor(Math.random() * 256)
          .toString(16)
          .padStart(2, "0")
      ).join("");

    try {
      const passportId = await passport.register(
        nullifierHash,
        proof,
        cred.jurisdiction
      );
      console.log(
        `[passports]   #${passportId}: ${cred.label} (${cred.jurisdiction}, ${cred.ownerKey.publicKey()})`
      );
      seeded.push({
        passportId: passportId as number,
        owner: cred.ownerKey.publicKey(),
        jurisdiction: cred.jurisdiction,
      });
    } catch (e: any) {
      console.log(`[passports]   Failed to register ${cred.label}: ${e.message}`);
    }
  }

  console.log(
    `[passports] Seeded ${seeded.length} KYC credentials`
  );
  return seeded;
}

export function getPassportSeedData() {
  return { credentials: TEST_CREDENTIALS };
}
