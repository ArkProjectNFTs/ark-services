import type { Row } from "@tanstack/react-table";
import { MoreHorizontal } from "lucide-react";

import { api } from "~/trpc/react";
import { Button } from "../ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "../ui/dropdown-menu";
import { useNetwork } from "./NetworkProvider";

interface DashboardRowActionsProps<TData> {
  row: Row<TData>;
}

export function DashboardRowActions<TData>({
  row,
}: DashboardRowActionsProps<TData>) {
  const { network } = useNetwork();

  const indexerQuery = api.indexer.allTasks.useQuery({ network });
  const deleteTaskMutation = api.indexer.deleteTask.useMutation({
    onSuccess: async () => {
      await indexerQuery.refetch();
    },
  });

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" className="h-8 w-8 p-0">
          <span className="sr-only">Open menu</span>
          <MoreHorizontal className="h-4 w-4" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuLabel>Actions</DropdownMenuLabel>
        <DropdownMenuItem
          className="cursor-pointer"
          onClick={() => {
            const taskId = row.getValue<string>("taskId");
            const url = `https://us-east-1.console.aws.amazon.com/cloudwatch/home?region=us-east-1#logsV2:log-groups/log-group/$252Fecs$252Fark-indexer-${network}/log-events/ecs$252Fark_indexer$252F${taskId}`;
            window.open(url, "_blank");
          }}
        >
          View logs
        </DropdownMenuItem>
        <DropdownMenuItem
          className="cursor-pointer"
          onClick={() => {
            const taskId = row.getValue<string>("taskId");
            const url = `https://us-east-1.console.aws.amazon.com/cloudwatch/home?region=us-east-1#logsV2:live-tail$3FlogGroupArns$3D~(~'arn*3aaws*3alogs*3aus-east-1*3a223605539824*3alog-group*3a*2fecs*2fark-indexer-${network}*3a*2a)$26logStreamNames$3D~(~'ecs*2fark_indexer*2f${taskId})`;
            window.open(url, "_blank");
          }}
        >
          Log stream
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem
          className="cursor-pointer"
          onClick={() => {
            const taskId = row.getValue<string>("taskId");
            deleteTaskMutation.mutate({
              network,
              taskId,
            });
          }}
        >
          Delete
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

export default DashboardRowActions;
