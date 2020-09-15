/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
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
    blocks::{BlockIdExt},
    error::BlockError,
    validators::ValidatorBaseInfo,
    Serializable, Deserializable,
    validators::ValidatorDescr
};
use ed25519::signature::{Signature, Verifier};
use std::{
    io::{Cursor, Write},
    collections::HashMap
};
use ton_types::{
    error, fail, Result,
    UInt256,
    BuilderData, Cell, IBitstring, SliceData, HashmapE, HashmapType
};

/*
ed25519_signature#5 R:bits256 s:bits256 = CryptoSignature; 
*/

///
/// CryptoSignature
/// 
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CryptoSignature(ed25519::Signature);

impl CryptoSignature {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self>
    {
        Ok(Self(ed25519::Signature::from_bytes(bytes)?))
    }

    pub fn from_str(string: &str) -> Result<Self> {
        let buf = hex::decode(string).map_err(
            |err| BlockError::InvalidData(format!("error parsing hex string: {}", err))
        )?;
        Self::from_bytes(&buf)
    }

    pub fn from_r_s(r: &[u8], s: &[u8]) -> Result<Self>
    {
        if r.len() != ed25519_dalek::SIGNATURE_LENGTH / 2 {
            fail!(BlockError::InvalidArg("`r` has invalid size".to_string()))
        }
        if s.len() != ed25519_dalek::SIGNATURE_LENGTH / 2 {
            fail!(BlockError::InvalidArg("`s` has invalid size".to_string()))
        }
        let mut sign = [0_u8; ed25519_dalek::SIGNATURE_LENGTH];
        {
            let mut cur = Cursor::new(&mut sign[..]);
            cur.write(r).unwrap();
            cur.write(s).unwrap();
        }
        Ok(Self(ed25519::Signature::from_bytes(&sign[..])?))
    }

    pub fn from_r_s_str(r: &str, s: &str) -> Result<Self> {
        let mut bytes = [0; ed25519_dalek::SIGNATURE_LENGTH];
        hex::decode_to_slice(r, &mut bytes[..ed25519_dalek::SIGNATURE_LENGTH / 2]).map_err(
            |err| BlockError::InvalidData(format!("error parsing `r` hex string: {}", err))
        )?;
        hex::decode_to_slice(s, &mut bytes[ed25519_dalek::SIGNATURE_LENGTH / 2..]).map_err(
            |err| BlockError::InvalidData(format!("error parsing `s` hex string: {}", err))
        )?;
        Self::from_bytes(&bytes)
    }

    pub fn to_bytes(&self) -> [u8; ed25519_dalek::SIGNATURE_LENGTH] {
        self.0.to_bytes()
    }

    pub fn to_r_s_bytes(&self) -> ([u8; ed25519_dalek::SIGNATURE_LENGTH / 2], [u8; ed25519_dalek::SIGNATURE_LENGTH / 2]) {
        let mut r_bytes = [0_u8; ed25519_dalek::SIGNATURE_LENGTH / 2];
        let mut s_bytes = [0_u8; ed25519_dalek::SIGNATURE_LENGTH / 2];
        let bytes = self.0.to_bytes();
        r_bytes.copy_from_slice(&bytes[..ed25519_dalek::SIGNATURE_LENGTH / 2]);
        s_bytes.copy_from_slice(&bytes[ed25519_dalek::SIGNATURE_LENGTH / 2..]);
        (r_bytes, s_bytes)
    }

    pub fn signature(&self) -> &ed25519::Signature {
        &self.0
    }
}

impl Default for CryptoSignature {
    fn default() -> Self {
        Self(ed25519::Signature::from_bytes(&[0; ed25519_dalek::SIGNATURE_LENGTH]).unwrap())
    }
}

const CRYPTO_SIGNATURE_TAG: u8 = 0x5;

impl Serializable for CryptoSignature {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_bits(CRYPTO_SIGNATURE_TAG as usize, 4)?;
        let bytes = self.to_bytes();
        cell.append_raw(&bytes, bytes.len() * 8)?;
        Ok(())
    }
}

impl Deserializable for CryptoSignature {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_bits(4)?;
        if tag[0] != 5 << 4 {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag[0] as u32,
                    s: "CryptoSignature".to_string()
                }
            )
        }
        let buf = cell.get_next_bits(ed25519_dalek::SIGNATURE_LENGTH * 8)?;
        self.0 = ed25519::Signature::from_bytes(&buf)?;
        Ok(())
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
    pub fn new() -> Self {
        CryptoSignaturePair {
            node_id_short: UInt256::default(),
            sign: CryptoSignature::default(),
        }
    }

    pub fn with_params(
        node_id_short: UInt256, 
        sign: CryptoSignature) -> Self 
    {
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
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct SigPubKey(ed25519_dalek::PublicKey);

const SIG_PUB_KEY_TAG: u32 = 0x8e81278a;

impl SigPubKey {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self>
    {
        Ok(SigPubKey(ed25519_dalek::PublicKey::from_bytes(bytes)?))
    }

    pub fn from_str(string: &str) -> Result<Self> {
        let key_buf = hex::decode(string).map_err(
            |err| BlockError::InvalidData(format!("error parsing hex string: {}", err))
        )?;
        Self::from_bytes(&key_buf)
    }

    pub fn key(&self) -> &ed25519_dalek::PublicKey {
        &self.0
    }

    pub fn key_bytes(&self) -> &[u8; ed25519_dalek::PUBLIC_KEY_LENGTH] {
        self.0.as_bytes()
    }

    pub fn verify_signature(&self, data: &[u8], signature: &CryptoSignature) -> bool {
        self.0.verify(data, signature.signature()).is_ok()
    }
}

impl Serializable for SigPubKey {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u32(SIG_PUB_KEY_TAG)?;
        cell.append_raw(self.key_bytes(), self.key_bytes().len() * 8)?;
        Ok(())
    }
}

impl Deserializable for SigPubKey {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_u32()?;
        if tag != SIG_PUB_KEY_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag,
                    s: "SigPubKey".to_string()
                } 
            )
        }
        let key_buf = cell.get_next_bits(ed25519_dalek::PUBLIC_KEY_LENGTH * 8)?;
        self.0 = ed25519_dalek::PublicKey::from_bytes(&key_buf)?;
        Ok(())
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockSignaturesPure {
    sig_count: u32, 
    sig_weight: u64, 
    signatures: HashmapE,
}

impl Default for BlockSignaturesPure {
    fn default() -> Self {
        BlockSignaturesPure {
            sig_count: 0,
            sig_weight: 0,
            signatures: HashmapE::with_bit_len(16),
        }
    }
}

impl BlockSignaturesPure {
    /// New empty instance of BlockSignaturesPure
    pub fn new() -> Self {
        BlockSignaturesPure::default()
    }
    
    /// New instance of BlockSignaturesPure
    pub fn with_weight(weight: u64) -> Self {
        BlockSignaturesPure {
            sig_count: 0,
            sig_weight: weight,
            signatures: HashmapE::with_bit_len(16),
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
        self.sig_count += 1;
        let key = (self.sig_count as u16).write_to_new_cell().unwrap();
        self.signatures.set(key.into(), &signature.write_to_new_cell().unwrap().into()).unwrap();
    }

    pub fn signatures(&self) -> &HashmapE {
        &self.signatures
    }

    pub fn check_signatures(&self, validators_list: Vec<ValidatorDescr>, data: &[u8]) -> Result<u64> {
        // Calc validators short ids
        let mut validators_map = HashMap::new();
        for vd in validators_list {
            validators_map.insert(vd.compute_node_id_short(), vd);
        };

        // Check signatures
        let mut weight = 0;
        self.signatures().iterate_slices(|ref mut _key, ref mut slice| {
            let sign = CryptoSignaturePair::construct_from(slice)?;
            if let Some(vd) = validators_map.get(&sign.node_id_short) {
                if !vd.public_key.verify_signature(data, &sign.sign) {
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
    pub pure_signatures: BlockSignaturesPure
}

impl BlockSignatures {
    /// Create new empty instance of BlockSignatures
    pub fn new() -> Self {
        BlockSignatures {
            validator_info: ValidatorBaseInfo::default(),
            pure_signatures: BlockSignaturesPure::default(),
        }
    }

    /// Create new instance of BlockSignatures
    pub fn with_params(
        validator_info: ValidatorBaseInfo,
        pure_signatures: BlockSignaturesPure
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
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: "BlockSignatures".to_string()
                }
            )
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
    pub fn new() -> Self {
        BlockProof {
            proof_for: BlockIdExt::default(),
            root: BuilderData::default().into(),
            signatures: None,
        }
    }

    /// Create new instance of BlockProof
    pub fn with_params(    
        proof_for: BlockIdExt,
        root: Cell,
        signatures: Option<BlockSignatures>
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
            cell.checked_append_reference(s.write_to_new_cell()?.into())?;
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
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32, 
                    s: "BlockProof".to_string()
                }
            )
        }
        self.proof_for.read_from(cell)?; 
        self.root = cell.checked_drain_reference()?.clone();
        self.signatures = if cell.get_next_bit()? {
            Some(BlockSignatures::construct_from(&mut cell.checked_drain_reference()?.into())?)
        } else {
            None
        };
        Ok(())
    }
}
