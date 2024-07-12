#!/bin/bash

folders=("ark-marketplace-api" "ark-marketplace-cron" "ark-orderbook-api")

rm -rf .sqlx/*

for folder in "${folders[@]}"
do
  cd $folder

  cargo sqlx prepare

  cd ..

  cp -R $folder/.sqlx/* .sqlx/
done

cd .sqlx
git add .
git commit -am "feat(sqlx): add sqlx files"
echo "Files committed"
