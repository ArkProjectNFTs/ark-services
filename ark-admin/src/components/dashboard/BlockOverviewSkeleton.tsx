"use client";

import { Separator } from "../ui/separator";
import { Skeleton } from "../ui/skeleton";

export default function BlocksOverviewSkeleton() {
  return (
    <>
      <div className="mb-6 flex items-center space-x-4">
        <h3 className="flex items-center space-x-2 text-2xl font-semibold tracking-tight">
          <span>Blocks</span>
        </h3>
        <div className="flex h-8 items-center space-x-2 rounded-md border px-3 text-xs shadow-sm">
          <div className="flex items-center space-x-1 py-1">
            <span>Lastest</span>
            <span className="font-semibold">#xxx</span>
          </div>
          <Separator orientation="vertical" />
          <div className="flex items-center space-x-1 py-1">
            <span className="">Unindexed</span>
            <span className="font-semibold">xxxx</span>
            <span className="text-muted-foreground">(xx%)</span>
          </div>
          <Separator orientation="vertical" />
          <div>Last refresh 2 days ago</div>
        </div>
        <div className="flex-grow" />
      </div>
      <Skeleton className="h-[432px] w-full rounded-md border shadow-sm" />
    </>
  );
}
