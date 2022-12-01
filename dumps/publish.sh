#!/bin/bash

# 1) handshake
clientId=$(
  curl \
    --silent \
    -c cookie.pub \
    -d '[{"id":"100","version":"1.0","minimumVersion":"1.0","channel":"/meta/handshake","supportedConnectionTypes":["long-polling"],"advice":{"timeout":60000,"interval":0}}]' \
    -H 'Content-Type: application/json' \
    '[::1]:1025/notifications/handshake' |
    jq -r '.[0].clientId'
)

# 2) publish
while true; do
  curl \
    --silent \
    -b cookie.pub \
    -d "[{\"id\":\"200\",\"channel\":\"/topic0\",\"data\":{\"msg\":\"Hello\"},\"clientId\":\"$clientId\"}]" \
    -H 'Content-Type: application/json' \
    '[::1]:1025/notifications/connect'
  sleep 1
  echo
done
