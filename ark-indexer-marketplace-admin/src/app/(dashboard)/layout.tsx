import { headers } from "next/headers";

import CollectionSearch from "~/components/CollectionSearch";
import Header from "~/components/dashboard/Header";
import { NetworkProvider } from "~/components/dashboard/NetworkProvider";
import { ThemeProvider } from "~/components/ThemeProvider";
import { Toaster } from "~/components/ui/toaster";
import { getCurrentUser } from "~/lib/session";
import { TRPCReactProvider } from "~/trpc/react";

export default async function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const user = await getCurrentUser();

  return (
    <html
      lang="en"
      className="light"
      style={{ colorScheme: "light" }}
      suppressHydrationWarning
    >
      <body>
        <ThemeProvider
          attribute="class"
          defaultTheme="light"
          enableSystem
          disableTransitionOnChange
        >
          <TRPCReactProvider headers={headers()}>
            <NetworkProvider>
              <div className="flex-col">
                <Header user={user} />
                {children}
              </div>
            </NetworkProvider>
          </TRPCReactProvider>
          <Toaster />
        </ThemeProvider>
      </body>
    </html>
  );
}
