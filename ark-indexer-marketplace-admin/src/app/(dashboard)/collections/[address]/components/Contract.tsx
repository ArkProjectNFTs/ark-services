import Link from "next/link";
import { LinkIcon } from "lucide-react";

import EditCollectionForm from "~/components/dashboard/EditCollectionForm";
import { Separator } from "~/components/ui/separator";
import { api } from "~/trpc/react";
import FlushCacheButton from "./FlushCacheButton";
import RefreshContractMetadataButton from "./RefreshContractMetadataButton";

export default function Contract(props: { address: string }) {
  const [contract] = api.contract.getContract.useSuspenseQuery({
    contractAddress: props.address,
  });

  return (
    <>
      <div className="container mx-auto flex items-center justify-between">
        <div className="space-y-0.5">
          <h2 className="text-2xl font-bold tracking-tight">Edit Collection</h2>
          <p className="text-muted-foreground">Edit the collection details.</p>
        </div>

        <nav className="flex items-center space-x-4">
          {contract?.contract_address && (
            <>
              <FlushCacheButton contractAddress={contract.contract_address} />
              <RefreshContractMetadataButton
                contractAddress={contract.contract_address}
                isRefreshing={contract.is_refreshing}
              />
            </>
          )}

          <Link
            target="_blank"
            href={`https://starkscan.co/nft-contract/${props.address}`}
            className="inline-flex items-center justify-center rounded-md bg-background px-4 py-2 text-sm font-medium text-muted-foreground shadow-sm transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50"
            prefetch={false}
          >
            <LinkIcon className="mr-2 h-4 w-4" />
            View on Starkscan
          </Link>
          <Link
            target="_blank"
            href={`https://market.arkproject.dev/collection/${props.address}`}
            className="inline-flex items-center justify-center rounded-md bg-background px-4 py-2 text-sm font-medium text-muted-foreground shadow-sm transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50"
            prefetch={false}
          >
            <LinkIcon className="mr-2 h-4 w-4" />
            View on ArkProject
          </Link>
        </nav>
      </div>

      <Separator className="my-6" />
      <div className="flex flex-col space-y-8 lg:flex-row lg:space-x-12 lg:space-y-0">
        {/* <aside className="-mx-4 lg:w-1/5"></aside> */}
        <div className="flex-1 lg:max-w-2xl">
          <div className="space-y-6">
            <EditCollectionForm contract={contract} />
          </div>
        </div>
      </div>
    </>
  );
}
