/*
* Copyright 2018-2019 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use super::*;

/*
ed25519_signature#5 R:bits256 s:bits256 = CryptoSignature; 
*/

///
/// CryptoSignature
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct CryptoSignature {
    r: UInt256,
    s: UInt256,
}

impl CryptoSignature {
    pub fn new() -> Self {
        CryptoSignature {
            r: UInt256::default(),
            s: UInt256::default(),
        }
    }

    pub fn with_params(
        r: UInt256, 
        s: UInt256) -> Self 
    {
        CryptoSignature {
            r,
            s,
        }
    }
}

impl Serializable for CryptoSignature {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        self.r.write_to(cell)?;
        self.s.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for CryptoSignature {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        self.r.read_from(cell)?;
        self.s.read_from(cell)?;
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
    node_id_short: UInt256,
    sign: CryptoSignature,
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
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        self.node_id_short.write_to(cell)?;
        self.sign.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for CryptoSignaturePair {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
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
pub struct SigPubKey {
    pubkey: UInt256,
}

const SIG_PUB_KEY_TAG: u32 = 0x8e81278a;

impl SigPubKey {
    pub fn new() -> Self {
        SigPubKey {
            pubkey: UInt256::default(),
        }
    }

    pub fn with_params(
        pubkey: UInt256) -> Self 
    {
        SigPubKey {
            pubkey,
        }
    }
}

impl Serializable for SigPubKey {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_u32(SIG_PUB_KEY_TAG)?;
        self.pubkey.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for SigPubKey {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        let tag = cell.get_next_u32()?;
        if tag != SIG_PUB_KEY_TAG {
            bail!(BlockErrorKind::InvalidConstructorTag {
                t: tag,
                s: "SigPubKey".into()
            })
        }
        self.pubkey.read_from(cell)?;
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
    pub fn get_count(&self) -> u32 {
        self.sig_count
    }

    /// Get weight
    pub fn get_weight(&self) -> u64 {
        self.sig_weight
    }

    /// Add crypto signature pair to BlockSignaturesPure
    pub fn add_sigpair(&mut self, signature: CryptoSignaturePair) {
        self.sig_count += 1;
        let key = (self.sig_count as u16).write_to_new_cell().unwrap();
        self.signatures.set(key.into(), &signature.write_to_new_cell().unwrap().into()).unwrap();
    }
}

impl Serializable for BlockSignaturesPure {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        self.sig_count.write_to(cell)?; 
        self.sig_weight.write_to(cell)?; 
        self.signatures.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for BlockSignaturesPure {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
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
        pure_signatures: BlockSignaturesPure) -> Self {
        BlockSignatures {
            validator_info,
            pure_signatures,
        }
    }
}

const BLOCK_SIGNATURES_TAG: u8 = 0x11;

impl Serializable for BlockSignatures {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_u8(BLOCK_SIGNATURES_TAG)?;
        self.validator_info.write_to(cell)?; 
        self.pure_signatures.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for BlockSignatures {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        let tag = cell.get_next_byte()?;
        if tag != BLOCK_SIGNATURES_TAG {
            bail!(BlockErrorKind::InvalidConstructorTag {
                t: tag as u32,
                s: "BlockSignatures".into()
            })
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
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_u8(BLOCK_PROOF_TAG)?;
        self.proof_for.write_to(cell)?;
        cell.append_reference(BuilderData::from(&self.root));
        self.signatures.write_maybe_to(cell)?;
        Ok(())
    }
}

impl Deserializable for BlockProof {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        let tag = cell.get_next_byte()?;
        if tag != BLOCK_PROOF_TAG {
            bail!(BlockErrorKind::InvalidConstructorTag {
                t: tag as u32,
                s: "BlockProof".into()
            })
        }
        self.proof_for.read_from(cell)?; 
        self.root = cell.checked_drain_reference()?.clone();
        self.signatures = BlockSignatures::read_maybe_from(cell)?;
        Ok(())
    }
}

