"use client";

import Link from "next/link";
import { type User } from "next-auth";
import { signOut } from "next-auth/react";

import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/avatar";
import { Button } from "~/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "~/components/ui/dropdown-menu";
import { Skeleton } from "~/components/ui/skeleton";
import CollectionSearch from "../CollectionSearch";
import Logo from "../icons/Logo";
import DashboardNav from "./DashboardNav";
import NetworkSelector from "./NetworkSelector";
import ThemeSelect from "./ThemeSelect";

interface DashboardHeaderProps extends React.HTMLAttributes<HTMLDivElement> {
  user: (User & { id: string }) | undefined;
}

export default function DashboardHeader({ user }: DashboardHeaderProps) {
  return (
    <header className="border-b">
      <div className="flex h-16 items-center px-4">
        <Logo className="h-8 w-auto" />
        <DashboardNav />
        <div className="flex-grow" />
        <NetworkSelector />
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" className="relative h-8 w-8 rounded-full">
              <Avatar className="h-8 w-8">
                <AvatarImage
                  src={
                    user?.image
                      ? user.image
                      : `https://avatar.vercel.sh/${user?.email}`
                  }
                  alt=""
                />
                <AvatarFallback>
                  <Skeleton className="h-full w-full rounded-full" />
                </AvatarFallback>
              </Avatar>
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent className="w-56" align="end" forceMount>
            <DropdownMenuLabel className="font-normal">
              <div className="flex flex-col space-y-1">
                <p className="text-sm font-medium leading-none">{user?.name}</p>
                <p className="text-xs leading-none text-muted-foreground">
                  {user?.email}
                </p>
              </div>
            </DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuGroup>
              <div className="flex items-center justify-between px-2 py-[6px]">
                <span className="text-sm">Theme</span>
                <ThemeSelect />
              </div>
            </DropdownMenuGroup>
            <DropdownMenuSeparator />
            <DropdownMenuItem
              className="cursor-pointer"
              onClick={(event) => {
                event.preventDefault();
                void signOut({
                  callbackUrl: `${window.location.origin}/login`,
                });
              }}
            >
              Log out
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </header>
  );
}
