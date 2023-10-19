"use client";

import React, {
  createContext,
  useContext,
  useState,
  type ReactNode,
} from "react";

export type NetworkType = "mainnet" | "testnet";

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
  const [network, setNetwork] = useState<NetworkType>("mainnet");

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
