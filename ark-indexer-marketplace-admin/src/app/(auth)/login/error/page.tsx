import Link from "next/link";

export default function LoginErrorPage() {
  return (
    <div className="flex h-full items-center justify-center">
      <div className="mx-auto flex w-full flex-col justify-center space-y-6 sm:w-[350px]">
        <div className="text-2xl font-semibold tracking-tight">
          An error occured
        </div>
        <div className="">
          Please try to login again with another
          <br />
          provider or with email.
        </div>
        <Link
          href="/login"
          className="focus-visible:ring-ring bg-primary text-primary-foreground hover:bg-primary/90 inline-flex h-9 items-center justify-center rounded-md px-4 py-2 text-sm font-medium shadow transition-colors focus-visible:outline-none focus-visible:ring-1 disabled:pointer-events-none disabled:opacity-50"
        >
          Go back to login
        </Link>
      </div>
    </div>
  );
}
