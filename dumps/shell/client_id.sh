#!/bin/bash

# 1) handshake
curl \
  --silent \
  -c $COOKIE \
  -d "$(jq -rc ".[0].id = $ID" json/handshake.json)" \
  -H 'Content-Type: application/json' \
  '[::1]:1025/notifications/handshake' |
  jq -r '.[0].clientId'
