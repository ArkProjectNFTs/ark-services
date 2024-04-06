import { MetadataIndicator } from "~/components/dashboard/MetadataIndicator";

export default function DashboardMetadataPage() {
  return (
    <div className="flex-1 p-6">
      <div className="flex space-x-12">
        <MetadataIndicator />
      </div>
    </div>
  );
}
