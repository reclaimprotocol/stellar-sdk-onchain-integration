#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Env};

// =============================================================================
// Helper Functions
// =============================================================================

fn setup_env() -> (Env, ReclaimContractClient<'static>, soroban_sdk::Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ReclaimContract);
    let client = ReclaimContractClient::new(&env, &contract_id);
    let user = soroban_sdk::Address::generate(&env);
    (env, client, user)
}

fn create_witness(env: &Env, hex_address: &str, host: &str) -> Witness {
    let bytes = hex::decode(hex_address).expect("invalid hex address");
    let items: &[u8; 20] = bytes[0..20].try_into().expect("slice with incorrect length");
    Witness {
        address: BytesN::<20>::from_array(env, items),
        host: String::from_str(env, host),
    }
}

fn create_witness_list(env: &Env, witnesses_data: &[(&str, &str)]) -> Vec<Witness> {
    let mut witnesses = Vec::new(env);
    for (hex_address, host) in witnesses_data {
        witnesses.push_back(create_witness(env, hex_address, host));
    }
    witnesses
}

fn hex_to_bytes_32(hex_str: &str) -> [u8; 32] {
    let bytes = hex::decode(hex_str).expect("invalid hex");
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes[0..32]);
    arr
}

fn hex_to_bytes_64(hex_str: &str) -> [u8; 64] {
    let bytes = hex::decode(hex_str).expect("invalid hex");
    let mut arr = [0u8; 64];
    arr.copy_from_slice(&bytes[0..64]);
    arr
}

// Known test data: witness address that matches the signature
const TEST_WITNESS_ADDRESS: &str = "244897572368eadf65bfbc5aec98d8e5443a9072";
const TEST_MESSAGE_DIGEST: &str = "c32e57b71247c1aab4b93bb0a2bb373186acc2d5c9bd8dfcd046e1d0553fd421";
const TEST_SIGNATURE: &str = "2888485f650f8ed02d18e32dd9a1512ca05feb83fc2cbf2df72fd8aa4246c5ee541fa53875c70eb64d3de9143446229a250c7a762202b7cc289ed31b74b31c81";
const TEST_RECOVERY_ID: u32 = 1;

// =============================================================================
// Initialization Tests
// =============================================================================

#[test]
fn test_instantiate_success() {
    let (_, client, user) = setup_env();
    assert_eq!(client.instantiate(&user), ());
}

#[test]
fn test_instantiate_sets_default_epoch() {
    let (env, client, user) = setup_env();
    client.instantiate(&user);

    // Verify proof with default witness should work
    // The default witness is 0x244897572368eadf65bfbc5aec98d8e5443a9072
    let message_digest = BytesN::from_array(&env, &hex_to_bytes_32(TEST_MESSAGE_DIGEST));
    let signature = BytesN::from_array(&env, &hex_to_bytes_64(TEST_SIGNATURE));

    assert_eq!(client.verify_proof(&message_digest, &signature, &TEST_RECOVERY_ID), ());
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_instantiate_fails_when_already_initialized() {
    let (_, client, user) = setup_env();

    client.instantiate(&user);
    // Second instantiation should fail with AlreadyInitialized error
    client.instantiate(&user);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_instantiate_fails_with_different_user() {
    let (env, client, user) = setup_env();
    let another_user = soroban_sdk::Address::generate(&env);

    client.instantiate(&user);
    // Another user trying to initialize should also fail
    client.instantiate(&another_user);
}

// =============================================================================
// Add Epoch Tests
// =============================================================================

#[test]
fn test_add_epoch_success() {
    let (env, client, user) = setup_env();
    client.instantiate(&user);

    let witnesses = create_witness_list(&env, &[(TEST_WITNESS_ADDRESS, "https://witness1.example.com")]);

    assert_eq!(client.add_epoch(&witnesses, &1_u32), ());
}

#[test]
fn test_add_epoch_with_multiple_witnesses() {
    let (env, client, user) = setup_env();
    client.instantiate(&user);

    let witnesses = create_witness_list(&env, &[
        (TEST_WITNESS_ADDRESS, "https://witness1.example.com"),
        ("1234567890abcdef1234567890abcdef12345678", "https://witness2.example.com"),
        ("abcdef1234567890abcdef1234567890abcdef12", "https://witness3.example.com"),
    ]);

    assert_eq!(client.add_epoch(&witnesses, &2_u32), ());
}

#[test]
fn test_add_multiple_epochs_increments_epoch_id() {
    let (env, client, user) = setup_env();
    client.instantiate(&user);

    let witnesses = create_witness_list(&env, &[(TEST_WITNESS_ADDRESS, "https://witness.example.com")]);

    // Add first epoch (will be epoch 1, since epoch 0 is created during instantiate)
    client.add_epoch(&witnesses, &1_u32);

    // Add second epoch (will be epoch 2)
    client.add_epoch(&witnesses, &1_u32);

    // Add third epoch (will be epoch 3)
    client.add_epoch(&witnesses, &1_u32);

    // Verify the latest epoch's witness works
    let message_digest = BytesN::from_array(&env, &hex_to_bytes_32(TEST_MESSAGE_DIGEST));
    let signature = BytesN::from_array(&env, &hex_to_bytes_64(TEST_SIGNATURE));

    assert_eq!(client.verify_proof(&message_digest, &signature, &TEST_RECOVERY_ID), ());
}

#[test]
fn test_add_epoch_replaces_previous_epoch_witnesses() {
    let (env, client, user) = setup_env();
    client.instantiate(&user);

    // Add epoch with a different witness
    let new_witnesses = create_witness_list(&env, &[
        ("1234567890abcdef1234567890abcdef12345678", "https://new-witness.example.com"),
    ]);
    client.add_epoch(&new_witnesses, &1_u32);

    // Now the old witness signature should fail
    let message_digest = BytesN::from_array(&env, &hex_to_bytes_32(TEST_MESSAGE_DIGEST));
    let signature = BytesN::from_array(&env, &hex_to_bytes_64(TEST_SIGNATURE));

    let result = client.try_verify_proof(&message_digest, &signature, &TEST_RECOVERY_ID);
    assert!(result.is_err());
}

// =============================================================================
// Verify Proof Tests
// =============================================================================

#[test]
fn test_verify_proof_success() {
    let (env, client, user) = setup_env();
    client.instantiate(&user);

    let witnesses = create_witness_list(&env, &[(TEST_WITNESS_ADDRESS, "https://witness.example.com")]);
    client.add_epoch(&witnesses, &1_u32);

    let message_digest = BytesN::from_array(&env, &hex_to_bytes_32(TEST_MESSAGE_DIGEST));
    let signature = BytesN::from_array(&env, &hex_to_bytes_64(TEST_SIGNATURE));

    assert_eq!(client.verify_proof(&message_digest, &signature, &TEST_RECOVERY_ID), ());
}

#[test]
fn test_verify_proof_with_witness_in_list() {
    let (env, client, user) = setup_env();
    client.instantiate(&user);

    // Add epoch with the test witness as second in list
    let witnesses = create_witness_list(&env, &[
        ("1111111111111111111111111111111111111111", "https://witness1.example.com"),
        (TEST_WITNESS_ADDRESS, "https://witness2.example.com"),
        ("2222222222222222222222222222222222222222", "https://witness3.example.com"),
    ]);
    client.add_epoch(&witnesses, &2_u32);

    let message_digest = BytesN::from_array(&env, &hex_to_bytes_32(TEST_MESSAGE_DIGEST));
    let signature = BytesN::from_array(&env, &hex_to_bytes_64(TEST_SIGNATURE));

    assert_eq!(client.verify_proof(&message_digest, &signature, &TEST_RECOVERY_ID), ());
}

#[test]
#[should_panic(expected = "Error(Crypto, InvalidInput)")]
fn test_verify_proof_fails_with_invalid_signature() {
    let (env, client, user) = setup_env();
    client.instantiate(&user);

    let witnesses = create_witness_list(&env, &[(TEST_WITNESS_ADDRESS, "https://witness.example.com")]);
    client.add_epoch(&witnesses, &1_u32);

    let message_digest = BytesN::from_array(&env, &hex_to_bytes_32(TEST_MESSAGE_DIGEST));
    // Invalid signature (all zeros) - causes low-level crypto error
    let invalid_signature = BytesN::from_array(&env, &[0u8; 64]);

    client.verify_proof(&message_digest, &invalid_signature, &TEST_RECOVERY_ID);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_verify_proof_fails_with_wrong_message_digest() {
    let (env, client, user) = setup_env();
    client.instantiate(&user);

    let witnesses = create_witness_list(&env, &[(TEST_WITNESS_ADDRESS, "https://witness.example.com")]);
    client.add_epoch(&witnesses, &1_u32);

    // Wrong message digest
    let wrong_digest = BytesN::from_array(&env, &[1u8; 32]);
    let signature = BytesN::from_array(&env, &hex_to_bytes_64(TEST_SIGNATURE));

    client.verify_proof(&wrong_digest, &signature, &TEST_RECOVERY_ID);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_verify_proof_fails_with_wrong_recovery_id() {
    let (env, client, user) = setup_env();
    client.instantiate(&user);

    let witnesses = create_witness_list(&env, &[(TEST_WITNESS_ADDRESS, "https://witness.example.com")]);
    client.add_epoch(&witnesses, &1_u32);

    let message_digest = BytesN::from_array(&env, &hex_to_bytes_32(TEST_MESSAGE_DIGEST));
    let signature = BytesN::from_array(&env, &hex_to_bytes_64(TEST_SIGNATURE));

    // Wrong recovery ID (0 instead of 1)
    client.verify_proof(&message_digest, &signature, &0_u32);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_verify_proof_fails_with_unknown_witness() {
    let (env, client, user) = setup_env();
    client.instantiate(&user);

    // Add epoch with a different witness (not the one that signed)
    let witnesses = create_witness_list(&env, &[
        ("1234567890abcdef1234567890abcdef12345678", "https://different-witness.example.com"),
    ]);
    client.add_epoch(&witnesses, &1_u32);

    let message_digest = BytesN::from_array(&env, &hex_to_bytes_32(TEST_MESSAGE_DIGEST));
    let signature = BytesN::from_array(&env, &hex_to_bytes_64(TEST_SIGNATURE));

    // This should fail because the recovered address won't match
    client.verify_proof(&message_digest, &signature, &TEST_RECOVERY_ID);
}

#[test]
fn test_verify_proof_respects_minimum_witness_count() {
    let (env, client, user) = setup_env();
    client.instantiate(&user);

    // Add epoch with test witness as third, but minimum_witness = 2
    // So only first 2 witnesses are checked
    let witnesses = create_witness_list(&env, &[
        ("1111111111111111111111111111111111111111", "https://witness1.example.com"),
        ("2222222222222222222222222222222222222222", "https://witness2.example.com"),
        (TEST_WITNESS_ADDRESS, "https://witness3.example.com"),
    ]);
    client.add_epoch(&witnesses, &2_u32);

    let message_digest = BytesN::from_array(&env, &hex_to_bytes_32(TEST_MESSAGE_DIGEST));
    let signature = BytesN::from_array(&env, &hex_to_bytes_64(TEST_SIGNATURE));

    // This should fail because only first 2 witnesses are checked (minimum_witness = 2)
    // and test witness is at position 3
    let result = client.try_verify_proof(&message_digest, &signature, &TEST_RECOVERY_ID);
    assert!(result.is_err());
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_add_epoch_with_minimum_witness_one() {
    let (env, client, user) = setup_env();
    client.instantiate(&user);

    let witnesses = create_witness_list(&env, &[
        (TEST_WITNESS_ADDRESS, "https://witness1.example.com"),
        ("1234567890abcdef1234567890abcdef12345678", "https://witness2.example.com"),
    ]);
    // Only require 1 witness signature
    client.add_epoch(&witnesses, &1_u32);

    let message_digest = BytesN::from_array(&env, &hex_to_bytes_32(TEST_MESSAGE_DIGEST));
    let signature = BytesN::from_array(&env, &hex_to_bytes_64(TEST_SIGNATURE));

    assert_eq!(client.verify_proof(&message_digest, &signature, &TEST_RECOVERY_ID), ());
}

#[test]
fn test_witness_with_different_hosts() {
    let (env, client, user) = setup_env();
    client.instantiate(&user);

    let witnesses = create_witness_list(&env, &[
        (TEST_WITNESS_ADDRESS, "https://api.reclaim.io"),
        (TEST_WITNESS_ADDRESS, "https://backup.reclaim.io"),
    ]);
    client.add_epoch(&witnesses, &1_u32);

    let message_digest = BytesN::from_array(&env, &hex_to_bytes_32(TEST_MESSAGE_DIGEST));
    let signature = BytesN::from_array(&env, &hex_to_bytes_64(TEST_SIGNATURE));

    assert_eq!(client.verify_proof(&message_digest, &signature, &TEST_RECOVERY_ID), ());
}
