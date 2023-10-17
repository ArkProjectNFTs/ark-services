import Link from "next/link";

import Logo from "~/components/icons/Logo";

export default function LoginVerifyPage() {
  return (
    <>
      <div className="flex h-full items-center justify-center text-center">
        <div className="mx-auto flex w-full flex-col justify-center space-y-6 sm:w-[350px]">
          <Logo className="mx-auto h-16 w-auto" />
          <div className="text-2xl font-semibold tracking-tight">
            Check your email
          </div>
          <div className="">
            We&apos;ve sent you a temporary login link,
            <br />
            please check your inbox.
          </div>
          <Link
            href="/login"
            className="focus-visible:ring-ring bg-primary text-primary-foreground hover:bg-primary/90 inline-flex h-9 items-center justify-center rounded-md px-4 py-2 text-sm font-medium shadow transition-colors focus-visible:outline-none focus-visible:ring-1 disabled:pointer-events-none disabled:opacity-50"
          >
            Back to login
          </Link>
        </div>
      </div>
    </>
  );
}
