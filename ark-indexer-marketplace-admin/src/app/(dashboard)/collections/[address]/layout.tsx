import { TaskFormProvider } from "~/components/dashboard/TaskFormProvider";

interface EditCollectionLayoutProps {
  children: React.ReactNode;
}

export default function EditCollectionLayout({
  children,
}: EditCollectionLayoutProps) {
  return (
    <TaskFormProvider>
      <div className="hidden space-y-6 p-10 pb-16 md:block">{children}</div>
    </TaskFormProvider>
  );
}
