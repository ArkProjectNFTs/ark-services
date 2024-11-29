#!/bin/bash

# Dont start the script if DBS are not set 
if [[ -n "${MARKETPLACE_DATABASE_URL}" ]]; then
  marketplace_db=$MARKETPLACE_DATABASE_URL
  folders=("ark-marketplace-api" "ark-marketplace-cron" "ark-indexer-transactions")
  rm -rf .sqlx/*
  for folder in "${folders[@]}"
  do
    default_db=$marketplace_db
    echo "run on marketplace DB" 
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
  echo "You should provide MARKETPLACE_DATABASE_URL environement variable at least"
fi






