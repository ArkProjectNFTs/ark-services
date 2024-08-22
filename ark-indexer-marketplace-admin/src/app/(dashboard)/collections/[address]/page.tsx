import { Suspense } from "react";

import Contract from "./components/Contract";

export default function CollectionPage({
  params,
}: {
  params: { address: string };
}) {
  return (
    <Suspense>
      <Contract address={params.address} />
    </Suspense>
  );
}
