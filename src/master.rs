/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
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
use super::hashmapaug::Augmentable;
use {IBitstring, SliceData, BuilderData};
use dictionary::HashmapE;
use {AccountId, UInt256};


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
define_HashmapAugE!{ShardFees, 96, ShardFeeCreated, ShardFeeCreated}

impl ShardHashes {
    pub fn iterate_shardes<F>(&self, func: &mut F) -> Result<bool>
    where F: FnMut(ShardIdent, ShardDescr) -> Result<bool> {
        self.iterate_with_keys(&mut |wc_id: i32, shardes_tree| {
            shardes_tree.0.iterate(&mut |prefix, shard_descr| {
                let shard_ident = ShardIdent::with_prefix_slice(wc_id, prefix)?;
                func(shard_ident, shard_descr)
            })
        })
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct McBlockExtra {
    pub hashes: ShardHashes, // workchain_id of ShardIdent from all blocks
    pub fees: ShardFees,
    pub prev_blk_signatures: CryptoSignatures,
    pub recover_create_msg: Option<InRefValue<InMsg>>,
    pub mint_msg: Option<InRefValue<InMsg>>,
    pub config: Option<ConfigParams>
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

        let ident = ShardIdent::with_workchain_id(workchain_id);

        let fee = ShardFeeCreated::with_fee(fee.clone());
        self.fees.0.set(ident.full_key(), &fee.write_to_new_cell()?.into(), &fee)?;
        Ok(ident)
    }
    /// Split Shard
    pub fn split_shard(&mut self, ident: &mut ShardIdent, descr: &ShardDescr, _fee: &CurrencyCollection) -> Result<()> {
        // TODO fee?
        let shards = match self.hashes.get(&ident.workchain_id())? {
            Some(InRefValue(mut shards)) => {
                shards.split(ident.shard_key(), descr)?;
                shards
            }
            None => {
                BinTree::with_item(descr)
            }
        };
        self.hashes.set(&ident.workchain_id(), &InRefValue(shards))?;
        Ok(())
    }

    pub fn hashes(&self) -> &ShardHashes {
        &self.hashes
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
    //     // let shard_key = ident.shard_key();
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
        Ok(match self.fees.get(&ident.workchain_id())? {
            Some(shards) => Some(shards.fees),
            None => None
        })
    }

    /// Get fees
    pub fn fees(&self) -> &ShardFees {
        &self.fees
    }

    pub fn config(&self) -> Option<&ConfigParams> {
        self.config.as_ref()
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
        self.recover_create_msg = InRefValue::<InMsg>::read_maybe_from(cell1)?;
        self.mint_msg = InRefValue::<InMsg>::read_maybe_from(cell1)?;

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
        self.recover_create_msg.write_maybe_to(&mut cell1)?;
        self.mint_msg.write_maybe_to(&mut cell1)?;
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
    key: bool,
    max_end_lt: u64
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
define_HashmapAugE!(OldMcBlocksInfo, 32, KeyExtBlkRef, KeyMaxLt);

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

/// counters#_ last_updated:uint32 total:uint64 cnt2048:uint64 cnt65536:uint64 = Counters;
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Counters {
    last_updated: u32,
    total: u64,
    cnt2048: u64,
    cnt65536: u64,
}

impl Counters {
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

/// block_create_stats#17 counters:(HashmapE 256 CreatorStats) = BlockCreateStats;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockCreateStats {
    counters: HashmapE,
}

impl Default for BlockCreateStats {
    fn default() -> Self {
        Self {
            counters: HashmapE::with_bit_len(256),
        }
    }
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
    hashes: ShardHashes,
    pub config: ConfigParams,
    pub validator_info: ValidatorInfo,
    pub prev_blocks: OldMcBlocksInfo,
    after_key_block: bool,
    pub last_key_block: Option<ExtBlkRef>,
    block_create_stats: Option<BlockCreateStats>,
    global_balance: CurrencyCollection,
}

impl McStateExtra {
    pub fn tag() -> u16 {
        0xcc26
    }

    /// Adds new workchain
    pub fn add_workchain(&mut self, workchain_id: i32, descr: &ShardDescr) -> Result<ShardIdent> {
        let shards = BinTree::with_item(descr);
        self.hashes.set(&workchain_id, &InRefValue(shards))?;
        Ok(ShardIdent::with_workchain_id(workchain_id))
    }

    /// Split Shard
    pub fn split_shard(&mut self, ident: &ShardIdent, descr: &ShardDescr) -> Result<()> {
        let shards = match self.hashes.get(&ident.workchain_id())? {
            Some(InRefValue(mut shards)) => {
                shards.split(ident.shard_key(), descr)?;
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
            Some(InRefValue(shards)) => shards.get(ident.shard_key())?.map(|s| s.seq_no),
            None => None
        })
    }

    ///
    /// Get shard last Logical Time
    /// 
    pub fn shard_lt(&self, ident: &ShardIdent) -> Result<Option<u64>> {
        Ok(match self.hashes.get(&ident.workchain_id())? {
            Some(InRefValue(shards)) => shards.get(ident.shard_key())?.map(|s| s.start_lt),
            None => None
        })
    }

    ///
    /// Get shard last block hash
    /// 
    pub fn shard_hash(&self, ident: &ShardIdent) -> Result<Option<UInt256>> {
        Ok(match self.hashes.get(&ident.workchain_id())? {
            Some(InRefValue(shards)) => shards.get(ident.shard_key())?.map(|s| s.root_hash),
            None => None
        })
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
    pub fn tag() -> u32 {
        0xb
    }

    pub fn tag_len_bits() -> usize {
        4
    }

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
}

impl Deserializable for ShardDescr {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        let tag = slice.get_next_int(Self::tag_len_bits())? as u32;
        if tag != Self::tag() {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag,
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
        self.fees_collected.read_from(slice)?;
        self.funds_created.read_from(slice)?;
        Ok(())
    }
}

impl Serializable for ShardDescr {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_bits(Self::tag() as usize, Self::tag_len_bits())?;

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
        self.fees_collected.write_to(cell)?;
        self.funds_created.write_to(cell)?;
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


/*
shared_lib_descr$00 lib:^Cell publishers:(Hashmap 256 False) = LibDescr;
*/
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LibDescr {
    lib: Cell,
    publishers: HashmapE // publishers:(Hashmap 256 False)
}

impl Default for LibDescr {
    fn default() -> Self {
        Self {
            lib: Cell::default(),
            publishers: HashmapE::with_bit_len(256)
        }
    }
}

impl LibDescr {
    pub fn from_lib_data_by_publisher(lib: Cell, publisher: AccountId) -> Self {
        let mut publishers = HashmapE::with_bit_len(256);
        publishers.set(
            publisher.write_to_new_cell().unwrap().into(),
            &SliceData::default()
        ).unwrap();
        Self {
            lib,
            publishers
        }
    }
    pub fn add_publisher(&mut self, publisher: AccountId) {
        self.publishers.set(
            publisher.write_to_new_cell().unwrap().into(),
            &SliceData::default()
        ).unwrap();
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
