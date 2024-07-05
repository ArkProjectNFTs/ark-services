"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";

import { Input } from "./ui/input";

export default function CollectionSearch() {
  const [text, setText] = useState<string>("");
  const router = useRouter();

  return (
    <div className="mr-2">
      <Input
        onChange={(event) => {
          setText(event.target.value);
        }}
        onKeyDown={(event) => {
          if (event.key === "Enter") {
            void router.push("/collections/search/" + text);
          }
        }}
        type="search"
        placeholder="Search collection..."
        className="md:w-[100px] lg:w-[300px]"
        value={text}
      />
    </div>
  );
}
