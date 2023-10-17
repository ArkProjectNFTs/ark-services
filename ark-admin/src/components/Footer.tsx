"use client";

import Image from "next/image";
import Link from "next/link";

import { Typography } from "./Typography";

export default function Footer() {
  return (
    <footer className="mt-5">
      <div className="mx-auto flex w-full max-w-7xl flex-col items-center justify-between gap-6 px-6 py-9 md:flex-row md:px-16 2xl:max-w-[100rem]">
        <Image
          className="block h-5 w-auto dark:hidden md:h-[30px]"
          src="/logos/arkProject.svg"
          height={30}
          width={230}
          alt="Ark Project logo"
        />
        <Image
          className="hidden h-5 w-auto dark:block md:h-[30px]"
          src="/logos/dark/arkProject.svg"
          height={30}
          width={230}
          alt="Ark Project logo"
        />
        <div>
          <Link
            className="text-sm leading-6 text-gray-300 hover:text-white"
            href="/privacy"
          >
            Privacy policy
          </Link>
        </div>
        <div>
          <Typography variant="body_text_16">© 2023 Screenshot Labs</Typography>
        </div>
      </div>
    </footer>
  );
}
