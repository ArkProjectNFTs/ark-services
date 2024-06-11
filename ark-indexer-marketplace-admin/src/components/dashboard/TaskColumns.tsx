"use client";

import { type ColumnDef } from "@tanstack/react-table";
import moment from "moment";

import { Progress } from "~/components/ui/progress";
import { Badge } from "../ui/badge";
import { Checkbox } from "../ui/checkbox";
import CopyPasteLabel from "./CopyPasteLabel";
import DashboardRowActions from "./DashboardRowActions";
import { DataTableColumnHeader } from "./DataTableColumnHeader";
import { type TaskData } from "./IndexerTasksList";

export const TaskColumns: ColumnDef<TaskData>[] = [
  {
    accessorKey: "taskId",
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title="Task Id" />
    ),
    cell: ({ row }) => {
      return (
        <div className="w-[150px] cursor-pointer overflow-hidden text-ellipsis">
          <CopyPasteLabel label={row.getValue("taskId")} />
        </div>
      );
    },
    enableSorting: true,
    enableHiding: true,
  },
  {
    accessorKey: "status",
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title="Status" />
    ),
    cell: ({ row }) => {
      return <Badge variant="outline">{row.getValue("status")}</Badge>;
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
        <div className="flex min-w-[160px] flex-row items-center justify-center gap-4">
          <span className="min-w-[40px]">
            {row.getValue("indexationProgress")}%
          </span>
          <Progress value={row.getValue("indexationProgress")} max={100} />
        </div>
      );
    },
  },
  {
    accessorKey: "currentBlockNumber",
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title="Block" />
    ),
    cell: ({ row }) => <div>{row.getValue("currentBlockNumber") ?? "---"}</div>,
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
    enableHiding: true,
    accessorKey: "forceMode",
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title="Force Mode" />
    ),
    cell: ({ row }) => {
      const value: boolean = row.getValue("forceMode") ?? false;
      return (
        <div className="flex items-center space-x-2">
          <Checkbox checked={value} />
        </div>
      );
    },
  },
  {
    id: "actions",
    enableHiding: false,
    cell: ({ row }) => <DashboardRowActions row={row} />,
  },
];
