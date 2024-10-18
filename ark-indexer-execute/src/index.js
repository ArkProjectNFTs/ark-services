// import { config as dotenvConfig } from "https://deno.land/x/dotenv/mod.ts";
import { Client } from "https://deno.land/x/postgres@v0.17.0/mod.ts";

const CONTRACT_ADDRESS = "0x007b42945bc47001db92fe1b9739d753925263f2f1036c2ae1f87536c916ee6a";
const STREAM_URL = "https://mainnet-v2.starknet.a5a.ch";
const STARTING_BLOCK = 786737;
const AUTH_TOKEN = "dna_wfWzg25bnZPKK25fAZMj";
const SELECTOR = "0xf10f06595d3d707241f604672ec4b6ae50eb82728ec2f3c65f6789e897760";
const DATABASE_URL = "postgresql://admin:dbpassword@localhost:5432/arkproject"

console.log("Connecting to database...");
const client = new Client(DATABASE_URL);

export const config = {
  streamUrl: STREAM_URL,
  startingBlock: STARTING_BLOCK,
  network: "starknet",
  authToken: AUTH_TOKEN,
  finality: "DATA_STATUS_ACCEPTED",
  filter: {
    header: {
      weak: true,
    },
    events: [
      {
        fromAddress: CONTRACT_ADDRESS,
        keys: [SELECTOR],
        includeReceipt: false,
      },
    ],
  },
  sinkType: "console",
  sinkOptions: {},
};

async function updateDatabase(orderhash) {
  console.log(`Updating database for orderhash: ${orderhash}`);
  try {
    const updateQuery = `
      UPDATE token
      SET 
        listing_start_amount = NULL,
        listing_start_date = NULL,
        listing_currency_address = NULL,
        listing_currency_chain_id = NULL,
        listing_timestamp = NULL,
        listing_broker_id = NULL,
        listing_orderhash = NULL,
        listing_end_amount = NULL,
        listing_end_date = NULL,
        top_bid_amount = NULL,
        top_bid_broker_id = NULL,
        top_bid_order_hash = NULL,
        has_bid = false,
        status = 'EXECUTED',
        buy_in_progress = false
      WHERE listing_orderhash = $1 OR top_bid_order_hash = $1
    `;

    const result = await client.queryArray(updateQuery, [orderhash]);
    console.log(`Updated ${result.rowCount} rows for orderhash: ${orderhash}`);
  } catch (error) {
    console.error(`Error updating database for orderhash ${orderhash}:`, error);
  }
}

export default async function transform({ header, events }) {
  console.log(`Processing block: ${header.blockNumber}`);

  if (!client.connected) {
    await client.connect();
    console.log("Connected to the database");
  }

  for (const { event } of events) {
    const orderhash = event.keys[1];
    console.log(`Processing orderhash: ${orderhash}`);
    await updateDatabase(orderhash);
  }

  console.log(`Finished processing block: ${header.blockNumber}`);
  return events.map(({ event }) => ({
    orderhash: event.keys[1],
  }));
}

// Ensure the database connection is closed when the script exits
globalThis.addEventListener("unload", () => {
  client.end();
  console.log("Database connection closed");
});