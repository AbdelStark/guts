use starknet::ContractAddress;

#[starknet::interface]
trait GutsTrait<T> {
    fn verify_signed_commit(ref self: T, pub_key: u256, msg: Span<u8>, signature: Span<u256>);
}

#[starknet::contract]
mod Guts {
    use super::ContractAddress;
    use starknet::get_caller_address;
    use alexandria_math::ed25519::{p, Point, verify_signature};


    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        GitCommitVerified: GitCommitVerified,
    }

    #[derive(Drop, starknet::Event)]
    struct GitCommitVerified {
        #[key]
        starknet_address: ContractAddress,
        signature: Span<u8>,
    }

    #[storage]
    struct Storage {
        owner: ContractAddress,
    }

    #[constructor]
    fn constructor(ref self: ContractState,) {}

    #[abi(embed_v0)]
    impl GutsImpl of super::GutsTrait<ContractState> {
        fn verify_signed_commit(
            ref self: ContractState, pub_key: u256, msg: Span<u8>, signature: Span<u256>
        ) {
            let is_valid: bool = verify_signature(msg, signature, pub_key);
            assert!(is_valid, "Invalid signature");
        }
    }


    #[generate_trait]
    impl GutsPrivateMethods of PrivateMethodsTrait {}
}
