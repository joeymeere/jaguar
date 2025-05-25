#!/usr/bin/env bash

set -ex

workspace_crates=(
    jaguar
    jaguar-derive
)

for crate in "${workspace_crates[@]}"; do
   echo "--- $crate"
   cargo package -p $crate
   cargo publish -p $crate
done