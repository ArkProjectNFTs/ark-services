"use client";

import Link from "next/link";

import { api } from "~/trpc/react";

export function MetadataIndicator() {
  const { data } = api.indexer.metadata.useQuery(undefined, {
    refetchInterval: 5000,
  });

  console.log("data", data);

  return (
    <div className="mb-6">
      <div>
        <h3 className="flex items-center space-x-2 text-2xl font-semibold tracking-tight">
          <span>Metadata</span>
          <div>{data?.totalCount ?? 0} tokens</div>
        </h3>

        {data && (
          <div>
            {Object.keys(data.contracts).map((contractAddress, idx) => {
              const contract = data.contracts[contractAddress];

              return (
                <div className="mt-4" key={`metadata-${idx}`}>
                  <Link
                    className="cursor-pointer"
                    target="_blank"
                    href={`https://explorer.arkproject.dev/mainnet/collections/${contractAddress}`}
                  >
                    <div className="font-bold">{contract.name}</div>
                  </Link>
                  <div>{contractAddress}</div>
                  <div>{contract.items.length} tokens</div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
