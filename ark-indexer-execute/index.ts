import { Client } from "https://deno.land/x/postgres@v0.17.0/mod.ts";

const CONTRACT_ADDRESS = Deno.env.get("CONTRACT_ADDRESS");
const STREAM_URL = Deno.env.get("STREAM_URL");
const STARTING_BLOCK = Deno.env.get("STARTING_BLOCK");
const AUTH_TOKEN = Deno.env.get("AUTH_TOKEN");
const SELECTOR = Deno.env.get("SELECTOR");
const DATABASE_URL = Deno.env.get("DATABASE_URL");
const client = new Client(DATABASE_URL);

export const config = {
  streamUrl: STREAM_URL,
  startingBlock: STARTING_BLOCK ? parseInt(STARTING_BLOCK) : undefined,
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

async function updateDatabase(orderhash: string) {
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

export default async function transform({
  header,
  events,
}: {
  header: { blockNumber: number };
  events: Array<{ event: { keys: string[] } }>;
}) {
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
