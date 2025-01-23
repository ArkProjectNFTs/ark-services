"use client";

import ContractGridItem from "~/components/dashboard/ContractGridItem";
import { api } from "~/trpc/react";

export default function SearchContracts(props: { search: string }) {
  const [contracts] = api.contract.searchContracts.useSuspenseQuery({
    contractName: props.search,
  });

  return (
    <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-6">
      {contracts.map((contract) => (
        <ContractGridItem key={contract.contract_address} contract={contract} />
      ))}
    </div>
  );
}
