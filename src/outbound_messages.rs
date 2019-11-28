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
use std::sync::Arc;
use {AccountId, UInt256};
use ton_types::{BuilderData, SliceData, CellType};
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
    enqueued_lt: u64,
    out_msg: Arc<MsgEnvelope>
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
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        self.enqueued_lt.write_to(cell)?;
        cell.append_reference(self.out_msg.write_to_new_cell()?);
        Ok(())
    }
}

impl Deserializable for EnqueuedMsg {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
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
    pub fn insert(&mut self, out_msg: &OutMsg) -> BlockResult<()> {
        let msg = out_msg.message()
            .ok_or(BlockErrorKind::InvalidOperation("OutMsg must contain message to be inserted into OutMsgDescr".into()))?;
        let hash = msg.hash()?;
        self.set(&hash, &out_msg, out_msg.expoted_value().unwrap_or(&CurrencyCollection::default()))
    }

    /// insert or replace existion record
    /// use to improve speed
    pub fn insert_serialized(&mut self, key: &SliceData, msg_slice: &SliceData, exported: &CurrencyCollection ) -> BlockResult<()> {
        if self.0.set(key.clone(), msg_slice, exported).is_ok() {
            Ok(())
        } else {
            block_err!(BlockErrorKind::Other("Error insert serialized message".to_string()))
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
    fn calc(&mut self, other: &Self) -> BlockResult<()> {
        if *self > *other {
            *self = *other;
        }
        Ok(())
    }
}

impl OutMsgQueue {
    /// insert OutMessage to OutMsgQueue
    pub fn insert(&mut self, address: u64, msg: &OutMsg) -> BlockResult<()> {
        let key = OutMsgQueueKey::with_workchain_id_and_message(0, address, &msg).unwrap();
        let lt = msg.at_and_lt().unwrap_or_default().1;
        self.set(&key, &msg, &lt)
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
    -> BlockResult<OutMsgQueueKey> {
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
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        self.workchain_id.write_to(cell)?;
        self.address.write_to(cell)?;
        self.hash.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for OutMsgQueueKey {
    fn read_from(&mut self, slice: &mut SliceData) -> BlockResult<()> {
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
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        self.out_queue.write_to(cell)?;
        self.proc_info.write_to(cell)?;
        self.ihr_pending.write_to(cell)?;

        Ok(())
    }
}

impl Deserializable for OutMsgQueueInfo {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
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
    /// the function returns the value exported by the message from the account
    ///
    pub fn expoted_value<'a>(&'a self) -> Option<&'a CurrencyCollection> {
        match self {
            OutMsg::External(ref x) => x.msg.get_value(),
            OutMsg::Immediately(ref x) => x.out_msg.get_message().get_value(),
            OutMsg::New(ref x) => x.out_msg.get_message().get_value(),
            OutMsg::Transit(ref _x) => None,
            OutMsg::Dequeue(ref _x) => None,
            OutMsg::DequeueImmediately(ref x) => x.out_msg.get_message().get_value(),
            OutMsg::TransitRequired(ref _x) => None,
            OutMsg::None => None,
        }
    }

    ///
    /// the function returns the message (if exists)
    ///
    pub fn message<'a>(&'a self) -> Option<&'a Message> {
        match self {
            OutMsg::External(ref x) => Some(&x.msg),
            OutMsg::Immediately(ref x) => Some(x.out_msg.get_message()),
            OutMsg::New(ref x) => Some(x.out_msg.get_message()),
            OutMsg::Transit(ref x) => Some(x.out_msg.get_message()),
            OutMsg::Dequeue(ref x) => Some(x.out_msg.get_message()),
            OutMsg::DequeueImmediately(ref x) => Some(x.out_msg.get_message()),
            OutMsg::TransitRequired(ref x) => Some(x.out_msg.get_message()),
            OutMsg::None => None,
        }
    }

    ///
    /// the function returns the message (if exists)
    ///
    pub fn message_mut<'a>(&'a mut self) -> Option<&'a mut Message> {
        match self {
            OutMsg::External(ref mut x) => Some(Arc::get_mut(&mut x.msg).unwrap()),
            OutMsg::Immediately(ref mut x) => x.out_msg.get_message_mut(),
            OutMsg::New(ref mut x) => x.out_msg.get_message_mut(),
            OutMsg::Transit(ref mut x) => x.out_msg.get_message_mut(),
            OutMsg::Dequeue(ref mut x) => x.out_msg.get_message_mut(),
            OutMsg::DequeueImmediately(ref mut x) => x.out_msg.get_message_mut(),
            OutMsg::TransitRequired(ref mut x) => x.out_msg.get_message_mut(),
            OutMsg::None => None,
        }
    }
    
    ///
    /// the function returns the fees exported by the message from the account
    ///
    pub fn exported_fee(&self) -> BlockResult<Option<Grams>> {
        match self {
            OutMsg::External(ref x) => x.msg.get_fee(),
            OutMsg::Immediately(ref x) => x.out_msg.get_message().get_fee(),
            OutMsg::New(ref x) => x.out_msg.get_message().get_fee(),
            OutMsg::Transit(ref x) => x.out_msg.get_message().get_fee(),
            OutMsg::Dequeue(ref x) => x.out_msg.get_message().get_fee(),
            OutMsg::DequeueImmediately(ref x) => x.out_msg.get_message().get_fee(),
            OutMsg::TransitRequired(ref x) => x.out_msg.get_message().get_fee(),
            OutMsg::None => Ok(None),
        }
    }

    ///
    /// set UNIX time and Logical Time for outbound message
    ///
    pub fn set_at_and_lt(&mut self, at: u32, lt: u64) {
        match self {
            OutMsg::External(ref mut x) => { Arc::get_mut(&mut x.msg).map(|m| m.set_at_and_lt(at, lt)); },
            OutMsg::Immediately(ref mut x) => { x.out_msg.get_message_mut().map(|m| m.set_at_and_lt(at, lt)); },
            OutMsg::New(ref mut x) => { x.out_msg.get_message_mut().map(|m| m.set_at_and_lt(at, lt)); },
            OutMsg::Transit(ref mut x) => { x.out_msg.get_message_mut().map(|m| m.set_at_and_lt(at, lt)); },
            OutMsg::Dequeue(ref mut x) => { x.out_msg.get_message_mut().map(|m| m.set_at_and_lt(at, lt)); },
            OutMsg::DequeueImmediately(ref mut x) => { x.out_msg.get_message_mut().map(|m| m.set_at_and_lt(at, lt)); },
            OutMsg::TransitRequired(ref mut x) => { x.out_msg.get_message_mut().map(|m| m.set_at_and_lt(at, lt)); },
            OutMsg::None => (),
        }
    }
    
    ///
    /// get UNIX time and Logical Time for outbound message
    ///
    pub fn at_and_lt(&self) -> Option<(u32, u64)> {
        match self {
            OutMsg::External(ref x) =>  { x.msg.at_and_lt() },
            OutMsg::Immediately(ref x) => { x.out_msg.get_message().at_and_lt() },
            OutMsg::New(ref x) => { x.out_msg.get_message().at_and_lt() },
            OutMsg::Transit(ref x) => { x.out_msg.get_message().at_and_lt() },
            OutMsg::Dequeue(ref x) => { x.out_msg.get_message().at_and_lt() },
            OutMsg::DequeueImmediately(ref x) => { x.out_msg.get_message().at_and_lt() },
            OutMsg::TransitRequired(ref x) => { x.out_msg.get_message().at_and_lt() },
            OutMsg::None => None,
        }
    }

    pub fn read_message_from(cell: &mut SliceData) -> BlockResult<Message> {
        let tag: u8 = (cell.get_next_bits(3)?[0] & 0xE0) >> 5;
        match tag {
            OUT_MSG_EXT => OutMsgExternal::read_message_from(cell),
            OUT_MSG_IMM => OutMsgImmediately::read_message_from(cell),
            OUT_MSG_NEW => OutMsgNew::read_message_from(cell),
            OUT_MSG_TR => OutMsgTransit::read_message_from(cell),
            OUT_MSG_DEQ_IMM => OutMsgDequeueImmediately::read_message_from(cell),
            OUT_MSG_DEQ =>  OutMsgDequeue::read_message_from(cell),
            OUT_MSG_TRDEQ => OutMsgTransitRequired::read_message_from(cell),
            tag => bail!(BlockErrorKind::InvalidConstructorTag(tag as u32, "OutMsg".into())),
        }
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
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        match self {
            OutMsg::External(ref x) => x.write_to(write_out_ctor_tag!(cell, OUT_MSG_EXT)),
            OutMsg::Immediately(ref x) => x.write_to(write_out_ctor_tag!(cell, OUT_MSG_IMM)),
            OutMsg::New(ref x) => x.write_to(write_out_ctor_tag!(cell, OUT_MSG_NEW)),
            OutMsg::Transit(ref x) => x.write_to(write_out_ctor_tag!(cell, OUT_MSG_TR)),
            OutMsg::Dequeue(ref x) => x.write_to(write_out_ctor_tag!(cell, OUT_MSG_DEQ)),
            OutMsg::DequeueImmediately(ref x) => x.write_to(write_out_ctor_tag!(cell, OUT_MSG_DEQ_IMM)),
            OutMsg::TransitRequired(ref x) => x.write_to(write_out_ctor_tag!(cell, OUT_MSG_TRDEQ)),
            OutMsg::None => 
                bail!(BlockErrorKind::InvalidOperation("OutMsg::None can't be sirialized".into())),
        }
    }
}

impl Deserializable for OutMsg {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        let tag: u8 = (cell.get_next_bits(3)?[0] & 0xE0) >> 5;
        *self =  match tag {
            OUT_MSG_EXT => read_out_msg_descr!(cell, OutMsgExternal, External),
            OUT_MSG_IMM => read_out_msg_descr!(cell, OutMsgImmediately, Immediately),
            OUT_MSG_NEW => read_out_msg_descr!(cell, OutMsgNew, New),
            OUT_MSG_TR => read_out_msg_descr!(cell, OutMsgTransit, Transit),
            OUT_MSG_DEQ_IMM => read_out_msg_descr!(cell, OutMsgDequeueImmediately, DequeueImmediately),
            OUT_MSG_DEQ =>  read_out_msg_descr!(cell, OutMsgDequeue, Dequeue),
            OUT_MSG_TRDEQ => read_out_msg_descr!(cell, OutMsgTransitRequired, TransitRequired),
            tag => bail!(BlockErrorKind::InvalidConstructorTag(tag as u32, "OutMsg".into())),
        };
        Ok(())
    }
}


///
/// msg_export_ext$000 msg:^Message transaction:^Transaction = OutMsg;
/// 
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OutMsgExternal {
    pub msg: Arc<Message>,
    pub transaction: Arc<Transaction>,
}

impl OutMsgExternal {
    pub fn with_params(msg: Arc<Message>, tr: Arc<Transaction>) -> Self{
        OutMsgExternal {
            msg: msg,
            transaction: tr,
        }
    }

    fn read_message_from(cell: &mut SliceData) -> BlockResult<Message> {
        let ref_cell = cell.checked_drain_reference()?;
        if ref_cell.cell_type() == CellType::PrunedBranch {
            bail!(BlockErrorKind::PrunedCellAccess("Message".into()))
        }
        Message::construct_from(&mut ref_cell.into())
    }
}

impl Serializable for OutMsgExternal {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_reference(self.msg.write_to_new_cell()?);
        cell.append_reference(self.transaction.write_to_new_cell()?);
        Ok(())
    }
}

impl Deserializable for OutMsgExternal {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        self.msg = Arc::new(Message::construct_from(&mut cell.checked_drain_reference()?.into())?);
        self.transaction = Arc::new(Transaction::construct_from(&mut cell.checked_drain_reference()?.into())?);
        Ok(())
    }
}

///
/// msg_export_imm$010 out_msg:^MsgEnvelope transaction:^Transaction reimport:^InMsg = OutMsg;
/// 

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OutMsgImmediately {
    pub out_msg: MsgEnvelope,
    pub transaction: Arc<Transaction>,
    pub reimport: Arc<InMsg>,
}

impl OutMsgImmediately {
    pub fn with_params(env: MsgEnvelope,
                        tr: Arc<Transaction>,
                        reimport: Arc<InMsg>) -> Self {
        OutMsgImmediately{
            out_msg: env,
            transaction: tr,
            reimport: reimport,
        }
    }

    pub fn read_message_from(cell: &mut SliceData) -> BlockResult<Message> {
        let ref_cell = cell.checked_drain_reference()?;
        if ref_cell.cell_type() == CellType::PrunedBranch {
            bail!(BlockErrorKind::PrunedCellAccess("MsgEnvelope".into()))
        }
        MsgEnvelope::read_message_from(&mut ref_cell.into())
    }
}

impl Serializable for OutMsgImmediately {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_reference(self.out_msg.write_to_new_cell()?);
        cell.append_reference(self.transaction.write_to_new_cell()?);
        cell.append_reference(self.reimport.write_to_new_cell()?);
        Ok(())
    }
}

impl Deserializable for OutMsgImmediately {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        self.out_msg.read_from(&mut cell.checked_drain_reference()?.into())?;
        self.transaction = Arc::new(Transaction::construct_from(&mut cell.checked_drain_reference()?.into())?);
        self.reimport = Arc::new(InMsg::construct_from(&mut cell.checked_drain_reference()?.into())?);
        Ok(())
    }
}

///
/// msg_export_new$001 out_msg:^MsgEnvelope transaction:^Transaction = OutMsg;
/// 

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OutMsgNew {
    pub out_msg: MsgEnvelope,
    pub transaction: Arc<Transaction>,
}

impl OutMsgNew {
    pub fn with_params(env: MsgEnvelope,
                       tr: Arc<Transaction>) -> Self {
        OutMsgNew{
            out_msg: env,
            transaction: tr,
        }
    }

    pub fn read_message_from(cell: &mut SliceData) -> BlockResult<Message> {
        let ref_cell = cell.checked_drain_reference()?;
        if ref_cell.cell_type() == CellType::PrunedBranch {
            bail!(BlockErrorKind::PrunedCellAccess("MsgEnvelope".into()))
        }
        MsgEnvelope::read_message_from(&mut ref_cell.into())
    }
}

impl Serializable for OutMsgNew {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_reference(self.out_msg.write_to_new_cell()?);
        cell.append_reference(self.transaction.write_to_new_cell()?);
        Ok(())
    }
}

impl Deserializable for OutMsgNew {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        self.out_msg.read_from(&mut cell.checked_drain_reference()?.into())?;
        self.transaction = Arc::new(Transaction::construct_from(&mut cell.checked_drain_reference()?.into())?);
        Ok(())
    }
}

///
/// msg_export_tr$011 out_msg:^MsgEnvelope imported:^InMsg = OutMsg;
/// 

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OutMsgTransit {
    pub out_msg: MsgEnvelope,
    pub imported: InMsg,
}

impl OutMsgTransit {
    pub fn with_params(env: MsgEnvelope, imported: InMsg) -> Self {
        OutMsgTransit{
            out_msg: env,
            imported: imported,
        }
    }

    pub fn read_message_from(cell: &mut SliceData) -> BlockResult<Message> {
        let ref_cell = cell.checked_drain_reference()?;
        if ref_cell.cell_type() == CellType::PrunedBranch {
            bail!(BlockErrorKind::PrunedCellAccess("MsgEnvelope".into()))
        }
        MsgEnvelope::read_message_from(&mut ref_cell.into())
    }
}

impl Serializable for OutMsgTransit {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_reference(self.out_msg.write_to_new_cell()?);
        cell.append_reference(self.imported.write_to_new_cell()?);
        Ok(())
    }
}

impl Deserializable for OutMsgTransit {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
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
    pub out_msg: MsgEnvelope,
    pub reimport: InMsg,
}

impl OutMsgDequeueImmediately {
    pub fn with_params(env: MsgEnvelope, reimport: InMsg) -> Self {
        Self {
            out_msg: env,
            reimport,
        }
    }

    pub fn read_message_from(cell: &mut SliceData) -> BlockResult<Message> {
        let ref_cell = cell.checked_drain_reference()?;
        if ref_cell.cell_type() == CellType::PrunedBranch {
            bail!(BlockErrorKind::PrunedCellAccess("MsgEnvelope".into()))
        }
        MsgEnvelope::read_message_from(&mut ref_cell.into())
    }
}

impl Serializable for OutMsgDequeueImmediately {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_reference(self.out_msg.write_to_new_cell()?);
        cell.append_reference(self.reimport.write_to_new_cell()?);
        Ok(())
    }
}

impl Deserializable for OutMsgDequeueImmediately {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
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
    pub out_msg: MsgEnvelope,
    pub import_block_lt: u64,
}

impl OutMsgDequeue {
    pub fn with_params(env: MsgEnvelope, lt: u64) -> Self {
        OutMsgDequeue{
            out_msg: env,
            import_block_lt: lt,
        }
    }

    pub fn read_message_from(cell: &mut SliceData) -> BlockResult<Message> {
        let ref_cell = cell.checked_drain_reference()?;
        if ref_cell.cell_type() == CellType::PrunedBranch {
            bail!(BlockErrorKind::PrunedCellAccess("MsgEnvelope".into()))
        }
        MsgEnvelope::read_message_from(&mut ref_cell.into())
    }
}

impl Serializable for OutMsgDequeue {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_reference(self.out_msg.write_to_new_cell()?);
        self.import_block_lt.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for OutMsgDequeue {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
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
    pub out_msg: MsgEnvelope,
    pub imported: InMsg,
}

impl OutMsgTransitRequired {
    pub fn with_params(env: MsgEnvelope,
                        imported: InMsg) -> Self {
        OutMsgTransitRequired{
            out_msg: env,
            imported: imported,
        }
    }

    pub fn read_message_from(cell: &mut SliceData) -> BlockResult<Message> {
        let ref_cell = cell.checked_drain_reference()?;
        if ref_cell.cell_type() == CellType::PrunedBranch {
            bail!(BlockErrorKind::PrunedCellAccess("MsgEnvelope".into()))
        }
        MsgEnvelope::read_message_from(&mut ref_cell.into())
    }
}

impl Serializable for OutMsgTransitRequired {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_reference(self.out_msg.write_to_new_cell()?);
        cell.append_reference(self.imported.write_to_new_cell()?);
        Ok(())
    }
}

impl Deserializable for OutMsgTransitRequired {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        self.out_msg.read_from(&mut cell.checked_drain_reference()?.into())?;
        self.imported.read_from(&mut cell.checked_drain_reference()?.into())?; 
        Ok(())
    }
}
