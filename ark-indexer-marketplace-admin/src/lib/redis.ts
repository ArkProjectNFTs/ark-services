import { createClient } from "redis";

import { env } from "~/env.mjs";

const client = createClient({
  url: env.REDIS_URL,
});

client.on("error", (err) => {
  console.error("Error connecting to Redis", err);
});

export async function clearListedTokensCache(contractAddress: string) {
  try {
    await client.connect();

    const pattern = `*_${contractAddress}_*`;
    const keys = [];
    for await (const key of client.scanIterator({ MATCH: pattern })) {
      keys.push(key);
    }

    if (keys.length > 0) {
      await client.del(keys);
      for (const key of keys) {
        console.log(`Deleted key: ${key}`);
      }
    } else {
      console.log("No keys found matching the pattern.");
    }
  } catch (err) {
    console.error("Error flushing cache:", err);
  } finally {
    await client.quit();
    console.log("Redis client disconnected.");
  }
}
