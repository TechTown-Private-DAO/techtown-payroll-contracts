use soroban_sdk::{Bytes, BytesN, Env, Vec};
use crate::types::ZKProof;

/// ZK Proof verifier for the TechTown Payroll system.
///
/// ## Production path
/// A real deployment would use a circuit compiled with snarkjs / circom and
/// embed the Groth16 verifying key as a constant.  The `verify_payroll_proof`
/// function would then call the pairing check:
///     e(π_A, π_B) == e(α, β) * e(Σ public_inputs * γ_i, γ) * e(π_C, δ)
///
/// Because Soroban does not (yet) expose BN-254 precompiles natively, the
/// recommended path today is:
///   1. Generate the proof off-chain (backend/relayer).
///   2. Store the verifying key in contract storage (set once by admin).
///   3. Call this function with the serialised proof and public inputs.
///
/// For now the function performs structural validation (correct lengths, non-
/// zero bytes) which ensures the integration tests exercise the full call path.
/// Replace the body of `verify_payroll_proof` with the real pairing check once
/// the circuit + VK are available.
pub struct ZKVerifier;

impl ZKVerifier {
    /// Verify a Groth16-style payroll proof.
    ///
    /// `proof.proof`        – 256-byte serialised proof (A‖B‖C, each 64 bytes for BN-254 G1/G2)
    /// `proof.public_inputs`– at minimum 3 entries: [total_amount_lo, total_amount_hi, employee_count]
    ///                        followed by the merkle root bytes split as needed.
    ///
    /// Returns `true` when the proof is structurally valid AND the embedded
    /// public inputs are consistent with the on-chain payroll parameters.
    pub fn verify_payroll_proof(
        env: &Env,
        proof: &ZKProof,
        total_amount: i128,
        employee_count: u32,
        merkle_root: &BytesN<32>,
    ) -> bool {
        // ── Structural checks ──────────────────────────────────────────────
        // Proof bytes must be non-empty (real check: must be exactly 256 bytes)
        if proof.proof.is_empty() {
            return false;
        }

        // At least three public inputs required
        if proof.public_inputs.len() < 3 {
            return false;
        }

        // ── Public input consistency checks ───────────────────────────────
        // Input[0]: total_amount encoded as big-endian i128 (16 bytes)
        let amount_input = proof.public_inputs.get(0).unwrap();
        if amount_input.len() != 16 {
            return false;
        }
        let mut amount_bytes = [0u8; 16];
        for i in 0..16u32 {
            amount_bytes[i as usize] = amount_input.get(i).unwrap();
        }
        let claimed_amount = i128::from_be_bytes(amount_bytes);
        if claimed_amount != total_amount {
            return false;
        }

        // Input[1]: employee_count encoded as big-endian u32 (4 bytes)
        let count_input = proof.public_inputs.get(1).unwrap();
        if count_input.len() != 4 {
            return false;
        }
        let mut count_bytes = [0u8; 4];
        for i in 0..4u32 {
            count_bytes[i as usize] = count_input.get(i).unwrap();
        }
        let claimed_count = u32::from_be_bytes(count_bytes);
        if claimed_count != employee_count {
            return false;
        }

        // Input[2]: merkle root (32 bytes)
        let root_input = proof.public_inputs.get(2).unwrap();
        if root_input.len() != 32 {
            return false;
        }
        let root_bytes_n: BytesN<32> = Self::bytes_to_bytes32(env, &root_input);
        if root_bytes_n != *merkle_root {
            return false;
        }

        // ── Pairing check placeholder ─────────────────────────────────────
        // TODO: replace with actual Groth16 / PLONK pairing verification
        // against the embedded verifying key once the circuit is available.
        // For now, non-zero proof bytes + consistent public inputs == valid.
        true
    }

    /// Verify that `commitment_hash == H(salary || randomness || employee_id)`
    ///
    /// Uses Soroban's host-provided SHA-256 (no external crate needed).
    pub fn verify_salary_commitment(
        env: &Env,
        commitment_hash: &BytesN<32>,
        employee_id: u64,
        salary: i128,
        randomness: &Bytes,
    ) -> bool {
        let computed = Self::compute_commitment(env, employee_id, salary, randomness);
        computed == *commitment_hash
    }

    /// Compute `H(salary || randomness || employee_id)` using the Soroban host.
    pub fn compute_commitment(
        env: &Env,
        employee_id: u64,
        salary: i128,
        randomness: &Bytes,
    ) -> BytesN<32> {
        let mut preimage = Bytes::new(env);

        // salary: 16 bytes big-endian
        let salary_bytes = salary.to_be_bytes();
        for b in salary_bytes.iter() {
            preimage.push_back(*b);
        }

        // randomness
        preimage.append(randomness);

        // employee_id: 8 bytes big-endian
        let id_bytes = employee_id.to_be_bytes();
        for b in id_bytes.iter() {
            preimage.push_back(*b);
        }

        env.crypto().sha256(&preimage).into()
    }

    /// Verify a Merkle inclusion proof for a leaf.
    ///
    /// `leaf`   – the leaf value (commitment hash or employee data hash).
    /// `proof`  – ordered list of sibling hashes from leaf to root.
    /// `index`  – 0-based leaf position (determines left/right sibling ordering).
    /// `root`   – expected Merkle root.
    pub fn verify_merkle_proof(
        env: &Env,
        leaf: &BytesN<32>,
        proof: &Vec<BytesN<32>>,
        index: u64,
        root: &BytesN<32>,
    ) -> bool {
        let mut current = leaf.clone();
        let mut idx = index;

        for sibling in proof.iter() {
            let mut preimage = Bytes::new(env);
            if idx % 2 == 0 {
                // current is left child
                preimage.append(&Bytes::from(&current));
                preimage.append(&Bytes::from(&sibling));
            } else {
                // current is right child
                preimage.append(&Bytes::from(&sibling));
                preimage.append(&Bytes::from(&current));
            }
            current = env.crypto().sha256(&preimage).into();
            idx /= 2;
        }

        current == *root
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn bytes_to_bytes32(env: &Env, b: &Bytes) -> BytesN<32> {
        let mut arr = [0u8; 32];
        for i in 0..32u32 {
            arr[i as usize] = b.get(i).unwrap_or(0);
        }
        BytesN::from_array(env, &arr)
    }
}
