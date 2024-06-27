import { MetadataIndicator } from "~/components/dashboard/MetadataIndicator";
import { api } from "~/trpc/server";
import RefreshingCollections from "./components/RefreshingCollections";

export default async function DashboardMetadataPage() {
  const contracts = await api.contract.getRefreshingContracts.query({});

  return (
    <div className=" p-6">
      <RefreshingCollections contracts={contracts} />
      {/* <MetadataIndicator /> */}
    </div>
  );
}
