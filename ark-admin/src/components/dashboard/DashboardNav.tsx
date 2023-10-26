"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

import { cn } from "~/lib/utils";

type ProjectNavProps = React.HTMLAttributes<HTMLElement>;

export default function DashboardNav({ className, ...props }: ProjectNavProps) {
  const pathname = usePathname();

  const items = [
    {
      title: "Dashboard",
      href: `/dashboard`,
    },
    {
      title: "Tasks",
      href: "/tasks",
    },
    {
      title: "Blocks",
      href: "/blocks",
    },
  ];

  return (
    <>
      <nav
        className={cn("flex items-center space-x-4 px-4 py-3", className)}
        {...props}
      >
        {items.map((item) => (
          <Link
            key={item.href}
            href={item.href}
            className={cn(
              pathname === item.href
                ? ""
                : "text-muted-foreground hover:text-muted-foreground",
              "text-sm font-medium transition-colors hover:text-primary",
            )}
          >
            {item.title}
          </Link>
        ))}
      </nav>
    </>
  );
}
