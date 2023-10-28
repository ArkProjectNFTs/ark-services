"use client";

import { MoveRight } from "lucide-react";

import { Button } from "../ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "../ui/popover";
import { useTaskForm } from "./TaskFormProvider";

type BlockRangeProps = {
  start: number;
  end: number;
  blocks: number[];
  hasUnindexed: boolean;
};

export default function BlockRange({
  start,
  end,
  blocks,
  hasUnindexed,
}: BlockRangeProps) {
  const { setValues } = useTaskForm();

  const handleAddtask = (blocks: number[]) => {
    setValues({
      from: blocks[0]!.toString(),
      to: blocks[blocks.length - 1]!.toString(),
      numberOfTasks: "3",
    });
  };

  if (hasUnindexed) {
    return (
      <Popover>
        <PopoverTrigger className="w-full cursor-default">
          <div className="flex h-6 w-full select-none justify-between bg-red-500 p-1 text-xs text-white">
            <div className="">{start?.toLocaleString()}</div>
            <div className="">{end?.toLocaleString()}</div>
          </div>
        </PopoverTrigger>
        <PopoverContent className="w-48 p-2">
          <div className="mb-4">
            <div className="mb-4 flex items-center text-xs">
              <div>#{start?.toLocaleString()}</div>
              <MoveRight size={14} className="flex-grow" />
              <div>#{end?.toLocaleString()}</div>
            </div>
            <div className="">
              <div className="text-xs text-muted-foreground">Blocks</div>
              <div className="text-2xl font-semibold">{end - start}</div>
            </div>
            <div className="">
              <div className="text-xs text-muted-foreground">
                Unindexed blocks
              </div>
              <div className="text-2xl font-semibold">
                <span>{blocks.length}</span>
              </div>
            </div>
          </div>
          <Button
            size="sm"
            className="w-full"
            onClick={() => handleAddtask(blocks)}
          >
            Add Task
          </Button>
        </PopoverContent>
      </Popover>
    );
  }

  return (
    <div className="flex h-6 w-full select-none justify-between bg-green-500 p-1 text-xs text-white">
      <div className="">{start?.toLocaleString()}</div>
      <div className="">{end?.toLocaleString()}</div>
    </div>
  );
}
