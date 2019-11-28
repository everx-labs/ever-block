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

use ton_types::{BuilderData, CellType, SliceData};
use {ExceptionCode, UInt256};
use super::*;
use dictionary::HashmapE;
use std::cmp::Ordering;
use std::sync::RwLock;


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

/* Blockchain 5.1.6 (outdated)

TL-B from Lite Client v11:

block_info version:uint32
  not_master:(## 1)
  after_merge:(## 1) before_split:(## 1)
  after_split:(## 1)
  want_split:Bool want_merge:Bool
  key_block:Bool vert_seqno_incr:(## 1)
  flags:(## 8)
  seq_no:# vert_seq_no:# { vert_seq_no >= vert_seqno_incr }
  { prev_seq_no:# } { ~prev_seq_no + 1 = seq_no }
  shard:ShardIdent gen_utime:uint32
  start_lt:uint64 end_lt:uint64
  gen_validator_list_hash_short:uint32
  gen_catchain_seqno:uint32
  min_ref_mc_seqno:uint32
  prev_key_block_seqno:uint32
  master_ref:not_master?^BlkMasterInfo
  prev_ref:^(BlkPrevInfo after_merge)
  prev_vert_ref:vert_seq_no?^(BlkPrevInfo 0)
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
    pub flags: u16, // 8 bit
    pub seq_no: u32,
    pub vert_seq_no: u32,
    pub prev_seq_no: u32,
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
            prev_seq_no: 0,
            shard: ShardIdent::default(),
            gen_utime: UnixTime32::default(),
            start_lt: 0,
            end_lt: 0,
            gen_validator_list_hash_short: 0,
            gen_catchain_seqno: 0,
            min_ref_mc_seqno: 0,
            prev_key_block_seqno: 0,
            master_ref: None,
            prev_ref: BlkPrevInfo::default(),
            prev_vert_ref: None,
        }
    }
}

impl BlockInfo {
    pub fn first(shard: ShardIdent, root_hash: UInt256, file_hash: UInt256) -> Self {
        Self::with_shard_ident_and_seq_no(
            shard,
            1,
            BlkPrevInfo {
                prev: ExtBlkRef {
                    end_lt: 0,
                    seq_no: 0,
                    root_hash,
                    file_hash,
                },
                prev_alt: None,
            },
            0,
            None,
        )
    }

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
        let mut info = BlockInfo::default();
        info.seq_no = seq_no;
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
prev_blk_info {merged:#} prev:ExtBlkRef
    prev_alt:merged?ExtBlkRef = BlkPrevInf merged;
*/
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BlkPrevInfo {
    pub prev: ExtBlkRef,
    pub prev_alt: Option<ExtBlkRef>, // depends on `merged` implicit field
}

impl BlkPrevInfo {
    // Creates new instance of struct based on given block and its hash
    pub fn with_prev_block(block: &Block, block_hash: UInt256) -> Self {
        BlkPrevInfo {
            prev: ExtBlkRef {
                end_lt: block.info().end_lt,
                seq_no: block.info().seq_no,
                root_hash: block_hash,
                file_hash: UInt256::default(),
            },
            prev_alt: None,
        }
    }
}

pub type BlockId = UInt256;

/*
unsigned_block info:^BlockInfo value_flow:^ValueFlow
    state_update:^(MERKLE_UPDATE ShardState)
    extra:^BlockExtra = Block;
*/
#[derive(Debug, Default)]
pub struct Block {
    // next 2 fields are stored only in JSON representation and not saved into BOC
    id: Option<UInt256>,
    pub status: BlockProcessingStatus,

    pub global_id: u32,
    pub info: BlockInfo,            // reference
    pub value_flow: ValueFlow,      // reference
    pub state_update: MerkleUpdate, // reference
    pub extra: BlockExtra,          // reference

    root_cell: RwLock<Option<SliceData>>,
}

impl GenericId for Block {
    fn id_mut_internal(&mut self) -> &mut Option<UInt256> {
        &mut self.id
    }

    fn id_internal(&self) -> Option<&UInt256> {
        self.id.as_ref()
    }
}

impl Clone for Block {
    fn clone(&self) -> Self {
        Block {
            id: self.id.clone(),
            status: self.status.clone(),
            global_id: self.global_id.clone(),
            info: self.info.clone(),
            value_flow: self.value_flow.clone(),
            state_update: self.state_update.clone(),
            extra: self.extra.clone(),
            root_cell: self.clone_root_cell(),
        }
    }
}

impl Block {
    pub fn with_params(
        global_id: u32,
        info: BlockInfo,
        value_flow: ValueFlow,
        state_update: MerkleUpdate,
        extra: BlockExtra,
    ) -> Self {
        Block {
            id: None,
            status: BlockProcessingStatus::default(),
            global_id,
            info,
            value_flow,
            extra,
            state_update,
            root_cell: RwLock::new(None),
        }
    }

    pub fn global_id(&self) -> u32 {
        self.global_id
    }

    pub fn set_global_id(&mut self, global_id: u32) {
        self.reset_root_cell();
        self.global_id = global_id
    }

    pub fn info(&self) -> &BlockInfo {
        &self.info
    }

    pub fn info_mut(&mut self) -> &mut BlockInfo {
        self.reset_root_cell();
        &mut self.info
    }

    pub fn value_flow(&self) -> &ValueFlow {
        &self.value_flow
    }

    pub fn value_flow_mut(&mut self) -> &mut ValueFlow {
        self.reset_root_cell();
        &mut self.value_flow
    }

    pub fn state_update(&self) -> &MerkleUpdate {
        &self.state_update
    }

    pub fn state_update_mut(&mut self) -> &mut MerkleUpdate {
        self.reset_root_cell();
        &mut self.state_update
    }

    pub fn extra(&self) -> &BlockExtra {
        &self.extra
    }

    pub fn extra_mut(&mut self) -> &mut BlockExtra {
        self.reset_root_cell();
        &mut self.extra
    }

    pub fn read_info_from(slice: &mut SliceData) -> BlockResult<BlockInfo> {
        let tag = slice.get_next_u32()?;
        if tag != BLOCK_TAG {
            bail!(BlockErrorKind::InvalidConstructorTag(tag, "Block".into()))
        }
        BlockInfo::construct_from(&mut slice.checked_drain_reference()?.into())
    }

    pub fn read_extra_slice_from(slice: &mut SliceData) -> BlockResult<SliceData> {
        let tag = slice.get_next_u32()?;
        if tag != BLOCK_TAG {
            bail!(BlockErrorKind::InvalidConstructorTag(tag, "Block".into()))
        }
        slice.checked_drain_reference()?;
        slice.checked_drain_reference()?;
        slice.checked_drain_reference()?;
        Ok(slice.checked_drain_reference()?.into())
    }
}

impl Ord for Block {
    fn cmp(&self, other: &Block) -> Ordering {
        self.info.seq_no.cmp(&other.info.seq_no)
    }
}

impl PartialOrd for Block {
    fn partial_cmp(&self, other: &Block) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Block {
    fn eq(&self, other: &Block) -> bool {
        self.info.seq_no() == other.info.seq_no()
    }
}

impl Eq for Block {}

/// block_extra in_msg_descr:^InMsgDescr
///     out_msg_descr:^OutMsgDescr
///     account_blocks:^ShardAccountBlocks
///     rand_seed:bits256
///     created_by:bits256
///     custom:(Maybe ^McBlockExtra)
/// = BlockExtra;
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BlockExtra {
    in_msg_descr: InMsgDescr, // reference
    out_msg_descr: OutMsgDescr, // reference
    pub account_blocks: ShardAccountBlocks,
    pub rand_seed: UInt256,
    pub created_by: UInt256,
    custom: Option<InRefValue<McBlockExtra>>, // Maybe reference // TODO write to DB (skipped for now)
}

impl BlockExtra {
    pub fn new() -> BlockExtra {
        let rand_seed: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();
        BlockExtra {
            in_msg_descr: InMsgDescr::default(),
            out_msg_descr: OutMsgDescr::default(),
            account_blocks: ShardAccountBlocks::default(),
            rand_seed: UInt256::from(rand_seed),
            created_by: UInt256::default(), // TODO: Need to fill?
            custom: None,
        }
    }

    pub fn in_msg_descr(&self) -> &InMsgDescr {
        &self.in_msg_descr
    }

    pub fn in_msg_descr_mut(&mut self) -> &mut InMsgDescr {
        &mut self.in_msg_descr
    }

    pub fn out_msg_descr(&self) -> &OutMsgDescr {
        &self.out_msg_descr
    }

    pub fn out_msg_descr_mut(&mut self) -> &mut OutMsgDescr {
        &mut self.out_msg_descr
    }

    pub fn account_blocks(&self) -> &ShardAccountBlocks {
        &self.account_blocks
    }

    pub fn account_blocks_mut(&mut self) -> &mut ShardAccountBlocks {
        &mut self.account_blocks
    }

    pub fn custom(&self) -> Option<&McBlockExtra> {
        self.custom.as_ref().map(|InRefValue(extra)| extra)
    }

    pub fn set_custom(&mut self, extra: McBlockExtra) {
        self.custom = Some(InRefValue(extra))
    }

    pub fn read_in_msg_descr_from(slice: &mut SliceData) -> BlockResult<InMsgDescr> {
        let tag = slice.get_next_u32()?;
        if tag != BLOCK_EXTRA_TAG {
            bail!(BlockErrorKind::InvalidConstructorTag(
                tag,
                "BlockExtra".into()
            ))
        }
        let im_cell = slice.checked_drain_reference()?;
        if im_cell.cell_type() == CellType::PrunedBranch {
            bail!(BlockErrorKind::PrunedCellAccess("InMsgDescr".into()))
        }
        Ok(InMsgDescr::construct_from(&mut im_cell.into())?)
    }

    pub fn read_out_msg_descr_from(slice: &mut SliceData) -> BlockResult<OutMsgDescr> {
        let tag = slice.get_next_u32()?;
        if tag != BLOCK_EXTRA_TAG {
            bail!(BlockErrorKind::InvalidConstructorTag(
                tag,
                "BlockExtra".into()
            ))
        }
        slice.checked_drain_reference()?;
        let om_cell = slice.checked_drain_reference()?;
        if om_cell.cell_type() == CellType::PrunedBranch {
            bail!(BlockErrorKind::PrunedCellAccess("OutMsgDescr".into()))
        }
        Ok(OutMsgDescr::construct_from(&mut om_cell.into())?)
    }

    pub fn read_account_blocks_from(slice: &mut SliceData) -> BlockResult<ShardAccountBlocks> {
        let tag = slice.get_next_u32()?;
        if tag != BLOCK_EXTRA_TAG {
            bail!(BlockErrorKind::InvalidConstructorTag(
                tag,
                "BlockExtra".into()
            ))
        }
        slice.checked_drain_reference()?;
        slice.checked_drain_reference()?;
        let ab_cell = slice.checked_drain_reference()?;
        if ab_cell.cell_type() == CellType::PrunedBranch {
            bail!(BlockErrorKind::PrunedCellAccess(
                "ShardAccountBlocks".into()
            ))
        }
        Ok(ShardAccountBlocks::construct_from(&mut ab_cell.into())?)
    }
}

const BLOCK_EXTRA_TAG: u32 = 0x4a33f6fd;

impl Deserializable for BlockExtra {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        let tag = cell.get_next_u32()?;
        if tag != BLOCK_EXTRA_TAG {
            bail!(BlockErrorKind::InvalidConstructorTag(
                tag,
                "BlockExtra".into()
            ))
        }
        self.in_msg_descr
            .read_from(&mut cell.checked_drain_reference()?.into())?;
        self.out_msg_descr
            .read_from(&mut cell.checked_drain_reference()?.into())?;
        self.account_blocks
            .read_from(&mut cell.checked_drain_reference()?.into())?;
        self.rand_seed.read_from(cell)?;
        self.created_by.read_from(cell)?;
        self.custom = InRefValue::<McBlockExtra>::read_maybe_from(cell)?;
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
        self.custom.write_maybe_to(cell)?;
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

impl BlkPrevInfo {
    // By the time we have to read a BlkPrevInfo, we already know its `merged` flag
    fn read_from(&mut self, cell: &mut SliceData, merged: bool) -> BlockResult<()> {
        if merged {
            self.prev
                .read_from(&mut cell.checked_drain_reference()?.into())?;
            self.prev_alt = Some(ExtBlkRef::construct_from(
                &mut cell.checked_drain_reference()?.into(),
            )?);
        } else {
            self.prev.read_from(cell)?;
            self.prev_alt = None;
        }
        Ok(())
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
            bail!(BlockErrorKind::InvalidData(
                "2 high bits in ShardIdent's first byte have to be zero".into()
            ))
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
                return block_err!(BlockErrorKind::InvalidConstructorTag(
                    tag,
                    "ShardState".into()
                ))
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
            bail!(BlockErrorKind::InvalidConstructorTag(
                tag,
                "ShardStateSplit".into()
            ))
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
#[derive(Debug)]
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

    root_cell: RwLock<Option<SliceData>>,
}

impl Clone for ShardStateUnsplit {
    fn clone(&self) -> Self {
        ShardStateUnsplit {
            global_id: self.global_id.clone(),
            shard_id: self.shard_id.clone(),
            seq_no: self.seq_no.clone(),
            vert_seq_no: self.vert_seq_no.clone(),
            gen_time: self.gen_time.clone(),
            gen_lt: self.gen_lt.clone(),
            min_ref_mc_seqno: self.min_ref_mc_seqno.clone(),
            out_msg_queue_info: self.out_msg_queue_info.clone(),
            before_split: self.before_split.clone(),
            accounts: self.accounts.clone(),
            overload_history: self.overload_history.clone(),
            underload_history: self.underload_history.clone(),
            total_balance: self.total_balance.clone(),
            total_validator_fees: self.total_validator_fees.clone(),
            libraries: self.libraries.clone(),
            master_ref: self.master_ref.clone(),
            custom: self.custom.clone(),
            root_cell: self.clone_root_cell(),
        }
    }
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
        self.reset_root_cell();
        self.global_id = value
    }

    pub fn shard_id(&self) -> &ShardIdent {
        &self.shard_id
    }

    pub fn shard_id_mut(&mut self) -> &mut ShardIdent {
        self.reset_root_cell();
        &mut self.shard_id
    }

    pub fn seq_no(&self) -> u32 {
        self.seq_no
    }

    pub fn set_seq_no(&mut self, seq_no: u32) {
        self.reset_root_cell();
        assert!(seq_no != 0);
        self.seq_no = seq_no
    }

    pub fn vert_seq_no(&self) -> u32 {
        self.vert_seq_no
    }

    pub fn set_vert_seq_no(&mut self, value: u32) {
        self.reset_root_cell();
        self.vert_seq_no = value
    }

    pub fn gen_time(&self) -> u32 {
        self.gen_time
    }

    pub fn set_gen_time(&mut self, value: u32) {
        self.reset_root_cell();
        self.gen_time = value
    }

    pub fn gen_lt(&self) -> u64 {
        self.gen_lt
    }

    pub fn set_gen_lt(&mut self, value: u64) {
        self.reset_root_cell();
        self.gen_lt = value
    }

    pub fn min_ref_mc_seqno(&self) -> u32 {
        self.min_ref_mc_seqno
    }

    pub fn set_(&mut self, value: u32) {
        self.reset_root_cell();
        self.min_ref_mc_seqno = value
    }

    pub fn read_out_msg_queue_info(&self) -> BlockResult<OutMsgQueueInfo> {
        self.out_msg_queue_info.read_struct()
    }

    pub fn write_out_msg_queue_info(&mut self, value: OutMsgQueueInfo) -> BlockResult<()> {
        self.out_msg_queue_info.write_struct(value)
    }

    pub fn before_split(&self) -> bool {
        self.before_split
    }

    pub fn set_before_split(&mut self, value: bool) {
        self.reset_root_cell();
        self.before_split = value
    }

    pub fn read_accounts(&self) -> BlockResult<ShardAccounts> {
        self.accounts.read_struct()
    }

    pub fn write_accounts(&mut self, value: ShardAccounts) -> BlockResult<()> {
        self.accounts.write_struct(value)
    }

    pub fn overload_history(&self) -> u64 {
        self.overload_history
    }

    pub fn set_overload_history(&mut self, value: u64) {
        self.reset_root_cell();
        self.overload_history = value
    }

    pub fn underload_history(&self) -> u64 {
        self.underload_history
    }

    pub fn set_underload_history(&mut self, value: u64) {
        self.reset_root_cell();
        self.underload_history = value
    }

    pub fn total_balance(&self) -> &CurrencyCollection {
        &self.total_balance
    }

    pub fn total_balance_mut(&mut self) -> &mut CurrencyCollection {
        self.reset_root_cell();
        &mut self.total_balance
    }

    pub fn total_validator_fees(&self) -> &CurrencyCollection {
        &self.total_validator_fees
    }

    pub fn total_validator_fees_mut(&mut self) -> &mut CurrencyCollection {
        self.reset_root_cell();
        &mut self.total_validator_fees
    }

    pub fn libraries(&self) -> &HashmapE {
        &self.libraries
    }

    pub fn libraries_mut(&mut self) -> &mut HashmapE {
        self.reset_root_cell();
        &mut self.libraries
    }

    pub fn master_ref(&self) -> Option<&BlkMasterInfo> {
        self.master_ref.as_ref()
    }

    pub fn master_ref_mut(&mut self) -> Option<&mut BlkMasterInfo> {
        self.reset_root_cell();
        self.master_ref.as_mut()
    }

    pub fn read_custom(&self) -> BlockResult<Option<McStateExtra>> {
        match self.custom {
            None => Ok(None),
            Some(ref custom) => Ok(Some(custom.read_struct()?))
        }
    }

    pub fn write_custom(&mut self, value: Option<McStateExtra>) -> BlockResult<()> {
        self.reset_root_cell();
        self.custom = match value {
            Some(custom) => Some(ChildCell::with_struct(custom)?),
            None => None
        };
        Ok(())
    }
}

impl PartialEq for ShardStateUnsplit {
    fn eq(&self, other: &Self) -> bool {
        self.global_id == other.global_id
            && self.shard_id == other.shard_id
            && self.seq_no == other.seq_no
            && self.vert_seq_no == other.vert_seq_no
            && self.gen_time == other.gen_time
            && self.gen_lt == other.gen_lt
            && self.min_ref_mc_seqno == other.min_ref_mc_seqno
            && self.out_msg_queue_info == other.out_msg_queue_info
            && self.before_split == other.before_split
            && self.accounts == other.accounts
            && self.overload_history == other.overload_history
            && self.underload_history == other.underload_history
            && self.total_balance == other.total_balance
            && self.total_validator_fees == other.total_validator_fees
            && self.libraries == other.libraries
            && self.master_ref == other.master_ref
            && self.custom == other.custom
    }
}
impl Eq for ShardStateUnsplit {}

impl LazySerializable for ShardStateUnsplit {
    fn root_cell(&self) -> &RwLock<Option<SliceData>> {
        &self.root_cell
    }

    fn do_read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        let tag = cell.get_next_u32()?;
        if tag != SHARD_STATE_UNSPLIT_PFX {
            bail!(BlockErrorKind::InvalidConstructorTag(
                tag as u32,
                "ShardStateUnsplit".into()
            ));
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

    fn do_write_to(&self, builder: &mut BuilderData) -> BlockResult<()> {
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
            out_msg_queue_info: ChildCell::with_struct(OutMsgQueueInfo::default()).unwrap(),
            before_split: false,
            accounts: ChildCell::with_struct(ShardAccounts::default()).unwrap(),
            overload_history: 0,
            underload_history: 0,
            total_balance: CurrencyCollection::default(),
            total_validator_fees: CurrencyCollection::default(),
            libraries: HashmapE::with_bit_len(256), // <AccountId, LibDescr>, // currently can be present only in masterchain blocks.
            master_ref: None,
            custom: None,
            root_cell: RwLock::new(None),
        }
    }
}

const BLOCK_TAG: u32 = 0x11ef55aa;

const BLOCK_INFO_TAG: u32 = 0x9bc7a987;

impl Serializable for BlockInfo {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        assert!(self.seq_no != 0);
        assert!((self.vert_seq_no == 0) ^ self.prev_vert_ref.is_some());

        let mut flags = self.flags & !0b1111_1111_0000_0000u16;
        if self.master_ref.is_some() {
            flags |= 1 << 15
        }
        if self.after_merge {
            flags |= 1 << 14
        }
        if self.before_split {
            flags |= 1 << 13;
        }
        if self.after_split {
            flags |= 1 << 12;
        }
        if self.want_split {
            flags |= 1 << 11;
        }
        if self.want_merge {
            flags |= 1 << 10;
        }
        if self.key_block {
            flags |= 1 << 9;
        }
        if self.vert_seqno_incr != 0 {
            flags |= 1 << 8;
        }
        cell.append_u32(BLOCK_INFO_TAG)?
            .append_u32(self.version)?
            .append_u16(flags)?
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
            bail!(BlockErrorKind::InvalidConstructorTag(
                tag,
                "ValueFlow".into()
            ))
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

impl Serializable for BlkPrevInfo {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        match &(self.prev_alt) {
            Some(prev_alt) => {
                cell.append_reference(self.prev.write_to_new_cell()?);
                cell.append_reference(prev_alt.write_to_new_cell()?);
            }
            None => {
                self.prev.write_to(cell)?;
            }
        }
        Ok(())
    }
}

impl Deserializable for BlockInfo {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        let tag = cell.get_next_u32()?;
        if tag != BLOCK_INFO_TAG {
            bail!(BlockErrorKind::InvalidConstructorTag(
                tag,
                "BlockInfo".into()
            ))
        }
        self.version = cell.get_next_u32()?;
        self.flags = cell.get_next_u16()?;
        let not_master = (self.flags >> 15) & 1 == 1;
        self.after_merge = (self.flags >> 14) & 1 == 1;
        self.before_split = (self.flags >> 13) & 1 == 1;
        self.after_split = (self.flags >> 12) & 1 == 1;
        self.want_split = (self.flags >> 11) & 1 == 1;
        self.want_merge = (self.flags >> 10) & 1 == 1;
        self.key_block = (self.flags >> 9) & 1 == 1;
        self.vert_seqno_incr = (self.flags as u32 >> 8) & 1;
        self.seq_no = cell.get_next_u32()?;
        self.vert_seq_no = cell.get_next_u32()?;
        if self.vert_seqno_incr > self.vert_seq_no {
            bail!(BlockErrorKind::InvalidData(format!(
                "BlockInfo {} < {}",
                self.vert_seqno_incr, self.vert_seq_no
            )))
        }
        if self.seq_no < 1 {
            bail!(BlockErrorKind::InvalidData(format!(
                "BlockInfo {}",
                self.seq_no
            )))
        }
        self.prev_seq_no = self.seq_no - 1;
        self.shard.read_from(cell)?;
        self.gen_utime.0 = cell.get_next_u32()?;
        self.start_lt = cell.get_next_u64()?;
        self.end_lt = cell.get_next_u64()?;
        self.gen_validator_list_hash_short = cell.get_next_u32()?;
        self.gen_catchain_seqno = cell.get_next_u32()?;
        self.min_ref_mc_seqno = cell.get_next_u32()?;
        self.prev_key_block_seqno = cell.get_next_u32()?;

        self.master_ref = match not_master {
            true => {
                let mut bli = BlkMasterInfo::default();
                bli.read_from(&mut cell.checked_drain_reference()?.into())?;
                Some(bli)
            }
            false => None,
        };

        // prev_ref:^(BlkPrevInfo after_merge)
        self.prev_ref = BlkPrevInfo::default();
        self.prev_ref.read_from(
            &mut cell.checked_drain_reference()?.into(),
            self.after_merge,
        )?;
        // prev_vert_ref:vert_seq_no?^(BlkPrevInfo 0)
        self.prev_vert_ref = match self.vert_seq_no {
            0 => None,
            _ => {
                let mut bpi = BlkPrevInfo::default();
                bpi.read_from(&mut cell.checked_drain_reference()?.into(), false)?;
                Some(bpi)
            }
        };

        Ok(())
    }
}

impl LazySerializable for Block {
    fn root_cell(&self) -> &RwLock<Option<SliceData>> {
        &self.root_cell
    }

    fn do_read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        let tag = cell.get_next_u32()?;
        if tag != BLOCK_TAG {
            bail!(BlockErrorKind::InvalidConstructorTag(tag, "Block".into()))
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

    fn do_write_to(&self, builder: &mut BuilderData) -> BlockResult<()> {
        builder.append_u32(BLOCK_TAG)?;
        builder.append_u32(self.global_id)?;
        builder.append_reference(self.info.write_to_new_cell()?); // info:^BlockInfo
        builder.append_reference(self.value_flow.write_to_new_cell()?); // value_flow:^ValueFlow
        builder.append_reference(self.state_update.write_to_new_cell()?); // state_update:^(MERKLE_UPDATE ShardState)
        builder.append_reference(self.extra.write_to_new_cell()?); // extra:^BlockExtra

        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum BlockProcessingStatus {
    Unknown,
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
    chain: Vec<Arc<CellData>>,
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
    pub fn append_proof(&mut self, cell: Arc<CellData>) {
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
            bail!(BlockErrorKind::InvalidConstructorTag(
                tag.into(),
                "TopBlockDescr".into()
            ))
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
