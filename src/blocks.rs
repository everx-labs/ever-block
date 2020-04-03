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

use ton_types::{BuilderData, SliceData};
use {ExceptionCode, UInt256};
use super::*;
use dictionary::HashmapE;
use std::cmp::Ordering;
use std::io::{Cursor, Write};
use std::fmt::{self, Display, Formatter};


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
    pub shard_id: ShardIdent,
    pub seq_no: u32,
    pub root_hash: UInt256,
    pub file_hash: UInt256,
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
    pub fn dummy_masterchain() -> Self {
        BlockIdExt {
            shard_id: ShardIdent::masterchain(),
            seq_no: 0,
            root_hash: UInt256::default(),
            file_hash: UInt256::default(),
        }
    }
    pub fn shard(&self) -> &ShardIdent {
        &self.shard_id
    }
}

impl Serializable for BlockIdExt {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.shard_id.write_to(cell)?;
        self.seq_no.write_to(cell)?;
        self.root_hash.write_to(cell)?;
        self.file_hash.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for BlockIdExt {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
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

const GEN_SOFTWARE_EXISTS_FLAG: u8 = 1;

/* 
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
  flags:(## 8) { flags <= 1 }
  seq_no:# 
  vert_seq_no:# 
  { vert_seq_no >= vert_seqno_incr } 
  { prev_seq_no:# } { ~prev_seq_no + 1 = seq_no } 

  shard:ShardIdent
  gen_utime:uint32
  start_lt:uint64 
  end_lt:uint64
  gen_validator_list_hash_short:uint32
  gen_catchain_seqno:uint32
  min_ref_mc_seqno:uint32
  prev_key_block_seqno:uint32
  gen_software:flags . 0?GlobalVersion

  master_ref:not_master?^BlkMasterInfo 
  prev_ref:^(BlkPrevInfo after_merge)
  prev_vert_ref:vert_seqno_incr?^(BlkPrevInfo 0)

= BlockInfo;
*/
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockInfo {

    version: u32,
    after_merge: bool,
    before_split: bool,
    after_split: bool,
    want_split: bool,
    want_merge: bool,
    key_block: bool,

    vert_seqno_incr: u32,
    flags: u8,
    seq_no: u32,
    vert_seq_no: u32,

    shard: ShardIdent,
    gen_utime: UnixTime32,
    start_lt: u64,
    end_lt: u64,
    gen_validator_list_hash_short: u32,
    gen_catchain_seqno: u32,
    min_ref_mc_seqno: u32,
    prev_key_block_seqno: u32,
    gen_software: Option<GlobalVersion>,

    master_ref: Option<ChildCell<BlkMasterInfo>>,
    prev_ref: ChildCell<BlkPrevInfo>,
    prev_vert_ref: Option<ChildCell<BlkPrevInfo>>,
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
            gen_software: None,
            master_ref: None,
            prev_ref: ChildCell::default(),
            prev_vert_ref: None,
        }
    }
}

impl BlockInfo {

    pub fn new() -> Self {
        Self::default()
    }

    pub fn version(&self) -> u32 { self.version }
    pub fn set_version(&mut self, version: u32) { self.version = version; }


    pub fn before_split(&self) -> bool { self.before_split }
    pub fn set_before_split(&mut self, before_split: bool) { self.before_split = before_split }

    pub fn after_split(&self) -> bool { self.after_split }
    pub fn set_after_split(&mut self, after_split: bool) { self.after_split = after_split }

    pub fn want_split(&self) -> bool { self.want_split }
    pub fn set_want_split(&mut self, want_split: bool) { self.want_split = want_split }

    pub fn want_merge(&self) -> bool { self.want_merge }
    pub fn set_want_merge(&mut self, want_merge: bool) { self.want_merge = want_merge }

    pub fn key_block(&self) -> bool { self.key_block }
    pub fn set_key_block(&mut self, key_block: bool) { self.key_block = key_block }


    pub fn flags(&self) -> u8 { self.flags }
    // For now flags is related only on gen_software, so it is set automatically if need
    //pub fn set_flags(&mut self, flags) { self.flags = flags }

    pub fn seq_no(&self) -> u32 { self.seq_no }
    pub fn set_seq_no(&mut self, seq_no: u32) -> Result<()> {
        if seq_no == 0 {
            fail!(BlockError::InvalidArg("`seq_no` can't be zero".to_string()))
        }
        self.seq_no = seq_no;
        Ok(())
    }


    pub fn shard(&self) -> &ShardIdent { &self.shard }
    pub fn set_shard(&mut self, shard: ShardIdent) { self.shard = shard }

    pub fn gen_utime(&self) -> UnixTime32 { self.gen_utime }
    pub fn set_gen_utime(&mut self, gen_utime: UnixTime32) { self.gen_utime = gen_utime }

    pub fn start_lt(&self) -> u64 { self.start_lt }
    pub fn set_start_lt(&mut self, start_lt: u64) { self.start_lt = start_lt }

    pub fn end_lt(&self) -> u64 { self.end_lt }
    pub fn set_end_lt(&mut self, end_lt: u64) { self.end_lt = end_lt }

    pub fn gen_validator_list_hash_short(&self) -> u32 { self.gen_validator_list_hash_short }
    pub fn set_gen_validator_list_hash_short(&mut self, hash: u32) { self.gen_validator_list_hash_short = hash }

    pub fn gen_catchain_seqno(&self) -> u32 { self.gen_catchain_seqno }
    pub fn set_gen_catchain_seqno(&mut self, cc_seqno: u32) { self.gen_catchain_seqno = cc_seqno }

    pub fn min_ref_mc_seqno(&self) -> u32 { self.min_ref_mc_seqno }
    pub fn set_min_ref_mc_seqno(&mut self, min_ref_mc_seqno: u32) { self.min_ref_mc_seqno = min_ref_mc_seqno }

    pub fn prev_key_block_seqno(&self) -> u32 { self.prev_key_block_seqno }
    pub fn set_prev_key_block_seqno(&mut self, prev_key_block_seqno: u32) { self.prev_key_block_seqno = prev_key_block_seqno }

    pub fn gen_software(&self) -> Option<&GlobalVersion> { self.gen_software.as_ref() }
    pub fn set_gen_software(&mut self, gen_software: Option<GlobalVersion>) {
        self.gen_software = gen_software;
        if self.gen_software.is_some() {
            self.flags |= GEN_SOFTWARE_EXISTS_FLAG;
        } else {
            self.flags &= !GEN_SOFTWARE_EXISTS_FLAG;
        }
    }

    pub fn read_master_ref(&self) -> Result<Option<BlkMasterInfo>> {
        self.master_ref.as_ref().map(|mr| mr.read_struct()).transpose()
    }

    pub fn write_master_ref(&mut self, value: Option<&BlkMasterInfo>) -> Result<()> {
        self.master_ref = value.map(|v| ChildCell::with_struct(v)).transpose()?;
        Ok(())
    }

    pub fn read_master_id(&self) -> Result<ExtBlkRef> {
        match self.master_ref {
            Some(ref mr) => Ok(mr.read_struct()?.master),
            None => self.read_prev_ref()?.prev1()
        }
    }

    pub fn after_merge(&self) -> bool { self.after_merge }
    pub fn read_prev_ref(&self) -> Result<BlkPrevInfo> {
        let mut prev_ref = if self.after_merge {
            BlkPrevInfo::default_blocks() 
        } else { 
            BlkPrevInfo::default_block()
        };
        prev_ref.read_from(&mut self.prev_ref.cell().into())?;
        Ok(prev_ref)
    }
    pub fn read_prev_ids(&self) -> Result<Vec<ExtBlkRef>> {
        let prev = self.read_prev_ref()?;
        let mut vec = vec!(prev.prev1()?);
        if let Some(prev2) = prev.prev2()? {
            vec.push(prev2);
        }
        Ok(vec)
    }
    pub fn set_prev_stuff(&mut self, after_merge: bool, prev_ref: &BlkPrevInfo) -> Result<()> {
        if !after_merge ^ prev_ref.is_one_prev() {
            fail!(BlockError::InvalidArg(
                "`prev_ref` may handle two blocks only if `after_merge`".to_string()))
        }
        self.after_merge = after_merge;
        self.prev_ref.write_struct(prev_ref)
    }

    pub fn vert_seq_no(&self) -> u32 { self.vert_seq_no }
    pub fn vert_seqno_incr(&self) -> u32 { self.vert_seqno_incr }
    pub fn read_prev_vert_ref(&self) -> Result<Option<BlkPrevInfo>> {
        self.prev_vert_ref.as_ref().map(|mr| mr.read_struct()).transpose()
    }
    pub fn set_vertical_stuff(
        &mut self,
        vert_seqno_incr: u32,
        vert_seq_no: u32,
        prev_vert_ref: Option<BlkPrevInfo>)
    -> Result<()> {
        if vert_seq_no < vert_seqno_incr {
            fail!(BlockError::InvalidArg(
                "`vert_seq_no` can't be less then `vert_seqno_incr`".to_string()))
        }
        if (vert_seqno_incr == 0) ^ prev_vert_ref.is_none() {
            fail!(BlockError::InvalidArg(
                "`prev_vert_ref` may be Some only if `vert_seqno_incr != 0` and vice versa".to_string()))
        }

        self.vert_seqno_incr = vert_seqno_incr;
        self.vert_seq_no = vert_seq_no;
        self.prev_vert_ref = prev_vert_ref.map(|v| ChildCell::with_struct(&v)).transpose()?;
        Ok(())
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

impl Default for BlkPrevInfo {
    fn default() -> BlkPrevInfo {
        BlkPrevInfo::Block{ prev: ExtBlkRef::default() }
    }
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

    pub fn prev1(&self) -> Result<ExtBlkRef> {
        Ok(
            match self {
                BlkPrevInfo::Block{prev} => prev.clone(),
                BlkPrevInfo::Blocks{prev1, prev2: _} => {
                    prev1.read_struct()?
                },
            }
        )
    }

    pub fn prev2(&self) -> Result<Option<ExtBlkRef>> {
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
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
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
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
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
    pub global_id: i32,
    pub info: ChildCell<BlockInfo>,            // reference
    pub value_flow: ChildCell<ValueFlow>,      // reference
    pub state_update: ChildCell<MerkleUpdate>, // reference
    pub extra: ChildCell<BlockExtra>,          // reference
}

impl Block {
    pub fn with_params(
        global_id: i32,
        info: BlockInfo,
        value_flow: ValueFlow,
        state_update: MerkleUpdate,
        extra: BlockExtra,
    ) -> Result<Self> {
        Ok(Block {
            global_id,
            info: ChildCell::with_struct(&info)?,
            value_flow: ChildCell::with_struct(&value_flow)?,
            extra: ChildCell::with_struct(&extra)?,
            state_update: ChildCell::with_struct(&state_update)?,
        })
    }

    pub fn global_id(&self) -> i32 {
        self.global_id
    }

    pub fn set_global_id(&mut self, global_id: i32) {
        self.global_id = global_id
    }

    pub fn read_info(&self) -> Result<BlockInfo> {
        self.info.read_struct()
    }

    pub fn write_info(&mut self, value: &BlockInfo) -> Result<()> {
        self.info.write_struct(value)
    }

    pub fn info_cell(&self) -> &Cell {
        self.info.cell()
    }

    pub fn read_value_flow(&self) -> Result<ValueFlow> {
        self.value_flow.read_struct()
    }

    pub fn write_value_flow(&mut self, value: &ValueFlow) -> Result<()> {
        self.value_flow.write_struct(value)
    }

    pub fn value_flow_cell(&self) -> &Cell {
        self.value_flow.cell()
    }

    pub fn read_state_update(&self) -> Result<MerkleUpdate> {
        self.state_update.read_struct()
    }

    pub fn write_state_update(&mut self, value: &MerkleUpdate) -> Result<()> {
        self.state_update.write_struct(value)
    }

    pub fn state_update_cell(&self) -> &Cell {
        self.state_update.cell()
    }

    pub fn read_extra(&self) -> Result<BlockExtra> {
        self.extra.read_struct()
    }

    pub fn write_extra(&mut self, value: &BlockExtra) -> Result<()> {
        self.extra.write_struct(value)
    }

    pub fn extra_cell(&self) -> &Cell {
        self.extra.cell()
    }

    const DATA_FOR_SIGN_SIZE: usize = 4 + 32 + 32;
    const DATA_FOR_SIGN_TAG: [u8; 4] = [0x70, 0x6e, 0x0b, 0xc5];

    pub fn build_data_for_sign(root_hash: &UInt256, file_hash: &UInt256) -> [u8; Self::DATA_FOR_SIGN_SIZE] {
        let mut data = [0_u8; Self::DATA_FOR_SIGN_SIZE];
        {
            let mut cur = Cursor::new(&mut data[..]);
            cur.write(&Self::DATA_FOR_SIGN_TAG).unwrap();
            cur.write(root_hash.as_slice()).unwrap();
            cur.write(file_hash.as_slice()).unwrap();
        }
        data
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

    pub fn read_in_msg_descr(&self) -> Result<InMsgDescr> {
        self.in_msg_descr.read_struct()
    }

    pub fn write_in_msg_descr(&mut self, value: &InMsgDescr) -> Result<()> {
        self.in_msg_descr.write_struct(value)
    }

    pub fn in_msg_descr_cell(&self) -> &Cell {
        self.in_msg_descr.cell()
    }

    pub fn read_out_msg_descr(&self) -> Result<OutMsgDescr> {
        self.out_msg_descr.read_struct()
    }

    pub fn write_out_msg_descr(&mut self, value: &OutMsgDescr) -> Result<()> {
        self.out_msg_descr.write_struct(value)
    }

    pub fn out_msg_descr_cell(&self) -> &Cell {
        self.out_msg_descr.cell()
    }

    pub fn read_account_blocks(&self) -> Result<ShardAccountBlocks> {
        self.account_blocks.read_struct()
    }

    pub fn write_account_blocks(&mut self, value: &ShardAccountBlocks) -> Result<()> {
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

    pub fn read_custom(&self) -> Result<Option<McBlockExtra>> {
        Ok(
            match self.custom {
                Some(ref custom) => Some(custom.read_struct()?),
                None => None
            }
        )
    }

    pub fn write_custom(&mut self, value: Option<McBlockExtra>) -> Result<()> {
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
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_u32()?;
        if tag != BLOCK_EXTRA_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag,
                    s: "BlockExtra".to_string()
                }
            )
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
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
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
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExtBlkRef {
    pub end_lt: u64,
    pub seq_no: u32,
    pub root_hash: UInt256,
    pub file_hash: UInt256,
}

impl Deserializable for ExtBlkRef {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.end_lt = cell.get_next_u64()?;
        self.seq_no = cell.get_next_u32()?;
        self.root_hash.read_from(cell)?;
        self.file_hash.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ExtBlkRef {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.end_lt.write_to(cell)?;
        self.seq_no.write_to(cell)?;
        self.root_hash.write_to(cell)?;
        self.file_hash.write_to(cell)?;
        Ok(())
    }
}

pub const MAX_SPLIT_DEPTH: u8 = 60;
pub const MASTERCHAIN_ID: i32 = -1;
pub const BASE_WORKCHAIN_ID: i32 = 0;
pub const INVALID_WORKCHAIN_ID: i32 = 0x8000_0000u32 as i32;
pub const SHARD_FULL: u64 = 0x8000_0000_0000_0000u64;

/*
shard_ident$00 
    shard_pfx_bits: (#<= 60)
    workchain_id: int32
    shard_prefix: uint64
= ShardIdent;
*/
#[derive(Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ShardIdent {
    workchain_id: i32,
    prefix: u64, // with terminated bit!
}

impl Default for ShardIdent {
    fn default() -> Self {
        ShardIdent {
            workchain_id: 0,
            prefix: SHARD_FULL,
        }
    }
}

impl ShardIdent {
    pub fn masterchain() -> Self {
        ShardIdent {
            workchain_id: MASTERCHAIN_ID,
            prefix: SHARD_FULL,
        }
    }
    pub fn with_prefix_len(shard_pfx_len: u8, workchain_id: i32, shard_prefix: u64) -> Result<Self> {
        if shard_pfx_len > MAX_SPLIT_DEPTH {
            fail!(BlockError::InvalidArg(
                    format!("Shard prefix length can't greater than {}", MAX_SPLIT_DEPTH)
            ))
        }
        Self::check_workchain_id(workchain_id)?;
        Ok(
            ShardIdent {
                workchain_id,
                prefix: Self::add_tag(shard_prefix, shard_pfx_len),
            }
        )
    }

    pub fn with_tagged_prefix(workchain_id: i32, shard_prefix_tagged: u64) -> Result<Self> {
        if (shard_prefix_tagged & (!0 >> (MAX_SPLIT_DEPTH + 1))) != 0 {
            fail!(
                BlockError::InvalidArg(
                    format!("Shard prefix can't longer than {}", MAX_SPLIT_DEPTH)
                )
            )
        }
        Self::check_workchain_id(workchain_id)?;
        Ok(
            ShardIdent {
                workchain_id,
                prefix: shard_prefix_tagged,
            }
        )
    }

    pub fn with_prefix_slice(workchain_id: i32, mut shard_prefix_slice: SliceData) -> Result<Self> {
        let mut shard_pfx_bits = 0;
        let mut shard_prefix = 0;
        while let Ok(bit) = shard_prefix_slice.get_next_bit_int() {
            shard_pfx_bits += 1;
            shard_prefix = shard_prefix | ((bit as u64) << 64 - shard_pfx_bits)
        }
        if shard_pfx_bits > MAX_SPLIT_DEPTH {
            fail!(
                BlockError::InvalidArg(
                    format!("Shard prefix length can't greater than {}", MAX_SPLIT_DEPTH)
                )
            )
        }
        Self::check_workchain_id(workchain_id)?;
        Ok(
            ShardIdent {
                workchain_id,
                prefix: Self::add_tag(shard_prefix, shard_pfx_bits),
            }
        )
    }

    pub fn with_workchain_id(workchain_id: i32) -> Result<Self> {
        Self::check_workchain_id(workchain_id)?;
        Ok(
            Self {
                workchain_id,
                prefix: SHARD_FULL,
            }
        )
    }

    pub fn check_workchain_id(workchain_id: i32) -> Result<()> {
        if workchain_id == INVALID_WORKCHAIN_ID {
            fail!(BlockError::InvalidArg(
                    format!("Workchain id 0x{:x} is invalid", INVALID_WORKCHAIN_ID)
            ))
        }
        Ok(())
    } 

    /// Get bitstring-key for BinTree operation for Shard
    pub fn shard_key(&self) -> SliceData {
        let mut cell = BuilderData::new();
        let mut p = self.prefix;
        debug_assert!(p != 0);
        while p != 1 << 63 {
            cell.append_bit_bool(p >> 63 != 0).unwrap(); // unsafe - cell is longer than 64 bit
            p = p << 1;
        }
        cell.into()
    }

    /// Get bitstring-key for BinTree operation for Shard
    pub fn full_key(&self) -> Result<SliceData> {
        let mut cell = BuilderData::new();
        cell.append_i32(self.workchain_id)?
            .append_u64(self.shard_prefix_without_tag())?;
        Ok(cell.into())
    }

    pub fn workchain_id(&self) -> i32 {
        self.workchain_id
    }

    pub fn is_child_for(&self, parent: &ShardIdent) -> bool {
        parent.is_parent_for(self)
    }

    pub fn is_parent_for(&self, child: &ShardIdent) -> bool {
        let parent = child.merge();
        self.workchain_id() == child.workchain_id() &&
            parent.is_ok() &&
            self.shard_prefix_with_tag() == parent.unwrap().shard_prefix_with_tag()
    }

    pub fn is_ancestor_for(&self, descendant: &ShardIdent) -> bool {
        descendant.prefix != SHARD_FULL &&
        self.workchain_id() == descendant.workchain_id() &&
        (
            self.prefix == SHARD_FULL ||
            ((descendant.prefix & !((self.prefix_lower_bits() << 1) - 1)) == self.shard_prefix_without_tag())
        )
    }

    pub fn can_split(&self) -> bool {
        self.prefix_len() == MAX_SPLIT_DEPTH
    }

    pub fn is_full(&self) -> bool {
        self.prefix == SHARD_FULL
    }

    pub fn is_masterchain(&self) -> bool {
        self.workchain_id == MASTERCHAIN_ID
    }

    pub fn is_masterchain_ext(&self) -> bool {
        self.is_masterchain() && self.is_full()
    }

    pub fn is_base_workchain(&self) -> bool {
        self.workchain_id() == BASE_WORKCHAIN_ID
    }

    pub fn contains_account(&self, mut acc_addr: AccountId) -> Result<bool> {
        Ok(
            if self.prefix == SHARD_FULL {
                true
            } else {
                // compare shard prefix and first bits of address 
                // (take as many bits of the address as the bits in the prefix)
                let len = self.prefix_len();
                let addr_pfx = acc_addr.get_next_int(len as usize)?;
                let shard_pfx = self.prefix >> (64 - len);
                addr_pfx == shard_pfx
            }
        )
    }

    pub fn shard_prefix_as_str_with_tag(&self) -> String {
        format!(
            "{:016x}",
            self.shard_prefix_with_tag()
        )
    }

    pub fn shard_prefix_with_tag(&self) -> u64 {
        self.prefix
    }

    pub fn shard_prefix_without_tag(self) -> u64 {
        self.prefix - self.prefix_lower_bits()
    }

    pub fn merge(&self) -> Result<ShardIdent> {
        let lb = self.prefix_lower_bits();
        if self.prefix == SHARD_FULL {
            fail!(
                BlockError::InvalidArg(
                    format!("Can't merge shard {}", self.shard_prefix_as_str_with_tag())
                )
            )
        } else {
            Ok(
                ShardIdent {
                    workchain_id: self.workchain_id,
                    prefix: (self.prefix - lb) | (lb << 1),
                }
            )
        }
    }

    pub fn split(&self) -> Result<(ShardIdent, ShardIdent)> {
        let lb = self.prefix_lower_bits() >> 1;
        if lb & (!0 >> MAX_SPLIT_DEPTH + 1) != 0 {
            fail!(
                BlockError::InvalidArg(
                    format!("Can't split shard {}, because of max split depth is {}",
                        self.shard_prefix_as_str_with_tag(), MAX_SPLIT_DEPTH)
                )
            )
        } else {
            Ok((
                ShardIdent {
                    workchain_id: self.workchain_id,
                    prefix: self.prefix - lb,
                },
                ShardIdent {
                    workchain_id: self.workchain_id,
                    prefix: self.prefix + lb,
                }
            ))
        }
    }

    // returns all 0 and first 1 from right to left
    // i.e. 1010000 -> 10000
    fn prefix_lower_bits(&self) -> u64 {
        self.prefix & (!self.prefix + 1)
    }

    fn add_tag(prefix: u64, len: u8) -> u64 { prefix | (1 << (63 - len)) }

    fn prefix_len(&self) -> u8 {
        let mut len = 0;
        let mut p = self.prefix;
        while p != (1 << 63) { 
            len = len + 1;
            p = p << 1;
        }
        len
    }
}

impl Display for ShardIdent {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}, {}", self.workchain_id, self.shard_prefix_as_str_with_tag())
    }
}

impl fmt::Debug for ShardIdent {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}, {}", self.workchain_id, self.shard_prefix_as_str_with_tag())
    }
}

impl Deserializable for ShardIdent {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let constructor_and_pfx = cell.get_next_byte()?;
        // check for 2 high bits to be zero
        if constructor_and_pfx & 0xC0 != 0 {
            fail!(
                BlockError::InvalidData(
                    "2 high bits in ShardIdent's first byte have to be zero".to_string()
                )
            )
        }
        let shard_pfx_bits = constructor_and_pfx & 0x3F;
        if shard_pfx_bits > MAX_SPLIT_DEPTH {
            fail!(
                BlockError::InvalidArg(
                    format!("Shard prefix can't longer than {}", MAX_SPLIT_DEPTH)
                )
            )
        }
        let workchain_id = cell.get_next_u32()? as i32;
        let shard_prefix = cell.get_next_u64()?;

        *self = Self::with_prefix_len(shard_pfx_bits, workchain_id, shard_prefix)?;

        Ok(())
    }
}

impl Serializable for ShardIdent {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.prefix_len().write_to(cell)?;
        self.workchain_id.write_to(cell)?;
        self.shard_prefix_without_tag().write_to(cell)?;
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
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
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
                fail!(
                    BlockError::InvalidConstructorTag {
                        t: tag,
                        s: "ShardState".to_string()
                    }
                )
            }
        };

        Ok(())
    }
}

const SHARD_STATE_SPLIT_PFX: u32 = 0x5f327da5;
const SHARD_STATE_UNSPLIT_PFX: u32 = 0x9023afe2;

impl Serializable for ShardState {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        match self {
            ShardState::UnsplitState(ss) => {
                ss.write_to(cell)?;
            }
            ShardState::SplitState(ss) => {
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
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_u32()?;
        if tag != SHARD_STATE_SPLIT_PFX {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag,
                    s: "ShardStateSplit".to_string()
                }
            )
        }
        self.left
            .read_from(&mut cell.checked_drain_reference()?.into())?;
        self.right
            .read_from(&mut cell.checked_drain_reference()?.into())?;
        Ok(())
    }
}

impl Serializable for ShardStateSplit {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
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

    pub fn shard(&self) -> &ShardIdent {
        &self.shard_id
    }

    pub fn shard_mut(&mut self) -> &mut ShardIdent {
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

    pub fn set_min_ref_mc_seqno(&mut self, value: u32) {
        self.min_ref_mc_seqno = value
    }

    pub fn read_out_msg_queue_info(&self) -> Result<OutMsgQueueInfo> {
        self.out_msg_queue_info.read_struct()
    }

    pub fn write_out_msg_queue_info(&mut self, value: &OutMsgQueueInfo) -> Result<()> {
        self.out_msg_queue_info.write_struct(value)
    }

    pub fn before_split(&self) -> bool {
        self.before_split
    }

    pub fn set_before_split(&mut self, value: bool) {
        self.before_split = value
    }

    pub fn read_accounts(&self) -> Result<ShardAccounts> {
        self.accounts.read_struct()
    }

    pub fn write_accounts(&mut self, value: &ShardAccounts) -> Result<()> {
        self.accounts.write_struct(value)
    }
    
    pub fn insert_account(&mut self, account_id: &AccountId, acc: &ShardAccount) -> Result<()> {
        // TODO: split depth
        let depth_balance_info = DepthBalanceInfo::new(0, acc.read_account()?.get_balance().unwrap())?;
        let mut accounts = self.read_accounts()?;
        accounts.set(account_id, acc, &depth_balance_info)?;
        self.write_accounts(&accounts)
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

    pub fn custom_cell(&self) -> Option<&Cell> {
        self.custom.as_ref().map(|c| c.cell())
    }

    pub fn read_custom(&self) -> Result<Option<McStateExtra>> {
        match self.custom {
            None => Ok(None),
            Some(ref custom) => Ok(Some(custom.read_struct()?))
        }
    }

    pub fn write_custom(&mut self, value: Option<McStateExtra>) -> Result<()> {
        self.custom = match value {
            Some(custom) => Some(ChildCell::with_struct(&custom)?),
            None => None
        };
        Ok(())
    }

    pub fn split(&self) -> Result<ShardStateSplit> {
        let mut left = self.clone();
        let mut right = self.clone();
        let (ls, rs) = self.shard().split()?;
        left.shard_id = ls;
        right.shard_id = rs;
        let split_key = self.shard_id.shard_key();
        let info = self.read_out_msg_queue_info()?;
        let (li, ri) = info.split(&split_key)?;
        left.write_out_msg_queue_info(&li)?;
        right.write_out_msg_queue_info(&ri)?;
        let accounts = self.read_accounts()?;
        let (al, ar) = accounts.split(&split_key)?;
        left.write_accounts(&al)?;
        right.write_accounts(&ar)?;
        left.total_balance = al.root_extra().balance().clone();
        right.total_balance = ar.root_extra().balance().clone();
        // debug_assert!(self.master_ref.is_some());
        Ok(ShardStateSplit { left, right })
    }

    pub fn merge_with(&mut self, _other: &ShardStateUnsplit) -> Result<()> {
        unimplemented!()
    }
}

impl Deserializable for ShardStateUnsplit {

    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_u32()?;
        if tag != SHARD_STATE_UNSPLIT_PFX {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: "ShardStateUnsplit".to_string()
                }
            )
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
    fn write_to(&self, builder: &mut BuilderData) -> Result<()> {
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
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {

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

        if self.flags & GEN_SOFTWARE_EXISTS_FLAG != 0 {
            if let Some(gen_software) = self.gen_software.as_ref() {
                gen_software.write_to(cell)?;
            } else {
                fail!(BlockError::InvalidData("GEN_SOFTWARE_EXISTS_FLAG is set but gen_software is None".to_string()))
            }
        } else if self.gen_software.is_some() {
            fail!(BlockError::InvalidData("GEN_SOFTWARE_EXISTS_FLAG is not set but gen_software is Some".to_string()))
        }

        if let Some(ref master) = self.master_ref {
            cell.append_reference(master.write_to_new_cell()?);
        }
        cell.append_reference(self.prev_ref.write_to_new_cell()?);
        if let Some(prev_vert_ref) = self.prev_vert_ref.as_ref() {
            cell.append_reference(prev_vert_ref.write_to_new_cell()?);
        }

        Ok(())
    }
}

const VALUE_FLOW_TAG: u32 = 0xb8e48dfb;

impl Serializable for ValueFlow {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
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
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_u32()?;
        if tag != VALUE_FLOW_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag,
                    s: "ValueFlow".to_string()
                }
            )
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
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_u32()?;
        if tag != BLOCK_INFO_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag,
                    s: "BlockInfo".to_string()
                }
            )
        }
        self.version = cell.get_next_u32()?;
        
        let next_byte = cell.get_next_byte()?;
        let not_master = (next_byte >> 7) & 1 == 1;
        let after_merge = (next_byte >> 6) & 1 == 1;
        self.before_split = (next_byte >> 5) & 1 == 1;
        self.after_split = (next_byte >> 4) & 1 == 1;
        self.want_split = (next_byte >> 3) & 1 == 1;
        self.want_merge = (next_byte >> 2) & 1 == 1;
        self.key_block = (next_byte >> 1) & 1 == 1;
        let vert_seqno_incr = ((next_byte) & 1) as u32;

        self.flags = cell.get_next_byte()?;
        let seq_no = cell.get_next_u32()?;
        self.set_seq_no(seq_no)?;
        let vert_seq_no = cell.get_next_u32()?;
        self.shard.read_from(cell)?;
        self.gen_utime.0 = cell.get_next_u32()?;
        self.start_lt = cell.get_next_u64()?;
        self.end_lt = cell.get_next_u64()?;
        self.gen_validator_list_hash_short = cell.get_next_u32()?;
        self.gen_catchain_seqno = cell.get_next_u32()?;
        self.min_ref_mc_seqno = cell.get_next_u32()?;
        self.prev_key_block_seqno = cell.get_next_u32()?;

        if self.flags & GEN_SOFTWARE_EXISTS_FLAG != 0 {
            self.gen_software = Some(GlobalVersion::construct_from(cell)?);
        }

        self.master_ref = if not_master {
            let mut bli = BlkMasterInfo::default();
            bli.read_from(&mut cell.checked_drain_reference()?.into())?;
            Some(ChildCell::with_struct(&bli)?)
        } else { 
            None
        };

        let mut prev_ref = if after_merge {
            BlkPrevInfo::default_blocks() 
        } else { 
            BlkPrevInfo::default_block()
        };
        prev_ref.read_from(&mut cell.checked_drain_reference()?.into())?;
        self.set_prev_stuff(after_merge, &prev_ref)?;

        let prev_vert_ref = if vert_seq_no == 0 {
            None
        } else {
            Some(BlkPrevInfo::construct_from(&mut cell.checked_drain_reference()?.into())?)
        };
        self.set_vertical_stuff(vert_seqno_incr, vert_seq_no, prev_vert_ref)?;

        Ok(())
    }
}

impl Deserializable for Block {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_u32()?;
        if tag != BLOCK_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag,
                    s: "Block".to_string()
                }
            )
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
    fn write_to(&self, builder: &mut BuilderData) -> Result<()> {
        builder.append_u32(BLOCK_TAG)?;
        builder.append_i32(self.global_id)?;
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
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
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
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        let tag = slice.get_next_byte()?;
        if tag != TOP_BLOCK_DESCR_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag.into(),
                    s: "TopBlockDescr".to_string()
                }
            )
        }
        self.proof_for.read_from(slice)?;
        self.signatures = BlockSignatures::read_maybe_from(slice)?;
        let len = slice.get_next_int(3)?;
        {
            let mut slice = slice.clone();
            for i in (0..len).rev() {
                if slice.remaining_references() == 0 {
                    fail!(BlockError::TvmException(ExceptionCode::CellUnderflow))
                }
                self.chain.push(slice.checked_drain_reference()?.clone());
                if i != 0 {
                    if slice.remaining_references() == 0 {
                        fail!(BlockError::TvmException(ExceptionCode::CellUnderflow))
                    }
                    slice = slice.checked_drain_reference()?.into();
                }
            }
        }
        Ok(())
    }
}
