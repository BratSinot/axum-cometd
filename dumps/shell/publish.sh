#!/bin/bash

ID=10

# 1) handshake
clientId=$(ID=$ID COOKIE=cookie.pub ./client_id.sh)

ID=$((ID + 10))

# 2) publish
while true; do
  curl \
    --silent \
    -b cookie.pub \
    -d "$(jq -rc --arg ID0 "$ID" --arg ID1 "$((ID + 10))" --arg clientId "$clientId" '.[0].id = $ID0 | .[1].id = $ID1 | .[].clientId = $clientId' json/publish.json)" \
    -H 'Content-Type: application/json' \
    '[::1]:1025/notifications/connect'
  ID=$((ID + 20))
  sleep 1
  echo
done
