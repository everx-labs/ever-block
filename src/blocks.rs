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
    define_HashmapE,
    config_params::{GlobalVersion, CatchainConfig},
    error::BlockError,
    inbound_messages::InMsgDescr,
    master::{BlkMasterInfo, McBlockExtra},
    merkle_update::MerkleUpdate,
    outbound_messages::OutMsgDescr,
    signature::BlockSignatures,
    shard::ShardIdent,
    transactions::ShardAccountBlocks,
    types::{ChildCell, CurrencyCollection, InRefValue, UnixTime32},
    Serializable, Deserializable, MaybeSerialize, MaybeDeserialize,
    validators::ValidatorSet,
};
use std::{
    cmp::Ordering,
    io::{Cursor, Write},
    fmt::{self, Display, Formatter},
    str::FromStr
};
use ton_types::{
    error, fail, Result,
    ExceptionCode, UInt256, BuilderData, Cell, IBitstring, SliceData, HashmapE, HashmapType,
};


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
#[derive(Clone, Debug, PartialEq, Eq, Default, Hash, Ord, PartialOrd)]
pub struct BlockIdExt {
    pub shard_id: ShardIdent,
    pub seq_no: u32,
    pub root_hash: UInt256,
    pub file_hash: UInt256,
}

impl BlockIdExt {
    /// New instance of BlockIdExt structure
    pub fn new(shard_id: ShardIdent, seq_no: u32) -> Self {
        Self::with_params(shard_id, seq_no, Default::default(), Default::default())
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
    pub fn from_ext_blk(blk: ExtBlkRef) -> Self {
        BlockIdExt {
            shard_id: ShardIdent::masterchain(),
            seq_no: blk.seq_no,
            root_hash: blk.root_hash,
            file_hash: blk.file_hash,
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

    pub fn shard(&self) -> &ShardIdent { &self.shard_id }

    pub fn seq_no(&self) -> u32 { self.seq_no }

    pub fn root_hash(&self) -> &UInt256 {
        &self.root_hash
    }

    pub fn file_hash(&self) -> &UInt256 {
        &self.file_hash
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

impl Display for BlockIdExt {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({}:{}, {}, rh {}, fh {})", 
            self.shard_id.workchain_id(), 
            self.shard_id.shard_prefix_as_str_with_tag(), 
            self.seq_no,
            self.root_hash.to_hex_string(),
            self.file_hash.to_hex_string())
    }
}

impl FromStr for BlockIdExt {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self> {
        // (0:1800000000000000, 1203696, rh 59b6e56610aa5df5e8ee4cc5f1081cd5d08473f10e0899f7763d580b2a635f90, fh 1b4d177339538562d10166d87823783b7e747ee80d85d033459928fd0605a126)
        let mut parts = s.trim_start_matches('(').trim_end_matches(')').split(',');
        let shard_parts = parts
            .next()
            .ok_or_else(|| error!("Can't read shard ident from {}", s))?
            .trim();
        let mut shard_parts = shard_parts.split(':');
        let workchain_id: i32 = shard_parts
            .next()
            .ok_or_else(|| error!("Can't read workchain_id from {}", s))?
            .trim()
            .parse()
            .map_err(|e| error!("Can't read workchain_id from {}: {}", s, e))?;
        let shard = u64::from_str_radix(
            shard_parts.next().ok_or_else(|| error!("Can't read shard from {}", s))?.trim(),
            16
        ).map_err(|e| error!("Can't read shard from {}: {}", s, e))?;
        let seq_no: u32 = parts
            .next()
            .ok_or_else(|| error!("Can't read seq_no from {}", s))?
            .trim()
            .parse()
            .map_err(|e| error!("Can't read seq_no from {}: {}", s, e))?;
        let root_hash = UInt256::from_str(parts
            .next()
            .ok_or_else(|| error!("Can't read root_hash from {}", s))?
            .trim_start_matches(" rh ")
        ).map_err(|e| error!("Can't read root_hash from {}: {}", s, e))?;
        let file_hash = UInt256::from_str(parts
            .next()
            .ok_or_else(|| error!("Can't read file_hash from {}", s))?
            .trim_start_matches(" fh ")
        ).map_err(|e| error!("Can't read file_hash from {}: {}", s, e))?;
        Ok(Self::with_params(
            ShardIdent::with_tagged_prefix(workchain_id, shard)?,
            seq_no,
            root_hash,
            file_hash,
        ))
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
    pub fn prev_ref_cell(&self)-> Cell {
        self.prev_ref.cell()
    }
    pub fn read_prev_ref(&self) -> Result<BlkPrevInfo> {
        let mut prev_ref = if self.after_merge {
            BlkPrevInfo::default_blocks() 
        } else { 
            BlkPrevInfo::default_block()
        };
        prev_ref.read_from(&mut self.prev_ref.cell().into())?;
        Ok(prev_ref)
    }
    pub fn read_prev_ids(&self) -> Result<Vec<BlockIdExt>> {
        let prev = self.read_prev_ref()?;
        if let Some(prev2) = prev.prev2()? {
            let (shard1, shard2) = self.shard.split()?;
            Ok(vec![prev.prev1()?.workchain_block_id(shard1).1, prev2.workchain_block_id(shard2).1])
        } else if self.after_split {
            Ok(vec!(prev.prev1()?.workchain_block_id(self.shard.merge()?).1))
        } else {
            Ok(vec!(prev.prev1()?.workchain_block_id(self.shard.clone()).1))
        }
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

    pub fn new(mut ext_block_refs: Vec<ExtBlkRef>) -> Result<Self> {
        match ext_block_refs.len() {
            2 => {
                let prev1 = ChildCell::with_struct(&ext_block_refs[0])?;
                let prev2 = ChildCell::with_struct(&ext_block_refs[1])?;
                Ok(BlkPrevInfo::Blocks { prev1, prev2 })
            }
            1 => {
                let prev = ext_block_refs.remove(0);
                Ok(BlkPrevInfo::Block{ prev })
            }
            _ => fail!("prev blocks must be 1 or 2")
        }
    }

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
                prev1.read_from_reference(cell)?;
                prev2.read_from_reference(cell)?;
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

    pub fn info_cell(&self)-> Cell {
        self.info.cell()
    }

    pub fn read_value_flow(&self) -> Result<ValueFlow> {
        self.value_flow.read_struct()
    }

    pub fn write_value_flow(&mut self, value: &ValueFlow) -> Result<()> {
        self.value_flow.write_struct(value)
    }

    pub fn value_flow_cell(&self)-> Cell {
        self.value_flow.cell()
    }

    pub fn read_state_update(&self) -> Result<MerkleUpdate> {
        self.state_update.read_struct()
    }

    pub fn write_state_update(&mut self, value: &MerkleUpdate) -> Result<()> {
        self.state_update.write_struct(value)
    }

    pub fn state_update_cell(&self)-> Cell {
        self.state_update.cell()
    }

    pub fn read_extra(&self) -> Result<BlockExtra> {
        self.extra.read_struct()
    }

    pub fn write_extra(&mut self, value: &BlockExtra) -> Result<()> {
        self.extra.write_struct(value)
    }

    pub fn extra_cell(&self)-> Cell {
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

    pub fn read_cur_validator_set_and_cc_conf(&self) -> Result<(ValidatorSet, CatchainConfig)> {
        let custom = self
            .read_extra()?
            .read_custom()?
            .ok_or_else(|| error!(BlockError::InvalidArg(
                "Block doesn't contain `extra->custom` field".to_string()
            )))?;
        let config = custom
            .config()
            .ok_or_else(|| error!(BlockError::InvalidArg(
                "Block doesn't contain `extra->custom->config` field, maybe no key block is used? ".to_string()
            )))?;

        Ok((
            config.validator_set()?,
            config.catchain_config()?
        ))
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

    pub fn in_msg_descr_cell(&self)-> Cell {
        self.in_msg_descr.cell()
    }

    pub fn read_out_msg_descr(&self) -> Result<OutMsgDescr> {
        self.out_msg_descr.read_struct()
    }

    pub fn write_out_msg_descr(&mut self, value: &OutMsgDescr) -> Result<()> {
        self.out_msg_descr.write_struct(value)
    }

    pub fn out_msg_descr_cell(&self)-> Cell {
        self.out_msg_descr.cell()
    }

    pub fn read_account_blocks(&self) -> Result<ShardAccountBlocks> {
        self.account_blocks.read_struct()
    }

    pub fn write_account_blocks(&mut self, value: &ShardAccountBlocks) -> Result<()> {
        self.account_blocks.write_struct(value)
    }

    pub fn account_blocks_cell(&self)-> Cell {
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

    pub fn write_custom(&mut self, value: Option<&McBlockExtra>) -> Result<()> {
        self.custom = match value {
                Some(v) => Some(ChildCell::with_struct(v)?),
                None => None
            };
        Ok(())
    }

    pub fn custom_cell(&self) -> Option<Cell> {
        self.custom.as_ref().map(|c| c.cell())
    }

    pub fn is_key_block(&self) -> bool {
        self.custom.is_some()
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
        self.in_msg_descr.read_from_reference(cell)?;
        self.out_msg_descr.read_from_reference(cell)?;
        self.account_blocks.read_from_reference(cell)?;
        self.rand_seed.read_from(cell)?;
        self.created_by.read_from(cell)?;
        self.custom = if cell.get_next_bit()? {
            Some(ChildCell::<McBlockExtra>::construct_from_reference(cell)?)
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

impl fmt::Display for ValueFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\
            from_prev_blk: {}, \
            to_next_blk: {}, \
            imported: {}, \
            exported: {}, \
            fees_collected: {}, \
            fees_imported: {}, \
            recovered: {}, \
            created: {}, \
            minted: {}",
            self.from_prev_blk,
            self.to_next_blk,
            self.imported,
            self.exported,
            self.fees_collected,
            self.fees_imported,
            self.recovered,
            self.created,
            self.minted
        )
    }
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

impl ExtBlkRef {
    pub fn master_block_id(self) -> (u64, BlockIdExt) {
        (self.end_lt, BlockIdExt::from_ext_blk(self))
    }
    pub fn workchain_block_id(self, shard_id: ShardIdent) -> (u64, BlockIdExt) {
        let block_id = BlockIdExt {
            shard_id,
            seq_no: self.seq_no,
            root_hash: self.root_hash,
            file_hash: self.file_hash,
        };
        (self.end_lt, block_id)
    }
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
            bli.read_from_reference(cell)?;
            Some(ChildCell::with_struct(&bli)?)
        } else { 
            None
        };

        let mut prev_ref = if after_merge {
            BlkPrevInfo::default_blocks() 
        } else { 
            BlkPrevInfo::default_block()
        };
        prev_ref.read_from_reference(cell)?;
        self.set_prev_stuff(after_merge, &prev_ref)?;

        let prev_vert_ref = if vert_seqno_incr == 0 {
            None
        } else {
            Some(BlkPrevInfo::construct_from_reference(cell)?)
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
        self.info.read_from_reference(cell)?;
        self.value_flow.read_from_reference(cell)?;
        self.state_update.read_from_reference(cell)?;
        self.extra.read_from_reference(cell)?;
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
    signatures: Option<InRefValue<BlockSignatures>>,
    chain: Vec<Cell>,
}

impl TopBlockDescr {
    pub fn with_id_and_signatures(
        proof_for: BlockIdExt,
        signatures: BlockSignatures,
    ) -> Self {
        Self {
            proof_for,
            signatures: Some(InRefValue(signatures)),
            chain: vec![],
        }
    }

    pub fn append_proof(&mut self, cell: Cell) {
        self.chain.push(cell);
    }

    pub fn proof_for(&self) -> &BlockIdExt {
        &self.proof_for
    }

    pub fn signatures(&self) -> Option<&BlockSignatures> {
        self.signatures.as_ref().map(|irf| &irf.0)
    }

    pub fn chain(&self) -> &Vec<Cell> {
        &self.chain
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
        cell.append_bits(self.chain.len(), 8)?;
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
        let len = slice.get_next_int(8)?;
        if !(len >= 1 && len <= 8) {
            fail!(
                BlockError::InvalidData(
                    "Failed check: `len >= 1 && len <= 8`".to_string()
                )
            )
        }
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

define_HashmapE!{TopBlockDescrCollection, 96, InRefValue<TopBlockDescr>}
/*
top_block_descr_set#4ac789f3 collection:(HashmapE 96 ^TopBlockDescr) = TopBlockDescrSet;
*/
#[derive(Clone, Debug, Default)]
pub struct TopBlockDescrSet {
    collection: TopBlockDescrCollection
}

impl TopBlockDescrSet {
    pub fn get_top_block_descr(&self, shard: &ShardIdent) -> Result<Option<TopBlockDescr>> {
        match self.collection.0.get(shard.full_key_with_tag()?)? {
            Some(slice) => TopBlockDescr::construct_from_cell(slice.reference(0)?).map(|r| Some(r)),
            None => Ok(None)
        }
    }
    pub fn insert(&mut self, shard: &ShardIdent, descr: &TopBlockDescr) -> Result<()> {
        let key = shard.full_key_with_tag()?;
        let value = descr.write_to_new_cell()?;
        self.collection.0.setref(key, &value.into()).map(|_|())
    }
    pub fn is_empty(&self) -> bool {
        self.collection.is_empty()
    }
    pub fn count(&self, max: usize) -> Result<usize> {
        self.collection.count(max)
    }
    pub fn collection(&self) -> &TopBlockDescrCollection {
        &self.collection
    }
}

const TOPBLOCK_DESCR_SET_TAG: u32 = 0x4ac789f3;

impl Serializable for TopBlockDescrSet {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        let tag = TOPBLOCK_DESCR_SET_TAG;
        cell.append_u32(tag)?;
        self.collection.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for TopBlockDescrSet {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_u32()?;
        if tag != TOPBLOCK_DESCR_SET_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: "TopBlockDescrSet".to_string()
                }
            )
        }
        self.collection = TopBlockDescrCollection::construct_from(cell)?;
        Ok(())
    }
}
