"use client";

import { type ColumnDef } from "@tanstack/react-table";
import moment from "moment";

import { Progress } from "~/components/ui/progress";
import { Checkbox } from "../ui/checkbox";
import CopyPasteLabel from "./CopyPasteLabel";
import DashboardRowActions from "./DashboardRowActions";
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
    cell: ({ row }) => {
      return (
        <div className="w-[150px] cursor-pointer overflow-hidden text-ellipsis">
          <CopyPasteLabel label={row.getValue("taskId")} />
        </div>
      );
    },
    enableSorting: false,
    enableHiding: false,
  },
  {
    accessorKey: "status",
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title="Status" />
    ),
    cell: ({ row }) => {
      return (
        <div className="flex space-x-2">
          <span className="max-w-[500px] truncate font-medium uppercase">
            {row.getValue("status")}
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
    accessorKey: "forceMode",
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title="Force Mode" />
    ),
    cell: ({ row }) => {
      const value: boolean = row.getValue("forceMode") ?? false;
      console.log("value force mode: ", value);
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
