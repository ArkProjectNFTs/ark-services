import Link from "next/link";
import { Link as LinkIcon } from "lucide-react";

import { TaskFormProvider } from "~/components/dashboard/TaskFormProvider";
import { Separator } from "~/components/ui/separator";

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
