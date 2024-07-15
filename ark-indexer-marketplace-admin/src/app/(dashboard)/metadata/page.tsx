/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import { api } from "~/trpc/server";
import RefreshingCollections from "./components/RefreshingCollections";

export default async function DashboardMetadataPage() {
  const contracts = await api.contract.getRefreshingContracts.query({
    refetchInterval: 10000,
  });

  return (
    <div className=" p-6">
      <RefreshingCollections contracts={contracts} />
      {/* <MetadataIndicator /> */}
    </div>
  );
}
