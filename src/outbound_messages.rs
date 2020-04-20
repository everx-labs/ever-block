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
    define_HashmapAugE,
    error::BlockError,
    envelope_message::MsgEnvelope,
    hashmapaug::{Augmentable, HashmapAugE},
    inbound_messages::InMsg,
    messages::{CommonMsgInfo, Message},
    miscellaneous::{IhrPendingInfo, ProcessedInfo},
    shard::{AccountIdPrefixFull, ShardIdent},
    types::{AddSub, ChildCell, CurrencyCollection},
    transactions::Transaction,
    GetRepresentationHash, Serializable, Deserializable,
};
use std::sync::Arc;
use ton_types::{
    error, fail, Result,
    AccountId, UInt256,
    BuilderData, Cell, IBitstring, HashmapType, SliceData
};


/*
        3.3 Outbound message queue and descriptors
 This section discusses OutMsgDescr, the structure representing all outbound
 messages of a block, along with their envelopes and brief descriptions of the
 reasons for including them into OutMsgDescr. This structure also describes
 all modifications of OutMsgQueue, which is a part of the shardchain state.
*/

//constructor tags of InMsg variants (only wrote bits are used (3 or 4))
const OUT_MSG_EXT: u8 = 0b000;
const OUT_MSG_IMM: u8 = 0b010;
const OUT_MSG_NEW: u8 = 0b001;
const OUT_MSG_TR: u8 = 0b011;
const OUT_MSG_DEQ_IMM: u8 = 0b100;
const OUT_MSG_DEQ: u8 = 0b1100;
const OUT_MSG_DEQ_SHORT: u8 = 0b1101;
const OUT_MSG_TRDEQ: u8 = 0b111;


/*
_ enqueued_lt:uint64 out_msg:^MsgEnvelope = EnqueuedMsg;
*/

///
/// EnqueuedMsg structure
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct EnqueuedMsg {
    pub enqueued_lt: u64,
    pub out_msg: ChildCell<MsgEnvelope>
}

impl EnqueuedMsg {
    /// New default instance EnqueuedMsg structure
    pub fn new() -> Self {
        Default::default()
    }

    /// New instance EnqueuedMsg structure
    pub fn with_param(enqueued_lt: u64, out_msg: &MsgEnvelope) -> Result<Self> {
        Ok(EnqueuedMsg {
            enqueued_lt,
            out_msg: ChildCell::with_struct(out_msg)?,
        })
    }

    pub fn enqueued_lt(&self) -> u64 {
        self.enqueued_lt
    }

    pub fn out_msg_cell(&self) -> &Cell {
        self.out_msg.cell()
    }

    pub fn read_out_msg(&self) -> Result<MsgEnvelope> {
        self.out_msg.read_struct()
    }
}

impl Serializable for EnqueuedMsg {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.enqueued_lt.write_to(cell)?;
        cell.append_reference(self.out_msg.write_to_new_cell()?);
        Ok(())
    }
}

impl Deserializable for EnqueuedMsg {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.enqueued_lt.read_from(cell)?;
        self.out_msg.read_from(&mut cell.checked_drain_reference()?.into())?;
        Ok(())
    }
}
/////////////////////////////////////////////////////////////////////////////////////////
// Blockchain: 3.3.5
// _ (HashmapAugE 256 OutMsg CurrencyCollection) = OutMsgDescr;
//
define_HashmapAugE!(OutMsgDescr, 256, OutMsg, CurrencyCollection);

impl OutMsgDescr {
    /// insert new or replace existing
    pub fn insert(&mut self, out_msg: &OutMsg) -> Result<()> {
        if let Some(msg) = out_msg.read_message()? {
            let value = match out_msg {
                OutMsg::External(_) => msg.get_value(),
                OutMsg::Immediately(_) => msg.get_value(),
                OutMsg::New(_) => msg.get_value(),
                OutMsg::Transit(_) => None,
                OutMsg::Dequeue(_) => None,
                OutMsg::DequeueShort(_) => None,
                OutMsg::DequeueImmediately(_) => msg.get_value(),
                OutMsg::TransitRequired(ref _x) => None,
                OutMsg::None => fail!("Try to insert uninited OutMsg")
            };
            self.set(&msg.hash()?, &out_msg, value.unwrap_or(&CurrencyCollection::default()))
        } else if let OutMsg::DequeueShort(msg) = out_msg {
            self.set(&msg.msg_env_hash, &out_msg, &CurrencyCollection::default())
        } else {
            fail!(BlockError::InvalidArg("Unsupported message type".to_string()))
        }
    }

    /// insert or replace existion record
    /// use to improve speed
    pub fn insert_serialized(&mut self, key: &SliceData, msg_slice: &SliceData, exported: &CurrencyCollection ) -> Result<()> {
        if self.0.set(key.clone(), msg_slice, exported).is_ok() {
            Ok(())
        } else {
            fail!(BlockError::Other("Error insert serialized message".to_string()))
        }
    }

    pub fn full_exported(&self) -> &CurrencyCollection {
        &self.root_extra()
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
// Blockchain: 3.3.6
// _ (HashmapAugE 352 EnqueuedMsg uint64) = OutMsgQueue;
// 352 = 32 - dest workchain_id, 64 - first 64 bit of dest account address, 256 - message hash
define_HashmapAugE!(OutMsgQueue, 352, EnqueuedMsg, MsgTime);

type MsgTime = u64;

impl Augmentable for MsgTime {
    fn calc(&mut self, other: &Self) -> Result<()> {
        if *self > *other {
            *self = *other;
        }
        Ok(())
    }
}

impl OutMsgQueue {
    /// insert OutMessage to OutMsgQueue
    pub fn insert(&mut self, workchain_id: i32, prefix: u64, env: Arc<MsgEnvelope>, msg_lt: u64) -> Result<()> {
        let hash = env.message_cell().repr_hash();
        let key = OutMsgQueueKey::with_workchain_id_and_prefix(workchain_id, prefix, hash);
        let enq = EnqueuedMsg::with_param(msg_lt, &env)?;
        self.set(&key, &enq, &msg_lt)
    }

    pub fn filter_queue(&mut self, _old_shard: &ShardIdent, _subshard: &ShardIdent) -> Result<()> {
        self.filter(|_key: OutMsgQueueKey, _enq, _lt| {
            todo!("need to port code")
        })
    }

}

///
/// The key used for an outbound message m is the concatenation of its 32-bit
/// next-hop workchain_id, the first 64 bits of the next-hop address inside that
/// workchain, and the representation hash Hash(m) of the message m itself
/// 

#[derive(Clone,Eq,Hash,Debug,PartialEq,Default)]
pub struct OutMsgQueueKey{
    pub workchain_id: i32,
    pub prefix: u64,
    pub hash: UInt256,
}

impl OutMsgQueueKey {
    pub fn with_workchain_id_and_prefix(workchain_id: i32, prefix: u64, hash: UInt256 ) -> Self {
        Self {
            workchain_id,
            prefix,
            hash,
        }
    }

    pub fn with_account_prefix(prefix: &AccountIdPrefixFull, hash: UInt256) -> Self {
        Self::with_workchain_id_and_prefix(prefix.workchain_id, prefix.prefix, hash)
    }

    pub fn with_msg_envelope(env: &MsgEnvelope, msg: &Message, hash: UInt256) -> Result<Self> {
        let src = msg.src().unwrap_or_default();
        let dst = msg.dst().unwrap_or_default();
        let src_prefix  = AccountIdPrefixFull::prefix(&src)?;
        let dest_prefix = AccountIdPrefixFull::prefix(&dst)?;
        let next_hop = src_prefix.interpolate_addr(&dest_prefix, &env.next_addr);
        Ok(Self::with_account_prefix(&next_hop, hash))
    }

    pub fn first_u64(acc: &AccountId) -> u64 { // TODO: remove to AccountId
        acc.clone().get_next_u64().unwrap()
    }

    pub fn to_hex_string(&self) -> String {
        match self.write_to_new_cell() {
            Ok(builder) => hex::encode(builder.data()),
            Err(err) => err.to_string() // impossible way
        }
    }
}

impl Serializable for OutMsgQueueKey {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.workchain_id.write_to(cell)?;
        self.prefix.write_to(cell)?;
        self.hash.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for OutMsgQueueKey {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        self.workchain_id.read_from(slice)?;
        self.prefix.read_from(slice)?;
        self.hash.read_from(slice)?;
        Ok(())
    }
}

/*
_ out_queue:OutMsgQueue proc_info:ProcessedInfo
ihr_pending:IhrPendingInfo = OutMsgQueueInfo;
*/
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct OutMsgQueueInfo {
    out_queue: OutMsgQueue,
    proc_info: ProcessedInfo,
    ihr_pending: IhrPendingInfo,
}

impl OutMsgQueueInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_params(
        out_queue: OutMsgQueue,
        proc_info: ProcessedInfo,
        ihr_pending: IhrPendingInfo
    ) -> Self {
        
        OutMsgQueueInfo {
            out_queue,
            proc_info,
            ihr_pending, 
        }
    }

    pub fn out_queue(&self) -> &OutMsgQueue {
        &self.out_queue
    }

    pub fn set_out_queue(&mut self, out_queue: OutMsgQueue) {
        self.out_queue = out_queue;
    }

    pub fn out_queue_mut(&mut self) -> &mut OutMsgQueue {
        &mut self.out_queue
    }

    pub fn proc_info(&self) -> &ProcessedInfo {
        &self.proc_info
    }

    pub fn set_proc_info(&mut self, proc_info: ProcessedInfo) {
        self.proc_info = proc_info;
    }

    pub fn ihr_pending(&self) -> &IhrPendingInfo {
        &self.ihr_pending
    }

    pub fn split(&self, shard: &ShardIdent) -> Result<(OutMsgQueueInfo, OutMsgQueueInfo)> {
        let mut left = self.clone();
        let mut right = self.clone();
        let split_key = shard.shard_key(true);
        let (left_queue, right_queue) = self.out_queue.split(&split_key)?;
        left.out_queue = left_queue;
        right.out_queue = right_queue;

        let split_key = shard.shard_key(false);
        let (left_info, right_info) = self.proc_info.split(&split_key)?;
        left.proc_info = left_info;
        right.proc_info = right_info;
        // TODO: check pending
        Ok((left, right))
    }

    pub fn split_inplace(&mut self, shard: &ShardIdent) -> Result<Self> {
        let (left, right) = self.split(&shard.merge()?)?;
        let mut pfx = shard.shard_key(true);
        let len = pfx.remaining_bits();
        pfx.move_by(len - 1)?;
        if pfx.get_next_bit()? {
            *self = right;
            Ok(left)
        } else {
            *self = left;
            Ok(right)
        }
    }
}

impl Serializable for OutMsgQueueInfo {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.out_queue.write_to(cell)?;
        self.proc_info.write_to(cell)?;
        self.ihr_pending.write_to(cell)?;

        Ok(())
    }
}

impl Deserializable for OutMsgQueueInfo {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.out_queue.read_from(cell)?;
        self.proc_info.read_from(cell)?;
        self.ihr_pending.read_from(cell)?;

        Ok(())
    }
}


///
/// OutMsg structure
/// blockchain spec 3.3.3. Descriptor of an outbound message
/// 
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OutMsg {
    None,
    /// External outbound messages, or “messages to nowhere”
    /// msg_export_ext$000 msg:^(Message Any) transaction:^Transaction = OutMsg;
    External(OutMsgExternal),           
    /// Ordinary (internal) outbound messages
    /// msg_export_new$001 out_msg:^MsgEnvelope transaction:^Transaction = OutMsg;
    New(OutMsgNew),
    /// Immediately processed internal outbound messages
    /// msg_export_imm$010 out_msg:^MsgEnvelope transaction:^Transaction reimport:^InMsg = OutMsg;
    Immediately(OutMsgImmediately),
    /// Transit (internal) outbound messages
    /// msg_export_tr$011 out_msg:^MsgEnvelope imported:^InMsg = OutMsg;
    Transit(OutMsgTransit),
    /// msg_export_deq$110 out_msg:^MsgEnvelope import_block_lt:uint64 = OutMsg;
    DequeueImmediately(OutMsgDequeueImmediately),
    /// msg_export_deq$1100 out_msg:^MsgEnvelope import_block_lt:uint63 = OutMsg;
    Dequeue(OutMsgDequeue),
    /// msg_export_deq_short$1101 msg_env_hash:bits256 next_workchain:int32 next_addr_pfx:uint64 import_block_lt:uint64 = OutMsg;
    DequeueShort(OutMsgDequeueShort),
    /// msg_export_tr_req$111 out_msg:^MsgEnvelope imported:^InMsg = OutMsg;
    TransitRequired(OutMsgTransitRequired),
}

impl Default for OutMsg {
    fn default() -> Self {
        OutMsg::None
    }
}

impl OutMsg {

    /// Check if is valid message
    pub fn is_valid(&self) -> bool {
        self != &OutMsg::None
    }

    pub fn tag(&self) -> u8 {
        match self {
            OutMsg::External(_)           => OUT_MSG_EXT,
            OutMsg::Immediately(_)        => OUT_MSG_IMM,
            OutMsg::New(_)                => OUT_MSG_NEW,
            OutMsg::Transit(_)            => OUT_MSG_TR,
            OutMsg::Dequeue(_)            => OUT_MSG_DEQ, // 4 bits
            OutMsg::DequeueShort(_)       => OUT_MSG_DEQ_SHORT, // 4 bits
            OutMsg::DequeueImmediately(_) => OUT_MSG_DEQ_IMM,
            OutMsg::TransitRequired(_)    => OUT_MSG_TRDEQ,
            OutMsg::None => 16
        }
    }

    ///
    /// the function returns the message envelop (if exists)
    ///
    pub fn read_out_message(&self) -> Result<Option<MsgEnvelope>> {
        Ok(
            match self {
                OutMsg::External(_) => None,
                OutMsg::Immediately(ref x) => Some(x.read_out_message()?),
                OutMsg::New(ref x) => Some(x.read_out_message()?),
                OutMsg::Transit(ref x) => Some(x.read_out_message()?),
                OutMsg::Dequeue(ref x) => Some(x.read_out_message()?),
                OutMsg::DequeueShort(_) => None,
                OutMsg::DequeueImmediately(ref x) => Some(x.read_out_message()?),
                OutMsg::TransitRequired(ref x) => Some(x.read_out_message()?),
                OutMsg::None => None
            }
        )
    }

    ///
    /// the function returns the message (if exists)
    ///
    pub fn read_message(&self) -> Result<Option<Message>> {
        Ok(
            match self {
                OutMsg::External(ref x) => Some(x.read_message()?),
                OutMsg::Immediately(ref x) => Some(x.read_out_message()?.read_message()?),
                OutMsg::New(ref x) => Some(x.read_out_message()?.read_message()?),
                OutMsg::Transit(ref x) => Some(x.read_out_message()?.read_message()?),
                OutMsg::Dequeue(ref x) => Some(x.read_out_message()?.read_message()?),
                OutMsg::DequeueShort(_) => None,
                OutMsg::DequeueImmediately(ref x) => Some(x.read_out_message()?.read_message()?),
                OutMsg::TransitRequired(ref x) => Some(x.read_out_message()?.read_message()?),
                OutMsg::None => None
            }
        )
    }

    ///
    /// the function returns the messages hash
    ///
    pub fn read_message_hash(&self) -> Result<UInt256> {
        Ok(
            match self {
                OutMsg::External(ref x) => x.message_cell().repr_hash(),
                OutMsg::Immediately(ref x) => x.read_out_message()?.message_cell().repr_hash(),
                OutMsg::New(ref x) => x.read_out_message()?.message_cell().repr_hash(),
                OutMsg::Transit(ref x) => x.read_out_message()?.message_cell().repr_hash(),
                OutMsg::Dequeue(ref x) => x.read_out_message()?.message_cell().repr_hash(),
                OutMsg::DequeueShort(ref x) => x.msg_env_hash.clone(),
                OutMsg::DequeueImmediately(ref x) => x.read_out_message()?.message_cell().repr_hash(),
                OutMsg::TransitRequired(ref x) => x.read_out_message()?.message_cell().repr_hash(),
                OutMsg::None => Default::default()
            }
        )
    }

    ///
    /// the function returns the message cell (if exists)
    ///
    pub fn message_cell(&self) -> Result<Option<Cell>> {
        Ok(
            match self {
                OutMsg::External(ref x) => Some(x.message_cell().clone()),
                OutMsg::Immediately(ref x) => Some(x.read_out_message()?.message_cell().clone()),
                OutMsg::New(ref x) => Some(x.read_out_message()?.message_cell().clone()),
                OutMsg::Transit(ref x) => Some(x.read_out_message()?.message_cell().clone()),
                OutMsg::Dequeue(ref x) => Some(x.read_out_message()?.message_cell().clone()),
                OutMsg::DequeueShort(_) => None,
                OutMsg::DequeueImmediately(ref x) => Some(x.read_out_message()?.message_cell().clone()),
                OutMsg::TransitRequired(ref x) => Some(x.read_out_message()?.message_cell().clone()),
                OutMsg::None => None
            }
        )
    }

    ///
    /// the function returns the message cell (if exists)
    ///
    pub fn envelope_message_cell(&self) -> Option<Cell> {
        match self {
            OutMsg::External(_) => None,
            OutMsg::Immediately(ref x) => Some(x.out_message_cell().clone()),
            OutMsg::New(ref x) => Some(x.out_message_cell().clone()),
            OutMsg::Transit(ref x) => Some(x.out_message_cell().clone()),
            OutMsg::Dequeue(ref x) => Some(x.out_message_cell().clone()),
            OutMsg::DequeueShort(_) => None,
            OutMsg::DequeueImmediately(ref x) => Some(x.out_message_cell().clone()),
            OutMsg::TransitRequired(ref x) => Some(x.out_message_cell().clone()),
            OutMsg::None => None
        }
    }

    pub fn transaction_cell(&self) -> Option<&Cell> {
        match self {
            OutMsg::External(ref x) => Some(x.transaction_cell()),
            OutMsg::Immediately(ref x) => Some(x.transaction_cell()),
            OutMsg::New(ref x) => Some(x.transaction_cell()),
            OutMsg::Transit(ref _x) => None,
            OutMsg::Dequeue(ref _x) => None,
            OutMsg::DequeueShort(ref _x) => None,
            OutMsg::DequeueImmediately(ref _x) => None,
            OutMsg::TransitRequired(ref _x) => None,
            OutMsg::None => None,
        }
    }

    pub fn read_transaction(&self) -> Result<Option<Transaction>> {
        self.transaction_cell().map(|cell| Transaction::construct_from(&mut cell.into())).transpose()
    }

    pub fn read_reimport_message(&self) -> Result<Option<InMsg>> {
        match self {
            OutMsg::Immediately(ref x) => Some(x.read_reimport_message()).transpose(),
            OutMsg::Transit(ref x) => Some(x.read_imported()).transpose(),
            OutMsg::DequeueImmediately(ref x) => Some(x.read_reimport_message()).transpose(),
            OutMsg::TransitRequired(ref x) => Some(x.read_imported()).transpose(),
            _ => Ok(None),
        }
    }

    pub fn exported_value(&self) -> Result<Option<CurrencyCollection>> {
        let mut exported = CurrencyCollection::default();
        match self {
            OutMsg::New(ref x) => {
                let env = x.read_out_message()?;
                let msg = env.read_message()?;
                // exported value = msg.value + msg.ihr_fee + fwd_fee_remaining
                exported.add(&msg.header().get_value().unwrap())?;
                if let CommonMsgInfo::IntMsgInfo(header) = msg.header() {
                    exported.grams.add(&header.ihr_fee)?;
                }
                exported.grams.add(&env.fwd_fee_remaining())?;
            }
            OutMsg::Transit(ref x) => {
                let env = x.read_out_message()?;
                let msg = env.read_message()?;
                // exported value = msg.value + msg.ihr_fee + fwd_fee_remaining
                exported.add(&msg.header().get_value().unwrap())?;
                if let CommonMsgInfo::IntMsgInfo(header) = msg.header() {
                    exported.grams.add(&header.ihr_fee)?;
                }
                exported.grams.add(&env.fwd_fee_remaining())?;
            }
            OutMsg::TransitRequired(ref x) => {
                let env = x.read_out_message()?;
                let msg = env.read_message()?;
                // exported value = msg.value + msg.ihr_fee + fwd_fee_remaining
                exported.add(&msg.header().get_value().unwrap())?;
                if let CommonMsgInfo::IntMsgInfo(header) = msg.header() {
                    exported.grams.add(&header.ihr_fee)?;
                }
                exported.grams.add(&env.fwd_fee_remaining())?;
            }
            // for other types - no value exported
            //OutMsg::External(ref x) => 
            //OutMsg::Immediately(ref x) => 
            //OutMsg::Dequeue(ref x) => 
            //OutMsg::DequeueImmediately(ref x) =>
            _ => return Ok(None)
        }
        Ok(Some(exported))
    }

    pub fn at_and_lt(&self) -> Result<Option<(u32, u64)>> {
        Ok(None)
    }
}

///internal helper macros for reading InMsg variants
macro_rules! read_out_msg_descr {
    ($cell:expr, $msg_descr:tt, $variant:ident) => {{
        let mut x = $msg_descr::default();
        x.read_from($cell)?;
        OutMsg::$variant(x)
    }}
}

 ///internal helper macros for reading InMsg variants
macro_rules! write_out_ctor_tag {
    ($builder:expr, $tag:ident, $tag_len:expr) => {{
        $builder.append_bits($tag as usize, $tag_len).unwrap();
        $builder
    }}
}


impl Serializable for OutMsg {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        match self {
            OutMsg::External(ref x) => x.write_to(write_out_ctor_tag!(cell, OUT_MSG_EXT, 3)),
            OutMsg::Immediately(ref x) => x.write_to(write_out_ctor_tag!(cell, OUT_MSG_IMM, 3)),
            OutMsg::New(ref x) => x.write_to(write_out_ctor_tag!(cell, OUT_MSG_NEW, 3)),
            OutMsg::Transit(ref x) => x.write_to(write_out_ctor_tag!(cell, OUT_MSG_TR, 3)),
            OutMsg::Dequeue(ref x) => x.write_to(write_out_ctor_tag!(cell, OUT_MSG_DEQ, 4)),
            OutMsg::DequeueShort(ref x) => x.write_to(write_out_ctor_tag!(cell, OUT_MSG_DEQ_SHORT, 4)),
            OutMsg::DequeueImmediately(ref x) => x.write_to(write_out_ctor_tag!(cell, OUT_MSG_DEQ_IMM, 3)),
            OutMsg::TransitRequired(ref x) => x.write_to(write_out_ctor_tag!(cell, OUT_MSG_TRDEQ, 3)),
            OutMsg::None => fail!(
                BlockError::InvalidOperation("OutMsg::None can't be serialized".to_string())
            )
        }
    }
}

impl Deserializable for OutMsg {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag: u8 = (cell.get_next_bits(3)?[0] & 0xE0) >> 5;
        *self =  match tag {
            OUT_MSG_EXT => read_out_msg_descr!(cell, OutMsgExternal, External),
            OUT_MSG_IMM => read_out_msg_descr!(cell, OutMsgImmediately, Immediately),
            OUT_MSG_NEW => read_out_msg_descr!(cell, OutMsgNew, New),
            OUT_MSG_TR => read_out_msg_descr!(cell, OutMsgTransit, Transit),
            OUT_MSG_DEQ_IMM => read_out_msg_descr!(cell, OutMsgDequeueImmediately, DequeueImmediately),
            OUT_MSG_TRDEQ => read_out_msg_descr!(cell, OutMsgTransitRequired, TransitRequired),
            tag if cell.remaining_bits() > 0 && (tag == OUT_MSG_DEQ >> 1 || tag == OUT_MSG_DEQ_SHORT >> 1) => {
                match (tag << 1) | cell.get_next_bit_int().unwrap() as u8 {
                    OUT_MSG_DEQ => read_out_msg_descr!(cell, OutMsgDequeue, Dequeue),
                    OUT_MSG_DEQ_SHORT => read_out_msg_descr!(cell, OutMsgDequeueShort, DequeueShort),
                    _ => unreachable!()
                }
            },
            tag => {
                fail!(
                    BlockError::InvalidConstructorTag {
                        t: tag as u32,
                        s: "OutMsg".to_string()
                    }
                );
            }
        };
        Ok(())
    }
}


///
/// msg_export_ext$000 msg:^Message transaction:^Transaction = OutMsg;
/// 
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OutMsgExternal {
    msg: ChildCell<Message>,
    transaction: ChildCell<Transaction>,
}

impl OutMsgExternal {
    pub fn with_params(msg: &Message, tr: &Transaction) -> Result<Self> {
        Ok(
            OutMsgExternal {
                msg: ChildCell::with_struct(msg)?,
                transaction: ChildCell::with_struct(tr)?,
            }
        )
    }

    pub fn read_message(&self) -> Result<Message> {
        self.msg.read_struct()
    }

    pub fn message_cell(&self) -> &Cell {
        self.msg.cell()
    }

    pub fn read_transaction(&self) -> Result<Transaction> {
        self.transaction.read_struct()
    }

    pub fn transaction_cell(&self) -> &Cell {
        self.transaction.cell()
    }
}

impl Serializable for OutMsgExternal {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_reference(self.msg.write_to_new_cell()?);
        cell.append_reference(self.transaction.write_to_new_cell()?);
        Ok(())
    }
}

impl Deserializable for OutMsgExternal {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.msg.read_from(&mut cell.checked_drain_reference()?.into())?;
        self.transaction.read_from(&mut cell.checked_drain_reference()?.into())?;
        Ok(())
    }
}

///
/// msg_export_imm$010 out_msg:^MsgEnvelope transaction:^Transaction reimport:^InMsg = OutMsg;
/// 

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OutMsgImmediately {
    out_msg: ChildCell<MsgEnvelope>,
    transaction: ChildCell<Transaction>,
    reimport: ChildCell<InMsg>,
}

impl OutMsgImmediately {
    pub fn with_params(env: &MsgEnvelope, tr: &Transaction, reimport: &InMsg) -> Result<Self> {
        Ok(
            OutMsgImmediately{
                out_msg: ChildCell::with_struct(env)?,
                transaction: ChildCell::with_struct(tr)?,
                reimport: ChildCell::with_struct(reimport)?,
            }
        )
    }

    pub fn read_out_message(&self) -> Result<MsgEnvelope> {
        self.out_msg.read_struct()
    }

    pub fn out_message_cell(&self) -> &Cell {
        self.out_msg.cell()
    }

    pub fn read_transaction(&self) -> Result<Transaction> {
        self.transaction.read_struct()
    }

    pub fn transaction_cell(&self) -> &Cell {
        self.transaction.cell()
    }

    pub fn read_reimport_message(&self) -> Result<InMsg> {
        self.reimport.read_struct()
    }

    pub fn reimport_message_cell(&self) -> &Cell {
        self.reimport.cell()
    }
}

impl Serializable for OutMsgImmediately {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_reference(self.out_msg.write_to_new_cell()?);
        cell.append_reference(self.transaction.write_to_new_cell()?);
        cell.append_reference(self.reimport.write_to_new_cell()?);
        Ok(())
    }
}

impl Deserializable for OutMsgImmediately {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.out_msg.read_from(&mut cell.checked_drain_reference()?.into())?;
        self.transaction.read_from(&mut cell.checked_drain_reference()?.into())?;
        self.reimport.read_from(&mut cell.checked_drain_reference()?.into())?;
        Ok(())
    }
}

///
/// msg_export_new$001 out_msg:^MsgEnvelope transaction:^Transaction = OutMsg;
/// 

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OutMsgNew {
    out_msg: ChildCell<MsgEnvelope>,
    transaction: ChildCell<Transaction>,
}

impl OutMsgNew {
    pub fn with_params(msg: &MsgEnvelope, tr: &Transaction) -> Result<Self> {
        Ok(
            OutMsgNew {
                out_msg: ChildCell::with_struct(msg)?,
                transaction: ChildCell::with_struct(tr)?,
            }
        )
    }

    pub fn read_out_message(&self) -> Result<MsgEnvelope> {
        self.out_msg.read_struct()
    }

    pub fn out_message_cell(&self) -> &Cell {
        self.out_msg.cell()
    }

    pub fn read_transaction(&self) -> Result<Transaction> {
        self.transaction.read_struct()
    }

    pub fn transaction_cell(&self) -> &Cell {
        self.transaction.cell()
    }
}

impl Serializable for OutMsgNew {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_reference(self.out_msg.write_to_new_cell()?);
        cell.append_reference(self.transaction.write_to_new_cell()?);
        Ok(())
    }
}

impl Deserializable for OutMsgNew {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.out_msg.read_from(&mut cell.checked_drain_reference()?.into())?;
        self.transaction.read_from(&mut cell.checked_drain_reference()?.into())?;
        Ok(())
    }
}

///
/// msg_export_tr$011 out_msg:^MsgEnvelope imported:^InMsg = OutMsg;
/// 

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OutMsgTransit {
    out_msg: ChildCell<MsgEnvelope>,
    imported: ChildCell<InMsg>,
}

impl OutMsgTransit {
    pub fn with_params(env: &MsgEnvelope, imported: &InMsg) -> Result<Self> {
        Ok(
            OutMsgTransit{
                out_msg: ChildCell::with_struct(env)?,
                imported: ChildCell::with_struct(imported)?,
            }
        )
    }

    pub fn read_out_message(&self) -> Result<MsgEnvelope> {
        self.out_msg.read_struct()
    }

    pub fn out_message_cell(&self) -> &Cell {
        self.out_msg.cell()
    }

    pub fn read_imported(&self) -> Result<InMsg> {
        self.imported.read_struct()
    }

    pub fn imported_cell(&self) -> &Cell {
        self.imported.cell()
    }
}

impl Serializable for OutMsgTransit {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_reference(self.out_msg.write_to_new_cell()?);
        cell.append_reference(self.imported.write_to_new_cell()?);
        Ok(())
    }
}

impl Deserializable for OutMsgTransit {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.out_msg.read_from(&mut cell.checked_drain_reference()?.into())?;
        self.imported.read_from(&mut cell.checked_drain_reference()?.into())?; 
        Ok(())
    }
}

///
/// msg_export_deq$110 out_msg:^MsgEnvelope import_block_lt:uint64 = OutMsg;
/// 

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OutMsgDequeueImmediately {
    out_msg: ChildCell<MsgEnvelope>,
    reimport: ChildCell<InMsg>,
}

impl OutMsgDequeueImmediately {
    pub fn with_params(env: &MsgEnvelope, reimport: &InMsg) -> Result<Self> {
        Ok(
            OutMsgDequeueImmediately{
                out_msg: ChildCell::with_struct(env)?,
                reimport: ChildCell::with_struct(reimport)?,
            }
        )
    }

    pub fn read_out_message(&self) -> Result<MsgEnvelope> {
        self.out_msg.read_struct()
    }

    pub fn out_message_cell(&self) -> &Cell {
        self.out_msg.cell()
    }

    pub fn read_reimport_message(&self) -> Result<InMsg> {
        self.reimport.read_struct()
    }

    pub fn reimport_message_cell(&self) -> &Cell {
        self.reimport.cell()
    }
}

impl Serializable for OutMsgDequeueImmediately {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_reference(self.out_msg.write_to_new_cell()?);
        cell.append_reference(self.reimport.write_to_new_cell()?);
        Ok(())
    }
}

impl Deserializable for OutMsgDequeueImmediately {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.out_msg.read_from(&mut cell.checked_drain_reference()?.into())?;
        self.reimport.read_from(&mut cell.checked_drain_reference()?.into())?;
        Ok(())
    }
}

///
/// msg_export_deq$1100 out_msg:^MsgEnvelope import_block_lt:uint63 = OutMsg;
/// 

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OutMsgDequeue {
    out_msg: ChildCell<MsgEnvelope>,
    import_block_lt: u64,
}

impl OutMsgDequeue {
    pub fn with_params(env: &MsgEnvelope, lt: u64) -> Result<Self> {
        if lt & 0x8000_0000_0000_0000 != 0 {
            fail!(BlockError::InvalidArg("`import_block_lt` can't have highest bit set".to_string()))
        }
        Ok(
            OutMsgDequeue{
                out_msg: ChildCell::with_struct(env)?,
                import_block_lt: lt,
            }
        )
    }

    pub fn read_out_message(&self) -> Result<MsgEnvelope> {
        self.out_msg.read_struct()
    }

    pub fn out_message_cell(&self) -> &Cell {
        self.out_msg.cell()
    }

    pub fn import_block_lt(&self) -> u64 {
        self.import_block_lt
    }

    pub fn set_import_block_lt(&mut self, value: u64) -> Result<()> {
        if value & 0x8000_0000_0000_0000 != 0 {
            fail!(BlockError::InvalidArg("`import_block_lt` can't have highest bit set".to_string()))
        }
        self.import_block_lt = value;
        Ok(())
    }
}

impl Serializable for OutMsgDequeue {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_reference(self.out_msg.write_to_new_cell()?);
        cell.append_bits(self.import_block_lt as usize, 63)?;
        Ok(())
    }
}

impl Deserializable for OutMsgDequeue {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.out_msg.read_from(&mut cell.checked_drain_reference()?.into())?;
        self.import_block_lt = cell.get_next_int(63)?;
        Ok(())
    }
}

///
/// msg_export_deq_short$1101 msg_env_hash:bits256 next_workchain:int32 next_addr_pfx:uint64 import_block_lt:uint64 = OutMsg;
///

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OutMsgDequeueShort {
    pub msg_env_hash: UInt256,
    pub next_workchain: i32,
    pub next_addr_pfx: u64,
    pub import_block_lt: u64,
}

impl Serializable for OutMsgDequeueShort {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.msg_env_hash.write_to(cell)?;
        self.next_workchain.write_to(cell)?;
        self.next_addr_pfx.write_to(cell)?;
        self.import_block_lt.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for OutMsgDequeueShort {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.msg_env_hash.read_from(cell)?; 
        self.next_workchain.read_from(cell)?; 
        self.next_addr_pfx.read_from(cell)?; 
        self.import_block_lt.read_from(cell)?; 
        Ok(())
    }
}

///
/// msg_export_tr_req$111 out_msg:^MsgEnvelope imported:^InMsg = OutMsg;
/// 

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OutMsgTransitRequired {
    out_msg: ChildCell<MsgEnvelope>,
    imported: ChildCell<InMsg>,
}

impl OutMsgTransitRequired {
    pub fn with_params(env: &MsgEnvelope, imported: &InMsg) -> Result<Self> {
        Ok(
            OutMsgTransitRequired{
                out_msg: ChildCell::with_struct(env)?,
                imported: ChildCell::with_struct(imported)?,
            }
        )
    }

    pub fn read_out_message(&self) -> Result<MsgEnvelope> {
        self.out_msg.read_struct()
    }

    pub fn out_message_cell(&self) -> &Cell {
        self.out_msg.cell()
    }

    pub fn read_imported(&self) -> Result<InMsg> {
        self.imported.read_struct()
    }

    pub fn imported_cell(&self) -> &Cell {
        self.imported.cell()
    }
}

impl Serializable for OutMsgTransitRequired {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_reference(self.out_msg.write_to_new_cell()?);
        cell.append_reference(self.imported.write_to_new_cell()?);
        Ok(())
    }
}

impl Deserializable for OutMsgTransitRequired {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.out_msg.read_from(&mut cell.checked_drain_reference()?.into())?;
        self.imported.read_from(&mut cell.checked_drain_reference()?.into())?; 
        Ok(())
    }
}
