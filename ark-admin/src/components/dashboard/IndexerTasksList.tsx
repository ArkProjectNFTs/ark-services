"use client";

import { api } from "~/trpc/react";
import { DataTable } from "./DataTable";
import { useNetwork } from "./NetworkProvider";
import { TaskColumns } from "./TaskColumns";

export interface TaskData {
  indexationProgress: number;
  taskId: string;
  from: number;
  to: number;
  version?: string;
  updatedAt?: string;
  createdAt?: string;
}

export default function IndexerTasksList() {
  const { network } = useNetwork();

  const [tasks] = api.indexer.allTasks.useSuspenseQuery(
    {
      network,
    },
    {
      refetchInterval: 5000,
    },
  );

  return (
    <div className="mt-10 w-full">
      <h3 className="mb-4 text-2xl font-semibold tracking-tight">Tasks</h3>
      <div className="grid h-full items-stretch gap-6">
        <div className="md:order-1">
          <DataTable<TaskData, TaskData>
            data={tasks ?? []}
            columns={TaskColumns}
            hasFilter={true}
          />
        </div>
      </div>
    </div>
  );
}
