/* eslint-disable @typescript-eslint/no-misused-promises */
"use client";

import Link from "next/link";
import { Loader2 } from "lucide-react";

import { percentageString } from "~/lib/percentageString";
import { api } from "~/trpc/react";
import { Button } from "../ui/button";
import { Separator } from "../ui/separator";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "../ui/tooltip";
import BlockRange from "./BlockRange";
import { useNetwork } from "./NetworkProvider";

export default function BlocksOverview() {
  const { network } = useNetwork();
  const [{ latest, ranges, rangeSize, count }, { refetch, isFetching }] =
    api.indexer.allBlocks.useSuspenseQuery(
      {
        network,
      },
      {
        refetchOnWindowFocus: false,
      },
    );
  const percentString = percentageString(latest, count);

  const handleRefresh = async () => {
    await refetch();
  };

  return (
    <div className="mb-6">
      <div className="mb-6 flex items-center space-x-4">
        <h3 className="flex items-center space-x-2 text-2xl font-semibold tracking-tight">
          <span>Blocks</span>
        </h3>
        <div className="flex h-8 items-center space-x-2 rounded-md border px-3 text-xs shadow-sm">
          <div className="flex select-none items-center space-x-1 py-1">
            <span>Network</span>
            <span className="font-semibold">{network}</span>
          </div>
          <Separator orientation="vertical" />
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <Link
                  href={`https://starkscan.co/block/${latest}`}
                  target="_blank"
                  className="flex items-center space-x-1 py-1"
                >
                  <span>Latest block</span>
                  <span className="font-semibold">
                    #{latest.toLocaleString()}
                  </span>
                </Link>
              </TooltipTrigger>
              <TooltipContent sideOffset={8} side="bottom" className="text-xs">
                View block on Starkscan
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
          <Separator orientation="vertical" />
          <div className="flex select-none items-center space-x-1 py-1">
            <span className="">Unindexed blocks</span>
            <span className="font-semibold">{count.toLocaleString()}</span>
            <span className="text-muted-foreground">({percentString}%)</span>
          </div>
        </div>
        <div className="flex-grow" />
        <Button
          size="sm"
          variant="outline"
          className="flex space-x-2"
          disabled={isFetching}
          onClick={handleRefresh}
        >
          {isFetching && <Loader2 size={14} className="animate-spin" />}
          <span>Refresh</span>
        </Button>
      </div>
      <div className="grid grid-cols-4 gap-1 md:grid-cols-10">
        {ranges.map((range) => (
          <BlockRange key={range.start} {...range} />
        ))}
      </div>
      <div className="mt-2 text-xs text-muted-foreground">
        * Blocks are displayed in ranges of {rangeSize.toLocaleString()} blocks
      </div>
    </div>
  );
}
