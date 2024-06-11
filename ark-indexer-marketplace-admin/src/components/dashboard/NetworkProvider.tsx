"use client";

import React, { createContext, useContext, type ReactNode } from "react";
import { useLocalStorage } from "usehooks-ts";

export type NetworkType =
  | "production-mainnet"
  | "production-sepolia"
  | "staging-mainnet"
  | "staging-sepolia";

interface NetworkContextType {
  network: NetworkType;
  setNetwork: React.Dispatch<React.SetStateAction<NetworkType>>;
}

const NetworkContext = createContext<NetworkContextType | undefined>(undefined);

interface NetworkProviderProps {
  children: ReactNode;
}

export const NetworkProvider: React.FC<NetworkProviderProps> = ({
  children,
}) => {
  const [network, setNetwork] = useLocalStorage<NetworkType>(
    "ark-admin:network",
    "production-mainnet",
  );

  return (
    <NetworkContext.Provider value={{ network, setNetwork }}>
      {children}
    </NetworkContext.Provider>
  );
};

export const useNetwork = (): NetworkContextType => {
  const context = useContext(NetworkContext);

  if (!context) {
    throw new Error("useNetwork must be used within a NetworkProvider");
  }
  return context;
};
