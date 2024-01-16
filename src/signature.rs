/*
* Copyright (C) 2019-2024 EverX. All Rights Reserved.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use crate::{
    blocks::BlockIdExt, define_HashmapE, error::BlockError, validators::ValidatorBaseInfo,
    validators::ValidatorDescr, Deserializable, Serializable,
};
use std::{collections::HashMap, str::FromStr, sync::Arc, convert::TryInto};
use ton_types::{
    error, fail, BuilderData, Cell, Ed25519KeyOption, HashmapE, HashmapType, IBitstring, KeyOption,
    Result, SliceData, UInt256,
    ED25519_PUBLIC_KEY_LENGTH, ED25519_SIGNATURE_LENGTH
};

/*
ed25519_signature#5 R:bits256 s:bits256 = CryptoSignature;
*/
///
/// CryptoSignature
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CryptoSignature([u8; ED25519_SIGNATURE_LENGTH]);

impl Default for CryptoSignature {
    fn default() -> Self {
        Self([0; ED25519_SIGNATURE_LENGTH])
    }
}

impl CryptoSignature {
    pub fn with_bytes(bytes: [u8; ED25519_SIGNATURE_LENGTH]) -> Self {
        Self(bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Self::with_bytes(bytes.try_into()?))
    }

    #[deprecated]
    pub fn from_r_s(r: &[u8], s: &[u8]) -> Result<Self>
    {
        if r.len() != ED25519_SIGNATURE_LENGTH / 2 {
            fail!(BlockError::InvalidArg("`r` has invalid size".to_string()))
        }
        if s.len() != ED25519_SIGNATURE_LENGTH / 2 {
            fail!(BlockError::InvalidArg("`s` has invalid size".to_string()))
        }
        let mut signature = Self::default();
        signature.0[..ED25519_SIGNATURE_LENGTH / 2].copy_from_slice(r);
        signature.0[ED25519_SIGNATURE_LENGTH / 2..].copy_from_slice(s);
        Ok(signature)
    }

    pub fn from_r_s_str(r: &str, s: &str) -> Result<Self> {
        let mut signature = Self::default();
        hex::decode_to_slice(r, &mut signature.0[..ED25519_SIGNATURE_LENGTH / 2]).map_err(|err| {
            BlockError::InvalidData(format!("error parsing `r` hex string: {}", err))
        })?;
        hex::decode_to_slice(s, &mut signature.0[ED25519_SIGNATURE_LENGTH / 2..]).map_err(|err| {
            BlockError::InvalidData(format!("error parsing `s` hex string: {}", err))
        })?;
        Ok(signature)
    }

    pub fn with_r_s(r: &[u8; 32], s: &[u8; 32]) -> Self {
        let mut signature = Self::default();
        signature.0[..ED25519_SIGNATURE_LENGTH / 2].copy_from_slice(r);
        signature.0[ED25519_SIGNATURE_LENGTH / 2..].copy_from_slice(s);
        signature
    }

    #[deprecated]
    pub fn to_r_s_bytes(&self) -> (&[u8], &[u8]) { self.as_r_s_bytes() }

    pub fn as_r_s_bytes(&self) -> (&[u8], &[u8]) {
        let r_bytes = &self.0[..ED25519_SIGNATURE_LENGTH / 2];
        let s_bytes = &self.0[ED25519_SIGNATURE_LENGTH / 2..];
        (r_bytes, s_bytes)
    }

    pub fn as_bytes(&self) -> &[u8; ED25519_SIGNATURE_LENGTH] {
        &self.0
    }

    #[deprecated]
    pub fn to_bytes(&self) -> [u8; ED25519_SIGNATURE_LENGTH] { *self.as_bytes() }
}

impl FromStr for CryptoSignature {
    type Err = ton_types::Error;
    fn from_str(s: &str) -> Result<Self> {
        let mut signature = Self::default();
        hex::decode_to_slice(s, &mut signature.0)?;
        Ok(signature)
    }
}

const CRYPTO_SIGNATURE_TAG: u8 = 0x5;

impl Serializable for CryptoSignature {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_bits(CRYPTO_SIGNATURE_TAG as usize, 4)?;
        cell.append_raw(&self.0, ED25519_SIGNATURE_LENGTH * 8)?;
        Ok(())
    }
}

impl Deserializable for CryptoSignature {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        let tag = slice.get_next_int(4)? as u8;
        if tag != CRYPTO_SIGNATURE_TAG {
            fail!(Self::invalid_tag(tag as u32))
        }
        slice.get_next_bytes_to_slice(&mut self.0)
    }
}

/*
sig_pair$_ node_id_short:bits256 sign:CryptoSignature = CryptoSignaturePair;
*/
///
/// CryptoSignaturePair
///
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct CryptoSignaturePair {
    pub node_id_short: UInt256,
    pub sign: CryptoSignature,
}

impl CryptoSignaturePair {
    pub fn new() -> Self { Self::default() }

    pub fn with_params(node_id_short: UInt256, sign: CryptoSignature) -> Self {
        CryptoSignaturePair {
            node_id_short,
            sign,
        }
    }
}

impl Serializable for CryptoSignaturePair {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.node_id_short.write_to(cell)?;
        self.sign.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for CryptoSignaturePair {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.node_id_short.read_from(cell)?;
        self.sign.read_from(cell)?;
        Ok(())
    }
}

/*
ed25519_pubkey#8e81278a pubkey:bits256 = SigPubKey;
*/

///
/// SigPubKey
///
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SigPubKey([u8; ED25519_PUBLIC_KEY_LENGTH]);

const SIG_PUB_KEY_TAG: u32 = 0x8e81278a;

impl SigPubKey {
    pub fn with_bytes(bytes: [u8; ED25519_PUBLIC_KEY_LENGTH]) -> Self {
        Self(bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Self(bytes.as_ref().try_into()?))
    }

    pub fn key_bytes(&self) -> &[u8; ED25519_PUBLIC_KEY_LENGTH] { self.as_bytes() }
    pub fn as_bytes(&self) -> &[u8; ED25519_PUBLIC_KEY_LENGTH] {
        &self.0
    }

    pub fn pub_key(&self) -> Arc<dyn KeyOption> {
        Ed25519KeyOption::from_public_key(&self.0)
    }

    pub fn key_id(&self) -> [u8; 32] {
        *self.pub_key().id().data()
    }

    // be careful here - we recreate public key object everytime
    pub fn verify_signature(&self, data: &[u8], signature: &CryptoSignature) -> bool {
        self.pub_key().verify(data, signature.as_bytes()).is_ok()
    }

    pub fn as_slice(&self) -> &[u8; 32] {
        &self.0
    }
}

impl PartialEq<UInt256> for SigPubKey {
    fn eq(&self, other: &UInt256) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl FromStr for SigPubKey {
    type Err = ton_types::Error;
    fn from_str(s: &str) -> Result<Self> {
        let mut public_key = Self::default();
        hex::decode_to_slice(s, &mut public_key.0)?;
        Ok(public_key)
    }
}

impl AsRef<[u8]> for SigPubKey {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl Serializable for SigPubKey {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u32(SIG_PUB_KEY_TAG)?;
        cell.append_raw(&self.0, ED25519_PUBLIC_KEY_LENGTH * 8)?;
        Ok(())
    }
}

impl Deserializable for SigPubKey {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        let tag = slice.get_next_u32()?;
        if tag != SIG_PUB_KEY_TAG {
            fail!(Self::invalid_tag(tag))
        }
        let mut public_key = Self::default();
        slice.get_next_bytes_to_slice(&mut public_key.0)?;
        Ok(public_key)
    }
}

/*
  PROOFS
*/

/*
block_signatures_pure#_
    sig_count:uint32
    sig_weight:uint64
    signatures:(HashmapE 16 CryptoSignaturePair)
= BlockSignaturesPure;
*/

define_HashmapE! {CryptoSignaturePairDict, 16, CryptoSignaturePair}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BlockSignaturesPure {
    sig_count: u32,
    sig_weight: u64,
    signatures: CryptoSignaturePairDict,
}

impl BlockSignaturesPure {
    pub fn new() -> Self { Self::default() }
    /// New instance of BlockSignaturesPure
    pub fn with_weight(sig_weight: u64) -> Self {
        Self {
            sig_count: 0,
            sig_weight,
            signatures: CryptoSignaturePairDict::default(),
        }
    }

    /// Get count of signatures
    pub fn count(&self) -> u32 {
        self.sig_count
    }

    /// Get weight
    pub fn weight(&self) -> u64 {
        self.sig_weight
    }

    pub fn set_weight(&mut self, weight: u64) {
        self.sig_weight = weight;
    }

    /// Add crypto signature pair to BlockSignaturesPure
    pub fn add_sigpair(&mut self, signature: CryptoSignaturePair) {
        self.signatures
            .set(&(self.sig_count as u16), &signature)
            .unwrap();
        self.sig_count += 1;
    }

    pub fn signatures(&self) -> &HashmapE {
        &self.signatures.0
    }

    pub fn check_signatures(&self, validators_list: &[ValidatorDescr], data: &[u8]) -> Result<u64> {
        // Calc validators short ids
        let mut validators_map = HashMap::new();
        for vd in validators_list {
            validators_map.insert(vd.compute_node_id_short(), vd);
        }

        // Check signatures
        let mut weight = 0;
        self.signatures()
            .iterate_slices(|ref mut _key, ref mut slice| {
                let sign = CryptoSignaturePair::construct_from(slice)?;
                if let Some(vd) = validators_map.get(&sign.node_id_short) {
                    if !vd.verify_signature(data, &sign.sign) {
                        fail!(BlockError::BadSignature)
                    }
                    weight += vd.weight;
                }
                Ok(true)
            })?;
        Ok(weight)
    }
}

impl Serializable for BlockSignaturesPure {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.sig_count.write_to(cell)?;
        self.sig_weight.write_to(cell)?;
        self.signatures.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for BlockSignaturesPure {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.sig_count.read_from(cell)?;
        self.sig_weight.read_from(cell)?;
        self.signatures.read_from(cell)?;
        Ok(())
    }
}

/*
block_signatures#11
    validator_info:ValidatorBaseInfo
    pure_signatures:BlockSignaturesPure
= BlockSignatures;
*/

///
/// BlockSignatures
///
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct BlockSignatures {
    pub validator_info: ValidatorBaseInfo,
    pub pure_signatures: BlockSignaturesPure,
}

impl BlockSignatures {
    /// Create new empty instance of BlockSignatures
    pub fn new() -> Self { Self::default() }

    /// Create new instance of BlockSignatures
    pub fn with_params(
        validator_info: ValidatorBaseInfo,
        pure_signatures: BlockSignaturesPure,
    ) -> Self {
        BlockSignatures {
            validator_info,
            pure_signatures,
        }
    }
}

const BLOCK_SIGNATURES_TAG: u8 = 0x11;

impl Serializable for BlockSignatures {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(BLOCK_SIGNATURES_TAG)?;
        self.validator_info.write_to(cell)?;
        self.pure_signatures.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for BlockSignatures {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_byte()?;
        if tag != BLOCK_SIGNATURES_TAG {
            fail!(Self::invalid_tag(tag as u32))
        }
        self.validator_info.read_from(cell)?;
        self.pure_signatures.read_from(cell)?;
        Ok(())
    }
}

/*
block_proof#c3
    proof_for:BlockIdExt
    root:^Cell
    signatures:(Maybe ^BlockSignatures)
= BlockProof;
*/

///
/// BlockProof
///
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct BlockProof {
    pub proof_for: BlockIdExt,
    pub root: Cell,
    pub signatures: Option<BlockSignatures>,
}

impl BlockProof {
    /// Create new empty instance of BlockProof
    pub fn new() -> Self { Self::default() }

    /// Create new instance of BlockProof
    pub fn with_params(
        proof_for: BlockIdExt,
        root: Cell,
        signatures: Option<BlockSignatures>,
    ) -> Self {
        BlockProof {
            proof_for,
            root,
            signatures,
        }
    }
}

const BLOCK_PROOF_TAG: u8 = 0xC3;

impl Serializable for BlockProof {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(BLOCK_PROOF_TAG)?;
        self.proof_for.write_to(cell)?;
        cell.checked_append_reference(self.root.clone())?;
        if let Some(s) = self.signatures.as_ref() {
            cell.append_bit_one()?;
            cell.checked_append_reference(s.serialize()?)?;
        } else {
            cell.append_bit_zero()?;
        }
        Ok(())
    }
}

impl Deserializable for BlockProof {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_byte()?;
        if tag != BLOCK_PROOF_TAG {
            fail!(Self::invalid_tag(tag as u32))
        }
        self.proof_for.read_from(cell)?;
        self.root = cell.checked_drain_reference()?;
        self.signatures = if cell.get_next_bit()? {
            Some(BlockSignatures::construct_from_reference(cell)?)
        } else {
            None
        };
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/test_signature.rs"]
mod tests;
