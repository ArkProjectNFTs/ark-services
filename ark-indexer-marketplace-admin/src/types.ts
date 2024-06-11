export type Network =
  | "production-mainnet"
  | "production-sepolia"
  | "staging-mainnet"
  | "staging-sepolia";

export type Block = {
  block_number: number;
  block_timestamp: number;
  block_status: string;
};
