"use client";

import { api } from "~/trpc/react";
import CreateIndexerTaskFrom from "./CreateIndexerTaskForm";
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
    <div className="container h-full py-6">
      <div className="grid h-full items-stretch gap-6 md:grid-cols-[1fr_200px]">
        <div className="md:order-1">
          <DataTable<TaskData, TaskData>
            data={tasks ?? []}
            columns={TaskColumns}
          />
        </div>
        <div className="hidden flex-col space-y-4 sm:flex md:order-2">
          <CreateIndexerTaskFrom network={network} />
        </div>
      </div>
    </div>
  );
}
