import { useState, useCallback } from "react";
import type { ForwardHedgeClient } from "../clients/forwardHedge";

export interface UseForwardHedgeReturn {
  placeHedge: (
    buyer: string,
    commodity: string,
    quantity: number,
    commitment: string,
    expiry: number,
  ) => Promise<number>;
  acceptHedge: (hedgeId: number, farmer: string) => Promise<string>;
  isPending: boolean;
  error: string | null;
}

export function useForwardHedge(
  hedgeClient: ForwardHedgeClient | null,
): UseForwardHedgeReturn {
  const [isPending, setIsPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const placeHedge = useCallback(
    async (
      buyer: string,
      commodity: string,
      quantity: number,
      commitment: string,
      expiry: number,
    ): Promise<number> => {
      if (!hedgeClient) throw new Error("Hedge client not connected");
      setIsPending(true);
      setError(null);
      try {
        const id = await hedgeClient.placeHedge(
          buyer,
          commodity,
          quantity,
          commitment,
          expiry,
        );
        return id;
      } catch (err) {
        const msg =
          err instanceof Error ? err.message : "Failed to place hedge";
        setError(msg);
        throw err;
      } finally {
        setIsPending(false);
      }
    },
    [hedgeClient],
  );

  const acceptHedge = useCallback(
    async (hedgeId: number, farmer: string): Promise<string> => {
      if (!hedgeClient) throw new Error("Hedge client not connected");
      setIsPending(true);
      setError(null);
      try {
        const hash = await hedgeClient.acceptHedge(hedgeId, farmer);
        return hash;
      } catch (err) {
        const msg =
          err instanceof Error ? err.message : "Failed to accept hedge";
        setError(msg);
        throw err;
      } finally {
        setIsPending(false);
      }
    },
    [hedgeClient],
  );

  return { placeHedge, acceptHedge, isPending, error };
}
