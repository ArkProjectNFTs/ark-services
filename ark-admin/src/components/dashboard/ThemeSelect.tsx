"use client";

import type { ReactElement } from "react";
import { ComputerDesktopIcon, SunIcon } from "@heroicons/react/24/outline";
import { MoonIcon } from "lucide-react";
import { useTheme } from "next-themes";

import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "~/components/ui/select";
import { capitalize } from "~/lib/utils";

const icons: Record<string, ReactElement> = {
  light: <SunIcon width={12} height={12} />,
  dark: <MoonIcon width={12} height={12} />,
  system: <ComputerDesktopIcon width={12} height={12} />,
};

export default function ThemeSelect() {
  const { setTheme, theme } = useTheme();

  const handleChange = (value: string) => {
    setTheme(value);
  };

  if (!theme) {
    return null;
  }

  return (
    <Select onValueChange={handleChange} value={theme}>
      <SelectTrigger className="h-6 w-24 px-2 py-1">
        <SelectValue>
          <div className="flex items-center space-x-1 text-xs">
            {icons[theme]}
            <span>{capitalize(theme)}</span>
          </div>
        </SelectValue>
      </SelectTrigger>
      <SelectContent className="">
        <SelectGroup>
          <SelectItem className="text-xs" value="light">
            Light
          </SelectItem>
          <SelectItem className="text-xs" value="dark">
            Dark
          </SelectItem>
          <SelectItem className="text-xs" value="system">
            System
          </SelectItem>
        </SelectGroup>
      </SelectContent>
    </Select>
  );
}
