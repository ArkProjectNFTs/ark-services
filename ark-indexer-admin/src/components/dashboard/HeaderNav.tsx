"use client";

import Link from "next/link";

import { useNetwork } from "./NetworkProvider";

export default function HeaderNav() {
  const { network } = useNetwork();

  return (
    <nav className="mx-6 flex items-center space-x-4 lg:space-x-6">
      <Link
        className="text-sm font-medium transition-colors hover:text-primary"
        href="/dashboard"
      >
        Dashboard
      </Link>
      <Link
        className="text-sm font-medium transition-colors hover:text-primary"
        href="/metadata"
      >
        Metadata
      </Link>
      <Link
        target="_blank"
        className="text-sm font-medium text-muted-foreground transition-colors hover:text-primary"
        href={
          network === "mainnet"
            ? "https://g-508abb969b.grafana-workspace.us-east-1.amazonaws.com/d/qOmka-94z/ark-project-dashboard"
            : "https://g-508abb969b.grafana-workspace.us-east-1.amazonaws.com/d/w85dxPzIz/ark-project-testnet"
        }
      >
        Monitoring
      </Link>
    </nav>
  );
}
