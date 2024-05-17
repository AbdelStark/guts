#!/bin/bash

# Deployment script for Guts contract

compiler_version="2.6.2"
network="sepolia"

target_dir="./target/dev"
contract_class_file="$target_dir/guts_Guts.contract_class.json"

# Declare the contract and capture the command output
#command_output=$(starkli declare $contract_class_file --compiler-version=$compiler_version --watch)

from_string="Class hash declared:"
#class_hash="${command_output#*$from_string}"
class_hash="0x005bae77c6195ffeee99f8ea5740ff77a0629482bc5beb90b062081d7c90756b"

echo "Deploying contract with class hash: $class_hash"

max_fee="--max-fee-raw 677831717532860"

# Deploy the contract using the extracted class hash
starkli deploy $class_hash $max_fee