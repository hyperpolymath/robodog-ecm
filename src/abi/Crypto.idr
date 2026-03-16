-- SPDX-License-Identifier: PMPL-1.0-or-later
-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
--
-- Crypto.idr — Post-quantum algorithm types with key size proofs.
--
-- DEFENSIVE USE ONLY — Wassenaar Category 5A2.

module Crypto

import Types

%default total

||| Key sizes for Kyber1024 (NIST ML-KEM-1024).
||| These are compile-time constants verified against the specification.
public export
Kyber1024_PK_Size : Nat
Kyber1024_PK_Size = 1568

public export
Kyber1024_SK_Size : Nat
Kyber1024_SK_Size = 3168

public export
Kyber1024_CT_Size : Nat
Kyber1024_CT_Size = 1568

public export
Kyber1024_SS_Size : Nat
Kyber1024_SS_Size = 32

||| Key sizes for Dilithium5 (NIST ML-DSA-87).
public export
Dilithium5_PK_Size : Nat
Dilithium5_PK_Size = 2592

public export
Dilithium5_SK_Size : Nat
Dilithium5_SK_Size = 4896

||| A byte vector with a proven length.
||| Used to carry key material with compile-time size guarantees.
public export
data SizedBytes : Nat -> Type where
  MkSized : (bytes : List Bits8) -> {auto prf : length bytes = n} -> SizedBytes n

||| A Kyber1024 public key with proven correct size.
public export
KyberPublicKey : Type
KyberPublicKey = SizedBytes Kyber1024_PK_Size

||| A Kyber1024 secret key with proven correct size.
public export
KyberSecretKey : Type
KyberSecretKey = SizedBytes Kyber1024_SK_Size

||| A Kyber1024 ciphertext with proven correct size.
public export
KyberCiphertext : Type
KyberCiphertext = SizedBytes Kyber1024_CT_Size

||| A shared secret with proven correct size.
public export
SharedSecret : Type
SharedSecret = SizedBytes Kyber1024_SS_Size

||| A Dilithium5 public key with proven correct size.
public export
DilithiumPublicKey : Type
DilithiumPublicKey = SizedBytes Dilithium5_PK_Size

||| A Dilithium5 secret key with proven correct size.
public export
DilithiumSecretKey : Type
DilithiumSecretKey = SizedBytes Dilithium5_SK_Size

||| Hashing algorithm selection.
public export
data HashAlgorithm : Type where
  SHA3_512  : HashAlgorithm
  SHAKE256  : HashAlgorithm
  BLAKE3    : HashAlgorithm
