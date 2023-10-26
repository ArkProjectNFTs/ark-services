"use client";

import { MoveRight, RefreshCw } from "lucide-react";

import { splitIntoRanges } from "~/lib/splitIntoRanges";
import { api } from "~/trpc/react";
import { Button } from "../ui/button";
import { Separator } from "../ui/separator";
import { useNetwork } from "./NetworkProvider";

export default function BlocksOverview() {
  const { network } = useNetwork();
  const [{ blocks, latest }] = api.indexer.allBlocks.useSuspenseQuery({
    network,
  });

  const ranges = splitIntoRanges(latest, 120);
  const percent = (blocks.length / latest) * 100;
  const percentString = percent < 1 ? percent.toFixed(2) : percent.toFixed(0);

  console.log(blocks);

  return (
    <div className="">
      <div className="mb-6 flex items-center space-x-4">
        <h3 className="flex items-center space-x-2 text-2xl font-semibold tracking-tight">
          <span>Blocks</span>
        </h3>
        <div className="flex h-8 items-center space-x-2 rounded-md border px-3 text-xs shadow-sm">
          <div className="flex items-center space-x-1 py-1">
            <span>Lastest</span>
            <span className="font-semibold">{latest.toLocaleString()}</span>
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
        <Button size="sm" variant="outline" className="flex space-x-2">
          <RefreshCw size={14} className="" />
          <span>Refresh</span>
        </Button>
        <Button size="sm" variant="outline" className="flex space-x-2">
          <span>View all blocks</span>
          <MoveRight size={14} />
        </Button>
      </div>
      <div className="grid grid-cols-10 gap-1">
        {ranges.map((range, i) => (
          <div
            key={i}
            className="rounded:md flex h-6 w-full justify-between bg-green-100 p-1 text-xs ring-1 ring-green-300 ring-opacity-50"
          >
            {/* <div className="">{range[0]?.toLocaleString()}</div>
            <div className="">{range[range.length - 1]?.toLocaleString()}</div> */}
          </div>
        ))}
      </div>
    </div>
  );
}
