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
    redirect("/dashboard");
  }

  return (
    <html
      lang="en"
      className="light"
      style={{ colorScheme: "light" }}
      suppressHydrationWarning
    >
      <ThemeProvider
        attribute="class"
        defaultTheme="light"
        enableSystem
        disableTransitionOnChange
      >
        <body className="h-full">
          {children}
          <Toaster />
        </body>
      </ThemeProvider>
    </html>
  );
}
