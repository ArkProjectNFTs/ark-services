"use client";

import { api } from "~/trpc/react";
import { BlockColumns } from "./BlockColumns";
import { DataTable } from "./DataTable";
import { useNetwork } from "./NetworkProvider";

export interface BlockData {
  blockId: number;
  timestamp: number;
}

export default function IndexerBlocksList() {
  const { network } = useNetwork();
  const [latestBlocks] = api.indexer.latestBlocks.useSuspenseQuery(
    {
      network,
    },
    {
      refetchInterval: 5000,
    },
  );

  return (
    <div className="mt-10 w-full">
      <h3 className="mb-4 text-2xl font-semibold tracking-tight">
        Latest Indexed Blocks
      </h3>
      <div className="grid h-full items-stretch gap-6">
        <div className="md:order-1">
          <DataTable<BlockData, BlockData>
            data={latestBlocks ?? []}
            columns={BlockColumns}
            hasFilter={false}
          />
        </div>
      </div>
    </div>
  );
}
