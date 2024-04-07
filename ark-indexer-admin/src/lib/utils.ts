import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

import type { Network } from "~/types";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function capitalize(string: string) {
  return string.charAt(0).toUpperCase() + string.slice(1);
}

export function getTableName(network: Network) {
  switch (network) {
    case "production-mainnet":
      return `${process.env.TABLE_NAME_PREFIX}mainnet`;
    case "production-sepolia":
      return `${process.env.TABLE_NAME_PREFIX}sepolia`;
    case "staging-mainnet":
      return `${process.env.TABLE_NAME_PREFIX}staging_mainnet`;
    case "staging-sepolia":
      return `${process.env.TABLE_NAME_PREFIX}staging_sepolia`;
  }
}
