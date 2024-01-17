"use client";

import { Separator } from "../ui/separator";
import { Skeleton } from "../ui/skeleton";

export default function BlocksOverviewSkeleton() {
  return (
    <div className="mb-6">
      <div className="mb-6 flex items-center space-x-4">
        <h3 className="flex items-center space-x-2 text-2xl font-semibold tracking-tight">
          <span>Blocks</span>
        </h3>
        <div className="flex h-8 items-center space-x-2 rounded-md border px-3 text-xs shadow-sm">
          <div className="flex items-center space-x-1 py-1">
            <span>Network</span>
            <Skeleton className="h-4 w-[42px]" />
          </div>
          <Separator orientation="vertical" />
          <div className="flex items-center space-x-1 py-1">
            <span>Latest block</span>
            <Skeleton className="h-4 w-[60px]" />
          </div>
          <Separator orientation="vertical" />
          <div className="flex items-center space-x-1 py-1">
            <span className="">Unindexed blocks</span>
            <Skeleton className="h-4 w-[90px]" />
          </div>
        </div>
      </div>
      <Skeleton className="h-[332px] w-full rounded-md border shadow-sm" />
      <div className="mt-2 text-xs text-muted-foreground">
        * Blocks are displayed in ranges of blocks
      </div>
    </div>
  );
}
