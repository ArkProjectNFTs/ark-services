import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/avatar";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "~/components/ui/card";
import { Progress } from "~/components/ui/progress";
import type { Contract } from "~/types";

export default function RefreshingCollections(props: {
  contracts: Contract[];
}) {
  return (
    <Card className="col-span-3">
      <CardHeader>
        <CardTitle>Refreshing Collections Metadata</CardTitle>
        <CardDescription></CardDescription>
      </CardHeader>
      <CardContent>
        <div className="space-y-8">
          {props.contracts.map((contract) => {
            return (
              <div
                key={contract.contract_address}
                className="flex items-center"
              >
                <Avatar className="h-9 w-9">
                  <AvatarImage src={contract.contract_image} alt="Avatar" />
                  <AvatarFallback>OM</AvatarFallback>
                </Avatar>
                <div className="ml-4 space-y-1">
                  <p className="text-sm font-medium leading-none">
                    {contract.contract_name}
                  </p>
                  <p className="min-w-[200px] text-sm text-muted-foreground">
                    {contract.contract_address}
                    <Progress value={33} />
                  </p>
                </div>
                <div className="ml-auto font-medium">100%</div>
              </div>
            );
          })}
        </div>
      </CardContent>
    </Card>
  );
}
