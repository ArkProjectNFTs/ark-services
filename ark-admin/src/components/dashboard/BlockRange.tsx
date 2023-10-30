"use client";

import { MoveRight } from "lucide-react";

import { cn } from "~/lib/utils";
import { Button } from "../ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "../ui/popover";
import { useTaskForm } from "./TaskFormProvider";

type BlockRangeProps = {
  start: number;
  end: number;
  blocks: number[];
};

export default function BlockRange({ start, end, blocks }: BlockRangeProps) {
  const { setValues } = useTaskForm();

  const handleAddtask = (blocks: number[]) => {
    setValues({
      from: blocks[0]!.toString(),
      to: blocks[blocks.length - 1]!.toString(),
      numberOfTasks: "3",
    });
  };

  return (
    <Popover>
      <PopoverTrigger className="w-full cursor-default">
        <div
          className={cn(
            "flex h-6 w-full select-none justify-between bg-green-500 p-1 text-xs text-white",
            blocks.length && "bg-red-500 ",
          )}
        />
      </PopoverTrigger>
      <PopoverContent className="w-48 p-2">
        <div className="mb-4">
          <div className="mb-4 flex items-center text-xs">
            <div>#{start.toLocaleString()}</div>
            <MoveRight size={14} className="flex-grow" />
            <div>#{end.toLocaleString()}</div>
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
        {blocks.length && (
          <Button
            size="sm"
            className="w-full"
            onClick={() => handleAddtask(blocks)}
          >
            Add Task
          </Button>
        )}
      </PopoverContent>
    </Popover>
  );
}
