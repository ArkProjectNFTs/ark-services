import { Suspense } from "react";

import SearchCollectionsLayout from "~/components/dashboard/SearchCollectionsLayout";
import SearchContracts from "~/components/SearchContracts";

export default function CollectionSearch({
  params,
}: {
  params: { search: string };
}) {
  return (
    <SearchCollectionsLayout search={params.search}>
      <Suspense>
        <SearchContracts search={params.search} />
      </Suspense>
    </SearchCollectionsLayout>
  );
}
