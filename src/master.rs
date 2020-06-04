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
    define_HashmapE, define_HashmapAugE,
    bintree::{BinTree, BinTreeType},
    blocks::{BlockIdExt, ExtBlkRef},
    config_params::ConfigParams,
    error::BlockError,
    hashmapaug::{Augmentable, HashmapAugType, TraverseNextStep},
    inbound_messages::InMsg,
    shard::{ShardIdent},
    signature::CryptoSignaturePair,
    types::{CurrencyCollection, InRefValue, ChildCell},
    validators::ValidatorInfo,
    Serializable, Deserializable, MaybeSerialize, MaybeDeserialize,
};
use std::fmt;
use ton_types::{
    error, fail, Result,
    AccountId, UInt256,
    Cell, IBitstring, SliceData, BuilderData, HashmapE, HashmapType, hm_label,
};


/*
_ (HashmapE 32 ^(BinTree ShardDescr)) = ShardHashes;
_ (HashmapAugE 96 ShardFeeCreated ShardFeeCreated) = ShardFees;

masterchain_block_extra#cca5
  key_block:(## 1)
  shard_hashes:ShardHashes
  shard_fees:ShardFees
  ^[ prev_blk_signatures:(HashmapE 16 CryptoSignaturePair)
     recover_create_msg:(Maybe ^InMsg)
     mint_msg:(Maybe ^InMsg) ]
  config:key_block?ConfigParams
= McBlockExtra;
*/
define_HashmapE!{ShardHashes, 32, InRefValue<BinTree<ShardDescr>>}
define_HashmapE!{CryptoSignatures, 16, CryptoSignaturePair}
define_HashmapAugE!{ShardFees, 96, ShardIdentFull, ShardFeeCreated, ShardFeeCreated}

#[derive(Clone, Debug, Default)]
pub struct ShardIdentFull {
    pub workchain_id: i32,
    pub prefix: u64, // with terminated bit!
}

impl ShardIdentFull {
    pub fn to_hex_string(&self) -> String {
        format!("{}:{:016X}", self.workchain_id, self.prefix)
    }
}

impl Serializable for ShardIdentFull {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.workchain_id.write_to(cell)?;
        self.prefix.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for ShardIdentFull {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.workchain_id.read_from(cell)?;
        self.prefix.read_from(cell)?;
        Ok(())
    }
}

impl ShardHashes {
    pub fn iterate_shards<F>(&self, mut func: F) -> Result<bool>
    where F: FnMut(ShardIdent, ShardDescr) -> Result<bool> {
        self.iterate_with_keys(|wc_id: i32, InRefValue(shardes_tree)| {
            shardes_tree.iterate(|prefix, shard_descr| {
                let shard_ident = ShardIdent::with_prefix_slice(wc_id, prefix)?;
                func(shard_ident, shard_descr)
            })
        })
    }
    pub fn has_workchain(&self, workchain_id: i32) -> Result<bool> {
        self.get_as_slice(&workchain_id).map(|result| result.is_some())
    }
    pub fn find_shard(&self, shard: &ShardIdent) -> Result<Option<McShardRecord>> {
        if let Some(InRefValue(bintree)) = self.get(&shard.workchain_id())? {
            let shard_id = shard.shard_key(false);
            if let Some((key, descr)) = bintree.find(shard_id)? {
                let shard = ShardIdent::with_prefix_slice(shard.workchain_id(), key)?;
                return Ok(Some(McShardRecord::new(shard, descr)))
            }
        }
        Ok(None)
    }
    pub fn get_shard(&self, shard: &ShardIdent) -> Result<Option<McShardRecord>> {
        if let Some(InRefValue(bintree)) = self.get(&shard.workchain_id())? {
            let shard_id = shard.shard_key(false);
            if let Some(descr) = bintree.get(shard_id)? {
                return Ok(Some(McShardRecord::new(shard.clone(), descr)))
            }
        }
        Ok(None)
    }
    pub fn get_neighbours(&self, shard: &ShardIdent) -> Result<Vec<McShardRecord>> {
        let mut vec = Vec::new();
        if let Some(InRefValue(bintree)) = self.get(&shard.workchain_id())? {
            bintree.iterate(|prefix, shard_descr| {
                let shard_ident = ShardIdent::with_prefix_slice(shard.workchain_id(), prefix)?;
                if shard.is_neighbor_for(&shard_ident) {
                    vec.push(McShardRecord::new(shard_ident, shard_descr));
                }
                Ok(true)
            })?;
        }
        Ok(vec)
    }
}

impl ShardHashes {
    pub fn dump(&self, heading: &str) {
        println!("dumping shard records for: {}", heading);
        self.iterate_with_keys(|workchain_id: i32, InRefValue(bintree)| {
            println!("workchain: {}", workchain_id);
            bintree.iterate(|prefix, descr| {
                let shard = ShardIdent::with_prefix_slice(workchain_id, prefix.clone().into())?;
                println!("shard: {}", shard);
                println!("seq_no: {}", descr.seq_no);
                println!("prefix: {}", prefix);
                Ok(true)
            })
        }).unwrap();
    }
}

#[derive(Default, Debug)]
pub struct McShardRecord {
    pub shard: ShardIdent,
    pub descr: ShardDescr,
    pub blk_id: BlockIdExt,
}

impl McShardRecord {
    pub fn new(shard: ShardIdent, descr: ShardDescr) -> Self {
        let blk_id = BlockIdExt::with_params(shard, descr.seq_no, descr.root_hash.clone(), descr.file_hash.clone());
        Self { shard, descr, blk_id }
    }

    pub fn shard(&self) -> &ShardIdent {
        &self.shard
    }

    pub fn descr(&self) -> &ShardDescr {
        &self.descr
    }

    pub fn blk_id(&self) -> &BlockIdExt {
        &self.blk_id
    }

    pub fn basic_info_equal(&self, other: &Self, compare_fees: bool, compare_reg_seqno: bool) -> bool {
        self.blk_id == other.blk_id
            && self.descr.start_lt == other.descr.start_lt
            && self.descr.end_lt == other.descr.end_lt
            && (!compare_reg_seqno || self.descr.reg_mc_seqno == other.descr.reg_mc_seqno)
            && self.descr.gen_utime == other.descr.gen_utime
            && self.descr.min_ref_mc_seqno == other.descr.min_ref_mc_seqno
            && self.descr.before_split == other.descr.before_split
            && self.descr.want_split == other.descr.want_split
            && self.descr.want_merge == other.descr.want_merge
            && (!compare_fees
                || (self.descr.fees_collected == other.descr.fees_collected
                    && self.descr.funds_created == other.descr.funds_created))
    }
}

/*
masterchain_block_extra#cca5
  key_block:(## 1)
  shard_hashes:ShardHashes
  shard_fees:ShardFees
  ^[ prev_blk_signatures:(HashmapE 16 CryptoSignaturePair)
     recover_create_msg:(Maybe ^InMsg)
     mint_msg:(Maybe ^InMsg) ]
  config:key_block?ConfigParams
= McBlockExtra;
*/
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct McBlockExtra {
    key_block: bool,
    hashes: ShardHashes, // workchain_id of ShardIdent from all blocks
    fees: ShardFees,
    prev_blk_signatures: CryptoSignatures,
    recover_create_msg: Option<ChildCell<InMsg>>,
    mint_msg: Option<ChildCell<InMsg>>,
    config: Option<ConfigParams>
}

pub fn shard_ident_to_u64(shard: &[u8]) -> u64 {
    let mut shard_key = [0; 8];
    let len = std::cmp::min(shard.len(), 8);
    shard_key[..len].copy_from_slice(&shard[..len]);
    u64::from_be_bytes(shard_key)
}

impl McBlockExtra {
    /// Adds new workchain
    pub fn add_workchain(&mut self, workchain_id: i32, descr: &ShardDescr, fee: &CurrencyCollection) -> Result<ShardIdent> {
        let shards = BinTree::with_item(descr);
        self.hashes.set(&workchain_id, &InRefValue(shards))?;

        let ident = ShardIdent::with_workchain_id(workchain_id)?;

        let fee = ShardFeeCreated::with_fee(fee.clone());
        self.fees.set_serialized(ident.full_key()?, &fee.write_to_new_cell()?.into(), &fee)?;
        Ok(ident)
    }
    /// Split Shard
    pub fn split_shard(&mut self, ident: &mut ShardIdent, descr: &ShardDescr, _fee: &CurrencyCollection) -> Result<()> {
        // TODO fee?
        let shards = match self.hashes.get(&ident.workchain_id())? {
            Some(InRefValue(mut shards)) => {
                shards.split(ident.shard_key(false), descr)?;
                shards
            }
            None => {
                BinTree::with_item(descr)
            }
        };
        self.hashes.set(&ident.workchain_id(), &InRefValue(shards))?;
        Ok(())
    }

    ///
    /// Get all fees for blockchain
    /// 
    pub fn total_fee(&self) -> &CurrencyCollection {
        &self.fees.root_extra().fees
    }
    
    // ///
    // /// Set fee value for selected shard
    // /// 
    // pub fn set_shard_fee(&mut self, _shard_ident: &ShardIdent, _shard_fee: &CurrencyCollection)
    // -> Option<ExceptionCode> {
    //     unimplemented!()
    //     // let shard_key = ident.shard_key(false);
    //     // if let Some(shards) = self.fees.get(&ident.workchain_id())? {
    //     //     shards.set_extra(shard_key, fee);
    //     // } else if shard_key.is_empty() {
    //     //     let shards = BinTreeAug::with_extra(fee);
    //     //     self.fees.insert(ident.workchain_id(), shards);
    //     // } else {
    //     //     return err_opt!(ExceptionCode::Other);
    //     // }
    //     // None
    // }

    ///
    /// Get total fees for shard
    /// 
    pub fn fee(&self, ident: &ShardIdent) -> Result<Option<CurrencyCollection>> {
        Ok(match self.fees.get_serialized(ident.full_key()?)? {
            Some(shards) => Some(shards.fees),
            None => None
        })
    }

    pub fn is_key_block(&self) -> bool { self.config.is_some() }

    pub fn hashes(&self) -> &ShardHashes { &self.hashes }
    pub fn hashes_mut(&mut self) -> &mut ShardHashes { &mut self.hashes }

    pub fn shards(&self) -> &ShardHashes { &self.hashes }
    pub fn shards_mut(&mut self) -> &mut ShardHashes { &mut self.hashes }

    pub fn fees(&self) -> &ShardFees { &self.fees }
    pub fn fees_mut(&mut self) -> &mut ShardFees { &mut self.fees }

    pub fn prev_blk_signatures(&self) -> &CryptoSignatures { &self.prev_blk_signatures }
    pub fn prev_blk_signatures_mut(&mut self) -> &mut CryptoSignatures { &mut self.prev_blk_signatures }

    pub fn config(&self) -> Option<&ConfigParams> { self.config.as_ref() }
    pub fn config_mut(&mut self) -> &mut Option<ConfigParams> { &mut self.config }

    pub fn read_recover_create_msg(&self) -> Result<Option<InMsg>> {
        self.recover_create_msg.as_ref().map(|mr| mr.read_struct()).transpose()
    }
    pub fn write_recover_create_msg(&mut self, value: Option<&InMsg>) -> Result<()> {
        self.recover_create_msg = value.map(|v| ChildCell::with_struct(v)).transpose()?;
        Ok(())
    }
    pub fn recover_create_msg_cell(&self) -> Option<&Cell> {
        self.recover_create_msg.as_ref().map(|mr| mr.cell())
    }

    pub fn read_mint_msg(&self) -> Result<Option<InMsg>> {
        self.mint_msg.as_ref().map(|mr| mr.read_struct()).transpose()
    }
    pub fn write_mint_msg(&mut self, value: Option<&InMsg>) -> Result<()> {
        self.mint_msg = value.map(|v| ChildCell::with_struct(v)).transpose()?;
        Ok(())
    }
    pub fn mint_msg_cell(&self) -> Option<&Cell> {
        self.mint_msg.as_ref().map(|mr| mr.cell())
    }
}

const MC_BLOCK_EXTRA_TAG : u16 = 0xCCA5;

impl Deserializable for McBlockExtra {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_u16()?;
        if tag != MC_BLOCK_EXTRA_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag.into(),
                    s: "McBlockExtra".to_string()
                }
            )
        }
        let key_block = cell.get_next_bit()?;
        self.hashes.read_from(cell)?;
        self.fees.read_from(cell)?;

        let ref mut cell1 = cell.checked_drain_reference()?.into();
        self.prev_blk_signatures.read_from(cell1)?;
        
        self.recover_create_msg = if cell1.get_next_bit()? {
            Some(ChildCell::construct_from(&mut cell1.checked_drain_reference()?.into())?)
        } else {
            None
        };
        
        self.mint_msg = if cell1.get_next_bit()? {
            Some(ChildCell::construct_from(&mut cell1.checked_drain_reference()?.into())?)
        } else {
            None
        };

        self.config = if key_block {
            Some(ConfigParams::construct_from(cell)?)
        } else {
            None
        };

        Ok(())
    }
}

impl Serializable for McBlockExtra {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u16(MC_BLOCK_EXTRA_TAG)?;
        self.config.is_some().write_to(cell)?;
        self.hashes.write_to(cell)?;
        self.fees.write_to(cell)?;

        let mut cell1 = BuilderData::new();
        self.prev_blk_signatures.write_to(&mut cell1)?;
        if let Some(msg) = self.recover_create_msg.as_ref() {
            cell1.append_bit_one()?;
            cell1.append_reference(msg.write_to_new_cell()?);
        } else {
            cell1.append_bit_zero()?;
        }
        
        if let Some(msg) = self.mint_msg.as_ref() {
            cell1.append_bit_one()?;
            cell1.append_reference(msg.write_to_new_cell()?);
        } else {
            cell1.append_bit_zero()?;
        }
        
        cell.append_reference(cell1);

        if let Some(config) = &self.config {
            config.write_to(cell)?;
        }

        Ok(())
    }
}

// _ key:Bool max_end_lt:uint64 = KeyMaxLt;
#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct KeyMaxLt {
    pub key: bool,
    pub max_end_lt: u64
}

impl Deserializable for KeyMaxLt {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        self.key.read_from(slice)?;
        self.max_end_lt.read_from(slice)?;
        Ok(())
    }
}

impl Serializable for KeyMaxLt {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.key.write_to(cell)?;
        self.max_end_lt.write_to(cell)?;
        Ok(())
    }
}

impl Augmentable for KeyMaxLt {
    fn calc(&mut self, other: &Self) -> Result<()> {
        if other.key {
            self.key = true
        }
        if self.max_end_lt < other.max_end_lt {
            self.max_end_lt = other.max_end_lt
        }
        Ok(())
    }
}

// _ key:Bool blk_ref:ExtBlkRef = KeyExtBlkRef;
#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct KeyExtBlkRef {
    key: bool,
    blk_ref: ExtBlkRef
}

impl KeyExtBlkRef {
    pub fn key(&self) -> bool {
        self.key
    }
    pub fn blk_ref(&self) -> &ExtBlkRef {
        &self.blk_ref
    }
}

impl Deserializable for KeyExtBlkRef {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        self.key.read_from(slice)?;
        self.blk_ref.read_from(slice)?;
        Ok(())
    }
}

impl Serializable for KeyExtBlkRef {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.key.write_to(cell)?;
        self.blk_ref.write_to(cell)?;
        Ok(())
    }
}

// _ (HashmapAugE 32 KeyExtBlkRef KeyMaxLt) = OldMcBlocksInfo;
// key - seq_no
define_HashmapAugE!(OldMcBlocksInfo, 32, u32, KeyExtBlkRef, KeyMaxLt);

impl OldMcBlocksInfo {

    // returns key block with max block.seqno and block.seqno <= req_seqno
    pub fn get_prev_key_block(&self, req_seqno: u32) -> Result<Option<ExtBlkRef>> {
        let found = self.traverse(|key_prefix, key_prefix_len, aug, value_opt| {
            if !aug.key {
                // no key blocks in subtree, skip
                return Ok(TraverseNextStep::Stop);
            }

            let x = Self::build_key_part(key_prefix, key_prefix_len)?;
            let d = 32 - key_prefix_len;
            if d == 0 {
                return if x <= req_seqno {
                    let value = value_opt.ok_or_else(|| error!(BlockError::InvalidData(
                        "OldMcBlocksInfo's node with max key length doesn't have value".to_string()
                    )))?;
                    Ok(TraverseNextStep::End(value))
                } else {
                    Ok(TraverseNextStep::Stop)
                }
            }
            let y = req_seqno >> (d - 1);
            if y < 2 * x {
                // (x << d) > req_seqno <=> x > (req_seqno >> d) = (y >> 1) <=> 2 * x > y
                return Ok(TraverseNextStep::Stop);  // all nodes in subtree have block.seqno > req_seqno => skip
            }
            return if y == 2 * x {
                Ok(TraverseNextStep::VisitZero) // visit only left ("0")
            } else {
                Ok(TraverseNextStep::VisitOneZero) // visit right, then left ("1" then "0")
            }
        })?;

        if let Some(id) = found {
            debug_assert!(id.blk_ref.seq_no <= req_seqno);
            debug_assert!(id.key);
            Ok(Some(id.blk_ref))
        } else {
            Ok(None)
        }
    }

    // returns key block with min block.seqno and block.seqno >= req_seqno
    pub fn get_next_key_block(&self, req_seqno: u32) -> Result<Option<ExtBlkRef>> {
        let found = self.traverse(|key_prefix, key_prefix_len, aug, value_opt| {
            if !aug.key {
                // no key blocks in subtree, skip
                return Ok(TraverseNextStep::Stop);
            }

            let x = Self::build_key_part(key_prefix, key_prefix_len)?;
            let d = 32 - key_prefix_len;
            if d == 0 {
                return if x >= req_seqno {
                    let value = value_opt.ok_or_else(|| error!(BlockError::InvalidData(
                        "OldMcBlocksInfo's node with max key length doesn't have value".to_string()
                    )))?;
                    Ok(TraverseNextStep::End(value))
                } else {
                    Ok(TraverseNextStep::Stop)
                }
            }
            let y = req_seqno >> (d - 1);
            if y > 2 * x + 1 {
                // ((x + 1) << d) <= req_seqno <=> (x+1) <= (req_seqno >> d) = (y >> 1) <=> 2*x+2 <= y <=> y > 2*x+1
                return Ok(TraverseNextStep::Stop);  // all nodes in subtree have block.seqno < req_seqno => skip
            }
            return if y == 2 * x + 1 {
                Ok(TraverseNextStep::VisitOne) // visit only right ("1")
            } else {
                Ok(TraverseNextStep::VisitZeroOne) // visit left, then right ("0" then "1")
            }
        })?;

        if let Some(id) = found {
            debug_assert!(id.blk_ref.seq_no >= req_seqno);
            debug_assert!(id.key);
            Ok(Some(id.blk_ref))
        } else {
            Ok(None)
        }
    }

    fn build_key_part(key_prefix: &[u8], key_prefix_len: usize) -> Result<u32> {
        if key_prefix_len > 32 {
            error!(BlockError::InvalidData("key_prefix_len > 32".to_string()));
        }
        let mut key_buf = [0_u8; 4];
        key_buf[..key_prefix.len()].copy_from_slice(key_prefix);
        Ok(
            u32::from_be_bytes(key_buf) >> (32 - key_prefix_len)
        )
    }
}

// _ fees:CurrencyCollection create:CurrencyCollection = ShardFeeCreated;
#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct ShardFeeCreated {
    pub fees: CurrencyCollection,
    pub create: CurrencyCollection,
}

impl ShardFeeCreated {
    pub fn with_fee(fees: CurrencyCollection) -> Self {
        Self {
            fees,
            create: CurrencyCollection::default(),
        }
    }
}

impl Augmentable for ShardFeeCreated {
    fn calc(&mut self, other: &Self) -> Result<()> {
        self.fees.calc(&other.fees)?;
        self.create.calc(&other.create)
    }
}

impl Deserializable for ShardFeeCreated {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.fees.read_from(cell)?;
        self.create.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ShardFeeCreated {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.fees.write_to(cell)?;
        self.create.write_to(cell)?;
        Ok(())
    }
}
fn umulnexps32(_x: u64, _k: u32, _trunc: bool) -> u64 {
    unimplemented!("https://www.notion.so/tonlabs/Port-NegExpInt64Table-from-TNode-75664c4ccf794ee9b2485f3ce7945b5d")
}

/// counters#_ last_updated:uint32 total:uint64 cnt2048:uint64 cnt65536:uint64 = Counters;
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Counters {
    valid: bool,
    last_updated: u32,
    total: u64,
    cnt2048: u64,
    cnt65536: u64,
}

impl Counters {
    pub fn validate(&mut self) -> bool {
        if !self.is_valid() {
            return false
        }
        if self.total == 0 {
            if (self.cnt2048 | self.cnt65536) != 0 {
                return self.invalidate()
            }
        } else if self.last_updated == 0 {
            return self.invalidate()
        }
        return true;
    }
    pub fn is_valid(&self) -> bool {
        self.valid
    }
    pub fn invalidate(&mut self) -> bool {
        self.valid = false;
        self.valid
    }
    pub fn is_zero(&self) -> bool {
        self.total == 0
    }
    pub fn almost_zero(&self) -> bool {
        (self.cnt2048 | self.cnt65536) <= 1
    }
    pub fn almost_equals(&self, other: &Self) -> bool {
        self.last_updated == other.last_updated
            && self.total == other.total
            && self.cnt2048 <= other.cnt2048 + 1
            && other.cnt2048 <= self.cnt2048 + 1
            && self.cnt65536 <= other.cnt65536 + 1
            && other.cnt65536 <= self.cnt65536 + 1
    }
    pub fn modified_since(&self, utime: u32) -> bool {
        self.last_updated >= utime
    }
    pub fn increase_by(&mut self, count: u64, now: u32) -> bool {
        if !self.validate() {
            return false
        }
        let scaled = count << 32;
        if self.total == 0 {
            self.last_updated = now;
            self.total = count;
            self.cnt2048 = scaled;
            self.cnt65536 = scaled;
            return true
        }
        if count > !self.total || self.cnt2048 > !scaled || self.cnt65536 > !scaled {
            return false /* invalidate() */  // overflow
        }
        let dt = now.checked_sub(self.last_updated).unwrap_or_default();
        if dt != 0 {
            // more precise version of cnt2048 = llround(cnt2048 * exp(-dt / 2048.));
            // (rounding error has absolute value < 1)
            self.cnt2048 = if dt >= 48 * 2048 {0} else {
                umulnexps32(self.cnt2048, dt << 5, false)
            };
            // more precise version of cnt65536 = llround(cnt65536 * exp(-dt / 65536.));
            // (rounding error has absolute value < 1)
            self.cnt65536 = umulnexps32(self.cnt65536, dt, false);
        }
        self.total += count;
        self.cnt2048 += scaled;
        self.cnt65536 += scaled;
        self.last_updated = now;
        true
    }
    pub fn total(&self) -> u64 {
        self.total
    }
    pub fn last_updated(&self) -> u32 {
        self.last_updated
    }
    pub fn cnt2048(&self) -> u64 {
        self.cnt2048
    }
    pub fn cnt65536(&self) -> u64 {
        self.cnt65536
    }
}

impl Deserializable for Counters {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        self.last_updated.read_from(slice)?;
        self.total.read_from(slice)?;
        self.cnt2048.read_from(slice)?;
        self.cnt65536.read_from(slice)?;
        Ok(())
    }
}

impl Serializable for Counters {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.last_updated.write_to(cell)?;
        self.total.write_to(cell)?;
        self.cnt2048.write_to(cell)?;
        self.cnt65536.write_to(cell)?;
        Ok(())
    }
}

/// creator_info#4 mc_blocks:Counters shard_blocks:Counters = CreatorStats;
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CreatorStats {
    mc_blocks: Counters,
    shard_blocks: Counters,
}

impl CreatorStats {
    pub fn tag() -> u32 {
        0x4
    }

    pub fn tag_len_bits() -> usize {
        4
    }

    pub fn mc_blocks(&self) -> &Counters {
        &self.mc_blocks
    }

    pub fn shard_blocks(&self) -> &Counters {
        &self.shard_blocks
    }
}

impl Deserializable for CreatorStats {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        let tag = slice.get_next_int(Self::tag_len_bits())? as u32;
        if tag != Self::tag() {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag.into(),
                    s: "CreatorStats".to_string()
                }
            )
        }

        self.mc_blocks.read_from(slice)?;
        self.shard_blocks.read_from(slice)?;
        Ok(())
    }
}

impl Serializable for CreatorStats {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_bits(Self::tag() as usize, Self::tag_len_bits())?;

        self.mc_blocks.write_to(cell)?;
        self.shard_blocks.write_to(cell)?;
        Ok(())
    }
}

define_HashmapE!{BlockCounters, 256, CreatorStats}

/// block_create_stats#17 counters:(HashmapE 256 CreatorStats) = BlockCreateStats;
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BlockCreateStats {
    pub counters: BlockCounters,
}

impl BlockCreateStats {
    pub fn tag() -> u32 {
        0x17
    }

    pub fn tag_len_bits() -> usize {
        8
    }
}

impl Deserializable for BlockCreateStats {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        let tag = slice.get_next_int(Self::tag_len_bits())? as u32;
        if tag != Self::tag() {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag.into(),
                    s: "BlockCreateStats".to_string()
                }
            )
        }

        self.counters.read_from(slice)?;
        Ok(())
    }
}

impl Serializable for BlockCreateStats {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_bits(Self::tag() as usize, Self::tag_len_bits())?;

        self.counters.write_to(cell)?;
        Ok(())
    }
}

/*
masterchain_state_extra#cc26
  shard_hashes:ShardHashes
  config:ConfigParams
  ^[ flags:(## 16) { flags <= 1 }
     validator_info:ValidatorInfo
     prev_blocks:OldMcBlocksInfo
     after_key_block:Bool
     last_key_block:(Maybe ExtBlkRef)
     block_create_stats:(flags . 0)?BlockCreateStats ]
  global_balance:CurrencyCollection
= McStateExtra;
*/
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct McStateExtra {
    pub hashes: ShardHashes, // TODO: correct name shards
    pub config: ConfigParams,
    pub validator_info: ValidatorInfo,
    pub prev_blocks: OldMcBlocksInfo,
    pub after_key_block: bool,
    pub last_key_block: Option<ExtBlkRef>,
    pub block_create_stats: Option<BlockCreateStats>,
    pub global_balance: CurrencyCollection,
}

impl McStateExtra {
    pub fn tag() -> u16 {
        0xcc26
    }

    /// Adds new workchain
    pub fn add_workchain(&mut self, workchain_id: i32, descr: &ShardDescr) -> Result<ShardIdent> {
        let shards = BinTree::with_item(descr);
        self.hashes.set(&workchain_id, &InRefValue(shards))?;
        Ok(ShardIdent::with_workchain_id(workchain_id)?)
    }

    /// Split Shard
    pub fn split_shard(&mut self, ident: &ShardIdent, descr: &ShardDescr) -> Result<()> {
        let shards = match self.hashes.get(&ident.workchain_id())? {
            Some(InRefValue(mut shards)) => {
                shards.split(ident.shard_key(false), descr)?;
                shards
            }
            None => BinTree::with_item(descr)
        };
        self.hashes.set(&ident.workchain_id(), &InRefValue(shards))?;
        Ok(())
    }

    ///
    /// Get Shard last seq_no
    ///
    pub fn shard_seq_no(&self, ident: &ShardIdent) -> Result<Option<u32>> {
        Ok(match self.hashes.get(&ident.workchain_id())? {
            Some(InRefValue(shards)) => shards.get(ident.shard_key(false))?.map(|s| s.seq_no),
            None => None
        })
    }

    ///
    /// Get shard last Logical Time
    /// 
    pub fn shard_lt(&self, ident: &ShardIdent) -> Result<Option<u64>> {
        Ok(match self.hashes.get(&ident.workchain_id())? {
            Some(InRefValue(shards)) => shards.get(ident.shard_key(false))?.map(|s| s.start_lt),
            None => None
        })
    }

    ///
    /// Get shard last block hash
    /// 
    pub fn shard_hash(&self, ident: &ShardIdent) -> Result<Option<UInt256>> {
        Ok(match self.hashes.get(&ident.workchain_id())? {
            Some(InRefValue(shards)) => shards.get(ident.shard_key(false))?.map(|s| s.root_hash),
            None => None
        })
    }

    pub fn shards(&self) -> &ShardHashes {
        &self.hashes
    }
}

impl Deserializable for McStateExtra {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_u16()?;
        if tag != Self::tag() {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag.into(),
                    s: "McStateExtra".to_string()
                }
            )
        }
        self.hashes.read_from(cell)?;
        self.config.read_from(cell)?;

        let ref mut cell1 = cell.checked_drain_reference()?.into();
        let mut flags = 0u16;
        flags.read_from(cell1)?;
        if flags > 1 {
            fail!(
                BlockError::InvalidData(
                    format!("Invalid flags value ({}). Must be <= 1.", flags)
                )
            )
        }
        self.validator_info.read_from(cell1)?;
        self.prev_blocks.read_from(cell1)?;
        self.after_key_block.read_from(cell1)?;
        self.last_key_block = ExtBlkRef::read_maybe_from(cell1)?;
        if flags & 1 == 0 {
            self.block_create_stats = None;
        } else {
            let mut block_create_stats = BlockCreateStats::default();
            block_create_stats.read_from(cell1)?;
            self.block_create_stats = Some(block_create_stats);
        }
        self.global_balance.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for McStateExtra {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u16(Self::tag())?;
        self.hashes.write_to(cell)?;
        self.config.write_to(cell)?;

        let mut cell1 = BuilderData::new();
        let flags = if self.block_create_stats.is_some() {
            1u16
        } else {
            0u16
        };
        flags.write_to(&mut cell1)?;
        self.validator_info.write_to(&mut cell1)?;
        self.prev_blocks.write_to(&mut cell1)?;
        self.after_key_block.write_to(&mut cell1)?;
        self.last_key_block.write_maybe_to(&mut cell1)?;
        if let Some(ref block_create_stats) = self.block_create_stats {
            block_create_stats.write_to(&mut cell1)?;
        }
        cell.append_reference(cell1);
        self.global_balance.write_to(cell)?;
        Ok(())
    }
}

/*
fsm_none$0

fsm_split$10 
    split_utime: uint32 
    interval: uint32
= FutureSplitMerge;

fsm_merge$11 
    merge_utime: uint32 
    interval: uint32 
= FutureSplitMerge;
*/
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FutureSplitMerge {
    None,
    Split {
        split_utime: u32,
        interval: u32,
    },
    Merge {
        merge_utime: u32, 
        interval: u32,
    }
}

impl Default for FutureSplitMerge {
    fn default() -> Self {
        FutureSplitMerge::None
    }
}

impl Deserializable for FutureSplitMerge {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        if !slice.get_next_bit()? {
            *self = FutureSplitMerge::None;
        } else if !slice.get_next_bit()? {
            *self = FutureSplitMerge::Split {
                split_utime: slice.get_next_u32()?,
                interval: slice.get_next_u32()?,
            };
        } else {
            *self = FutureSplitMerge::Merge {
                merge_utime: slice.get_next_u32()?,
                interval: slice.get_next_u32()?,
            };
        }
        Ok(())
    }
}

impl Serializable for FutureSplitMerge {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        match self {
            FutureSplitMerge::None => {
                cell.append_bit_zero()?;
            },
            FutureSplitMerge::Split { split_utime, interval } => {
                cell.append_bit_one()?;
                cell.append_bit_zero()?;
                split_utime.write_to(cell)?;
                interval.write_to(cell)?;
            },
            FutureSplitMerge::Merge { merge_utime, interval } => {
                cell.append_bit_one()?;
                cell.append_bit_one()?;
                merge_utime.write_to(cell)?;
                interval.write_to(cell)?;
            }
        }
        Ok(())
    }
}

/*
shard_descr$_ seq_no:uint32 lt:uint64 hash:uint256
split_merge_at:FutureSplitMerge = ShardDescr;

shard_descr#b seq_no:uint32 reg_mc_seqno:uint32
  start_lt:uint64 end_lt:uint64
  root_hash:bits256 file_hash:bits256 
  before_split:Bool before_merge:Bool
  want_split:Bool want_merge:Bool
  nx_cc_updated:Bool flags:(## 3) { flags = 0 }
  next_catchain_seqno:uint32 next_validator_shard:uint64
  min_ref_mc_seqno:uint32 gen_utime:uint32
  split_merge_at:FutureSplitMerge
  fees_collected:CurrencyCollection
  funds_created:CurrencyCollection = ShardDescr;
*/
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ShardDescr {
    pub seq_no: u32,
    pub reg_mc_seqno: u32,
    pub start_lt: u64,
    pub end_lt: u64,
    pub root_hash: UInt256,
    pub file_hash: UInt256 ,
    pub before_split: bool,
    pub before_merge: bool,
    pub want_split: bool,
    pub want_merge: bool,
    pub nx_cc_updated: bool,
    pub flags: u8,
    pub next_catchain_seqno: u32,
    pub next_validator_shard: u64,
    pub min_ref_mc_seqno: u32,
    pub gen_utime: u32,
    pub split_merge_at: FutureSplitMerge,
    pub fees_collected: CurrencyCollection,
    pub funds_created: CurrencyCollection,
}

impl ShardDescr {

    /// Constructs ShardDescr as slice with its params
    pub fn with_params(seq_no: u32, start_lt: u64, end_lt: u64, root_hash: UInt256, split_merge_at: FutureSplitMerge) -> Self {
        
        ShardDescr {
            seq_no, 
            reg_mc_seqno: 0,
            start_lt,
            end_lt,
            root_hash: root_hash,
            file_hash: UInt256::from([0;32]),
            before_split: false,
            before_merge: false,
            want_split: false,
            want_merge: false,
            nx_cc_updated: false, 
            flags: 0,
            next_catchain_seqno: 0, 
            next_validator_shard: 0,
            min_ref_mc_seqno: 0,
            gen_utime: 0,
            split_merge_at,
            fees_collected: CurrencyCollection::default(),
            funds_created: CurrencyCollection::default(),
        }
    }
    pub fn fsm_equal(&self, other: &Self) -> bool {
        self.is_fsm_none() == other.is_fsm_none()
            && self.is_fsm_split() == other.is_fsm_split()
            && self.is_fsm_merge() == other.is_fsm_merge()
    }
    pub fn is_fsm_merge(&self) -> bool {
        match self.split_merge_at {
            FutureSplitMerge::Merge{merge_utime: _, interval: _} => true,
            _ => false
        }
    }
    pub fn is_fsm_split(&self) -> bool {
        match self.split_merge_at {
            FutureSplitMerge::Split{split_utime: _, interval: _} => true,
            _ => false
        }
    }
    pub fn is_fsm_none(&self) -> bool {
        match self.split_merge_at {
            FutureSplitMerge::None => true,
            _ => false
        }
    }
    pub fn fsm_utime(&self) -> u32 {
        match self.split_merge_at {
            FutureSplitMerge::Split{split_utime, interval: _} => split_utime,
            FutureSplitMerge::Merge{merge_utime, interval: _} => merge_utime,
            _ => 0
        }
    }
    pub fn fsm_utime_end(&self) -> u32 {
        match self.split_merge_at {
            FutureSplitMerge::Split{split_utime, interval} => split_utime + interval,
            FutureSplitMerge::Merge{merge_utime, interval} => merge_utime + interval,
            _ => 0
        }
    }
}

const SHARD_IDENT_TAG_A: u8 = 0xa; // 4 bit
const SHARD_IDENT_TAG_B: u8 = 0xb; // 4 bit
const SHARD_IDENT_TAG_LEN: usize = 4;

impl Deserializable for ShardDescr {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        let tag = slice.get_next_int(SHARD_IDENT_TAG_LEN)? as u8;
        if tag != SHARD_IDENT_TAG_A && tag != SHARD_IDENT_TAG_B {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: "ShardDescr".to_string()
                } 
            )
        }

        self.seq_no.read_from(slice)?;
        self.reg_mc_seqno.read_from(slice)?;
        self.start_lt.read_from(slice)?;
        self.end_lt.read_from(slice)?;
        self.root_hash.read_from(slice)?;
        self.file_hash.read_from(slice)?;
        let mut flags: u8 = 0;
        flags.read_from(slice)?;
        self.before_split = (flags >> 7) & 1 == 1;
        self.before_merge = (flags >> 6) & 1 == 1;
        self.want_split = (flags >> 5) & 1 == 1;
        self.want_merge = (flags >> 4) & 1 == 1;
        self.nx_cc_updated = (flags >> 3) & 1 == 1;

        self.next_catchain_seqno.read_from(slice)?;
        self.next_validator_shard.read_from(slice)?;
        self.min_ref_mc_seqno.read_from(slice)?;
        self.gen_utime.read_from(slice)?;
        self.split_merge_at.read_from(slice)?;
        if tag == SHARD_IDENT_TAG_B {
            self.fees_collected.read_from(slice)?;
            self.funds_created.read_from(slice)?;
        } else {
            let mut slice1 = slice.checked_drain_reference()?.into();
            self.fees_collected.read_from(&mut slice1)?;
            self.funds_created.read_from(&mut slice1)?;
        }
        Ok(())
    }
}

impl Serializable for ShardDescr {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_bits(SHARD_IDENT_TAG_A as usize, SHARD_IDENT_TAG_LEN)?;

        self.seq_no.write_to(cell)?;
        self.reg_mc_seqno.write_to(cell)?;
        self.start_lt.write_to(cell)?;
        self.end_lt.write_to(cell)?;
        self.root_hash.write_to(cell)?;
        self.file_hash.write_to(cell)?;

        let mut flags: u8 = 0;
        if self.before_split {
            flags |= 1 << 7
        }
        if self.before_merge {
            flags |= 1 << 6;
        }
        if self.want_split {
            flags |= 1 << 5;
        }
        if self.want_merge {
            flags |= 1 << 4;
        }
        if self.nx_cc_updated {
            flags |= 1 << 3;
        }
        flags |= self.flags & 0x7;
        
        flags.write_to(cell)?;

        self.next_catchain_seqno.write_to(cell)?;
        self.next_validator_shard.write_to(cell)?;
        self.min_ref_mc_seqno.write_to(cell)?;
        self.gen_utime.write_to(cell)?;
        self.split_merge_at.write_to(cell)?;

        let mut child = BuilderData::new();
        self.fees_collected.write_to(&mut child)?;
        self.funds_created.write_to(&mut child)?;
        cell.append_reference(child);

        Ok(())
    }
}

/*
master_info$_ master:ExtBlkRef = BlkMasterInfo;
*/
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlkMasterInfo {
    pub master: ExtBlkRef
}

impl Default for BlkMasterInfo {
    fn default() -> Self {
        BlkMasterInfo { master: ExtBlkRef::default() }
    }
}

impl Deserializable for BlkMasterInfo {
     fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.master.read_from(cell)        
    }
}

impl Serializable for BlkMasterInfo {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.master.write_to(cell)
    }
}


define_HashmapE!(Publishers, 256, ());
/*
shared_lib_descr$00 lib:^Cell publishers:(Hashmap 256 True) = LibDescr;
*/
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LibDescr {
    lib: Cell,
    publishers: Publishers
}

impl LibDescr {
    pub fn from_lib_data_by_publisher(lib: Cell, publisher: AccountId) -> Self {
        let mut publishers = Publishers::default();
        publishers.set(&publisher, &()).unwrap();
        Self {
            lib,
            publishers
        }
    }
    pub fn add_publisher(&mut self, publisher: AccountId) {
        self.publishers.set(&publisher, &()).unwrap();
    }
    pub fn publishers(&self) -> &Publishers {
        &self.publishers
    }
    pub fn lib(&self) -> &Cell {
        &self.lib
    }
    pub fn is_public_library(&self, _key: &UInt256) -> bool {
        unimplemented!()
    }
}

impl Deserializable for LibDescr {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        self.lib.read_from(slice)?;
        self.publishers.read_hashmap_root(slice)?;
        Ok(())
    }
}

impl Serializable for LibDescr {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        if self.publishers.is_empty() {
            fail!(BlockError::InvalidData("self.publishers is empty".to_string()))
        }
        self.lib.write_to(cell)?;
        self.publishers.write_hashmap_root(cell)?;
        Ok(())
    }
}
