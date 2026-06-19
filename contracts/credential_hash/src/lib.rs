#![no_std]

//! credential_hash — Academic Credential Hash Anchor
//!
//! Schools (issuers) anchor a hash of every diploma / certificate they
//! hand out on the Stellar blockchain using a Soroban smart contract.
//! The full credential document stays off-chain (e.g. on IPFS or the
//! school's own storage), preserving student privacy and minimizing
//! on-chain cost. Any verifier — an employer, consulate, another
//! university — can later confirm whether a presented hash exists,
//! which school issued it, when it was issued, and whether it has been
//! revoked or has expired, all without trusting a third party.

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Symbol};

// ---------------------------------------------------------------------------
// Constants & types
// ---------------------------------------------------------------------------

/// Minimum remaining TTL (in ledgers) before we extend storage.
const TTL_THRESHOLD: u32 = 1_000;
/// TTL (in ledgers) to extend storage to on every write.
const TTL_EXTEND_TO: u32 = 1_000_000;

/// Status codes returned by `verify()`.
pub const STATUS_UNKNOWN: u32 = 0;
pub const STATUS_VALID: u32 = 1;
pub const STATUS_EXPIRED: u32 = 2;
pub const STATUS_REVOKED: u32 = 3;

/// A registered academic credential hash plus its on-chain metadata.
#[contracttype]
#[derive(Clone)]
pub struct Credential {
    /// Stellar address of the school that issued the credential.
    pub school: Address,
    /// Stellar address of the recipient (student / graduate).
    pub holder: Address,
    /// UNIX timestamp at which the credential was anchored.
    pub issued_at: u64,
    /// UNIX timestamp after which the credential is treated as expired.
    pub expires_at: u64,
    /// `true` once the issuing school has explicitly revoked it.
    pub revoked: bool,
    /// Free-text reason recorded at revocation time.
    pub revoke_reason: String,
}

/// Audit-trail record stored at revocation time.
#[contracttype]
#[derive(Clone)]
pub struct RevokeInfo {
    /// UNIX timestamp at which the revocation was recorded.
    pub revoked_at: u64,
    /// Human-readable reason provided by the school.
    pub reason: String,
}

/// All keys written by this contract to persistent storage.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Full `Credential` record, keyed by the credential hash.
    Cred(Symbol),
    /// Revocation audit record, keyed by the credential hash.
    RevokeInfo(Symbol),
    /// Per-holder counter of anchored credentials, keyed by holder address.
    HolderCount(Address),
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

/// The credential-hash anchor contract.
#[contract]
pub struct CredentialHash;

#[contractimpl]
impl CredentialHash {
    /// Register a new academic credential hash on-chain.
    ///
    /// The `school` (issuer) must sign the transaction. The full diploma
    /// / certificate stays off-chain — only its hash, the recipient's
    /// Stellar address, and the expiry timestamp are anchored.
    ///
    /// * `cred_hash`  — symbolic identifier for the credential hash.
    /// * `holder`     — Stellar address of the recipient.
    /// * `expires_at` — UNIX timestamp after which the credential is expired.
    ///
    /// Returns `1` on success.
    pub fn register_credential(
        env: Env,
        school: Address,
        cred_hash: Symbol,
        holder: Address,
        expires_at: u64,
    ) -> u32 {
        // The issuing school must authorize this write.
        school.require_auth();

        let key = DataKey::Cred(cred_hash);
        if env.storage().persistent().has(&key) {
            panic!("credential already registered");
        }
        if expires_at <= env.ledger().timestamp() {
            panic!("expires_at must be in the future");
        }

        let cred = Credential {
            school: school.clone(),
            holder: holder.clone(),
            issued_at: env.ledger().timestamp(),
            expires_at,
            revoked: false,
            revoke_reason: String::from_str(&env, ""),
        };
        env.storage().persistent().set(&key, &cred);
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND_TO);

        // Increment the holder's anchored-credential counter.
        let count_key = DataKey::HolderCount(holder);
        let count: u32 = env
            .storage()
            .persistent()
            .get(&count_key)
            .unwrap_or(0u32);
        env.storage()
            .persistent()
            .set(&count_key, &(count + 1));
        env.storage()
            .persistent()
            .extend_ttl(&count_key, TTL_THRESHOLD, TTL_EXTEND_TO);

        1
    }

    /// Revoke a previously registered credential hash.
    ///
    /// Only the issuing `school` may revoke. The revocation is
    /// tamper-evident: it records both a timestamp and a free-text
    /// reason, and flips the `revoked` flag on the credential record
    /// so that `verify()` instantly returns `STATUS_REVOKED`.
    ///
    /// Returns `1` on success.
    pub fn revoke_credential(
        env: Env,
        school: Address,
        cred_hash: Symbol,
        reason: String,
    ) -> u32 {
        school.require_auth();

        let key = DataKey::Cred(cred_hash.clone());
        let mut cred: Credential = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| panic!("unknown credential"));

        if cred.school != school {
            panic!("only issuing school can revoke");
        }
        if cred.revoked {
            panic!("credential already revoked");
        }

        cred.revoked = true;
        cred.revoke_reason = reason.clone();
        env.storage().persistent().set(&key, &cred);
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND_TO);

        // Audit-trail record so the history is independent of the
        // (potentially later overwritten) credential record.
        let rkey = DataKey::RevokeInfo(cred_hash);
        let info = RevokeInfo {
            revoked_at: env.ledger().timestamp(),
            reason,
        };
        env.storage().persistent().set(&rkey, &info);
        env.storage()
            .persistent()
            .extend_ttl(&rkey, TTL_THRESHOLD, TTL_EXTEND_TO);

        1
    }

    /// Verify the current status of a credential hash.
    ///
    /// Returns one of:
    /// * `0` — `STATUS_UNKNOWN` (the hash was never registered)
    /// * `1` — `STATUS_VALID`   (registered, not expired, not revoked)
    /// * `2` — `STATUS_EXPIRED` (registered, expiry has passed)
    /// * `3` — `STATUS_REVOKED` (registered, but explicitly revoked)
    pub fn verify(env: Env, cred_hash: Symbol) -> u32 {
        let key = DataKey::Cred(cred_hash);
        let cred: Credential = match env.storage().persistent().get(&key) {
            Some(c) => c,
            None => return STATUS_UNKNOWN,
        };
        if cred.revoked {
            return STATUS_REVOKED;
        }
        if env.ledger().timestamp() >= cred.expires_at {
            return STATUS_EXPIRED;
        }
        STATUS_VALID
    }

    /// Return the `Address` of the school that issued the given
    /// credential hash. Panics if the hash is unknown.
    pub fn get_school(env: Env, cred_hash: Symbol) -> Address {
        let key = DataKey::Cred(cred_hash);
        let cred: Credential = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| panic!("unknown credential"));
        cred.school
    }

    /// Return the number of credentials currently anchored on-chain for
    /// `holder`. Useful for portfolio UIs, degree aggregators, and
    /// background-check dashboards.
    pub fn list_credentials(env: Env, holder: Address) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::HolderCount(holder))
            .unwrap_or(0u32)
    }

    /// Extend the expiry of an existing credential in place.
    ///
    /// Only the issuing `school` may renew. Revoked credentials cannot
    /// be renewed — a fresh `register_credential` call is required for
    /// re-issuance. This keeps the on-chain history clean.
    ///
    /// Returns `1` on success.
    pub fn renew(
        env: Env,
        school: Address,
        cred_hash: Symbol,
        new_expiry: u64,
    ) -> u32 {
        school.require_auth();

        let key = DataKey::Cred(cred_hash);
        let mut cred: Credential = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| panic!("unknown credential"));

        if cred.school != school {
            panic!("only issuing school can renew");
        }
        if cred.revoked {
            panic!("cannot renew revoked credential");
        }
        if new_expiry <= cred.expires_at {
            panic!("new expiry must be later than current expiry");
        }

        cred.expires_at = new_expiry;
        env.storage().persistent().set(&key, &cred);
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND_TO);

        1
    }
}
