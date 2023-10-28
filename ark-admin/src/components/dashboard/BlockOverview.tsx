"use client";

import { Loader2, MoveRight } from "lucide-react";

import { percentageString } from "~/lib/percentageString";
import { api } from "~/trpc/react";
import { Button } from "../ui/button";
import { Separator } from "../ui/separator";
import BlockRange from "./BlockRange";
import { useNetwork } from "./NetworkProvider";

export default function BlocksOverview() {
  const { network } = useNetwork();
  const [{ blocks, latest, ranges }, { refetch, isLoading, isFetching }] =
    api.indexer.allBlocks.useSuspenseQuery(
      {
        network,
      },
      {
        refetchOnWindowFocus: false,
      },
    );
  const percentString = percentageString(latest, blocks.length);

  const handleRefresh = async () => {
    await refetch();
  };

  console.log("BlocksOverview.render", isLoading);

  return (
    <div className="">
      <div className="mb-6 flex items-center space-x-4">
        <h3 className="flex items-center space-x-2 text-2xl font-semibold tracking-tight">
          <span>Blocks</span>
        </h3>
        <div className="flex h-8 items-center space-x-2 rounded-md border px-3 text-xs shadow-sm">
          <div className="flex items-center space-x-1 py-1">
            <span>Lastest</span>
            <span className="font-semibold">#{latest.toLocaleString()}</span>
          </div>
          <Separator orientation="vertical" />
          <div className="flex items-center space-x-1 py-1">
            <span className="">Unindexed</span>
            <span className="font-semibold">
              {blocks.length.toLocaleString()}
            </span>
            <span className="text-muted-foreground">({percentString}%)</span>
          </div>
          <Separator orientation="vertical" />
          <div>Last refresh 2 days ago</div>
        </div>
        <div className="flex-grow" />
        <Button
          size="sm"
          variant="outline"
          className="flex space-x-2"
          // eslint-disable-next-line @typescript-eslint/no-misused-promises
          onClick={handleRefresh}
        >
          {isFetching && <Loader2 size={14} className="animate-spin" />}
          <span>Refresh</span>
        </Button>
        <Button size="sm" variant="outline" className="flex space-x-2">
          <span>View all blocks</span>
          <MoveRight size={14} />
        </Button>
      </div>
      <div className="grid grid-cols-10 gap-1">
        {ranges.map((range) => (
          <BlockRange key={range.start} {...range} />
        ))}
      </div>
    </div>
  );
}
