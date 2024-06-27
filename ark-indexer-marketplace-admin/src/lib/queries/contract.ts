import type { Contract } from "~/types";
import { pool } from "../postgres";

export async function fetchContract(contractAddress: string, chainId: string) {
  const res = await pool.query<Contract>(
    `SELECT contract_address, chain_id, updated_timestamp, contract_type, contract_name, contract_symbol, contract_image, metadata_ok, is_spam, is_nsfw, deployed_timestamp, is_verified, save_images
     FROM contract
     WHERE contract_address = $1 AND chain_id = $2`,
    [contractAddress, chainId],
  );

  return res.rows.length > 0 ? res.rows[0] : undefined;
}

export async function searchContracts(contractName: string, chainId: string) {
  const res = await pool.query<Contract>(
    `SELECT contract_address, chain_id, updated_timestamp, contract_type, contract_name, contract_symbol, contract_image, metadata_ok, is_spam, is_nsfw, deployed_timestamp, is_verified, save_images
     FROM contract
     WHERE contract_name ILIKE $1 AND chain_id = $2 ORDER BY contract_image ASC LIMIT 50`,
    ["%" + contractName + "%", chainId],
  );

  return res.rows;
}

export async function updateContract(
  contractName: string,
  contractSymbol: string,
  isSpam: boolean,
  isNSFW: boolean,
  isVerified: boolean,
  saveImages: boolean,
  contractAddress: string,
  chainId: string,
  contractImage?: string,
) {
  let query = `UPDATE contract
               SET contract_name = $1, contract_symbol = $2, is_spam = $3, is_nsfw = $4, is_verified = $5, save_images = $6`;
  const values = [
    contractName,
    contractSymbol,
    isSpam,
    isNSFW,
    isVerified,
    saveImages,
    contractAddress,
    chainId,
  ];

  if (contractImage !== undefined) {
    query += `, contract_image = $7`;
    values.splice(6, 0, contractImage);
  }

  query += ` WHERE contract_address = $${values.length - 1} AND chain_id = $${
    values.length
  }`;

  await pool.query(query, values);
}
