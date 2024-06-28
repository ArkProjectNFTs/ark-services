/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import Link from "next/link";
import { LinkIcon, RotateCcw } from "lucide-react";

import EditCollectionForm from "~/components/dashboard/EditCollectionForm";
import { Button } from "~/components/ui/button";
import { Separator } from "~/components/ui/separator";
import { api } from "~/trpc/server";

export default async function CollectionPage({
  params,
}: {
  params: { address: string };
}) {
  const contract = await api.contract.getContract.query({
    contractAddress: params.address,
  });

  return (
    <>
      <div className="container mx-auto flex items-center justify-between">
        <div className="space-y-0.5">
          <h2 className="text-2xl font-bold tracking-tight">Edit Collection</h2>
          <p className="text-muted-foreground">Edit the collection details.</p>
        </div>

        <nav className="flex items-center space-x-4">
          <Link
            target="_blank"
            href={`https://starkscan.co/nft-contract/${params.address}`}
            className="inline-flex items-center justify-center rounded-md bg-background px-4 py-2 text-sm font-medium text-muted-foreground shadow-sm transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50"
            prefetch={false}
          >
            <LinkIcon className="mr-2 h-4 w-4" />
            Starkscan
          </Link>
          <Link
            target="_blank"
            href={`https://market.arkproject.dev/collection/${params.address}`}
            className="inline-flex items-center justify-center rounded-md bg-background px-4 py-2 text-sm font-medium text-muted-foreground shadow-sm transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50"
            prefetch={false}
          >
            <LinkIcon className="mr-2 h-4 w-4" />
            Ark Market
          </Link>
          <Button variant="secondary">
            <RotateCcw className="mr-2 h-4 w-4" />
            Refresh Metadata
          </Button>
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
