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

use crate::{
    define_HashmapE,
    accounts::ShardAccount,
    envelope_message::IntermediateAddress,
    error::BlockError,
    master::{BlkMasterInfo, LibDescr, McStateExtra},
    messages::MsgAddressInt,
    outbound_messages::OutMsgQueueInfo,
    shard_accounts::{DepthBalanceInfo, ShardAccounts},
    types::{ChildCell, CurrencyCollection},
    Serializable, Deserializable, MaybeSerialize, MaybeDeserialize,
};
use std::fmt::{self, Display, Formatter};
use ton_types::{
    error, fail, Result,
    types::AccountId,
    BuilderData, Cell, HashmapE, HashmapType, IBitstring, SliceData
};


pub const MAX_SPLIT_DEPTH: u8 = 60;
pub const MASTERCHAIN_ID: i32 = -1;
pub const BASE_WORKCHAIN_ID: i32 = 0;
pub const INVALID_WORKCHAIN_ID: i32 = 0x8000_0000u32 as i32;
pub const SHARD_FULL: u64 = 0x8000_0000_0000_0000u64;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountIdPrefixFull {
    pub workchain_id: i32,
    pub prefix: u64,
}

impl Default for AccountIdPrefixFull {
    fn default() -> Self {
        Self {
            workchain_id: INVALID_WORKCHAIN_ID,
            prefix: 0,
        }
    }
}

/// https://www.notion.so/tonlabs/Implement-AccountIdPrefixFull-14fa126991074ec3a06a3ab8d36e8223
impl AccountIdPrefixFull {
    pub fn is_valid(&self) -> bool {
        self.workchain_id != INVALID_WORKCHAIN_ID
    }
    pub fn prefix(_address: &MsgAddressInt) -> Result<Self> {
        unimplemented!("ton::AccountIdPrefixFull MsgAddressInt::get_prefix(vm::CellSlice&& cs)")
    }
    pub fn cheked_prefix(_address: &MsgAddressInt) -> Result<Self> {
        unimplemented!("ton::AccountIdPrefixFull MsgAddressInt::get_prefix(vm::CellSlice&& cs)")
    }
    pub fn interpolate_addr(&self, _to: &Self, _ia: &IntermediateAddress) -> Self {
        unimplemented!()
        // if (d <= 0) {
        //     return src;
        // } else if (d >= 96) {
        //     return dest;
        // } else if (d >= 32) {
        //     unsigned long long mask = (std::numeric_limits<td::uint64>::max() >> (d - 32));
        //     return ton::AccountIdPrefixFull{dest.workchain, (dest.account_id_prefix & ~mask) | (src.account_id_prefix & mask)};
        // } else {
        //     int mask = (-1 >> d);
        //     return ton::AccountIdPrefixFull{(dest.workchain & ~mask) | (src.workchain & mask), src.account_id_prefix};
        // }
    }
    pub fn count_matching_bits(&self, _other: &Self) -> u8 {
        unimplemented!()
    }
}

impl fmt::Display for AccountIdPrefixFull {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.workchain_id, self.prefix)
    }
}

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

    /// It is copy from t-node. TODO: investigate, add comment and tests
    pub fn is_neighbor_for(&self, other: &Self) -> bool {
        if self.is_masterchain() || other.is_masterchain() {
            return true;
        }
        let xs = self.shard_prefix_with_tag();
        let ys = other.shard_prefix_with_tag();
        let xl = self.prefix_lower_bits();
        let yl = other.prefix_lower_bits();
        let z = (xs ^ ys) & (!(std::cmp::max(xl, yl) << 1) + 1);
        if z == 0 {
            return true
        }
        if self.workchain_id() != other.workchain_id() {
            return false
        }
        let c1 = z.leading_zeros() >> 2;
        let c2 = z.trailing_zeros() >> 2;
        c1 + c2 == 15
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

    pub fn contains_full_prefix(&self, prefix: &AccountIdPrefixFull) -> bool {
        self.contains_prefix(prefix.workchain_id, prefix.prefix)
    }

    pub fn contains_prefix(&self, workchain_id: i32, prefix_without_tag: u64) -> bool {
        if self.workchain_id == workchain_id {
            if self.prefix == SHARD_FULL {
                return true
            }
            let shift = 64 - self.prefix_len();
            return self.prefix >> shift == prefix_without_tag >> shift
        }
        false
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

    pub fn sibling(&self) -> ShardIdent {
        let prefix = self.prefix ^ ((self.prefix & (!self.prefix + 1)) << 1);
        Self {
            workchain_id: self.workchain_id,
            prefix,
        }
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

    pub fn minus_one(&self) -> Result<Self> {
        Self::with_tagged_prefix(self.workchain_id, self.prefix - 1)
    }

    pub fn plus_one(&self) -> Result<Self> {
        Self::with_tagged_prefix(self.workchain_id, self.prefix + 1)
    }

    // returns all 0 and first 1 from right to left
    // i.e. 1010000 -> 10000
    fn prefix_lower_bits(&self) -> u64 {
        self.prefix & (!self.prefix + 1)
    }

    fn add_tag(prefix: u64, len: u8) -> u64 { prefix | (1 << (63 - len)) }

    pub fn prefix_len(&self) -> u8 {
        63 - self.prefix.trailing_zeros() as u8
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

define_HashmapE!(Libraries, 256, LibDescr);

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
#[derive(Debug, Default, Clone, Eq, PartialEq)]
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
    libraries: Libraries, // <AccountId, LibDescr>, // currently can be present only in masterchain blocks.
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

    pub fn id(&self) -> String {
        format!("shard: {}, seq_no: {}", self.shard(), self.seq_no)
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

    pub fn libraries(&self) -> &Libraries {
        &self.libraries
    }

    pub fn libraries_mut(&mut self) -> &mut Libraries {
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

    pub fn is_key_state(&self) -> bool {
        self.custom.is_some()
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
        // TODO: other
        Ok(ShardStateSplit { left, right })
    }

    pub fn merge_with(&mut self, other: &ShardStateUnsplit) -> Result<()> {
        self.shard_id = self.shard_id.merge()?;
        let merge_key = self.shard_id.shard_key();
        let mut accounts = self.read_accounts()?;
        accounts.merge(&other.read_accounts()?, &merge_key)?;
        self.write_accounts(&accounts)?;
        self.total_balance = accounts.root_extra().balance().clone();
        // debug_assert!(self.master_ref.is_some());
        // TODO: merge other
        Ok(())
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
