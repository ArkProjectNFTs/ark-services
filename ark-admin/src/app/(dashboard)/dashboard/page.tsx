import { Suspense } from "react";
import type { NextPage } from "next";

import IndexerTasksList from "~/components/dashboard/IndexerTasksList";

const DashboardPage: NextPage = () => {
  return (
    <div className="hidden h-full flex-col px-6 md:flex">
      <Suspense fallback={null}>
        <IndexerTasksList />
      </Suspense>
    </div>
  );
};

export default DashboardPage;
