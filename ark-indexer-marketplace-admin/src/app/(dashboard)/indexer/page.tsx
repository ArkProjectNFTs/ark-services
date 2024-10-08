import { Suspense } from "react";
import type { NextPage } from "next";

import BlocksOverview from "~/components/dashboard/BlockOverview";
import BlocksOverviewSkeleton from "~/components/dashboard/BlockOverviewSkeleton";
import CreateIndexerTaskForm from "~/components/dashboard/CreateIndexerTaskForm";
import IndexerBlocksList from "~/components/dashboard/IndexerBlocksList";
import IndexerTasksList from "~/components/dashboard/IndexerTasksList";
import { TaskFormProvider } from "~/components/dashboard/TaskFormProvider";

const IndexerPage: NextPage = () => {
  return (
    <TaskFormProvider>
      <div className="flex space-x-12">
        <div className="flex-1 p-6">
          <Suspense fallback={<BlocksOverviewSkeleton />}>
            <BlocksOverview />
          </Suspense>
          <Suspense fallback={null}>
            <IndexerTasksList />
          </Suspense>
          <Suspense fallback={null}>
            <IndexerBlocksList />
          </Suspense>
        </div>
        <div className="sticky top-0 h-screen border-l p-6">
          <CreateIndexerTaskForm />
        </div>
      </div>
    </TaskFormProvider>
  );
};

export default IndexerPage;
