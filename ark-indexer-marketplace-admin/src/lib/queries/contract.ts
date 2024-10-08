import type { Contract, RefreshingContract } from "~/types";
import { pool } from "../postgres";

export async function updateIsRefreshingContract(
  contractAddress: string,
  chainId: string,
  isRefreshing: boolean,
) {
  const query = `UPDATE contract SET is_refreshing = $1, updated_timestamp=EXTRACT(epoch FROM now())::bigint 
                 WHERE contract_address=$2 AND chain_id=$3`;

  await pool.query(query, [isRefreshing, contractAddress, chainId]);
}

export async function fetchRefreshingContracts(chainId: string) {
  const query = `
  SELECT
    c.contract_address,
    c.chain_id,
    c.updated_timestamp,
    c.contract_type,
    c.contract_name,
    c.contract_symbol,
    c.contract_image,
    CAST(c.token_count AS INTEGER),
    (
        SELECT CAST(COUNT(*) AS INTEGER)
        FROM token as t 
        WHERE t.contract_address = c.contract_address 
        AND t.chain_id = c.chain_id 
        AND t.metadata_status = 'OK'
    ) AS refreshed_token_count
    FROM contract c 
    WHERE
        c.is_spam = false
        AND c.is_refreshing = true
        AND c.chain_id = $1
        AND c.contract_type = 'ERC721'
    LIMIT 20;`;
  const res = await pool.query<RefreshingContract>(query, [chainId]);

  return res.rows;
}

export async function fetchContract(contractAddress: string, chainId: string) {
  const res = await pool.query<Contract>(
    `SELECT contract_address, chain_id, updated_timestamp, contract_type, contract_name, contract_symbol, contract_image, metadata_ok, is_spam, is_nsfw, deployed_timestamp, is_verified, save_images, is_refreshing
     FROM contract
     WHERE contract_address = $1 AND chain_id = $2`,
    [contractAddress, chainId],
  );

  return res.rows.length > 0 ? res.rows[0] : undefined;
}

export async function getDefaultContracts(chainId: string) {
  const res = await pool.query<Contract>(
    `SELECT contract_address, chain_id, updated_timestamp, contract_type, contract_name, contract_symbol, contract_image, metadata_ok, is_spam, is_nsfw, deployed_timestamp, is_verified, save_images, is_refreshing
     FROM contract
     WHERE contract_type = 'ERC721' AND is_spam = false AND chain_id = $1 AND contract_name != '' ORDER BY contract_name ASC LIMIT 50`,
    [chainId],
  );

  return res.rows;
}

export async function searchContracts(contractName: string, chainId: string) {
  const res = await pool.query<Contract>(
    `SELECT contract_address, chain_id, updated_timestamp, contract_type, contract_name, contract_symbol, contract_image, metadata_ok, is_spam, is_nsfw, deployed_timestamp, is_verified, save_images, is_refreshing
     FROM contract
     WHERE contract_type = 'ERC721' AND contract_name ILIKE $1 AND chain_id = $2 ORDER BY contract_image ASC, contract_name ASC LIMIT 50`,
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
