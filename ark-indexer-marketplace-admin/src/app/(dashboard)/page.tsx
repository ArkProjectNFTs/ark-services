import { Suspense } from "react";

import CollectionSearch from "~/components/CollectionSearch";
import ContractGridItem from "~/components/dashboard/ContractGridItem";
import { api } from "~/trpc/server";
import SearchContracts from "../../components/SearchContracts";

export default async function DashboardPage({
  params,
}: {
  params: { search: string };
}) {
  const contracts = await api.contract.getContracts.query();

  return (
    <div className="container mx-auto px-4 py-12 sm:px-6 lg:px-8">
      <div className="mb-8 flex flex-col gap-2">
        <h1 className="text-3xl font-bold">Contracts</h1>
        <p className="text-muted-foreground">
          Explore the contracts that match your search query.
        </p>
        <CollectionSearch />
      </div>
      <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-6">
        {contracts.map((contract) => (
          <ContractGridItem
            key={contract.contract_address}
            contract={contract}
          />
        ))}
      </div>
    </div>
  );
}
