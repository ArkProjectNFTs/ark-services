import "../styles/globals.css";

import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "ArkProject Admin",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return children;
}
