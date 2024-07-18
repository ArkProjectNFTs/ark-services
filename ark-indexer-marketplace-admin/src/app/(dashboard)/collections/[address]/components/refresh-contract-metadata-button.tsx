"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { RotateCcw } from "lucide-react";

import { Button } from "~/components/ui/button";
import { api } from "~/trpc/react";

export default function RefreshContractMetadataButton({
  contractAddress,
  isRefreshing,
}: {
  contractAddress: string;
  isRefreshing: boolean;
}) {
  const router = useRouter();
  const refreshContractMetadata =
    api.contract.refreshContractMetadata.useMutation({});

  async function refreshMetadata() {
    await refreshContractMetadata.mutateAsync({
      contractAddress,
    });

    router.push("/metadata");
  }

  if (isRefreshing) {
    return (
      <Link
        href="/metadata"
        className="inline-flex items-center justify-center rounded-md bg-background px-4 py-2 text-sm font-medium text-muted-foreground shadow-sm transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50"
      >
        Check Metadata Update
      </Link>
    );
  }

  if (refreshContractMetadata.isLoading) {
    return <Button disabled>Loading...</Button>;
  }

  return (
    <Button onClick={() => void refreshMetadata()} variant="secondary">
      <RotateCcw className="mr-2 h-4 w-4" />
      Refresh Metadata
    </Button>
  );
}
