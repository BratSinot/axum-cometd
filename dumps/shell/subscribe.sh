#!/bin/bash

ID=10

# 1) handshake
clientId=$(ID=$ID COOKIE=cookie.sub ./client_id.sh)

ID=$((ID + 10))

# 2) subscribe
curl \
  --silent \
  -b cookie.sub \
  -d "$(jq -rc --arg ID "$ID" --arg clientId "$clientId" '.[0].id = $ID | .[].clientId = $clientId' json/subscribe.json)" \
  -H 'Content-Type: application/json' \
  '[::1]:1025/notifications'
echo

ID=$((ID + 10))

# 3) connect
while true; do
  curl \
    --silent \
    -b cookie.sub \
    -d "$(jq -rc --arg ID "$ID" --arg clientId "$clientId" '.[0].id = $ID | .[].clientId = $clientId' json/connect.json)" \
    -H 'Content-Type: application/json' \
    '[::1]:1025/notifications/connect'
  ID=$((ID + 10))
  echo
done
