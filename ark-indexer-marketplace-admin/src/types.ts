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

export type Contract = {
  contract_address: string;
  chain_id: string;
  updated_timestamp: number;
  contract_type: string;
  contract_name?: string;
  contract_symbol?: string;
  contract_image?: string;
  metadata_ok: boolean;
  is_spam: boolean;
  is_nsfw: boolean;
  deployed_timestamp?: number;
  is_verified: boolean;
  save_images: boolean;
};

export type RefreshingContract = {
  contract_address: string;
  chain_id: string;
  updated_timestamp: number;
  contract_type: string;
  contract_name?: string;
  contract_symbol?: string;
  contract_image?: string;
  token_count: number;
};
