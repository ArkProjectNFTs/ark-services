/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import Link from "next/link";

import EditCollectionForm from "~/components/dashboard/EditCollectionForm";
import { TaskFormProvider } from "~/components/dashboard/TaskFormProvider";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "~/components/ui/card";
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
    <TaskFormProvider>
      <div className="flex h-screen items-center justify-center">
        <Card>
          <CardHeader>
            <div className="flex items-center">
              <div>
                <CardTitle>Edit Collection</CardTitle>
                <CardDescription>Edit the collection details.</CardDescription>
              </div>
              <div className="ml-auto gap-2 text-xs text-foreground">
                <div>
                  <Link
                    className="hover:underline"
                    target="_blank"
                    href={`https://market.arkproject.dev/collection/${params.address}`}
                  >
                    Ark Market
                  </Link>
                </div>
                <div>
                  <Link
                    className="hover:underline"
                    target="_blank"
                    href={`https://starkscan.co/nft-contract/${params.address}#items`}
                  >
                    Starkscan
                  </Link>
                </div>
              </div>
            </div>
          </CardHeader>
          <CardContent>
            <EditCollectionForm contract={contract} />
          </CardContent>
        </Card>
      </div>
    </TaskFormProvider>
  );
}
