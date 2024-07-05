import { redirect } from "next/navigation";

import { ThemeProvider } from "~/components/ThemeProvider";
import { Toaster } from "~/components/ui/toaster";
import { getCurrentUser } from "~/lib/session";

export default async function AuthLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const user = await getCurrentUser();

  if (user) {
    redirect("/");
  }

  return (
    <html
      lang="en"
      className="light h-full"
      style={{ colorScheme: "light" }}
      suppressHydrationWarning
    >
      <body className="h-full">
        <ThemeProvider
          attribute="class"
          defaultTheme="light"
          enableSystem
          disableTransitionOnChange
        >
          {children}
          <Toaster />
        </ThemeProvider>
      </body>
    </html>
  );
}
