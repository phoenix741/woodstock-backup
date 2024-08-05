#!/bin/bash
VERSION=$(jq -r .version package.json)
cargo workspaces version --no-git-commit -y custom $VERSION