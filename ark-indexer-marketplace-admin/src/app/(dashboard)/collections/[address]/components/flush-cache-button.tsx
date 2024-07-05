"use client";

import { Eraser } from "lucide-react";

import { Button } from "~/components/ui/button";
import { api } from "~/trpc/react";

export default function FlushCacheButton(props: { contractAddress: string }) {
  const flushCacheMutation = api.contract.flushCache.useMutation();

  function flushCache() {
    flushCacheMutation.mutate({
      contractAddress: props.contractAddress,
    });
  }

  return (
    <Button onClick={() => flushCache()} variant="secondary">
      <Eraser className="mr-2 h-4 w-4" />
      Flush Cache
    </Button>
  );
}
