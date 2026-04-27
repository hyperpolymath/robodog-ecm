// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

//! Post-Quantum Cryptographic Protocols for secure autonomous communications.
//!
//! DEFENSIVE USE ONLY — Wassenaar Category 5A2 (Information Security).
//! Open-source publication; may qualify for Wassenaar Note 3 exemption.
//!
//! # Algorithm Suite
//!
//! | Function           | Algorithm             | Fallback       |
//! |--------------------|-----------------------|----------------|
//! | Key encapsulation  | Kyber1024             | X25519         |
//! | Digital signatures | Ed448 + Dilithium5    | SPHINCS+       |
//! | Symmetric encrypt  | AES-256-GCM           | ChaCha20-Poly  |
//! | Hashing            | SHA3-512              | SHAKE256       |
//! | KDF                | HKDF-SHA3-512         | —              |

use pqcrypto_traits::kem::{
    Ciphertext as KemCiphertextTrait, PublicKey as KemPublicKeyTrait,
    SecretKey as KemSecretKeyTrait, SharedSecret as KemSharedSecretTrait,
};
use pqcrypto_traits::sign::{
    PublicKey as SigPublicKeyTrait, SecretKey as SigSecretKeyTrait,
    SignedMessage as SigSignedMessageTrait,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors from cryptographic operations.
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("key generation failed: {0}")]
    KeyGeneration(String),

    #[error("encapsulation failed: {0}")]
    Encapsulation(String),

    #[error("decapsulation failed: {0}")]
    Decapsulation(String),

    #[error("signature verification failed")]
    SignatureVerification,

    #[error("unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),
}

/// Supported key encapsulation mechanisms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KemAlgorithm {
    /// ML-KEM-1024 (formerly Kyber1024) — NIST PQC standard.
    Kyber1024,
}

/// Supported digital signature algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureAlgorithm {
    /// ML-DSA-87 (formerly Dilithium5) — NIST PQC standard.
    Dilithium5,
    /// SLH-DSA (formerly SPHINCS+) — hash-based fallback.
    SphincsPlusSha2256f,
}

/// A KEM keypair (public + secret).
#[derive(Debug, Clone)]
pub struct KemKeypair {
    /// The KEM algorithm used.
    pub algorithm: KemAlgorithm,
    /// Public key bytes.
    pub public_key: Vec<u8>,
    /// Secret key bytes (zeroised on drop in production).
    pub secret_key: Vec<u8>,
}

/// Result of a KEM encapsulation: ciphertext + shared secret.
#[derive(Debug, Clone)]
pub struct KemEncapsulation {
    /// Ciphertext to send to the secret key holder.
    pub ciphertext: Vec<u8>,
    /// Shared secret derived from the encapsulation.
    pub shared_secret: Vec<u8>,
}

/// Generate a Kyber1024 keypair.
///
/// # Errors
///
/// Returns `CryptoError::KeyGeneration` if the underlying library fails.
pub fn kem_keygen(algorithm: KemAlgorithm) -> Result<KemKeypair, CryptoError> {
    match algorithm {
        KemAlgorithm::Kyber1024 => {
            let (pk, sk) = pqcrypto_kyber::kyber1024::keypair();
            Ok(KemKeypair {
                algorithm,
                public_key: pk.as_bytes().to_vec(),
                secret_key: sk.as_bytes().to_vec(),
            })
        }
    }
}

/// Encapsulate a shared secret using a KEM public key.
///
/// # Errors
///
/// Returns `CryptoError::Encapsulation` on failure.
pub fn kem_encapsulate(public_key: &[u8], algorithm: KemAlgorithm) -> Result<KemEncapsulation, CryptoError> {
    match algorithm {
        KemAlgorithm::Kyber1024 => {
            let pk = pqcrypto_kyber::kyber1024::PublicKey::from_bytes(public_key)
                .map_err(|e| CryptoError::Encapsulation(format!("{e:?}")))?;
            let (ss, ct) = pqcrypto_kyber::kyber1024::encapsulate(&pk);
            Ok(KemEncapsulation {
                ciphertext: ct.as_bytes().to_vec(),
                shared_secret: ss.as_bytes().to_vec(),
            })
        }
    }
}

/// Decapsulate a shared secret using a KEM secret key.
///
/// # Errors
///
/// Returns `CryptoError::Decapsulation` on failure.
pub fn kem_decapsulate(
    ciphertext: &[u8],
    secret_key: &[u8],
    algorithm: KemAlgorithm,
) -> Result<Vec<u8>, CryptoError> {
    match algorithm {
        KemAlgorithm::Kyber1024 => {
            let sk = pqcrypto_kyber::kyber1024::SecretKey::from_bytes(secret_key)
                .map_err(|e| CryptoError::Decapsulation(format!("{e:?}")))?;
            let ct = pqcrypto_kyber::kyber1024::Ciphertext::from_bytes(ciphertext)
                .map_err(|e| CryptoError::Decapsulation(format!("{e:?}")))?;
            let ss = pqcrypto_kyber::kyber1024::decapsulate(&ct, &sk);
            Ok(ss.as_bytes().to_vec())
        }
    }
}

/// A signature keypair (public + secret).
#[derive(Debug, Clone)]
pub struct SignatureKeypair {
    pub algorithm: SignatureAlgorithm,
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
}

/// Generate a signature keypair.
///
/// # Errors
///
/// Returns `CryptoError::KeyGeneration` on failure.
pub fn sig_keygen(algorithm: SignatureAlgorithm) -> Result<SignatureKeypair, CryptoError> {
    match algorithm {
        SignatureAlgorithm::Dilithium5 => {
            let (pk, sk) = pqcrypto_dilithium::dilithium5::keypair();
            Ok(SignatureKeypair {
                algorithm,
                public_key: pk.as_bytes().to_vec(),
                secret_key: sk.as_bytes().to_vec(),
            })
        }
        SignatureAlgorithm::SphincsPlusSha2256f => {
            let (pk, sk) = pqcrypto_sphincsplus::sphincssha2256fsimple::keypair();
            Ok(SignatureKeypair {
                algorithm,
                public_key: pk.as_bytes().to_vec(),
                secret_key: sk.as_bytes().to_vec(),
            })
        }
    }
}

/// Sign a message.
///
/// # Errors
///
/// Returns `CryptoError::KeyGeneration` if the secret key is malformed.
pub fn sign(message: &[u8], secret_key: &[u8], algorithm: SignatureAlgorithm) -> Result<Vec<u8>, CryptoError> {
    match algorithm {
        SignatureAlgorithm::Dilithium5 => {
            let sk = pqcrypto_dilithium::dilithium5::SecretKey::from_bytes(secret_key)
                .map_err(|e| CryptoError::KeyGeneration(format!("{e:?}")))?;
            let sig = pqcrypto_dilithium::dilithium5::sign(message, &sk);
            Ok(sig.as_bytes().to_vec())
        }
        SignatureAlgorithm::SphincsPlusSha2256f => {
            let sk = pqcrypto_sphincsplus::sphincssha2256fsimple::SecretKey::from_bytes(secret_key)
                .map_err(|e| CryptoError::KeyGeneration(format!("{e:?}")))?;
            let sig = pqcrypto_sphincsplus::sphincssha2256fsimple::sign(message, &sk);
            Ok(sig.as_bytes().to_vec())
        }
    }
}

/// Verify a signature.
///
/// # Errors
///
/// Returns `CryptoError::SignatureVerification` if verification fails.
pub fn verify(
    signed_message: &[u8],
    public_key: &[u8],
    algorithm: SignatureAlgorithm,
) -> Result<Vec<u8>, CryptoError> {
    match algorithm {
        SignatureAlgorithm::Dilithium5 => {
            let pk = pqcrypto_dilithium::dilithium5::PublicKey::from_bytes(public_key)
                .map_err(|_| CryptoError::SignatureVerification)?;
            let sig = pqcrypto_dilithium::dilithium5::SignedMessage::from_bytes(signed_message)
                .map_err(|_| CryptoError::SignatureVerification)?;
            pqcrypto_dilithium::dilithium5::open(&sig, &pk)
                .map_err(|_| CryptoError::SignatureVerification)
        }
        SignatureAlgorithm::SphincsPlusSha2256f => {
            let pk = pqcrypto_sphincsplus::sphincssha2256fsimple::PublicKey::from_bytes(public_key)
                .map_err(|_| CryptoError::SignatureVerification)?;
            let sig = pqcrypto_sphincsplus::sphincssha2256fsimple::SignedMessage::from_bytes(signed_message)
                .map_err(|_| CryptoError::SignatureVerification)?;
            pqcrypto_sphincsplus::sphincssha2256fsimple::open(&sig, &pk)
                .map_err(|_| CryptoError::SignatureVerification)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kyber1024_round_trip() {
        let keypair = kem_keygen(KemAlgorithm::Kyber1024).unwrap();
        let encap = kem_encapsulate(&keypair.public_key, KemAlgorithm::Kyber1024).unwrap();
        let decap = kem_decapsulate(&encap.ciphertext, &keypair.secret_key, KemAlgorithm::Kyber1024).unwrap();
        assert_eq!(encap.shared_secret, decap);
    }

    #[test]
    fn dilithium5_sign_verify() {
        let keypair = sig_keygen(SignatureAlgorithm::Dilithium5).unwrap();
        let message = b"DEFENSIVE USE ONLY";
        let signed = sign(message, &keypair.secret_key, SignatureAlgorithm::Dilithium5).unwrap();
        let opened = verify(&signed, &keypair.public_key, SignatureAlgorithm::Dilithium5).unwrap();
        assert_eq!(opened, message);
    }

    #[test]
    fn sphincs_sign_verify() {
        let keypair = sig_keygen(SignatureAlgorithm::SphincsPlusSha2256f).unwrap();
        let message = b"post-quantum fallback";
        let signed = sign(message, &keypair.secret_key, SignatureAlgorithm::SphincsPlusSha2256f).unwrap();
        let opened = verify(&signed, &keypair.public_key, SignatureAlgorithm::SphincsPlusSha2256f).unwrap();
        assert_eq!(opened, message);
    }
}
