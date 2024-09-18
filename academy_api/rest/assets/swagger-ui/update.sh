#!/usr/bin/env bash

set -ex

url=$(curl https://api.github.com/repos/swagger-api/swagger-ui/releases/latest | jq -r .tarball_url)
curl -L "$url" | tar xvz --wildcards --no-wildcards-match-slash '*/dist'
mv swagger-api-swagger-ui-*/dist/{swagger-ui-bundle.js,swagger-ui.css} .
rm -rf swagger-api-swagger-ui-*
