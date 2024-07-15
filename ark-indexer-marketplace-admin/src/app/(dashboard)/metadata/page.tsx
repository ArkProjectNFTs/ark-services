import { Suspense } from "react";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "~/components/ui/card";
import { Skeleton } from "~/components/ui/skeleton";
import RefreshingCollections from "./components/RefreshingCollections";

export default function DashboardMetadataPage() {
  return (
    <div className=" p-6">
      <Card className="col-span-3">
        <CardHeader>
          <CardTitle>Refreshing Collections Metadata</CardTitle>
          <CardDescription></CardDescription>
        </CardHeader>
        <CardContent>
          <Suspense
            fallback={
              <div className="flex items-center space-x-4">
                <Skeleton className="h-12 w-12 rounded-full" />
                <div className="space-y-2">
                  <Skeleton className="h-4 w-[250px]" />
                  <Skeleton className="h-4 w-[200px]" />
                </div>
              </div>
            }
          >
            <RefreshingCollections />
          </Suspense>
        </CardContent>
      </Card>

      {/* <MetadataIndicator /> */}
    </div>
  );
}
