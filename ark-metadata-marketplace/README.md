# Ark Metadata Marketplace

## Introduction

This service is designed to manage and process metadata related to NFTs (Non-Fungible Tokens) on the blockchain. Before running the program, it is important to set up an Elasticsearch index that will store the NFT metadata.

## Prerequisites

- **Elasticsearch 8.x or higher**
- **Kibana** (optional, for visualizing the data)
- **Docker** (if running Elasticsearch in a container)
- **Rust** (for compiling and running the project)

## Setup

### 1. Create the `nft-metadata` Index in Elasticsearch

Before starting the program, you need to create an index in Elasticsearch where the NFT metadata will be stored. This can be done using a simple `PUT` request to Elasticsearch.

You can use the following command with `curl` to create the index:

```bash
curl -X PUT "https://localhost:9200/nft-metadata" -H 'Content-Type: application/json' -u elastic:your_password_here -k -d'
{
  "mappings": {
    "properties": {
      "chain_id": { "type": "text" },
      "contract_address": { "type": "text" },
      "metadata": {
        "properties": {
          "animation_key": { "type": "text" },
          "animation_mime_type": { "type": "text" },
          "animation_url": { "type": "text" },
          "attributes": {
            "type": "nested",
            "properties": {
              "trait_type": { "type": "keyword" },
              "value": { "type": "keyword" }
            }
          },
          "background_color": { "type": "text" },
          "description": { "type": "text" },
          "external_url": { "type": "text" },
          "image": { "type": "text" },
          "image_key": { "type": "text" },
          "image_mime_type": { "type": "text" },
          "name": { "type": "text" },
          "properties": { "type": "text" },
          "youtube_url": { "type": "text" }
        }
      },
      "metadata_updated_at": { "type": "long" },
      "raw_metadata": { "type": "text" },
      "token_id": { "type": "text" }
    }
  }
}
'
```
### 2. Configure the Environment

Make sure to set the appropriate environment variables for your Elasticsearch instance and other required services. For example:

```bash
export ELASTICSEARCH_URL=https://localhost:9200
export ELASTICSEARCH_USERNAME=elastic
export ELASTICSEARCH_PASSWORD=your_password_here
```

### 3. Build and Run the Project
```bash
cargo build --release
./target/release/ark-metadata-marketplace
```

### 4. Testing the Setup
After running the program, you can verify that the NFT metadata is being indexed in Elasticsearch by checking the contents of the nft-metadata index.

You can use Kibana or the following curl command to inspect the data:
```bash
curl -X GET "https://localhost:9200/nft-metadata/_search" -u elastic:your_password_here -k
```

and should see something like:
```json
{"took":1,"timed_out":false,"_shards":{"total":1,"successful":1,"skipped":0,"failed":0},"hits":{"total":{"value":0,"relation":"eq"},"max_score":null,"hits":[]}}
```
