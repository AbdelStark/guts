use alexandria_math::ed25519::{p, Point, verify_signature, SpanU8TryIntoPoint};

fn main() -> bool {
    let msg: Span<u8> = gen_msg();
    let sig: Span<u8> = gen_sig();
    let pub_key: Span<u8> = gen_pub_key();

    let is_valid = verify_signature(msg, sig, pub_key);

    println!("valid: {is_valid}");

    is_valid
}

fn gen_msg() -> Span<u8> {
    array![0xab, 0xcd].span()
}

fn gen_sig() -> Span<u8> {
    // 71eb4ef992551292a9ba5a4817df47fdda2372b2065ed60758b7ee346b7a9e786a9473f6492676e988709498b228df873fe3cfdf59255b1a9e1add4f87ec610b
    let mut sig: Array<u8> = array![
        0x71,
        0xeb,
        0x4e,
        0xf9,
        0x92,
        0x55,
        0x12,
        0x92,
        0xa9,
        0xba,
        0x5a,
        0x48,
        0x17,
        0xdf,
        0x47,
        0xfd,
        0xda,
        0x23,
        0x72,
        0xb2,
        0x06,
        0x5e,
        0xd6,
        0x07,
        0x58,
        0xb7,
        0xee,
        0x34,
        0x6b,
        0x7a,
        0x9e,
        0x78,
        0x6a,
        0x94,
        0x73,
        0xf6,
        0x49,
        0x26,
        0x76,
        0xe9,
        0x88,
        0x70,
        0x94,
        0x98,
        0xb2,
        0x28,
        0xdf,
        0x87,
        0x3f,
        0xe3,
        0xcf,
        0xdf,
        0x59,
        0x25,
        0x5b,
        0x1a,
        0x9e,
        0x1a,
        0xdd,
        0x4f,
        0x87,
        0xec,
        0x61,
        0x0b
    ];
    sig.span()
}

fn gen_pub_key() -> Span<u8> {
    let mut pub_key: Array<u8> = array![
        0x1e,
        0x6c,
        0x5b,
        0x38,
        0x58,
        0x80,
        0x84,
        0x9f,
        0x46,
        0x71,
        0x6d,
        0x69,
        0x1b,
        0x8a,
        0x44,
        0x7d,
        0x7c,
        0xbe,
        0x4a,
        0x7e,
        0xf1,
        0x54,
        0xf3,
        0xe2,
        0x17,
        0x4f,
        0xfb,
        0x3c,
        0x52,
        0x56,
        0xfc,
        0xfe
    ];
    pub_key.span()
}

#[cfg(test)]
mod tests {}
