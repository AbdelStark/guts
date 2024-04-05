#!/bin/bash

# Deployment script for Guts contract

compiler_version="2.6.2"
network="sepolia"

target_dir="../../target/dev"
contract_class_file="$target_dir/guts_Guts.contract_class.json"

# Declare the contract and capture the command output
command_output=$(starkli declare $contract_class_file --compiler-version=$compiler_version)

from_string="Class hash declared:"
class_hash="${command_output#*$from_string}"

# Deploy the contract using the extracted class hash
starkli deploy $class_hash