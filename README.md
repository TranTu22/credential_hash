# credential_hash

## Project Title
credential_hash

## Project Description
Academic credentials — diplomas, certificates, micro-credentials, professional licenses — are routinely forged, lost, or hard to verify across borders. `credential_hash` is a Soroban smart contract that lets an issuing school anchor a tamper-evident fingerprint of every credential it hands out on the Stellar blockchain. The full document never leaves the holder's possession (it can live on IPFS, the school's portal, or the graduate's own wallet); only the hash, the recipient's Stellar address, and an expiration timestamp are written on-chain. Any verifier — an employer, consulate, another university — can later call a single function to confirm that a presented hash exists, which school issued it, when it was issued, and whether it has been revoked or has expired, all without trusting a third party or paying for a background-check service.

## Project Vision
A world where every academic credential can be verified by anyone, anywhere, in a single blockchain read, with no central authority in the loop. `credential_hash` aims to become a public, permissionless registry of academic integrity for the Stellar ecosystem — one where every diploma, certificate, and badge carries a permanent, privacy-preserving anchor that outlives the issuing institution, survives institutional change, and is censorship-resistant. In the long run, the same primitive can be reused for professional licenses, KYC attestations, and any other "off-chain document + on-chain proof of issuance" use case.

## Key Features
- **Hash-only anchoring** — the actual diploma is never stored on-chain; only a short hash identifier is anchored, protecting student privacy and keeping ledger costs low.
- **Issuer authentication** — only the school that owns the credential can register, revoke, or renew it. Every state-mutating call requires the issuer's Soroban `require_auth()` signature.
- **Four-state verification in one call** — `verify(cred_hash)` returns `0 = unknown, 1 = valid, 2 = expired, 3 = revoked`, giving verifiers a single, deterministic answer.
- **Holder portfolio lookups** — `list_credentials(holder)` returns the number of credentials anchored for any recipient, powering degree-aggregator and portfolio UIs.
- **In-place renewal** — `renew()` extends an existing credential's expiry without minting a new on-chain record, saving fees on long-lived credentials (e.g. professional licenses).
- **Tamper-evident revocation** — every revocation records the issuer's signature, a free-text reason, and a ledger timestamp, giving employers a clear audit trail when a credential has been withdrawn.
- **Privacy by default** — no personally identifying information (name, photo, grades) lives on-chain, so the contract is GDPR-friendly by design.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** identity dApp — see `contracts/credential_hash/src/lib.rs` for the full credential_hash business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** `<CDOPW2TSSRZWCZKQAISYCFTEXQKM63LT7R5RWUBWZ36CQOY7UWYS6ZSF>`
- **Explorer template:** `https://stellar.expert/explorer/testnet/tx/6d0caeb8deac18ca21b1d21d7535ffb6589ea951e1e9f31cc2f1b0bd571c276c`
- **Screenshot of deployed contract on Stellar Expert:**
  `_(Screenshot of the contract page on Stellar Expert will appear here after deploy.)_`


## Future Scope
- **Multi-signature issuance** — require N-of-M school administrators to co-sign every credential registration, so a single compromised key cannot mint fake diplomas.
- **Batch registration** — let a registrar register an entire graduating class in a single transaction, cutting per-credential fees for bulk events.
- **On-chain metadata pointer** — store an IPFS / content-addressed CID alongside the hash, so verifiers with the holder's consent can fetch the full document without trusting a server.
- **Schema versioning** — add a `version: u32` field so old `Credential` records keep verifying while the contract grows to support Open Badges v3, W3C Verifiable Credentials, and other formats.
- **Cross-chain bridge** — mirror credential hashes to a Stellar Classic account-memo stream for explorers and wallets that don't yet support Soroban invocations.
- **Freighter-powered frontend** — a React + Freighter dApp where schools register, graduates view their anchored portfolio, and employers verify a hash in a single click.
- **KYC / professional-license reuse** — generalize the same primitive beyond academia to any "off-chain document + on-chain proof of issuance" use case.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `credential_hash` (identity)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
