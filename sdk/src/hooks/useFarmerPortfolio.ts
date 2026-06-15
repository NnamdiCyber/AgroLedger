import { useState, useEffect, useCallback } from "react";
import type { CropTokenClient } from "../clients/cropToken";
import type { CollateralVaultClient } from "../clients/collateralVault";
import type { ForwardHedgeClient } from "../clients/forwardHedge";
import type { LotMeta, VaultState, HedgeState } from "../types";

export interface FarmerPortfolio {
  tokens: { lotId: string; meta: LotMeta }[];
  vaults: { id: number; state: VaultState }[];
  hedges: { id: number; state: HedgeState }[];
  isLoading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
}

export function useFarmerPortfolio(
  farmerAddress: string | null,
  cropTokenClient: CropTokenClient | null,
  vaultClient: CollateralVaultClient | null,
  hedgeClient: ForwardHedgeClient | null,
): FarmerPortfolio {
  const [tokens, setTokens] = useState<{ lotId: string; meta: LotMeta }[]>(
    [],
  );
  const [vaults, setVaults] = useState<{ id: number; state: VaultState }[]>(
    [],
  );
  const [hedges, setHedges] = useState<{ id: number; state: HedgeState }[]>(
    [],
  );
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    if (!farmerAddress || !cropTokenClient || !vaultClient || !hedgeClient) {
      return;
    }
    setIsLoading(true);
    setError(null);
    try {
      setTokens([]);
      setVaults([]);
      setHedges([]);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch portfolio");
    } finally {
      setIsLoading(false);
    }
  }, [farmerAddress, cropTokenClient, vaultClient, hedgeClient]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { tokens, vaults, hedges, isLoading, error, refresh };
}
