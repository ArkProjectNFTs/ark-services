"use client";

import { MoveRight, RefreshCw } from "lucide-react";

import { percentageString } from "~/lib/percentageString";
import { cn } from "~/lib/utils";
import { api } from "~/trpc/react";
import { Button } from "../ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "../ui/popover";
import { Separator } from "../ui/separator";
import { useNetwork } from "./NetworkProvider";
import { useTaskForm } from "./TaskFormProvider";

export default function BlocksOverview() {
  const { network } = useNetwork();
  const [{ blocks, latest, ranges }, { refetch, isFetching }] =
    api.indexer.allBlocks.useSuspenseQuery({
      network,
    });
  const { setState } = useTaskForm();
  const percentString = percentageString(latest, blocks.length);

  const handleAddtask = (blocks: number[]) => {
    setState({
      from: blocks[0]!.toString(),
      to: blocks[blocks.length - 1]!.toString(),
      numberOfTasks: Math.round(blocks.length / 10).toString(),
    });
  };

  const handleRefresh = async () => {
    await refetch();
  };

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
          <span>Refresh</span>
          <RefreshCw size={14} className={cn(isFetching && "animate-spin")} />
        </Button>
        <Button size="sm" variant="outline" className="flex space-x-2">
          <span>View all blocks</span>
          <MoveRight size={14} />
        </Button>
      </div>
      <div className="grid grid-cols-10 gap-1">
        {ranges.map((range, i) => (
          <>
            {range.hasUnindexed ? (
              <Popover key={i}>
                <PopoverTrigger>
                  <div className="flex h-6 w-full select-none justify-between bg-red-500 p-1 text-xs text-white">
                    <div className="">{range?.start?.toLocaleString()}</div>
                    <div className="">{range?.end?.toLocaleString()}</div>
                  </div>
                </PopoverTrigger>
                <PopoverContent className="w-48 p-2">
                  <div className="mb-4">
                    <div className="mb-4 flex items-center text-xs">
                      <div>#{range?.start?.toLocaleString()}</div>
                      <MoveRight size={14} className="flex-grow" />
                      <div>#{range?.end?.toLocaleString()}</div>
                    </div>
                    <div className="">
                      <div className="text-xs text-muted-foreground">
                        Blocks
                      </div>
                      <div className="text-2xl font-semibold">
                        {range.end! - range.start!}
                      </div>
                    </div>
                    <div className="">
                      <div className="text-xs text-muted-foreground">
                        Unindexed blocks
                      </div>
                      <div className="text-2xl font-semibold">
                        <span>{range.blocks.length}</span>
                        {/* <span>{percentString(blocks.length)}</span> */}
                      </div>
                    </div>
                  </div>
                  <Button
                    size="sm"
                    className="w-full"
                    onClick={() => handleAddtask(range.blocks)}
                  >
                    Add Task
                  </Button>
                </PopoverContent>
              </Popover>
            ) : (
              <div className="flex h-6 w-full select-none justify-between bg-green-500 p-1 text-xs text-white">
                <div className="">{range?.start?.toLocaleString()}</div>
                <div className="">{range?.end?.toLocaleString()}</div>
              </div>
            )}
          </>
        ))}
      </div>
    </div>
  );
}
