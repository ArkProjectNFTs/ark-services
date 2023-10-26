import { Suspense } from "react";
import type { NextPage } from "next";

import BlocksOverview from "~/components/dashboard/BlockOverview";
import CreateIndexerTaskFrom from "~/components/dashboard/CreateIndexerTaskForm";
import IndexerTasksList from "~/components/dashboard/IndexerTasksList";

const DashboardPage: NextPage = () => {
  return (
    <div className="flex space-x-12">
      <div className="flex flex-1 flex-col space-y-6 p-6">
        <Suspense fallback={null}>
          <BlocksOverview />
        </Suspense>
        <Suspense fallback={null}>
          <IndexerTasksList />
        </Suspense>
      </div>
      <div className="sticky top-0 h-screen border-l p-6">
        <CreateIndexerTaskFrom />
      </div>
    </div>
  );
};

export default DashboardPage;
