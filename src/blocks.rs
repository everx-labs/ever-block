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

use ton_types::{BuilderData, SliceData};
use {ExceptionCode, UInt256};
use super::*;
use dictionary::HashmapE;
use std::cmp::Ordering;


/*
block_id_ext$_
    shard_id:ShardIdent
    seq_no:uint32
    root_hash:bits256
    file_hash:bits256
= BlockIdExt;
*/
///
/// BlockIdExt
///
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct BlockIdExt {
    shard_id: ShardIdent,
    seq_no: u32,
    root_hash: UInt256,
    file_hash: UInt256,
}

impl BlockIdExt {
    /// New empty instance of BlockIdExt structure
    pub fn new() -> Self {
        BlockIdExt::default()
    }

    // New instance of BlockIdExt structure
    pub fn with_params(
        shard_id: ShardIdent,
        seq_no: u32,
        root_hash: UInt256,
        file_hash: UInt256,
    ) -> Self {
        BlockIdExt {
            shard_id,
            seq_no,
            root_hash,
            file_hash,
        }
    }
}

impl Serializable for BlockIdExt {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        self.shard_id.write_to(cell)?;
        self.seq_no.write_to(cell)?;
        self.root_hash.write_to(cell)?;
        self.file_hash.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for BlockIdExt {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        self.shard_id.read_from(cell)?;
        self.seq_no.read_from(cell)?;
        self.root_hash.read_from(cell)?;
        self.file_hash.read_from(cell)?;
        Ok(())
    }
}

/// Additional struct, used for convenience
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct BlockSeqNoAndShard {
    pub seq_no: u32,
    pub vert_seq_no: u32,
    pub shard_id: ShardIdent,
}

/* Blockchain 5.1.6 (outdated)

TL-B from Lite Client v11:

block_info#9bc7a987 
  
  version:uint32 
  
  not_master:(## 1) 
  after_merge:(## 1)
  before_split:(## 1) 
  after_split:(## 1) 
  want_split:Bool
  want_merge:Bool
  key_block:Bool

  vert_seqno_incr:(## 1)
  flags:(## 8)
  seq_no:# 
  vert_seq_no:# { vert_seq_no >= vert_seqno_incr } 
  { prev_seq_no:# } { ~prev_seq_no + 1 = seq_no } 
  
  shard:ShardIdent 
  gen_utime:uint32
  start_lt:uint64
  end_lt:uint64
  gen_validator_list_hash_short:uint32
  gen_catchain_seqno:uint32
  min_ref_mc_seqno:uint32
  prev_key_block_seqno:uint32
  master_ref:not_master?^BlkMasterInfo 
  prev_ref:^(BlkPrevInfo after_merge)
  prev_vert_ref:vert_seqno_incr?^(BlkPrevInfo 0)
  = BlockInfo;
*/
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockInfo {
    
    pub version: u32,
    
    pub after_merge: bool,
    pub before_split: bool,
    pub after_split: bool,
    pub want_split: bool,
    pub want_merge: bool,
    pub key_block: bool,

    pub vert_seqno_incr: u32,
    pub flags: u8,
    pub seq_no: u32,
    pub vert_seq_no: u32,

    pub shard: ShardIdent,
    pub gen_utime: UnixTime32,
    pub start_lt: u64,
    pub end_lt: u64,
    pub gen_validator_list_hash_short: u32,
    pub gen_catchain_seqno: u32,
    pub min_ref_mc_seqno: u32,
    pub prev_key_block_seqno: u32,

    pub master_ref: Option<BlkMasterInfo>,  // reference
    pub prev_ref: BlkPrevInfo,              // reference, master = after_merge
    pub prev_vert_ref: Option<BlkPrevInfo>, // reference, depends on `vert_seq_no`, master = 0
}

impl Default for BlockInfo {
    fn default() -> Self {
        BlockInfo {
            version: 0,
            after_merge: false,
            before_split: false,
            after_split: false,
            want_split: false,
            want_merge: false,
            key_block: false,
            vert_seqno_incr: 0,
            flags: 0,
            seq_no: 1,
            vert_seq_no: 0,
            shard: ShardIdent::default(),
            gen_utime: UnixTime32::default(),
            start_lt: 0,
            end_lt: 0,
            gen_validator_list_hash_short: 0,
            gen_catchain_seqno: 0,
            min_ref_mc_seqno: 0,
            prev_key_block_seqno: 0,
            master_ref: None,
            prev_ref: BlkPrevInfo::Block{prev: ExtBlkRef::default()},
            prev_vert_ref: None,
        }
    }
}

impl BlockInfo {
    ///
    /// Create new instance BlockInfo with sequence number of block
    ///
    pub fn with_seq_no(
        seq_no: u32,
        prev_ref: BlkPrevInfo,
        vert_seq_no: u32,
        prev_vert_ref: Option<BlkPrevInfo>,
    ) -> Self {
        assert!(seq_no != 0, "seq_no should not be 0");
        assert!(
            (vert_seq_no == 0) ^ prev_vert_ref.is_some(),
            "if vert_sec_no !=0 then vert_prev_ref can't be equal to None"
        );
        assert!(
            prev_vert_ref.is_none() || prev_vert_ref.as_ref().unwrap().is_one_prev(),
            "prev_vert_ref can have only one prev block"
        );

        let mut info = BlockInfo::default();

        info.seq_no = seq_no;

        info.after_merge = !prev_ref.is_one_prev();
        info.prev_ref = prev_ref;

        info.vert_seq_no = vert_seq_no;
        info.prev_vert_ref = prev_vert_ref;

        info
    }

    ///
    /// Create new instance BlockInfo with shard_ident
    /// and sequence number of block
    ///
    pub fn with_shard_ident_and_seq_no(
        shard: ShardIdent,
        seq_no: u32,
        prev_ref: BlkPrevInfo,
        vert_seq_no: u32,
        prev_vert_ref: Option<BlkPrevInfo>,
    ) -> Self {
        let mut info = BlockInfo::with_seq_no(seq_no, prev_ref, vert_seq_no, prev_vert_ref);
        info.shard = shard;
        info
    }

    pub fn seq_no(&self) -> u32 {
        self.seq_no
    }
}

/*
prev_blk_info$_
    prev:ExtBlkRef
    = BlkPrevInfo 0;

prev_blks_info$_
    prev1:^ExtBlkRef 
    prev2:^ExtBlkRef 
    = BlkPrevInfo 1;
*/
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlkPrevInfo {
    Block {
        prev: ExtBlkRef
    },
    Blocks {
        prev1: ChildCell<ExtBlkRef>,
        prev2: ChildCell<ExtBlkRef>
    },
}

impl BlkPrevInfo {

    pub fn default_block() -> Self {
        BlkPrevInfo::Block {
            prev: ExtBlkRef::default()
        }
    }

    pub fn default_blocks() -> Self {
        BlkPrevInfo::Blocks {
            prev1: ChildCell::default(),
            prev2: ChildCell::default(),
        }
    }

    pub fn is_one_prev(&self) -> bool {
        match self {
            BlkPrevInfo::Block{prev: _} => true,
            BlkPrevInfo::Blocks{prev1: _, prev2: _} => false,
        }
    }

    pub fn prev1(&self) -> BlockResult<ExtBlkRef> {
        Ok(
            match self {
                BlkPrevInfo::Block{prev} => prev.clone(),
                BlkPrevInfo::Blocks{prev1, prev2: _} => {
                    prev1.read_struct()?
                },
            }
        )
    }

    pub fn prev2(&self) -> BlockResult<Option<ExtBlkRef>> {
        Ok(
            match self {
                BlkPrevInfo::Block{prev: _} => None,
                BlkPrevInfo::Blocks{prev1: _, prev2} => {
                    Some(prev2.read_struct()?)
                },
            }
        )
    }
}

impl Deserializable for BlkPrevInfo {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        match self {
            BlkPrevInfo::Block{prev} => {
                prev.read_from(cell)?;
            },
            BlkPrevInfo::Blocks{prev1, prev2} => {
                prev1.read_from(&mut cell.checked_drain_reference()?.into())?;
                prev2.read_from(&mut cell.checked_drain_reference()?.into())?;
            },
        }
        Ok(())
    }
}

impl Serializable for BlkPrevInfo {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        match self {
            BlkPrevInfo::Block{prev} => {
                prev.write_to(cell)?;
            }
            BlkPrevInfo::Blocks{prev1, prev2} => {
                cell.append_reference(prev1.write_to_new_cell()?);
                cell.append_reference(prev2.write_to_new_cell()?);
            },
        }
        Ok(())
    }
}

pub type BlockId = UInt256;

/*
unsigned_block info:^BlockInfo value_flow:^ValueFlow
    state_update:^(MERKLE_UPDATE ShardState)
    extra:^BlockExtra = Block;
*/
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Block {
    pub global_id: u32,
    pub info: ChildCell<BlockInfo>,            // reference
    pub value_flow: ChildCell<ValueFlow>,      // reference
    pub state_update: ChildCell<MerkleUpdate>, // reference
    pub extra: ChildCell<BlockExtra>,          // reference
}

impl Block {
    pub fn with_params(
        global_id: u32,
        info: BlockInfo,
        value_flow: ValueFlow,
        state_update: MerkleUpdate,
        extra: BlockExtra,
    ) -> BlockResult<Self> {
        Ok(Block {
            global_id,
            info: ChildCell::with_struct(&info)?,
            value_flow: ChildCell::with_struct(&value_flow)?,
            extra: ChildCell::with_struct(&extra)?,
            state_update: ChildCell::with_struct(&state_update)?,
        })
    }

    pub fn global_id(&self) -> u32 {
        self.global_id
    }

    pub fn set_global_id(&mut self, global_id: u32) {
        self.global_id = global_id
    }

    pub fn read_info(&self) -> BlockResult<BlockInfo> {
        self.info.read_struct()
    }

    pub fn write_info(&mut self, value: &BlockInfo) -> BlockResult<()> {
        self.info.write_struct(value)
    }

    pub fn info_cell(&self) -> &Cell {
        self.info.cell()
    }

    pub fn read_value_flow(&self) -> BlockResult<ValueFlow> {
        self.value_flow.read_struct()
    }

    pub fn write_value_flow(&mut self, value: &ValueFlow) -> BlockResult<()> {
        self.value_flow.write_struct(value)
    }

    pub fn value_flow_cell(&self) -> &Cell {
        self.value_flow.cell()
    }

    pub fn read_state_update(&self) -> BlockResult<MerkleUpdate> {
        self.state_update.read_struct()
    }

    pub fn write_state_update(&mut self, value: &MerkleUpdate) -> BlockResult<()> {
        self.state_update.write_struct(value)
    }

    pub fn state_update_cell(&self) -> &Cell {
        self.state_update.cell()
    }

    pub fn read_extra(&self) -> BlockResult<BlockExtra> {
        self.extra.read_struct()
    }

    pub fn write_extra(&mut self, value: &BlockExtra) -> BlockResult<()> {
        self.extra.write_struct(value)
    }

    pub fn extra_cell(&self) -> &Cell {
        self.extra.cell()
    }
}

impl Ord for Block {
    fn cmp(&self, other: &Block) -> Ordering {
        self.read_info().unwrap().seq_no.cmp(&other.read_info().unwrap().seq_no)
    }
}

impl PartialOrd for Block {
    fn partial_cmp(&self, other: &Block) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// block_extra
//    in_msg_descr:^InMsgDescr
//    out_msg_descr:^OutMsgDescr
//    account_blocks:^ShardAccountBlocks
//    rand_seed:bits256
//    created_by:bits256
//    custom:(Maybe ^McBlockExtra)
//    = BlockExtra;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BlockExtra {
    in_msg_descr: ChildCell<InMsgDescr>,
    out_msg_descr: ChildCell<OutMsgDescr>,
    account_blocks: ChildCell<ShardAccountBlocks>,
    pub rand_seed: UInt256,
    pub created_by: UInt256,
    custom: Option<ChildCell<McBlockExtra>>,
}

impl BlockExtra {
    pub fn new() -> BlockExtra {
        let rand_seed: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();
        BlockExtra {
            in_msg_descr: ChildCell::default(),
            out_msg_descr: ChildCell::default(),
            account_blocks: ChildCell::default(),
            rand_seed: UInt256::from(rand_seed),
            created_by: UInt256::default(), // TODO: Need to fill?
            custom: None,
        }
    }

    pub fn read_in_msg_descr(&self) -> BlockResult<InMsgDescr> {
        self.in_msg_descr.read_struct()
    }

    pub fn write_in_msg_descr(&mut self, value: &InMsgDescr) -> BlockResult<()> {
        self.in_msg_descr.write_struct(value)
    }

    pub fn in_msg_descr_cell(&self) -> &Cell {
        self.in_msg_descr.cell()
    }

    pub fn read_out_msg_descr(&self) -> BlockResult<OutMsgDescr> {
        self.out_msg_descr.read_struct()
    }

    pub fn write_out_msg_descr(&mut self, value: &OutMsgDescr) -> BlockResult<()> {
        self.out_msg_descr.write_struct(value)
    }

    pub fn out_msg_descr_cell(&self) -> &Cell {
        self.out_msg_descr.cell()
    }

    pub fn read_account_blocks(&self) -> BlockResult<ShardAccountBlocks> {
        self.account_blocks.read_struct()
    }

    pub fn write_account_blocks(&mut self, value: &ShardAccountBlocks) -> BlockResult<()> {
        self.account_blocks.write_struct(value)
    }

    pub fn account_blocks_cell(&self) -> &Cell {
        self.account_blocks.cell()
    }

    pub fn rand_seed(&self) -> &UInt256 {
        &self.rand_seed
    }

    pub fn rand_seed_mut(&mut self) -> &mut UInt256 {
        &mut self.rand_seed
    }

    pub fn created_by(&self) -> &UInt256 {
        &self.created_by
    }

    pub fn created_by_mut(&mut self) -> &mut UInt256 {
        &mut self.created_by
    }

    pub fn read_custom(&self) -> BlockResult<Option<McBlockExtra>> {
        Ok(
            match self.custom {
                Some(ref custom) => Some(custom.read_struct()?),
                None => None
            }
        )
    }

    pub fn write_custom(&mut self, value: Option<McBlockExtra>) -> BlockResult<()> {
        self.custom = match value {
                Some(v) => Some(ChildCell::with_struct(&v)?),
                None => None
            };
        Ok(())
    }

    pub fn custom_cell(&self) -> Option<&Cell> {
        self.custom.as_ref().map(|c| c.cell())
    }
}

const BLOCK_EXTRA_TAG: u32 = 0x4a33f6fd;

impl Deserializable for BlockExtra {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        let tag = cell.get_next_u32()?;
        if tag != BLOCK_EXTRA_TAG {
            bail!(BlockErrorKind::InvalidConstructorTag {
                t: tag,
                s: "BlockExtra".into()
            })
        }
        self.in_msg_descr
            .read_from(&mut cell.checked_drain_reference()?.into())?;
        self.out_msg_descr
            .read_from(&mut cell.checked_drain_reference()?.into())?;
        self.account_blocks
            .read_from(&mut cell.checked_drain_reference()?.into())?;
        self.rand_seed.read_from(cell)?;
        self.created_by.read_from(cell)?;
        self.custom = if cell.get_next_bit()? {
            Some(ChildCell::<McBlockExtra>::construct_from(&mut cell.checked_drain_reference()?.into())?)
        } else {
            None
        };
        Ok(())
    }
}

impl Serializable for BlockExtra {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_u32(BLOCK_EXTRA_TAG)?;
        cell.append_reference(self.in_msg_descr.write_to_new_cell()?);
        cell.append_reference(self.out_msg_descr.write_to_new_cell()?);

        let mut account_blocks_builder = BuilderData::new();
        self.account_blocks.write_to(&mut account_blocks_builder)?;
        cell.append_reference(account_blocks_builder);

        self.rand_seed.write_to(cell)?;
        self.created_by.write_to(cell)?;
        if let Some(custrom) = &self.custom {
            cell.append_bit_one()?;
            cell.append_reference(custrom.write_to_new_cell()?);
        } else {
            cell.append_bit_zero()?;
        }
        Ok(())
    }
}

/// value_flow ^[ from_prev_blk:CurrencyCollection
///   to_next_blk:CurrencyCollection
///   imported:CurrencyCollection
///   exported:CurrencyCollection ]
///   fees_collected:CurrencyCollection
///   ^[
///   fees_imported:CurrencyCollection
///   recovered:CurrencyCollection
///   created:CurrencyCollection
///   minted:CurrencyCollection
/// ] = ValueFlow;
///
/// TON Blockchain 4.3.5:
/// The TL-B construct _:ˆ[...] describes a reference to a cell containing the fields
/// listed inside the square brackets. In this way, several fields can be moved from
/// a cell containing a large record into a separate subcell.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ValueFlow {
    pub from_prev_blk: CurrencyCollection, // serialized into another cell 1
    pub to_next_blk: CurrencyCollection,   // serialized into another cell 1
    pub imported: CurrencyCollection,      // serialized into another cell 1
    pub exported: CurrencyCollection,      // serialized into another cell 1
    pub fees_collected: CurrencyCollection,
    pub fees_imported: CurrencyCollection, // serialized into another cell 2
    pub recovered: CurrencyCollection,     // serialized into another cell 2
    pub created: CurrencyCollection,       // serialized into another cell 2
    pub minted: CurrencyCollection,        // serialized into another cell 2
}

/*
ext_blk_ref$_ start_lt:uint64 end_lt:uint64
    seq_no:uint32 hash:uint256 = ExtBlkRef;
*/
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtBlkRef {
    pub end_lt: u64,
    pub seq_no: u32,
    pub root_hash: UInt256,
    pub file_hash: UInt256,
}

impl Deserializable for ExtBlkRef {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        self.end_lt = cell.get_next_u64()?;
        self.seq_no = cell.get_next_u32()?;
        self.root_hash.read_from(cell)?;
        self.file_hash.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ExtBlkRef {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        self.end_lt.write_to(cell)?;
        self.seq_no.write_to(cell)?;
        self.root_hash.write_to(cell)?;
        self.file_hash.write_to(cell)?;
        Ok(())
    }
}

impl Default for ExtBlkRef {
    fn default() -> Self {
        Self {
            end_lt: 0,
            seq_no: 0,
            root_hash: UInt256::from([0; 32]),
            file_hash: UInt256::from([0; 32]),
        }
    }
}

/*
shard_ident$00 shard_pfx_bits:(#<= 60)
    workchain_id:int32 shard_prefix:uint64 = ShardIdent;
*/
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct ShardIdent {
    pub shard_pfx_bits: u8, // 6 bits
    pub workchain_id: i32,
    pub shard_prefix: u64,
}

impl ShardIdent {
    pub fn new() -> Self {
        ShardIdent::default()
    }

    /// Creates new
    pub fn with_workchain_id(workchain_id: i32) -> Self {
        Self {
            shard_pfx_bits: 0,
            workchain_id,
            shard_prefix: 0,
        }
    }
    ///
    /// Get bitstring-key for BinTree operation for Shard
    ///
    pub fn shard_key(&self) -> SliceData {
        let mut cell = BuilderData::new();
        cell.append_bits(self.shard_prefix as usize, self.shard_pfx_bits as usize)
            .unwrap();
        cell.into()
    }

    ///
    /// Get bitstring-key for BinTree operation for Shard
    ///
    pub fn full_key(&self) -> SliceData {
        let mut cell = BuilderData::new();
        cell.append_i32(self.workchain_id)
            .unwrap()
            .append_u64(self.shard_prefix)
            .unwrap();
        cell.into()
    }

    pub fn enlarge(&mut self) {
        debug_assert!(self.shard_pfx_bits < 60);
        self.shard_prefix |= 1u64 << self.shard_pfx_bits as u64;
        self.shard_pfx_bits += 1;
    }

    ///
    /// Get workchain_id
    ///
    pub fn workchain_id(&self) -> u32 {
        self.workchain_id as u32
    }

    pub fn contains_account(&self, mut acc_addr: AccountId) -> BlockResult<bool> {
        Ok(self.shard_pfx_bits == 0
            || acc_addr.get_next_int(self.shard_pfx_bits as usize)?
                == self.shard_prefix >> 64 - self.shard_pfx_bits)
    }

    pub fn shard_prefix_as_str_with_tag(&self) -> String {
        format!(
            "{:x}",
            self.shard_prefix | (1 << (63 - self.shard_pfx_bits))
        )
    }
}

impl Deserializable for ShardIdent {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        let constructor_and_pfx = cell.get_next_byte()?;
        // check for 2 high bits to be zero
        if constructor_and_pfx & 0xC0 != 0 {
            bail!(BlockErrorKind::InvalidData {
                msg: "2 high bits in ShardIdent's first byte have to be zero".into()
            })
        }
        self.shard_pfx_bits = constructor_and_pfx & 0x3F;
        self.workchain_id = cell.get_next_u32()? as i32;
        self.shard_prefix = cell.get_next_u64()?;

        Ok(())
    }
}

impl Serializable for ShardIdent {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        (self.shard_pfx_bits & 0x3F).write_to(cell)?;
        self.workchain_id.write_to(cell)?;
        self.shard_prefix.write_to(cell)?;
        Ok(())
    }
}

/*
_ ShardStateUnsplit = ShardState;
split_state#5f327da5
    left:^ShardStateUnsplit
    right:^ShardStateUnsplit
= ShardState;
*/

///
/// Enum ShardState
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShardState {
    UnsplitState(ShardStateUnsplit),
    SplitState(ShardStateSplit),
}

impl Default for ShardState {
    fn default() -> Self {
        ShardState::UnsplitState(ShardStateUnsplit::default())
    }
}

impl Deserializable for ShardState {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        let tag = cell.clone().get_next_u32()?;
        *self = match tag {
            SHARD_STATE_UNSPLIT_PFX => {
                let mut ss = ShardStateUnsplit::default();
                ss.read_from(cell)?;
                ShardState::UnsplitState(ss)
            }
            SHARD_STATE_SPLIT_PFX => {
                let mut ss = ShardStateSplit::default();
                ss.read_from(cell)?;
                ShardState::SplitState(ss)
            }
            _ => {
                bail!(BlockErrorKind::InvalidConstructorTag {
                    t: tag,
                    s: "ShardState".into()
                })
            }
        };

        Ok(())
    }
}

const SHARD_STATE_SPLIT_PFX: u32 = 0x5f327da5;
const SHARD_STATE_UNSPLIT_PFX: u32 = 0x9023afe2;

impl Serializable for ShardState {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        match self {
            ShardState::UnsplitState(ss) => {
                cell.append_u32(SHARD_STATE_UNSPLIT_PFX)?;
                ss.write_to(cell)?;
            }
            ShardState::SplitState(ss) => {
                cell.append_u32(SHARD_STATE_SPLIT_PFX)?;
                ss.write_to(cell)?;
            }
        }
        Ok(())
    }
}

///
/// Struct ShardStateSplit
///
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ShardStateSplit {
    pub left: ShardStateUnsplit,
    pub right: ShardStateUnsplit,
}

impl ShardStateSplit {
    pub fn new() -> Self {
        ShardStateSplit::default()
    }

    pub fn with_left_right(left: ShardStateUnsplit, right: ShardStateUnsplit) -> Self {
        ShardStateSplit { left, right }
    }
}

impl Deserializable for ShardStateSplit {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        let tag = cell.get_next_u32()?;
        if tag != SHARD_STATE_SPLIT_PFX {
            bail!(BlockErrorKind::InvalidConstructorTag {
                t: tag,
                s: "ShardStateSplit".into()
            })
        }
        self.left
            .read_from(&mut cell.checked_drain_reference()?.into())?;
        self.right
            .read_from(&mut cell.checked_drain_reference()?.into())?;
        Ok(())
    }
}

impl Serializable for ShardStateSplit {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_u32(SHARD_STATE_SPLIT_PFX)?;
        cell.append_reference(self.left.write_to_new_cell()?);
        cell.append_reference(self.right.write_to_new_cell()?);
        Ok(())
    }
}

///
/// Struct ShardStateUnsplit
///
// shard_state#9023afe2
//     global_id:int32
//     shard_id:ShardIdent
//     seq_no:uint32
//     vert_seq_no:#
//     gen_utime:uint32
//     gen_lt:uint64
//     min_ref_mc_seqno:uint32
//     out_msg_queue_info:^OutMsgQueueInfo
//     before_split:(## 1)
//     accounts:^ShardAccounts
//     ^[
//         overload_history:uint64
//         underload_history:uint64
//         total_balance:CurrencyCollection
//         total_validator_fees:CurrencyCollection
//         libraries:(HashmapE 256 LibDescr)
//         master_ref:(Maybe BlkMasterInfo)
//     ]
//     custom:(Maybe ^McStateExtra)
// = ShardStateUnsplit;
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ShardStateUnsplit {
    global_id: i32,
    shard_id: ShardIdent,
    seq_no: u32,
    vert_seq_no: u32,
    gen_time: u32,
    gen_lt: u64,
    min_ref_mc_seqno: u32,
    out_msg_queue_info: ChildCell<OutMsgQueueInfo>,
    before_split: bool,
    accounts: ChildCell<ShardAccounts>,
    // next fields in separate cell
    overload_history: u64,
    underload_history: u64,
    total_balance: CurrencyCollection,
    total_validator_fees: CurrencyCollection,
    libraries: HashmapE, // <AccountId, LibDescr>, // currently can be present only in masterchain blocks.
    master_ref: Option<BlkMasterInfo>,

    custom: Option<ChildCell<McStateExtra>>, // The field custom is usually present only
    // in the masterchain and contains all the masterchain-specific data.
}

impl ShardStateUnsplit {
    pub fn with_ident(shard_id: ShardIdent) -> Self {
        let mut shard_state = ShardStateUnsplit::default();
        shard_state.shard_id = shard_id;
        shard_state
    }

    pub fn global_id(&self) -> i32 {
        self.global_id
    }

    pub fn set_global_id(&mut self, value: i32) {
        self.global_id = value
    }

    pub fn shard_id(&self) -> &ShardIdent {
        &self.shard_id
    }

    pub fn shard_id_mut(&mut self) -> &mut ShardIdent {
        &mut self.shard_id
    }

    pub fn seq_no(&self) -> u32 {
        self.seq_no
    }

    pub fn set_seq_no(&mut self, seq_no: u32) {
        assert!(seq_no != 0);
        self.seq_no = seq_no
    }

    pub fn vert_seq_no(&self) -> u32 {
        self.vert_seq_no
    }

    pub fn set_vert_seq_no(&mut self, value: u32) {
        self.vert_seq_no = value
    }

    pub fn gen_time(&self) -> u32 {
        self.gen_time
    }

    pub fn set_gen_time(&mut self, value: u32) {
        self.gen_time = value
    }

    pub fn gen_lt(&self) -> u64 {
        self.gen_lt
    }

    pub fn set_gen_lt(&mut self, value: u64) {
        self.gen_lt = value
    }

    pub fn min_ref_mc_seqno(&self) -> u32 {
        self.min_ref_mc_seqno
    }

    pub fn set_(&mut self, value: u32) {
        self.min_ref_mc_seqno = value
    }

    pub fn read_out_msg_queue_info(&self) -> BlockResult<OutMsgQueueInfo> {
        self.out_msg_queue_info.read_struct()
    }

    pub fn write_out_msg_queue_info(&mut self, value: &OutMsgQueueInfo) -> BlockResult<()> {
        self.out_msg_queue_info.write_struct(value)
    }

    pub fn before_split(&self) -> bool {
        self.before_split
    }

    pub fn set_before_split(&mut self, value: bool) {
        self.before_split = value
    }

    pub fn read_accounts(&self) -> BlockResult<ShardAccounts> {
        self.accounts.read_struct()
    }

    pub fn write_accounts(&mut self, value: &ShardAccounts) -> BlockResult<()> {
        self.accounts.write_struct(value)
    }

    pub fn overload_history(&self) -> u64 {
        self.overload_history
    }

    pub fn set_overload_history(&mut self, value: u64) {
        self.overload_history = value
    }

    pub fn underload_history(&self) -> u64 {
        self.underload_history
    }

    pub fn set_underload_history(&mut self, value: u64) {
        self.underload_history = value
    }

    pub fn total_balance(&self) -> &CurrencyCollection {
        &self.total_balance
    }

    pub fn total_balance_mut(&mut self) -> &mut CurrencyCollection {
        &mut self.total_balance
    }

    pub fn total_validator_fees(&self) -> &CurrencyCollection {
        &self.total_validator_fees
    }

    pub fn total_validator_fees_mut(&mut self) -> &mut CurrencyCollection {
        &mut self.total_validator_fees
    }

    pub fn libraries(&self) -> &HashmapE {
        &self.libraries
    }

    pub fn libraries_mut(&mut self) -> &mut HashmapE {
        &mut self.libraries
    }

    pub fn master_ref(&self) -> Option<&BlkMasterInfo> {
        self.master_ref.as_ref()
    }

    pub fn master_ref_mut(&mut self) -> Option<&mut BlkMasterInfo> {
        self.master_ref.as_mut()
    }

    pub fn read_custom(&self) -> BlockResult<Option<McStateExtra>> {
        match self.custom {
            None => Ok(None),
            Some(ref custom) => Ok(Some(custom.read_struct()?))
        }
    }

    pub fn write_custom(&mut self, value: Option<McStateExtra>) -> BlockResult<()> {
        self.custom = match value {
            Some(custom) => Some(ChildCell::with_struct(&custom)?),
            None => None
        };
        Ok(())
    }
}

impl Deserializable for ShardStateUnsplit {

    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        let tag = cell.get_next_u32()?;
        if tag != SHARD_STATE_UNSPLIT_PFX {
            bail!(BlockErrorKind::InvalidConstructorTag {
                t: tag as u32,
                s: "ShardStateUnsplit".into()
            });
        }
        self.global_id.read_from(cell)?;
        self.shard_id.read_from(cell)?;
        self.seq_no.read_from(cell)?;
        self.vert_seq_no.read_from(cell)?;
        self.gen_time.read_from(cell)?;
        self.gen_lt.read_from(cell)?;
        self.min_ref_mc_seqno.read_from(cell)?;
        self.out_msg_queue_info
            .read_from(&mut cell.checked_drain_reference()?.into())?;
        self.before_split = cell.get_next_bit()?;
        self.accounts
            .read_from(&mut cell.checked_drain_reference()?.into())?;

        let ref mut cell1 = cell.checked_drain_reference()?.into();
        self.overload_history.read_from(cell1)?;
        self.underload_history.read_from(cell1)?;
        self.total_balance.read_from(cell1)?;
        self.total_validator_fees.read_from(cell1)?;
        self.libraries.read_from(cell1)?;
        self.master_ref = BlkMasterInfo::read_maybe_from(cell1)?;

        self.custom = if cell.get_next_bit()? {
            let mse = ChildCell::<McStateExtra>::construct_from(&mut cell.checked_drain_reference()?.into())?;
            Some(mse)
        } else {
            None
        };
        Ok(())
    }
}

impl Serializable for ShardStateUnsplit {
    fn write_to(&self, builder: &mut BuilderData) -> BlockResult<()> {
        builder.append_u32(SHARD_STATE_UNSPLIT_PFX)?;
        self.global_id.write_to(builder)?;
        self.shard_id.write_to(builder)?;
        self.seq_no.write_to(builder)?;
        self.vert_seq_no.write_to(builder)?;
        self.gen_time.write_to(builder)?;
        self.gen_lt.write_to(builder)?;
        self.min_ref_mc_seqno.write_to(builder)?;
        builder.append_reference(self.out_msg_queue_info.write_to_new_cell()?);
        builder.append_bit_bool(self.before_split)?;

        let mut accounts_builder = BuilderData::new();
        self.accounts.write_to(&mut accounts_builder)?;
        builder.append_reference(accounts_builder);

        let mut b2 = BuilderData::new();
        self.overload_history.write_to(&mut b2)?;
        self.underload_history.write_to(&mut b2)?;
        self.total_balance.write_to(&mut b2)?;
        self.total_validator_fees.write_to(&mut b2)?;
        self.libraries.write_to(&mut b2)?;
        self.master_ref.write_maybe_to(&mut b2)?;
        builder.append_reference(b2);

        builder.append_bit_bool(self.custom.is_some())?;
        if let Some(ref custom) = self.custom {
            builder.append_reference(custom.write_to_new_cell()?);
        }

        Ok(())
    }
}

impl Default for ShardStateUnsplit {
    fn default() -> Self {
        Self {
            global_id: 0,
            shard_id: ShardIdent::default(),
            seq_no: 0,
            vert_seq_no: 0,
            gen_time: 0,
            gen_lt: 0,
            min_ref_mc_seqno: 0,
            out_msg_queue_info: ChildCell::default(),
            before_split: false,
            accounts: ChildCell::default(),
            overload_history: 0,
            underload_history: 0,
            total_balance: CurrencyCollection::default(),
            total_validator_fees: CurrencyCollection::default(),
            libraries: HashmapE::with_bit_len(256), // <AccountId, LibDescr>, // currently can be present only in masterchain blocks.
            master_ref: None,
            custom: None,
        }
    }
}

const BLOCK_TAG: u32 = 0x11ef55aa;

const BLOCK_INFO_TAG: u32 = 0x9bc7a987;

impl Serializable for BlockInfo {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        assert!(self.seq_no != 0);
        assert!((self.vert_seq_no == 0) ^ self.prev_vert_ref.is_some());

        let mut byte = 0;
        if self.master_ref.is_some() {
            byte |= 1 << 7
        }
        if self.after_merge {
            byte |= 1 << 6
        }
        if self.before_split {
            byte |= 1 << 5;
        }
        if self.after_split {
            byte |= 1 << 4;
        }
        if self.want_split {
            byte |= 1 << 3;
        }
        if self.want_merge {
            byte |= 1 << 2;
        }
        if self.key_block {
            byte |= 1 << 1;
        }
        if self.vert_seqno_incr != 0 {
            byte |= 1;
        }

        cell.append_u32(BLOCK_INFO_TAG)?
            .append_u32(self.version)?
            .append_u8(byte)?
            .append_u8(self.flags)?
            .append_u32(self.seq_no)?
            .append_u32(self.vert_seq_no)?;

        // shard:ShardIdent
        self.shard.write_to(cell)?;
        cell.append_u32(self.gen_utime.0)?
            .append_u64(self.start_lt)?
            .append_u64(self.end_lt)?
            .append_u32(self.gen_validator_list_hash_short)?
            .append_u32(self.gen_catchain_seqno)?
            .append_u32(self.min_ref_mc_seqno)?
            .append_u32(self.prev_key_block_seqno)?;

        // master_ref:not_master?^BlkMasterInfo
        if let Some(ref master) = self.master_ref {
            cell.append_reference(master.write_to_new_cell()?);
        }

        // prev_ref:^(BlkPrevInfo after_merge)
        cell.append_reference(self.prev_ref.write_to_new_cell()?);

        // prev_vert_ref:vert_seq_no?^(BlkPrevInfo 0)
        match &self.prev_vert_ref {
            Some(prev) => {
                cell.append_reference(prev.write_to_new_cell()?);
            }
            None => (),
        }

        Ok(())
    }
}

const VALUE_FLOW_TAG: u32 = 0xb8e48dfb;

impl Serializable for ValueFlow {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_u32(VALUE_FLOW_TAG)?;

        let mut cell1 = BuilderData::new();
        self.from_prev_blk.write_to(&mut cell1)?;
        self.to_next_blk.write_to(&mut cell1)?;
        self.imported.write_to(&mut cell1)?;
        self.exported.write_to(&mut cell1)?;
        cell.append_reference(cell1);
        self.fees_collected.write_to(cell)?;

        let mut cell2 = BuilderData::new();
        self.fees_imported.write_to(&mut cell2)?;
        self.recovered.write_to(&mut cell2)?;
        self.created.write_to(&mut cell2)?;
        self.minted.write_to(&mut cell2)?;
        cell.append_reference(cell2);

        Ok(())
    }
}

impl Deserializable for ValueFlow {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        let tag = cell.get_next_u32()?;
        if tag != VALUE_FLOW_TAG {
            bail!(BlockErrorKind::InvalidConstructorTag {
                t: tag,
                s: "ValueFlow".into()
            })
        }
        let ref mut cell1 = cell.checked_drain_reference()?.into();
        self.from_prev_blk.read_from(cell1)?;
        self.to_next_blk.read_from(cell1)?;
        self.imported.read_from(cell1)?;
        self.exported.read_from(cell1)?;
        self.fees_collected.read_from(cell)?;

        let ref mut cell2 = cell.checked_drain_reference()?.into();
        self.fees_imported.read_from(cell2)?;
        self.recovered.read_from(cell2)?;
        self.created.read_from(cell2)?;
        self.minted.read_from(cell2)?;
        Ok(())
    }
}

impl Deserializable for BlockInfo {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        let tag = cell.get_next_u32()?;
        if tag != BLOCK_INFO_TAG {
            bail!(BlockErrorKind::InvalidConstructorTag {
                t: tag,
                s: "BlockInfo".into()
            })
        }
        self.version = cell.get_next_u32()?;
        
        let next_byte = cell.get_next_byte()?;
        let not_master = (next_byte >> 7) & 1 == 1;
        self.after_merge = (next_byte >> 6) & 1 == 1;
        self.before_split = (next_byte >> 5) & 1 == 1;
        self.after_split = (next_byte >> 4) & 1 == 1;
        self.want_split = (next_byte >> 3) & 1 == 1;
        self.want_merge = (next_byte >> 2) & 1 == 1;
        self.key_block = (next_byte >> 1) & 1 == 1;
        self.vert_seqno_incr = ((next_byte) & 1) as u32;

        self.flags = cell.get_next_byte()?;
        self.seq_no = cell.get_next_u32()?;
        self.vert_seq_no = cell.get_next_u32()?;
        if self.vert_seqno_incr > self.vert_seq_no {
            bail!(BlockErrorKind::InvalidData {
                msg: format!("BlockInfo {} < {}", self.vert_seqno_incr, self.vert_seq_no)
            })
        }
        if self.seq_no < 1 {
            bail!(BlockErrorKind::InvalidData {
                msg: format!("BlockInfo {}", self.seq_no)
            })
        }
        self.shard.read_from(cell)?;
        self.gen_utime.0 = cell.get_next_u32()?;
        self.start_lt = cell.get_next_u64()?;
        self.end_lt = cell.get_next_u64()?;
        self.gen_validator_list_hash_short = cell.get_next_u32()?;
        self.gen_catchain_seqno = cell.get_next_u32()?;
        self.min_ref_mc_seqno = cell.get_next_u32()?;
        self.prev_key_block_seqno = cell.get_next_u32()?;

        // master_ref:not_master?^BlkMasterInfo 
        self.master_ref = if not_master {
                let mut bli = BlkMasterInfo::default();
                bli.read_from(&mut cell.checked_drain_reference()?.into())?;
                Some(bli)
            } else { 
                None
            };

        // prev_ref:^(BlkPrevInfo after_merge)
        self.prev_ref = if self.after_merge {
                BlkPrevInfo::default_blocks() 
            } else { 
                BlkPrevInfo::default_block()
            };
        self.prev_ref.read_from(&mut cell.checked_drain_reference()?.into())?;

        // prev_vert_ref:vert_seqno_incr?^(BlkPrevInfo 0)
        self.prev_vert_ref = match self.vert_seq_no {
            0 => None,
            _ => {
                let mut bpi = BlkPrevInfo::default_block();
                bpi.read_from(&mut cell.checked_drain_reference()?.into())?;
                Some(bpi)
            }
        };

        Ok(())
    }
}

impl Deserializable for Block {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        let tag = cell.get_next_u32()?;
        if tag != BLOCK_TAG {
            bail!(BlockErrorKind::InvalidConstructorTag {
                t: tag,
                s: "Block".into()
            })
        }
        self.global_id.read_from(cell)?;
        self.info
            .read_from(&mut cell.checked_drain_reference()?.into())?;
        self.value_flow
            .read_from(&mut cell.checked_drain_reference()?.into())?;
        self.state_update
            .read_from(&mut cell.checked_drain_reference()?.into())?;
        self.extra
            .read_from(&mut cell.checked_drain_reference()?.into())?;

        Ok(())
    }
}

impl Serializable for Block {
    fn write_to(&self, builder: &mut BuilderData) -> BlockResult<()> {
        builder.append_u32(BLOCK_TAG)?;
        builder.append_u32(self.global_id)?;
        builder.append_reference(self.info.write_to_new_cell()?); // info:^BlockInfo
        builder.append_reference(self.value_flow.write_to_new_cell()?); // value_flow:^ValueFlow
        builder.append_reference(self.state_update.write_to_new_cell()?); // state_update:^(MERKLE_UPDATE ShardState)
        builder.append_reference(self.extra.write_to_new_cell()?); // extra:^BlockExtra

        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum BlockProcessingStatus {
    Unknown = 0,
    Proposed,
    Finalized,
    Refused,
}

impl Default for BlockProcessingStatus {
    fn default() -> Self {
        BlockProcessingStatus::Unknown
    }
}

/*
chain_empty$_ = ProofChain 0;
chain_link$_
    {n:#}
    root:^Cell
    prev:n?^(ProofChain n)
= ProofChain (n + 1);
top_block_descr#d5
    proof_for:BlockIdExt
    signatures:(Maybe ^BlockSignatures)
    len:(## 8) { len >= 1 } { len <= 8 }
    chain:(ProofChain len)
= TopBlockDescr;
*/
#[derive(Debug, Default, Eq, PartialEq)]
pub struct TopBlockDescr {
    proof_for: BlockIdExt,
    signatures: Option<BlockSignatures>,
    chain: Vec<Cell>,
}

impl TopBlockDescr {
    pub fn with_id_and_signatures(
        proof_for: BlockIdExt,
        signatures: Option<BlockSignatures>,
    ) -> Self {
        Self {
            proof_for,
            signatures,
            chain: vec![],
        }
    }
    pub fn append_proof(&mut self, cell: Cell) {
        self.chain.push(cell);
    }
}

const TOP_BLOCK_DESCR_TAG: u8 = 0xD5;

impl Serializable for TopBlockDescr {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        TOP_BLOCK_DESCR_TAG.write_to(cell)?;
        self.proof_for.write_to(cell)?;
        self.signatures.write_maybe_to(cell)?;
        let mut prev = BuilderData::new();
        for (i, c) in self.chain.iter().rev().enumerate() {
            let mut builder = BuilderData::new();
            builder.append_reference(BuilderData::from(&c));
            if i != 0 {
                builder.append_reference(prev);
            }
            prev = builder;
        }
        cell.append_bits(self.chain.len(), 3)?;
        cell.checked_append_references_and_data(&prev.into())?;
        Ok(())
    }
}

impl Deserializable for TopBlockDescr {
    fn read_from(&mut self, slice: &mut SliceData) -> BlockResult<()> {
        let tag = slice.get_next_byte()?;
        if tag != TOP_BLOCK_DESCR_TAG {
            bail!(BlockErrorKind::InvalidConstructorTag {
                t: tag.into(),
                s: "TopBlockDescr".into()
            })
        }
        self.proof_for.read_from(slice)?;
        self.signatures = BlockSignatures::read_maybe_from(slice)?;
        let len = slice.get_next_int(3)?;
        {
            let mut slice = slice.clone();
            for i in (0..len).rev() {
                ensure!(
                    slice.remaining_references() != 0,
                    BlockError::from(ExceptionCode::CellUnderflow)
                );
                self.chain.push(slice.checked_drain_reference()?.clone());
                if i != 0 {
                    ensure!(
                        slice.remaining_references() != 0,
                        BlockError::from(ExceptionCode::CellUnderflow)
                    );
                    slice = slice.checked_drain_reference()?.into();
                }
            }
        }
        Ok(())
    }
}
