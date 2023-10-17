import { Suspense } from "react";
import type { NextPage } from "next";

import IndexerTasksList from "~/components/dashboard/IndexerTasksList";

const HomePage: NextPage = () => {
  return (
    <div className="hidden h-full flex-col px-6 md:flex">
      <Suspense fallback={<div>Loading...</div>}>
        <IndexerTasksList />
      </Suspense>
    </div>
  );
};

export default HomePage;
