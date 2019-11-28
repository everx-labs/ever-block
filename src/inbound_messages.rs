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

//! # message descriptors
//!
//! Implementation of blockchain spec (3.2) structs: InMsg and InMsgDescr.
//! Serialization and deserialization of this structs.

use super::*;
use ton_types::CellType;
use std::sync::Arc;
use self::hashmapaug::Augmentable;


///internal helper macros for reading InMsg variants
macro_rules! read_msg_descr {
    ($cell:expr, $msg_descr:tt, $variant:ident) => {{
        let mut x = $msg_descr::default();
        x.read_from($cell)?;
        InMsg::$variant(x)
    }}
}

///internal helper macros for writing constructor tags in InMsg variants
macro_rules! write_ctor_tag {
    ($builder:expr, $tag:ident) => {{
        $builder.append_bits($tag as usize, 3).unwrap();
        $builder
    }}
}

//3.2.7. Augmentation of InMsgDescr
#[derive(Default, PartialEq, Eq, Clone, Debug)]
pub struct ImportFees {
    fees_collected: Grams,
    value_imported: CurrencyCollection,
}

impl Augmentable for ImportFees {
    fn calc(&mut self, other: &Self) -> BlockResult<()> {
        self.fees_collected.calc(&other.fees_collected)?;
        self.value_imported.calc(&other.value_imported)?;
        Ok(())
    }
}

impl ImportFees {
    pub fn with_grams(grams: u64) -> Self {
        Self {
            fees_collected: Grams(grams.into()),
            value_imported: CurrencyCollection::default()
        }
    }
}

impl Serializable for ImportFees {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        self.fees_collected.write_to(cell)?;
        self.value_imported.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for ImportFees {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        self.fees_collected.read_from(cell)?;
        self.value_imported.read_from(cell)?;
        Ok(())
    }
}

//constructor tags of InMsg variants (only 3 bits are used)
const MSG_IMPORT_EXT: u8 = 0b00000000;
const MSG_IMPORT_IHR: u8 = 0b00000010;
const MSG_IMPORT_IMM: u8 = 0b00000011;
const MSG_IMPORT_FIN: u8 = 0b00000100;
const MSG_IMPORT_TR: u8 = 0b00000101;
const MSG_DISCARD_FIN: u8 = 0b00000110;
const MSG_DISCARD_TR: u8 = 0b00000111;

/// 
/// Inbound message
/// blockchain spec 3.2.2. Descriptor of an inbound message.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InMsg {
    None,
    /// Inbound external messages
    External(InMsgExternal),        
    /// Internal IHR messages with destination addresses in this block
    IHR(InMsgIHR),                  
    /// Internal messages with destinations in this block
    Immediatelly(InMsgFinal),       
    /// Immediately routed internal messages
    Final(InMsgFinal),              
    /// Transit internal messages
    Transit(InMsgTransit),          
    /// Discarded internal messages with destinations in this block
    DiscardedFinal(InMsgDiscardedFinal), 
    /// Discarded transit internal messages
    DiscardedTransit(InMsgDiscardedTransit), 
}

impl Default for InMsg {
    fn default() -> Self {
        InMsg::None
    }
}


impl InMsg {

    ///
    /// Get transaction from inbound message
    /// Transaction exist only in External, IHR, Immediatlly and Final inbound messages.
    /// For other messages function returned None
    ///
    pub fn transaction(&self) -> Option<Arc<Transaction>> {
        match self {
            InMsg::External(ref x) => Some(x.transaction.clone()),
            InMsg::IHR(ref x) => Some(x.transaction.clone()),
            InMsg::Immediatelly(ref x) => Some(x.transaction.clone()),
            InMsg::Final(ref x) => Some(x.transaction.clone()),
            InMsg::Transit(ref _x) => None,
            InMsg::DiscardedFinal(ref _x) => None,
            InMsg::DiscardedTransit(ref _x) => None,
            InMsg::None => None,
        }
    }

    ///
    /// Get transaction from inbound message
    ///
    /*pub fn get_transaction_mut(&mut self) -> Option<&mut Transaction> {
        match self {
            InMsg::External(ref mut x) => Arc::get_mut(&mut x.transaction),
            InMsg::IHR(ref mut x) => Arc::get_mut(&mut x.transaction),
            InMsg::Immediatelly(ref mut x) => Arc::get_mut(&mut x.transaction),
            InMsg::Final(ref mut x) => Arc::get_mut(&mut x.transaction),
            InMsg::Transit(ref _x) => None,
            InMsg::DiscardedFinal(ref _x) => None,
            InMsg::DiscardedTransit(ref _x) => None,
            InMsg::None => None,
        }
    }*/

    ///
    /// Get value imported message
    /// Message exist only in External, IHR, Immediatlly and Final inbound messages.
    /// For other messages function returned None
    ///
    pub fn imported_value<'a>(&'a self) -> Option<&'a CurrencyCollection> {
        match self {
            InMsg::External(ref x) => x.msg.get_value(),
            InMsg::IHR(ref x) => x.msg.get_value(),
            InMsg::Immediatelly(ref x) => x.in_msg.get_message().get_value(),
            InMsg::Final(ref x) => x.in_msg.get_message().get_value(),
            InMsg::Transit(ref _x) => None,
            InMsg::DiscardedFinal(ref _x) => None,
            InMsg::DiscardedTransit(ref _x) => None,
            InMsg::None => None,
        }
    }
    
    ///
    /// Get message
    ///
    pub fn message<'a>(&'a self) -> Option<&'a Message> {
        match self {
            InMsg::External(ref x) => Some(&x.msg),
            InMsg::IHR(ref x) => Some(&x.msg),
            InMsg::Immediatelly(ref x) => Some(x.in_msg.get_message()),
            InMsg::Final(ref x) => Some(x.in_msg.get_message()),
            InMsg::Transit(ref x) => Some(x.in_msg.get_message()),
            InMsg::DiscardedFinal(ref x) => Some(x.in_msg.get_message()),
            InMsg::DiscardedTransit(ref x) => Some(x.in_msg.get_message()),
            InMsg::None => None,
        }
    }

    pub fn message_mut<'a>(&'a mut self) -> Option<&'a mut Message> {
        match self {
            InMsg::External(ref mut x) => Some(Arc::get_mut(&mut x.msg).unwrap()),
            InMsg::IHR(ref mut x) => Some(Arc::get_mut(&mut x.msg).unwrap()),
            InMsg::Immediatelly(ref mut x) => x.in_msg.get_message_mut(),
            InMsg::Final(ref mut x) => x.in_msg.get_message_mut(),
            InMsg::Transit(ref mut x) => x.in_msg.get_message_mut(),
            InMsg::DiscardedFinal(ref mut x) => x.in_msg.get_message_mut(),
            InMsg::DiscardedTransit(ref mut x) => x.in_msg.get_message_mut(),
            InMsg::None => None,
        }
    }

    pub fn get_fee<'a>(&'a self) -> Option<&'a ImportFees> {
        None
    }

    pub fn read_message_from(cell: &mut SliceData) -> BlockResult<Message> {
        let tag: u8 = (cell.get_next_bits(3)?[0] & 0xE0) >> 5;
        match tag {
            MSG_IMPORT_EXT => InMsgExternal::read_message_from(cell),
            MSG_IMPORT_IHR => InMsgIHR::read_message_from(cell),
            MSG_IMPORT_IMM => InMsgFinal::read_message_from(cell),
            MSG_IMPORT_FIN => InMsgFinal::read_message_from(cell),
            MSG_IMPORT_TR =>  InMsgTransit::read_message_from(cell),
            MSG_DISCARD_FIN => InMsgDiscardedFinal::read_message_from(cell),
            MSG_DISCARD_TR => InMsgDiscardedTransit::read_message_from(cell),
            tag => bail!(BlockErrorKind::InvalidConstructorTag(tag as u32, "InMsg".into())),
        }
    }
}


impl Serializable for InMsg {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        match self {
            InMsg::External(ref x) => x.write_to(write_ctor_tag!(cell, MSG_IMPORT_EXT)),
            InMsg::IHR(ref x) => x.write_to(write_ctor_tag!(cell, MSG_IMPORT_IHR)),
            InMsg::Immediatelly(ref x) => x.write_to(write_ctor_tag!(cell, MSG_IMPORT_IMM)),
            InMsg::Final(ref x) => x.write_to(write_ctor_tag!(cell, MSG_IMPORT_FIN)),
            InMsg::Transit(ref x) => x.write_to(write_ctor_tag!(cell, MSG_IMPORT_TR)),
            InMsg::DiscardedFinal(ref x) => x.write_to(write_ctor_tag!(cell, MSG_DISCARD_FIN)),
            InMsg::DiscardedTransit(ref x) => x.write_to(write_ctor_tag!(cell, MSG_DISCARD_TR)),
            InMsg::None => bail!(BlockErrorKind::InvalidOperation("can't serialize InMsg::None".into())),
        }
    }
}

impl Deserializable for InMsg {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        let tag: u8 = (cell.get_next_bits(3)?[0] & 0xE0) >> 5;
        *self =  match tag {
            MSG_IMPORT_EXT => read_msg_descr!(cell, InMsgExternal, External),
            MSG_IMPORT_IHR => read_msg_descr!(cell, InMsgIHR, IHR),
            MSG_IMPORT_IMM => read_msg_descr!(cell, InMsgFinal, Immediatelly),
            MSG_IMPORT_FIN => read_msg_descr!(cell, InMsgFinal, Final),
            MSG_IMPORT_TR =>  read_msg_descr!(cell, InMsgTransit, Transit),
            MSG_DISCARD_FIN => read_msg_descr!(cell, InMsgDiscardedFinal, DiscardedFinal),
            MSG_DISCARD_TR => read_msg_descr!(cell, InMsgDiscardedTransit, DiscardedTransit),
            tag => bail!(BlockErrorKind::InvalidConstructorTag(tag as u32, "InMsg".into())),
        };
        
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMsgExternal {
    pub msg: Arc<Message>,
    pub transaction: Arc<Transaction>,
}

impl InMsgExternal {
    pub fn with_params(msg: Arc<Message>, tr: Arc<Transaction>) -> Self{
        InMsgExternal {
            msg: msg,
            transaction: tr,
        }
    }

    pub fn read_message_from(cell: &mut SliceData) -> BlockResult<Message> {
        let ref_cell = cell.checked_drain_reference()?;
        if ref_cell.cell_type() == CellType::PrunedBranch {
            bail!(BlockErrorKind::PrunedCellAccess("Message".into()))
        }
        Message::construct_from(&mut ref_cell.into())
    }
}

impl Serializable for InMsgExternal {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_reference(self.msg.write_to_new_cell()?);
        cell.append_reference(self.transaction.write_to_new_cell()?);
        Ok(())
    }
}

impl Deserializable for InMsgExternal {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        self.msg = Arc::new(Message::construct_from(&mut cell.checked_drain_reference()?.into())?);
        self.transaction = Arc::new(Transaction::construct_from(&mut cell.checked_drain_reference()?.into())?);
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMsgIHR {
    pub msg: Arc<Message>,
    pub transaction: Arc<Transaction>,
    pub ihr_fee: Grams,
    pub proof_created: Arc<CellData>,
}


impl InMsgIHR {
    pub fn with_params(msg: Arc<Message>,
                        tr: Arc<Transaction>,
                        fee: Grams,
                        proof: Arc<CellData>) -> Self{
        InMsgIHR {
            msg: msg,
            transaction: tr,
            ihr_fee: fee,
            proof_created: proof
        }
    }

    pub fn read_message_from(cell: &mut SliceData) -> BlockResult<Message> {
        let ref_cell = cell.checked_drain_reference()?;
        if ref_cell.cell_type() == CellType::PrunedBranch {
            bail!(BlockErrorKind::PrunedCellAccess("Message".into()))
        }
        Message::construct_from(&mut ref_cell.into())
    }
}


impl Serializable for InMsgIHR {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_reference(self.msg.write_to_new_cell()?);
        cell.append_reference(self.transaction.write_to_new_cell()?);
        self.ihr_fee.write_to(cell)?;
        cell.append_reference(BuilderData::from(&self.proof_created));
        Ok(())
    }
}

impl Deserializable for InMsgIHR {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        self.msg = Arc::new(Message::construct_from(&mut cell.checked_drain_reference()?.into())?);
        self.transaction = Arc::new(Transaction::construct_from(&mut cell.checked_drain_reference()?.into())?);
        self.ihr_fee.read_from(cell)?;
        self.proof_created = cell.checked_drain_reference()?.clone();
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMsgFinal {
    pub in_msg: MsgEnvelope,
    pub transaction: Arc<Transaction>,
    pub fwd_fee: Grams,
}

impl InMsgFinal {
    pub fn with_params(msg: MsgEnvelope,
                        tr: Arc<Transaction>,
                        fee: Grams) -> Self{
        InMsgFinal {
            in_msg: msg,
            transaction: tr,
            fwd_fee: fee,
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

impl Serializable for InMsgFinal {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_reference(self.in_msg.write_to_new_cell()?);
        cell.append_reference(self.transaction.write_to_new_cell()?);
        self.fwd_fee.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for InMsgFinal {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        self.in_msg.read_from(&mut cell.checked_drain_reference()?.into())?;
        self.transaction = Arc::new(Transaction::construct_from(&mut cell.checked_drain_reference()?.into())?);
        self.fwd_fee.read_from(cell)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMsgTransit {
    pub in_msg: MsgEnvelope,
    pub out_msg: MsgEnvelope,
    pub transit_fee: Grams,
}

impl InMsgTransit {
    pub fn with_params(in_msg: MsgEnvelope,
                        out_msg: MsgEnvelope,
                        fee: Grams) -> Self{
        InMsgTransit {
            in_msg: in_msg,
            out_msg: out_msg,
            transit_fee: fee,
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

impl Serializable for InMsgTransit {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_reference(self.in_msg.write_to_new_cell()?);
        cell.append_reference(self.out_msg.write_to_new_cell()?);
        self.transit_fee.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for InMsgTransit {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        self.in_msg.read_from(&mut cell.checked_drain_reference()?.into())?;
        self.out_msg.read_from(&mut cell.checked_drain_reference()?.into())?;
        self.transit_fee.read_from(cell)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMsgDiscardedFinal {
    pub in_msg: MsgEnvelope,
    pub transaction_id: u64,
    pub fwd_fee: Grams,
}

impl InMsgDiscardedFinal {
    pub fn with_params(in_msg: MsgEnvelope,
                        transaction_id: u64,
                        fee: Grams) -> Self{
        InMsgDiscardedFinal {
            in_msg: in_msg,
            transaction_id: transaction_id,
            fwd_fee: fee,
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

impl Serializable for InMsgDiscardedFinal {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_reference(self.in_msg.write_to_new_cell()?);
        self.transaction_id.write_to(cell)?;
        self.fwd_fee.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for InMsgDiscardedFinal {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        self.in_msg.read_from(&mut cell.checked_drain_reference()?.into())?;
        self.transaction_id.read_from(cell)?;
        self.fwd_fee.read_from(cell)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMsgDiscardedTransit {
    pub in_msg: MsgEnvelope,
    pub transaction_id: u64,
    pub fwd_fee: Grams,
    pub proof_delivered: Arc<CellData>,
}

impl InMsgDiscardedTransit {
    pub fn with_params(in_msg: MsgEnvelope,
                        transaction_id: u64,
                        fee: Grams,
                        proof: Arc<CellData>) -> Self{
        InMsgDiscardedTransit {
            in_msg: in_msg,
            transaction_id: transaction_id,
            fwd_fee: fee,
            proof_delivered: proof
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


impl Serializable for InMsgDiscardedTransit {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.append_reference(self.in_msg.write_to_new_cell()?);
        self.transaction_id.write_to(cell)?;
        self.fwd_fee.write_to(cell)?;
        cell.append_reference(BuilderData::from(&self.proof_delivered));
        Ok(())
    }
}

impl Deserializable for InMsgDiscardedTransit {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        self.in_msg.read_from(&mut cell.checked_drain_reference()?.into())?;
        self.transaction_id.read_from(cell)?;
        self.fwd_fee.read_from(cell)?;
        self.proof_delivered = cell.checked_drain_reference()?.clone();
        Ok(())
    }
}

//3.2.8. Structure of InMsgDescr
//_ (HashmapAugE 256 InMsg ImportFees) = InMsgDescr
define_HashmapAugE!(InMsgDescr, 256, InMsg, ImportFees);

impl InMsgDescr {
    /// insert new or replace existing
    pub fn insert(&mut self, in_msg: &InMsg) -> BlockResult<()> {
        let msg = in_msg.message()
            .ok_or(BlockErrorKind::InvalidOperation("InMsg must contain message to be inserted into InMsgDescr".into()))?;
        let hash = msg.hash()?;
        self.set(&hash, &in_msg, in_msg.get_fee().unwrap_or(&ImportFees::default()))
    }

    /// insert or replace existion record
    /// use to improve speed
    pub fn insert_serialized(&mut self, key: &SliceData, msg_slice: &SliceData, fees: &ImportFees ) -> BlockResult<()> {
        if self.0.set(key.clone(), msg_slice, fees).is_ok() {
            Ok(())
        } else {
            block_err!(BlockErrorKind::Other("Error insert serialized message".to_string()))
        }
    }

}
