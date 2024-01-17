"use client";

import { type ColumnDef } from "@tanstack/react-table";
import moment from "moment";

import { DataTableColumnHeader } from "./DataTableColumnHeader";
import type { BlockData } from "./IndexerBlocksList";

export const BlockColumns: ColumnDef<BlockData>[] = [
  {
    accessorKey: "blockId",
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title="Block Number" />
    ),
    cell: ({ row }) => {
      return <div>{row.getValue("blockId")}</div>;
    },
    enableSorting: false,
    enableHiding: false,
  },
  {
    accessorKey: "timestamp",
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title="Timestamp" />
    ),
    cell: ({ row }) => {
      return <div>{moment.unix(row.getValue("timestamp")).fromNow()}</div>;
    },
    enableSorting: false,
    enableHiding: false,
  },
];
