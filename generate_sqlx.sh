#!/bin/bash

# Dont start the script if DBS are not set 
if [[ -n "${MARKETPLACE_DATABASE_URL}"  || -n "${ORDERBOOK_DATABASE_URL}" ]]; then
  marketplace_db=$MARKETPLACE_DATABASE_URL
  orderbook_db=$ORDERBOOK_DATABASE_URL
  folders=("ark-marketplace-api" "ark-marketplace-cron" "ark-orderbook-api" "ark-indexer-transactions")
  rm -rf .sqlx/*
  for folder in "${folders[@]}"
  do
    default_db=$marketplace_db
    if [ "$folder" = "ark-orderbook-api" ]; then 
      default_db=$orderbook_db
      echo "run on orderbook DB" 
    else
        echo "run on marketplace DB" 
    fi;
    cd $folder

    cargo sqlx prepare --database-url $default_db

    cd ..

    cp -R $folder/.sqlx/* .sqlx/
  done

  cd .sqlx
  # git add .
  # git commit -am "feat(sqlx): add sqlx files"
  echo "check files before commit"
else
  echo "You should provide MARKETPLACE_DATABASE_URL && ORDERBOOK_DATABASE_URL environement variable at least"
fi






