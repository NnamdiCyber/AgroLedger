import { useState, useCallback } from "react";
import type { CollateralVaultClient } from "../clients/collateralVault";

export interface UseCollateralVaultReturn {
  openVault: (
    user: string,
    cropToken: string,
    commodity: string,
    passportId: number,
    jurisdiction: string,
    collateralAmount: number,
    borrowAmountUsdc: number,
  ) => Promise<number>;
  repay: (user: string, vaultId: number, amount: number) => Promise<string>;
  liquidate: (liquidator: string, vaultId: number) => Promise<string>;
  isPending: boolean;
  error: string | null;
}

export function useCollateralVault(
  vaultClient: CollateralVaultClient | null,
): UseCollateralVaultReturn {
  const [isPending, setIsPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const openVault = useCallback(
    async (
      user: string,
      cropToken: string,
      commodity: string,
      passportId: number,
      jurisdiction: string,
      collateralAmount: number,
      borrowAmountUsdc: number,
    ): Promise<number> => {
      if (!vaultClient) throw new Error("Vault client not connected");
      setIsPending(true);
      setError(null);
      try {
        const id = await vaultClient.open(
          user,
          cropToken,
          commodity,
          passportId,
          jurisdiction,
          collateralAmount,
          borrowAmountUsdc,
        );
        return id;
      } catch (err) {
        const msg =
          err instanceof Error ? err.message : "Failed to open vault";
        setError(msg);
        throw err;
      } finally {
        setIsPending(false);
      }
    },
    [vaultClient],
  );

  const repay = useCallback(
    async (
      user: string,
      vaultId: number,
      amount: number,
    ): Promise<string> => {
      if (!vaultClient) throw new Error("Vault client not connected");
      setIsPending(true);
      setError(null);
      try {
        const hash = await vaultClient.repay(user, vaultId, amount);
        return hash;
      } catch (err) {
        const msg =
          err instanceof Error ? err.message : "Failed to repay vault";
        setError(msg);
        throw err;
      } finally {
        setIsPending(false);
      }
    },
    [vaultClient],
  );

  const liquidate = useCallback(
    async (liquidator: string, vaultId: number): Promise<string> => {
      if (!vaultClient) throw new Error("Vault client not connected");
      setIsPending(true);
      setError(null);
      try {
        const hash = await vaultClient.liquidate(liquidator, vaultId);
        return hash;
      } catch (err) {
        const msg =
          err instanceof Error ? err.message : "Failed to liquidate vault";
        setError(msg);
        throw err;
      } finally {
        setIsPending(false);
      }
    },
    [vaultClient],
  );

  return { openVault, repay, liquidate, isPending, error };
}
