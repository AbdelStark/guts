use alexandria_math::ed25519::{p, Point, verify_signature};

#[test]
fn test_simple_ed25519_signature() {
    let pub_key: u256 = 0x26cd99663f8fcd42ea8a68aaf69bb811d3d0193aff830ce874527eae0adb8a9e;

    let msg: Span<u8> = array![0x01, 0x02, 0x03, 0x04, 0xab, 0xcd, 0xef, 0xaa].span();

    let r_sign = 0x8be5e9fac46d8fd1921d3f001e74e00afb39fd4935124bc49e223ccf7eb74db1;
    let s_sign = 0x8c1d3c4769f499517347b66b3b19042f8af8752703aee4e00da2e97b8d566702;
    let signature = array![r_sign, s_sign];
    let is_valid: bool = verify_signature(msg, signature.span(), pub_key);

    assert!(is_valid, "works");
}
