#!/usr/bin/env bash
# Simple test for your /download REST endpoint

URL="http://localhost:3090/api/v1/download_docs"

curl -X GET "$URL" \
     -H "Content-Type: application/json" \
     -d '{
           "source_vkntckn": "1950031086",
           "after_this": 25000,
           "download_type": "html",
           "format": "zip"
         }'
echo
