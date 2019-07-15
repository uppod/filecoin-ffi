use std::slice::from_raw_parts;

use ffi_toolkit::{c_str_to_rust_str, raw_ptr, rust_str_to_c_str};
use filecoin_proofs as api_fns;
use filecoin_proofs::types as api_types;
use libc;
use slog::info;

use crate::helpers;
use crate::responses::*;
use crate::singletons::FCPFFI_LOG;

/// Verifies the output of seal.
///
#[no_mangle]
pub unsafe extern "C" fn verify_seal(
    sector_size: u64,
    comm_r: &[u8; 32],
    comm_d: &[u8; 32],
    comm_r_star: &[u8; 32],
    prover_id: &[u8; 31],
    sector_id: &[u8; 31],
    proof_ptr: *const u8,
    proof_len: libc::size_t,
) -> *mut VerifySealResponse {
    info!(FCPFFI_LOG, "verify_seal: {}", "start"; "target" => "FFI");

    let porep_bytes = helpers::try_into_porep_proof_bytes(proof_ptr, proof_len);

    let result = porep_bytes.and_then(|bs| {
        helpers::porep_proof_partitions_try_from_bytes(&bs).and_then(|ppp| {
            let cfg = api_types::PoRepConfig(api_types::SectorSize(sector_size), ppp);

            api_fns::verify_seal(
                cfg,
                *comm_r,
                *comm_d,
                *comm_r_star,
                prover_id,
                sector_id,
                &bs,
            )
        })
    });

    let mut response = VerifySealResponse::default();

    match result {
        Ok(true) => {
            response.status_code = 0;
            response.is_valid = true;
        }
        Ok(false) => {
            response.status_code = 0;
            response.is_valid = false;
        }
        Err(err) => {
            response.status_code = 1;
            response.error_msg = rust_str_to_c_str(format!("{}", err));
        }
    };

    info!(FCPFFI_LOG, "verify_seal: {}", "finish"; "target" => "FFI");

    raw_ptr(response)
}

/// Verifies that a proof-of-spacetime is valid.
///
#[no_mangle]
pub unsafe extern "C" fn verify_post(
    sector_size: u64,
    proof_partitions: u8,
    flattened_comm_rs_ptr: *const u8,
    flattened_comm_rs_len: libc::size_t,
    challenge_seed: &[u8; 32],
    flattened_proofs_ptr: *const u8,
    flattened_proofs_len: libc::size_t,
    faults_ptr: *const u64,
    faults_len: libc::size_t,
) -> *mut VerifyPoStResponse {
    info!(FCPFFI_LOG, "verify_post: {}", "start"; "target" => "FFI");

    let post_bytes = helpers::try_into_post_proofs_bytes(
        proof_partitions,
        flattened_proofs_ptr,
        flattened_proofs_len,
    );

    let result = post_bytes.and_then(|bs| {
        let cfg = api_types::PoStConfig(
            api_types::SectorSize(sector_size),
            api_types::PoStProofPartitions(proof_partitions),
        );

        api_fns::verify_post(
            cfg,
            helpers::into_commitments(flattened_comm_rs_ptr, flattened_comm_rs_len),
            helpers::into_safe_challenge_seed(challenge_seed),
            bs,
            from_raw_parts(faults_ptr, faults_len).to_vec(),
        )
    });

    let mut response = VerifyPoStResponse::default();

    match result {
        Ok(dynamic) => {
            response.status_code = 0;
            response.is_valid = dynamic.is_valid;
        }
        Err(err) => {
            response.status_code = 1;
            response.error_msg = rust_str_to_c_str(format!("{}", err));
        }
    }

    info!(FCPFFI_LOG, "verify_post: {}", "finish"; "target" => "FFI");

    raw_ptr(response)
}

#[allow(dead_code)]
#[no_mangle]
pub unsafe extern "C" fn verify_piece_inclusion_proof(
    comm_d: &[u8; 32],
    comm_p: &[u8; 32],
    piece_inclusion_proof_ptr: *const u8,
    piece_inclusion_proof_len: libc::size_t,
    padded_and_aligned_piece_size: u64,
    sector_size: u64,
) -> *mut VerifyPieceInclusionProofResponse {
    info!(FCPFFI_LOG, "verify_piece_inclusion_proof: {}", "start"; "target" => "FFI");

    let bytes = from_raw_parts(piece_inclusion_proof_ptr, piece_inclusion_proof_len);

    let padded_and_aligned_piece_size = api_types::PaddedBytesAmount(padded_and_aligned_piece_size);
    let sector_size = api_types::SectorSize(sector_size);

    let result =
        api_fns::verify_piece_inclusion_proof(bytes, comm_d, comm_p, padded_and_aligned_piece_size, sector_size);

    let mut response = VerifyPieceInclusionProofResponse::default();

    match result {
        Ok(true) => {
            response.status_code = 0;
            response.is_valid = true;
        }
        Ok(false) => {
            response.status_code = 0;
            response.is_valid = false;
        }
        Err(err) => {
            response.status_code = 1;
            response.error_msg = rust_str_to_c_str(format!("{}", err));
        }
    };

    info!(FCPFFI_LOG, "verify_piece_inclusion_proof: {}", "finish"; "target" => "FFI");

    raw_ptr(response)
}

#[no_mangle]
pub unsafe extern "C" fn destroy_verify_piece_inclusion_proof_response(
    ptr: *mut VerifyPieceInclusionProofResponse,
) {
    let _ = Box::from_raw(ptr);
}

#[allow(dead_code)]
#[no_mangle]
pub unsafe extern "C" fn generate_piece_commitment(
    piece_path: *const libc::c_char,
    unpadded_piece_size: u64,
) -> *mut GeneratePieceCommitmentResponse {
    let unpadded_piece_size = api_types::UnpaddedBytesAmount(unpadded_piece_size);

    let result = api_fns::generate_piece_commitment(
        c_str_to_rust_str(piece_path).to_string(),
        unpadded_piece_size,
    );

    let mut response = GeneratePieceCommitmentResponse::default();

    match result {
        Ok((comm_p, padded_and_aligned_piece_size)) => {
            response.status_code = 0;
            response.comm_p = comm_p;
            response.padded_and_aligned_piece_size = padded_and_aligned_piece_size.into();
        }
        Err(err) => {
            response.status_code = 1;
            response.error_msg = rust_str_to_c_str(format!("{}", err));
        }
    }

    raw_ptr(response)
}

#[no_mangle]
pub unsafe extern "C" fn destroy_generate_piece_commitment_response(
    ptr: *mut GeneratePieceCommitmentResponse,
) {
    let _ = Box::from_raw(ptr);
}

/// Returns the number of user bytes that will fit into a staged sector.
///
#[no_mangle]
pub unsafe extern "C" fn get_max_user_bytes_per_staged_sector(sector_size: u64) -> u64 {
    u64::from(api_types::UnpaddedBytesAmount::from(api_types::SectorSize(
        sector_size,
    )))
}

/// Deallocates a VerifySealResponse.
///
#[no_mangle]
pub unsafe extern "C" fn destroy_verify_seal_response(ptr: *mut VerifySealResponse) {
    let _ = Box::from_raw(ptr);
}

/// Deallocates a VerifyPoStResponse.
///
#[no_mangle]
pub unsafe extern "C" fn destroy_verify_post_response(ptr: *mut VerifyPoStResponse) {
    let _ = Box::from_raw(ptr);
}
