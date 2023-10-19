import Link from "next/link";

import { LoginClientForm } from "~/components/auth/LoginClientForm";
import Logo from "~/components/icons/Logo";

export default function LoginPage() {
  return (
    <div className="flex h-full items-center justify-center">
      <div className="mx-auto flex w-full flex-col justify-center space-y-6 sm:w-[350px]">
        <div className="flex justify-center">
          <Logo />
        </div>
        <div className="flex flex-col space-y-2 text-center">
          <h1 className="text-2xl font-semibold tracking-tight">
            Sign in to ArkProject
          </h1>
          <p className="text-sm text-muted-foreground">
            Enter your email below to sign in to your account
          </p>
        </div>
        <div className="grid gap-6">
          <LoginClientForm />
        </div>
        <p className="px-8 text-center text-sm text-muted-foreground">
          By clicking continue, you agree to our{" "}
          <Link
            href="/terms"
            className="underline underline-offset-4 hover:text-primary"
          >
            Terms of Service
          </Link>{" "}
          and{" "}
          <Link
            href="/privacy"
            className="underline underline-offset-4 hover:text-primary"
          >
            Privacy Policy
          </Link>
          .
        </p>
      </div>
    </div>
  );
}
