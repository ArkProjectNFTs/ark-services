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

const GRADIENT_OFFSET = 30;

export default function BlockRange({ start, end, blocks }: BlockRangeProps) {
  const { setValues } = useTaskForm();
  const percent = 100 - Math.round((blocks.length / (end - start + 1)) * 100);

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
          className="h-6 w-full select-none items-end p-1 text-right text-xs text-white"
          style={{
            background: `linear-gradient(90deg, #10B981 ${
              percent === 100 ? 100 : percent - GRADIENT_OFFSET
            }%, #EF4444 ${percent ? percent + GRADIENT_OFFSET : 0}%)`,
          }}
        >
          {blocks.length ? blocks.length : ""}
        </div>
      </PopoverTrigger>
      <PopoverContent className="w-48 p-2">
        <div className="">
          <div className={cn("mb-4 flex items-center rounded-sm p-1 text-xs")}>
            <div>#{start.toLocaleString()}</div>
            <MoveRight size={14} className="flex-grow" />
            <div>#{end.toLocaleString()}</div>
          </div>
          <div className="">
            <div className="text-xs text-muted-foreground">Blocks</div>
            <div className="text-2xl font-semibold">{end - start + 1}</div>
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
        {blocks.length > 0 && (
          <Button
            size="sm"
            className="mt-4 w-full"
            onClick={() => handleAddtask(blocks)}
          >
            Add Task
          </Button>
        )}
      </PopoverContent>
    </Popover>
  );
}
