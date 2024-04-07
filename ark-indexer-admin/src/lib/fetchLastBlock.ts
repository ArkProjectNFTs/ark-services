import axios, { type AxiosResponse } from "axios";

type NetworkType =
  | "production-mainnet"
  | "production-sepolia"
  | "staging-mainnet"
  | "staging-sepolia";

interface BlockNumberResponse {
  jsonrpc: string;
  id: number;
  result: string;
}

export async function fetchLastBlock(network: NetworkType): Promise<number> {
  const payload = {
    jsonrpc: "2.0",
    id: 1,
    method: "starknet_blockNumber",
    params: [],
  };
  const url = network.includes("mainnet")
    ? `https://juno.mainnet.arkproject.dev`
    : `https://sepolia.arkproject.dev`;
  const response: AxiosResponse<BlockNumberResponse> = await axios.post(
    url,
    payload,
  );

  return parseInt(response.data.result);
}
