import CollectionSearch from "~/components/CollectionSearch";

export default function SearchCollectionsLayout({
  children,
  search,
}: {
  search?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="container mx-auto px-4 py-12 sm:px-6 lg:px-8">
      <div className="mb-8 flex flex-col gap-2">
        <h1 className="text-3xl font-bold">Contracts</h1>
        <p className="text-muted-foreground">
          Explore the contracts that match your search query.
        </p>
        <CollectionSearch search={search} />
      </div>
      {children}
    </div>
  );
}
