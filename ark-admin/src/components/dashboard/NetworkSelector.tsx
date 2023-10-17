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
          <SelectItem value="mainnet">Mainnet</SelectItem>
          <SelectItem value="testnet">Testnet</SelectItem>
        </SelectContent>
      </Select>
    </div>
  );
}
