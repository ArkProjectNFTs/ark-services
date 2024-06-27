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
            <CardTitle>Edit Collection</CardTitle>
            <CardDescription>
              <Link
                className="hover:underline"
                target="_blank"
                href={`https://market.arkproject.dev/collection/${params.address}`}
              >
                View Marketplace Collection
              </Link>
            </CardDescription>
          </CardHeader>
          <CardContent>
            <EditCollectionForm contract={contract} />
          </CardContent>
        </Card>
      </div>
    </TaskFormProvider>
  );
}
