import { Suspense } from "react";

import SearchContracts from "./components/SearchContracts";

export default function CollectionSearch({
  params,
}: {
  params: { search: string };
}) {
  return (
    <div className="container mx-auto px-4 py-12 sm:px-6 lg:px-8">
      <div className="mb-8">
        <h1 className="text-3xl font-bold">Search Results</h1>
        <p className="text-muted-foreground">
          Explore the contracts that match your search query.
        </p>
      </div>
      <Suspense>
        <SearchContracts search={params.search} />
      </Suspense>
    </div>
  );
}
