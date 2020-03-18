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
use std::sync::Arc;
use {AccountId, UInt256};
use ton_types::{BuilderData, SliceData};
use self::hashmapaug::Augmentable;


/*
        3.3 Outbound message queue and descriptors
 This section discusses OutMsgDescr, the structure representing all outbound
 messages of a block, along with their envelopes and brief descriptions of the
 reasons for including them into OutMsgDescr. This structure also describes
 all modifications of OutMsgQueue, which is a part of the shardchain state.
*/

//constructor tags of InMsg variants (only 3 bits are used)
const OUT_MSG_EXT: u8 = 0b00000000;
const OUT_MSG_IMM: u8 = 0b00000010;
const OUT_MSG_NEW: u8 = 0b00000001;
const OUT_MSG_TR: u8 = 0b00000011;
const OUT_MSG_DEQ_IMM: u8 = 0b00000100;
const OUT_MSG_DEQ: u8 = 0b00000110;
const OUT_MSG_TRDEQ: u8 = 0b00000111;


/*
_ 
	enqueued_lt:uint64 
	out_msg:^MsgEnvelope 
= EnqueuedMsg;
*/

///
/// EnqueuedMsg structure
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct EnqueuedMsg {
    pub enqueued_lt: u64,
    pub out_msg: Arc<MsgEnvelope>
}

impl EnqueuedMsg {
    /// New default instance EnqueuedMsg structure
    pub fn new() -> Self {
        EnqueuedMsg {
            enqueued_lt: 0,
            out_msg: Arc::new(MsgEnvelope::default())
        }
    }

    /// New instance EnqueuedMsg structure
    pub fn with_param(enqueued_lt: u64, out_msg: Arc<MsgEnvelope>) -> Self {
        EnqueuedMsg {
            enqueued_lt,
            out_msg,
        }
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
        let mut msg_e = MsgEnvelope::default();
        msg_e.read_from(&mut cell.checked_drain_reference()?.into())?;
        self.out_msg = Arc::new(msg_e);
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
        let msg = out_msg.read_message()?;
        let hash = msg.hash()?;
        let value = match out_msg {
            OutMsg::External(_) => msg.get_value(),
            OutMsg::Immediately(_) => msg.get_value(),
            OutMsg::New(_) => msg.get_value(),
            OutMsg::Transit(_) => None,
            OutMsg::Dequeue(_) => None,
            OutMsg::DequeueImmediately(_) => msg.get_value(),
            OutMsg::TransitRequired(ref _x) => None,
            OutMsg::None => unreachable!(),
        };
        self.set(&hash, &out_msg, value.unwrap_or(&CurrencyCollection::default()))
    }

    /// insert or replace existion record
    /// use to improve speed
    pub fn insert_serialized(&mut self, key: &SliceData, msg_slice: &SliceData, exported: &CurrencyCollection ) -> Result<()> {
        if self.0.set(key.clone(), msg_slice, exported).is_ok() {
            Ok(())
        } else {
            failure::bail!(BlockError::Other("Error insert serialized message".to_string()))
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
// Blockchain: 3.3.6
// _ (HashmapAugE 352 OutMsg uint64) = OutMsgQueue;
//
define_HashmapAugE!(OutMsgQueue, 352, OutMsg, MsgTime);

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
    pub fn insert(&mut self, address: u64, msg: &OutMsg, msg_lt: u64) -> Result<()> {
        let key = OutMsgQueueKey::with_workchain_id_and_message(0, address, &msg).unwrap();
        self.set(&key, &msg, &msg_lt)
    }
}

///
/// The key used for an outbound message m is the concatenation of its 32-bit
/// next-hop workchain_id, the first 64 bits of the next-hop address inside that
/// workchain, and the representation hash Hash(m) of the message m itself
/// 

#[derive(Clone,Eq,Hash,Debug,PartialEq,Default)]
pub struct OutMsgQueueKey{
    pub workchain_id:i32,
    pub address: u64,
    pub hash: UInt256,
}

impl OutMsgQueueKey {
    pub fn with_workchain_id_and_message(id: i32, address: u64, out_msg: &OutMsg )
    -> Result<OutMsgQueueKey> {
        let hash = out_msg.hash()?;
        Ok(OutMsgQueueKey {
            workchain_id: id,
            address,
            hash,
        })
    }

    pub fn first_u64(acc: &AccountId) -> u64 { // TODO: remove to AccountId
        acc.clone().get_next_u64().unwrap()
    }
}

impl Serializable for OutMsgQueueKey {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.workchain_id.write_to(cell)?;
        self.address.write_to(cell)?;
        self.hash.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for OutMsgQueueKey {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        self.workchain_id.read_from(slice)?;
        self.address.read_from(slice)?;
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

    pub fn out_queue_mut(&mut self) -> &mut OutMsgQueue {
        &mut self.out_queue
    }

    pub fn proc_info(&self) -> &ProcessedInfo {
        &self.proc_info
    }

    pub fn ihr_pending(&self) -> &IhrPendingInfo {
        &self.ihr_pending
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
    External(OutMsgExternal),           
    /// Immediately processed internal outbound messages
    Immediately(OutMsgImmediately),
    /// Ordinary (internal) outbound messages
    New(OutMsgNew),
    /// Transit (internal) outbound messages
    Transit(OutMsgTransit),
    DequeueImmediately(OutMsgDequeueImmediately),
    Dequeue(OutMsgDequeue),
    TransitRequired(OutMsgTransitRequired),
}

impl Default for OutMsg {
    fn default() -> Self {
        OutMsg::None
    }
}

impl OutMsg {

    ///
    /// the function returns the message
    ///
    pub fn read_message(&self) -> Result<Message> {
        Ok(
            match self {
                OutMsg::External(ref x) => x.read_message()?,
                OutMsg::Immediately(ref x) => x.read_out_message()?.read_message()?,
                OutMsg::New(ref x) => x.read_out_message()?.read_message()?,
                OutMsg::Transit(ref x) => x.read_out_message()?.read_message()?,
                OutMsg::Dequeue(ref x) => x.read_out_message()?.read_message()?,
                OutMsg::DequeueImmediately(ref x) => x.read_out_message()?.read_message()?,
                OutMsg::TransitRequired(ref x) => x.read_out_message()?.read_message()?,
                OutMsg::None => unreachable!(),
            }
        )
    }

    ///
    /// the function returns the message cell (if exists)
    ///
    pub fn message_cell(&self) -> Result<Cell> {
        Ok(
            match self {
                OutMsg::External(ref x) => x.message_cell().clone(),
                OutMsg::Immediately(ref x) => x.read_out_message()?.message_cell().clone(),
                OutMsg::New(ref x) => x.read_out_message()?.message_cell().clone(),
                OutMsg::Transit(ref x) => x.read_out_message()?.message_cell().clone(),
                OutMsg::Dequeue(ref x) => x.read_out_message()?.message_cell().clone(),
                OutMsg::DequeueImmediately(ref x) => x.read_out_message()?.message_cell().clone(),
                OutMsg::TransitRequired(ref x) => x.read_out_message()?.message_cell().clone(),
                OutMsg::None => unreachable!(),
            }
        )
    }

    pub fn transaction_cell(&self) -> Option<&Cell> {
        match self {
            OutMsg::External(ref x) => Some(x.transaction_cell()),
            OutMsg::Immediately(ref x) => Some(x.transaction_cell()),
            OutMsg::New(ref x) => Some(x.transaction_cell()),
            OutMsg::Transit(ref _x) => None,
            OutMsg::Dequeue(ref _x) => None,
            OutMsg::DequeueImmediately(ref _x) => None,
            OutMsg::TransitRequired(ref _x) => None,
            OutMsg::None => None,
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
    ($builder:expr, $tag:ident) => {{
        $builder.append_bits($tag as usize, 3).unwrap();
        $builder
    }}
}


impl Serializable for OutMsg {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        match self {
            OutMsg::External(ref x) => x.write_to(write_out_ctor_tag!(cell, OUT_MSG_EXT)),
            OutMsg::Immediately(ref x) => x.write_to(write_out_ctor_tag!(cell, OUT_MSG_IMM)),
            OutMsg::New(ref x) => x.write_to(write_out_ctor_tag!(cell, OUT_MSG_NEW)),
            OutMsg::Transit(ref x) => x.write_to(write_out_ctor_tag!(cell, OUT_MSG_TR)),
            OutMsg::Dequeue(ref x) => x.write_to(write_out_ctor_tag!(cell, OUT_MSG_DEQ)),
            OutMsg::DequeueImmediately(ref x) => x.write_to(write_out_ctor_tag!(cell, OUT_MSG_DEQ_IMM)),
            OutMsg::TransitRequired(ref x) => x.write_to(write_out_ctor_tag!(cell, OUT_MSG_TRDEQ)),
            OutMsg::None => failure::bail!(
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
            OUT_MSG_DEQ =>  read_out_msg_descr!(cell, OutMsgDequeue, Dequeue),
            OUT_MSG_TRDEQ => read_out_msg_descr!(cell, OutMsgTransitRequired, TransitRequired),
            tag => failure::bail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: "OutMsg".to_string()
                }
            )
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

    pub fn message_cell(&self) -> &Cell {
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
/// msg_export_deq$110 out_msg:^MsgEnvelope import_block_lt:uint64 = OutMsg;
/// 

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OutMsgDequeue {
    out_msg: ChildCell<MsgEnvelope>,
    pub import_block_lt: u64,
}

impl OutMsgDequeue {
    pub fn with_params(env: &MsgEnvelope, lt: u64) -> Result<Self> {
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
}

impl Serializable for OutMsgDequeue {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_reference(self.out_msg.write_to_new_cell()?);
        self.import_block_lt.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for OutMsgDequeue {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.out_msg.read_from(&mut cell.checked_drain_reference()?.into())?;
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
