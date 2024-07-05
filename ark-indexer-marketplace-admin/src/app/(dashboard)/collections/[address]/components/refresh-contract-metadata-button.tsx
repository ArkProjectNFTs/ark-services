"use client";

import { RotateCcw } from "lucide-react";

import { Button } from "~/components/ui/button";
import { api } from "~/trpc/react";

export default function RefreshContractMetadataButton({
  contractAddress,
}: {
  contractAddress: string;
}) {
  const refreshContractMetadata =
    api.contract.refreshContractMetadata.useMutation();

  return (
    <Button
      onClick={() => {
        refreshContractMetadata.mutate({
          contractAddress,
        });
      }}
      variant="secondary"
    >
      <RotateCcw className="mr-2 h-4 w-4" />
      Refresh Metadata
    </Button>
  );
}
