/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import { api } from "~/trpc/server";
import RefreshingCollections from "./components/RefreshingCollections";

export default async function DashboardMetadataPage() {
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access
  const contracts = await api.contract.getRefreshingContracts.query({});

  return (
    <div className=" p-6">
      <RefreshingCollections contracts={contracts} />
      {/* <MetadataIndicator /> */}
    </div>
  );
}
