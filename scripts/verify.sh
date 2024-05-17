#!/bin/bash

guts_contract="0x07e99abfda19044d7aaf6e296b6ccd78aeabb8d9e205ae919474db03aed7e822"
pub_key="u256:0x26cd99663f8fcd42ea8a68aaf69bb811d3d0193aff830ce874527eae0adb8a9e"
msg="8 0x01 0x02 0x03 0x04 0xab 0xcd 0xef 0xaa"
signature="2 u256:0x8be5e9fac46d8fd1921d3f001e74e00afb39fd4935124bc49e223ccf7eb74db1 u256:0x8c1d3c4769f499517347b66b3b19042f8af8752703aee4e00da2e97b8d566702"
verify_selector="selector:verify_signed_commit"
max_fee="--max-fee-raw 677831717532860"

starkli invoke $guts_contract $verify_selector $pub_key $msg $signature $max_fee


