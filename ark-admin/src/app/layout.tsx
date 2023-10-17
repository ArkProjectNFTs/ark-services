import "../styles/globals.css";

import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "ArkProject",
  description: "NFTs Made Easy With Shared Liquidity.",
  twitter: {
    title: "ArkProject",
    card: "summary_large_image",
    site: "@ArkProjectNFTs",
    creator: "@ArkProjectNFTs",
    description: "NFTs Made Easy With Shared Liquidity.",
    images: ["https://www.arkproject.dev/medias/ark_project_thumbnail.png"],
  },
  openGraph: {
    title: "ArkProject",
    description: "NFTs Made Easy With Shared Liquidity.",
    url: "https://www.arkproject.dev",
    type: "website",
    images: ["https://www.arkproject.dev/medias/ark_project_thumbnail.png"],
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return children;
}
