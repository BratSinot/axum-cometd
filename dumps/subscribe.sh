#!/bin/bash

# 1) handshake
clientId=$(
  curl \
    --silent \
    -c cookie.sub \
    -d '[{"id":"10","version":"1.0","minimumVersion":"1.0","channel":"/meta/handshake","supportedConnectionTypes":["long-polling"],"advice":{"timeout":60000,"interval":0}}]' \
    -H 'Content-Type: application/json' \
    '[::1]:1025/notifications/handshake' |
    jq -r '.[0].clientId'
)

# 2) subscribe
curl \
  --silent \
  -b cookie.sub \
  -d "[{\"id\":\"20\",\"channel\":\"/meta/subscribe\",\"subscription\":\"/topic0\",\"clientId\":\"$clientId\"}]" \
  -H 'Content-Type: application/json' \
  '[::1]:1025/notifications'
echo

# 3) connect
while true; do
  curl \
    --silent \
    -b cookie.sub \
    -d "[{\"id\":\"30\",\"channel\":\"/meta/connect\",\"connectionType\":\"long-polling\",\"advice\":{\"timeout\":5000},\"clientId\":\"$clientId\"}]" \
    -H 'Content-Type: application/json' \
    '[::1]:1025/notifications/connect'
echo
done
