#!/bin/bash

# Deployment script for Guts contract

compiler_version="2.6.2"
network="sepolia"

target_dir="./target/dev"
contract_class_file="$target_dir/guts_Guts.contract_class.json"

# Declare the contract and capture the command output
#command_output=$(starkli declare $contract_class_file --compiler-version=$compiler_version)

from_string="Class hash declared:"
#class_hash="${command_output#*$from_string}"
class_hash="0x05e540e69c3a834dc920c45ecb0d57b902130ece8a9a5d8c96b9c30dec5d0733"

echo "Deploying contract with class hash: $class_hash"

max_fee="--max-fee-raw 677831717532860"

# Deploy the contract using the extracted class hash
starkli deploy $class_hash $max_fee