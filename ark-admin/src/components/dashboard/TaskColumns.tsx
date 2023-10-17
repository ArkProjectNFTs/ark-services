"use client";

import { type ColumnDef } from "@tanstack/react-table";
import { MoreHorizontal } from "lucide-react";
import moment from "moment";

import { Button } from "~/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuTrigger,
} from "~/components/ui/dropdown-menu";
import { Progress } from "~/components/ui/progress";
import { DataTableColumnHeader } from "./DataTableColumnHeader";
import { type TaskData } from "./IndexerTasksList";

export const TaskColumns: ColumnDef<TaskData>[] = [
  // {
  //   id: "select",
  //   header: ({ table }) => (
  //     <Checkbox
  //       checked={table.getIsAllPageRowsSelected()}
  //       onCheckedChange={(value) => table.toggleAllPageRowsSelected(!!value)}
  //       aria-label="Select all"
  //       className="translate-y-[2px]"
  //     />
  //   ),
  //   cell: ({ row }) => (
  //     <Checkbox
  //       checked={row.getIsSelected()}
  //       onCheckedChange={(value) => row.toggleSelected(!!value)}
  //       aria-label="Select row"
  //       className="translate-y-[2px]"
  //     />
  //   ),
  //   enableSorting: false,
  //   enableHiding: false,
  // },
  {
    accessorKey: "taskId",
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title="Task Id" />
    ),
    cell: ({ row }) => (
      <div className="w-[150px] overflow-hidden text-ellipsis">
        {row.getValue("taskId")}
      </div>
    ),
    enableSorting: false,
    enableHiding: false,
  },
  {
    accessorKey: "isRunning",
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title="Status" />
    ),
    cell: ({ row }) => {
      return (
        <div className="flex space-x-2">
          <span className="max-w-[500px] truncate font-medium">
            {row.getValue("isRunning") ? "Running" : "Stopped"}
          </span>
        </div>
      );
    },
  },
  {
    accessorKey: "from",
    header: () => <div>From</div>,
    cell: ({ row }) => {
      return <div>{row.getValue("from")}</div>;
    },
  },
  {
    accessorKey: "to",
    header: () => <div>To</div>,
    cell: ({ row }) => {
      return <div>{row.getValue("to")}</div>;
    },
  },
  {
    accessorKey: "indexationProgress",
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title="Progress" />
    ),
    cell: ({ row }) => {
      return (
        <div className="flex flex-row gap-4">
          <span>{row.getValue("indexationProgress")}%</span>
          <Progress value={row.getValue("indexationProgress")} max={100} />
        </div>
      );
    },
  },
  {
    accessorKey: "createdAt",
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title="Creation Date" />
    ),
    cell: ({ row }) => {
      const momentTimestamp = row.getValue("createdAt")
        ? moment.unix(row.getValue("createdAt")).fromNow()
        : "---";
      return <div className="flex items-center">{momentTimestamp}</div>;
    },
  },
  // {
  //   accessorKey: "updatedAt",
  //   header: ({ column }) => (
  //     <DataTableColumnHeader column={column} title="Last Update" />
  //   ),
  //   cell: ({ row }) => {
  //     const momentTimestamp = moment.unix(row.getValue("updatedAt"));
  //     return (
  //       <div className="flex items-center">{momentTimestamp.fromNow()}</div>
  //     );
  //   },
  // },
  {
    accessorKey: "version",
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title="Version" />
    ),
    cell: ({ row }) => {
      return <div className="flex items-center">{row.getValue("version")}</div>;
    },
  },
  {
    id: "actions",
    enableHiding: false,
    cell: ({ row }) => {
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
                const url = `https://us-east-1.console.aws.amazon.com/cloudwatch/home?region=us-east-1#logsV2:log-groups/log-group/$252Fecs$252Fark-indexer-mainnet/log-events/ecs$252Fark_indexer$252F${taskId}`;
                window.open(url, "_blank");
              }}
            >
              View logs
            </DropdownMenuItem>
            <DropdownMenuItem
              className="cursor-pointer"
              onClick={() => {
                const taskId = row.getValue<string>("taskId");
                const url = `https://us-east-1.console.aws.amazon.com/cloudwatch/home?region=us-east-1#logsV2:live-tail$3FlogGroupArns$3D~(~'arn*3aaws*3alogs*3aus-east-1*3a223605539824*3alog-group*3a*2fecs*2fark-indexer-mainnet*3a*2a)$26logStreamNames$3D~(~'ecs*2fark_indexer*2f${taskId})`;
                window.open(url, "_blank");
              }}
            >
              Log stream
            </DropdownMenuItem>
            {/* <DropdownMenuSeparator />
            <DropdownMenuItem>View logs</DropdownMenuItem> */}
          </DropdownMenuContent>
        </DropdownMenu>
      );
    },
  },
];
