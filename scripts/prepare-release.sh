#!/bin/bash
VERSION=$(jq -r .version package.json)
echo "Preparing release $VERSION"
npm version $VERSION --allow-same-version --no-git-tag-version --workspaces
cargo workspaces version -a --force '*' --no-git-commit -y custom $VERSION