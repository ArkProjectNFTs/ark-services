"use client";

// import { DataTableViewOptions } from "@/app/examples/tasks/components/data-table-view-options";
// import { Button } from "@/registry/new-york/ui/button";
// import { Input } from "@/registry/new-york/ui/input";
import { Cross2Icon } from "@radix-ui/react-icons";
import type { Table } from "@tanstack/react-table";

import { Button } from "../ui/button";
// import { priorities, statuses } from "../data/data";
import { Input } from "../ui/input";
// import { DataTableFacetedFilter } from "./data-table-faceted-filter";
import { DataTableViewOptions } from "./DataTableViewOptions";

interface DataTableToolbarProps<TData> {
  table: Table<TData>;
}

export function DataTableToolbar<TData>({
  table,
}: DataTableToolbarProps<TData>) {
  const isFiltered = table.getState().columnFilters.length > 0;

  return (
    <div className="flex items-center justify-between">
      <div className="flex flex-1 items-center space-x-2">
        <Input
          placeholder="Filter Task ID..."
          value={(table.getColumn("taskId")?.getFilterValue() as string) ?? ""}
          onChange={(event) =>
            table.getColumn("taskId")?.setFilterValue(event.target.value)
          }
          className="h-8 w-[150px] lg:w-[250px]"
        />
        {isFiltered && (
          <Button
            variant="ghost"
            onClick={() => table.resetColumnFilters()}
            className="h-8 px-2 lg:px-3"
          >
            Reset
            <Cross2Icon className="ml-2 h-4 w-4" />
          </Button>
        )}
      </div>
      <DataTableViewOptions table={table} />
    </div>
  );
}
