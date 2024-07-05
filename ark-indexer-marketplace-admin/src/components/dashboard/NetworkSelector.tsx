"use client";

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "~/components/ui/select";
import { useNetwork, type NetworkType } from "./NetworkProvider";

export default function NetworkSelector() {
  const { network, setNetwork } = useNetwork();
  return (
    <div className="mr-4">
      <Select
        value={network}
        onValueChange={(value) => setNetwork(value as NetworkType)}
      >
        <SelectTrigger className="w-[180px]">
          <SelectValue placeholder="Network" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="production-mainnet">Production Mainnet</SelectItem>
          {/* <SelectItem value="production-sepolia">Production Sepolia</SelectItem>
          <SelectItem value="staging-mainnet">Staging Mainnet</SelectItem>
          <SelectItem value="staging-sepolia">Staging Sepolia</SelectItem> */}
        </SelectContent>
      </Select>
    </div>
  );
}
