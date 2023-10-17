"use client";

import Link from "next/link";

import { useNetwork } from "./NetworkProvider";

export default function HeaderNav() {
  const { network } = useNetwork();

  return (
    <nav className="mx-6 flex items-center space-x-4 lg:space-x-6">
      <Link
        className="hover:text-primary text-sm font-medium transition-colors"
        href="/dashboard"
      >
        Dashboard
      </Link>
      <Link
        target="_blank"
        className="text-muted-foreground hover:text-primary text-sm font-medium transition-colors"
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
