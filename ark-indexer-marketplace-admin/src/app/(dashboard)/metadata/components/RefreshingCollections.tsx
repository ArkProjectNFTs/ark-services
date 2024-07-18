"use client";

import Link from "next/link";
import { LinkIcon } from "lucide-react";

import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/avatar";
import { Button } from "~/components/ui/button";
import { Progress } from "~/components/ui/progress";
import { api } from "~/trpc/react";

export default function RefreshingCollections() {
  const [contracts] = api.contract.getRefreshingContracts.useSuspenseQuery(
    {},
    {
      refetchInterval: 5000,
    },
  );

  return (
    <div className="space-y-8">
      {contracts.length === 0 ? (
        <p className="text-muted-foreground">
          No collections are currently refreshing.
        </p>
      ) : (
        <>
          {contracts.map((contract) => {
            return (
              <div
                key={contract.contract_address}
                className="flex items-center"
              >
                <Avatar className="h-9 w-9">
                  <AvatarImage src={contract.contract_image} alt="Avatar" />
                  <AvatarFallback>
                    {contract.contract_symbol?.substring(0, 2)}
                  </AvatarFallback>
                </Avatar>
                <div className="ml-4 space-y-1">
                  <Link
                    href={`/collections/${contract.contract_address}`}
                    className="text-sm font-medium leading-none"
                  >
                    {contract.contract_name}
                  </Link>
                  <p className="min-w-[200px] text-sm text-muted-foreground">
                    {contract.contract_address}
                  </p>
                  <div className="flex items-center gap-4">
                    <div className="w-[200px]">
                      <Progress value={contract.progression} />
                    </div>
                    <div className="text-xs">
                      {contract.refreshed_token_count}/{contract.token_count}{" "}
                      token
                      {contract.token_count > 1 && "s"}
                    </div>
                  </div>
                </div>
                <div className="ml-auto flex items-center justify-center gap-4">
                  <div className=" font-medium">{contract.progression}%</div>
                  <Link
                    target="_blank"
                    href={`https://market.arkproject.dev/collection/${contract.contract_address}`}
                  >
                    <Button>View on ArkProject</Button>
                  </Link>
                </div>
              </div>
            );
          })}
        </>
      )}
    </div>
  );
}
