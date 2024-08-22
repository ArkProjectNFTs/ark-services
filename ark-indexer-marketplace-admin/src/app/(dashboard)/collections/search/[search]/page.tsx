import Link from "next/link";

import { api } from "~/trpc/server";

export default async function CollectionSearch({
  params,
}: {
  params: { search: string };
}) {
  console.log("params", params);

  const contracts = await api.contract.searchContracts.query({
    contractName: "test",
  });

  return (
    <div className="container mx-auto px-4 py-12 sm:px-6 lg:px-8">
      <div className="mb-8">
        <h1 className="text-3xl font-bold">Search Results</h1>
        <p className="text-muted-foreground">
          Explore the contracts that match your search query.
        </p>
      </div>
      <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-6">
        {contracts.map((contract) => (
          <div key={contract.contract_address} className="space-y-3">
            <Link
              href={`/collections/${contract.contract_address}`}
              className="relative block overflow-hidden rounded-md"
              prefetch={false}
            >
              {contract.contract_image ? (
                <div>
                  <img
                    src={contract.contract_image}
                    alt={contract.contract_name ?? ""}
                    width={400}
                    height={300}
                    className="aspect-square h-auto w-auto object-cover transition-all hover:scale-105"
                  />
                </div>
              ) : (
                <div className="h-[202px] w-full object-cover transition-opacity group-hover:opacity-80" />
              )}

              <div className="absolute inset-0 flex items-end bg-gradient-to-t from-black/70 to-transparent p-4">
                <div>
                  <h3 className="text-lg font-semibold text-white">
                    {contract.contract_name}
                  </h3>
                  <p className="w-full overflow-hidden text-ellipsis whitespace-nowrap text-sm text-white/80"></p>
                  <div className="mt-2">
                    {contract.is_spam && (
                      <div className="mr-2 inline-flex items-center rounded-md bg-red-500 px-2 py-1 text-xs font-medium text-primary-foreground shadow focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50">
                        SPAM
                      </div>
                    )}

                    {contract.is_verified && (
                      <div className="inline-flex items-center rounded-md bg-primary px-2 py-1 text-xs font-medium text-primary-foreground shadow transition-colors hover:bg-primary/90 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50">
                        VERIFIED
                      </div>
                    )}
                  </div>
                </div>
              </div>
            </Link>
          </div>
        ))}
      </div>
    </div>
  );
}
