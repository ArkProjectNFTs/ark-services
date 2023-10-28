import { Suspense } from "react";
import type { NextPage } from "next";

import BlocksOverview from "~/components/dashboard/BlockOverview";
import BlocksOverviewSkeleton from "~/components/dashboard/BlockOverviewSkeleton";
import CreateIndexerTaskForm from "~/components/dashboard/CreateIndexerTaskForm";
import IndexerTasksList from "~/components/dashboard/IndexerTasksList";
import { TaskFormProvider } from "~/components/dashboard/TaskFormProvider";

const DashboardPage: NextPage = () => {
  return (
    <TaskFormProvider>
      <div className="flex space-x-12">
        <div className="flex flex-1 flex-col space-y-6 p-6">
          <Suspense fallback={<BlocksOverviewSkeleton />}>
            <BlocksOverview />
          </Suspense>
          <Suspense fallback={null}>
            <IndexerTasksList />
          </Suspense>
        </div>
        <div className="sticky top-0 h-screen border-l p-6">
          <CreateIndexerTaskForm />
        </div>
      </div>
    </TaskFormProvider>
  );
};

export default DashboardPage;
