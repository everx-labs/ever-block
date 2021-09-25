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

//! # message descriptors
//!
//! Implementation of blockchain spec (3.2) structs: InMsg and InMsgDescr.
//! Serialization and deserialization of this structs.

use crate::{
    define_HashmapAugE,
    envelope_message::MsgEnvelope,
    error::BlockError,
    hashmapaug::{Augmentable, Augmentation, HashmapAugType},
    messages::Message,
    transactions::Transaction,
    types::{AddSub, ChildCell, CurrencyCollection, Grams},
    Serializable, Deserializable,
};
use std::fmt;
use ton_types::{
    error, fail, Result,
    BuilderData, Cell, IBitstring, SliceData, HashmapType, UInt256, hm_label,
};


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
    pub fees_collected: Grams,
    pub value_imported: CurrencyCollection,
}

impl Augmentable for ImportFees {
    fn calc(&mut self, other: &Self) -> Result<()> {
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
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.fees_collected.write_to(cell)?;
        self.value_imported.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for ImportFees {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
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
    /// msg_import_ext$000 msg:^(Message Any) transaction:^Transaction = InMsg;
    External(InMsgExternal),
    /// Internal IHR messages with destination addresses in this block
    /// msg_import_ihr$010 msg:^(Message Any) transaction:^Transaction ihr_fee:Grams proof_created:^Cell = InMsg;
    IHR(InMsgIHR),
    /// Internal messages with destinations in this block
    /// msg_import_imm$011 in_msg:^MsgEnvelope transaction:^Transaction fwd_fee:Grams = InMsg;
    Immediatelly(InMsgFinal),
    /// Immediately routed internal messages
    /// msg_import_fin$100 in_msg:^MsgEnvelope transaction:^Transaction fwd_fee:Grams = InMsg;
    Final(InMsgFinal),
    /// Transit internal messages
    /// msg_import_tr$101  in_msg:^MsgEnvelope out_msg:^MsgEnvelope transit_fee:Grams = InMsg;
    Transit(InMsgTransit),
    /// Discarded internal messages with destinations in this block
    /// msg_discard_fin$110 in_msg:^MsgEnvelope transaction_id:uint64 fwd_fee:Grams = InMsg;
    DiscardedFinal(InMsgDiscardedFinal),
    /// Discarded transit internal messages
    /// msg_discard_tr$111 in_msg:^MsgEnvelope transaction_id:uint64 fwd_fee:Grams proof_delivered:^Cell = InMsg;
    DiscardedTransit(InMsgDiscardedTransit),
}

impl Default for InMsg {
    fn default() -> Self {
        InMsg::None
    }
}


impl InMsg {
    /// Create external
    pub fn external(msg: &Message, tr: &Transaction) -> Result<InMsg> {
        Ok(InMsg::External(InMsgExternal::with_params(msg, tr)?))
    }
    /// Create IHR
    pub fn ihr(msg: &Message, tr: &Transaction, ihr_fee: Grams, proof: Cell) -> Result<InMsg> {
        Ok(InMsg::IHR(InMsgIHR::with_params(msg, tr, ihr_fee, proof)?))
    }
    /// Create Immediatelly
    pub fn immediatelly(env: &MsgEnvelope, tr: &Transaction, fwd_fee: Grams) -> Result<InMsg> {
        Ok(InMsg::Immediatelly(InMsgFinal::with_params(env, tr, fwd_fee)?))
    }
    /// Create Final
    pub fn finally(env: &MsgEnvelope, tr: &Transaction, fwd_fee: Grams) -> Result<InMsg> {
        Ok(InMsg::Final(InMsgFinal::with_params(env, tr, fwd_fee)?))
    }
    /// Create Transit
    pub fn transit(in_msg: &MsgEnvelope, out_msg: &MsgEnvelope, fwd_fee: Grams) -> Result<InMsg> {
        Ok(InMsg::Transit(InMsgTransit::with_params(in_msg, out_msg, fwd_fee)?))
    }
    /// Create DiscardedFinal
    pub fn discard_final(in_msg: &MsgEnvelope, tr_id: u64, fwd_fee: Grams) -> Result<InMsg> {
        Ok(InMsg::DiscardedFinal(InMsgDiscardedFinal::with_params(in_msg, tr_id, fwd_fee)?))
    }
    /// Create DiscardedTransit
    pub fn discard_transit(msg: &MsgEnvelope, tr_id: u64, fwd_fee: Grams, proof: Cell) -> Result<InMsg> {
        Ok(InMsg::DiscardedTransit(InMsgDiscardedTransit::with_params(msg, tr_id, fwd_fee, proof)?))
    }

    /// Check if is valid message
    pub fn is_valid(&self) -> bool {
        self != &InMsg::None
    }

    pub fn tag(&self) -> u8 {
        match self {
            InMsg::External(_)         => MSG_IMPORT_EXT,
            InMsg::IHR(_)              => MSG_IMPORT_IHR,
            InMsg::Immediatelly(_)     => MSG_IMPORT_IMM,
            InMsg::Final(_)            => MSG_IMPORT_FIN,
            InMsg::Transit(_)          => MSG_IMPORT_TR,
            InMsg::DiscardedFinal(_)   => MSG_DISCARD_FIN,
            InMsg::DiscardedTransit(_) => MSG_DISCARD_TR,
            InMsg::None => 8
        }
    }

    ///
    /// Get transaction from inbound message
    /// Transaction exist only in External, IHR, Immediatlly and Final inbound messages.
    /// For other messages function returned None
    ///
    pub fn read_transaction(&self) -> Result<Option<Transaction>> {
        Ok(
            match self {
                InMsg::External(ref x) => Some(x.read_transaction()?),
                InMsg::IHR(ref x) => Some(x.read_transaction()?),
                InMsg::Immediatelly(ref x) => Some(x.read_transaction()?),
                InMsg::Final(ref x) => Some(x.read_transaction()?),
                InMsg::Transit(ref _x) => None,
                InMsg::DiscardedFinal(ref _x) => None,
                InMsg::DiscardedTransit(ref _x) => None,
                InMsg::None => fail!("wrong message type")
            }
        )
    }

    ///
    /// Get transaction cell from inbound message
    /// Transaction exist only in External, IHR, Immediatlly and Final inbound messages.
    /// For other messages function returned None
    ///
    pub fn transaction_cell(&self) -> Option<Cell> {
        match self {
            InMsg::External(ref x) => Some(x.transaction_cell()),
            InMsg::IHR(ref x) => Some(x.transaction_cell()),
            InMsg::Immediatelly(ref x) => Some(x.transaction_cell()),
            InMsg::Final(ref x) => Some(x.transaction_cell()),
            InMsg::Transit(ref _x) => None,
            InMsg::DiscardedFinal(ref _x) => None,
            InMsg::DiscardedTransit(ref _x) => None,
            InMsg::None => None,
        }
    }

    ///
    /// Get message
    ///
    pub fn read_message(&self) -> Result<Message> {
        Ok(
            match self {
                InMsg::External(ref x) => x.read_message()?,
                InMsg::IHR(ref x) => x.read_message()?,
                InMsg::Immediatelly(ref x) => x.read_message()?.read_message()?,
                InMsg::Final(ref x) => x.read_message()?.read_message()?,
                InMsg::Transit(ref x) => x.read_in_message()?.read_message()?,
                InMsg::DiscardedFinal(ref x) => x.read_message()?.read_message()?,
                InMsg::DiscardedTransit(ref x) => x.read_message()?.read_message()?,
                InMsg::None => Default::default()
            }
        )
    }

    ///
    /// Get message cell
    ///
    pub fn message_cell(&self) -> Result<Cell> {
        Ok(
            match self {
                InMsg::External(ref x) => x.message_cell(),
                InMsg::IHR(ref x) => x.message_cell(),
                InMsg::Immediatelly(ref x) => x.read_message()?.message_cell(),
                InMsg::Final(ref x) => x.read_message()?.message_cell(),
                InMsg::Transit(ref x) => x.read_in_message()?.message_cell(),
                InMsg::DiscardedFinal(ref x) => x.read_message()?.message_cell(),
                InMsg::DiscardedTransit(ref x) => x.read_message()?.message_cell(),
                InMsg::None => Default::default()
            }
        )
    }

    ///
    /// Get in envelope message cell
    ///
    pub fn in_msg_envelope_cell(&self) -> Option<Cell> {
        match self {
            InMsg::External(_) => None,
            InMsg::IHR(_) => None,
            InMsg::Immediatelly(ref x) => Some(x.message_cell()),
            InMsg::Final(ref x) => Some(x.message_cell()),
            InMsg::Transit(ref x) => Some(x.in_message_cell()),
            InMsg::DiscardedFinal(ref x) => Some(x.message_cell()),
            InMsg::DiscardedTransit(ref x) => Some(x.message_cell()),
            InMsg::None => None,
        }
    }

    ///
    /// Get in envelope message
    ///
    pub fn read_in_msg_envelope(&self) -> Result<Option<MsgEnvelope>> {
        match self {
            InMsg::External(_) => Ok(None),
            InMsg::IHR(_) => Ok(None),
            InMsg::Immediatelly(ref x) => Some(x.read_message()).transpose(),
            InMsg::Final(ref x) => Some(x.read_message()).transpose(),
            InMsg::Transit(ref x) => Some(x.read_in_message()).transpose(),
            InMsg::DiscardedFinal(ref x) => Some(x.read_message()).transpose(),
            InMsg::DiscardedTransit(ref x) => Some(x.read_message()).transpose(),
            InMsg::None => fail!("wrong message type"),
        }
    }

    ///
    /// Get out envelope message cell
    ///
    pub fn out_msg_envelope_cell(&self) -> Option<Cell> {
        match self {
            InMsg::External(_) => None,
            InMsg::IHR(_) => None,
            InMsg::Immediatelly(_) => None,
            InMsg::Final(_) => None,
            InMsg::Transit(ref x) => Some(x.out_message_cell()),
            InMsg::DiscardedFinal(_) => None,
            InMsg::DiscardedTransit(_) => None,
            InMsg::None => None,
        }
    }

    ///
    /// Get out envelope message
    ///
    pub fn read_out_msg_envelope(&self) -> Result<Option<MsgEnvelope>> {
        match self {
            InMsg::External(_) => Ok(None),
            InMsg::IHR(_) => Ok(None),
            InMsg::Immediatelly(_) => Ok(None),
            InMsg::Final(_) => Ok(None),
            InMsg::Transit(ref x) => Some(x.read_out_message()).transpose(),
            InMsg::DiscardedFinal(_) => Ok(None),
            InMsg::DiscardedTransit(_) => Ok(None),
            InMsg::None => fail!("wrong message type")
        }
    }

    pub fn get_fee(&self) -> Result<ImportFees> { self.aug() }
}

impl Augmentation<ImportFees> for InMsg {
    fn aug(&self) -> Result<ImportFees> {
        let msg = self.read_message()?;
        let header = match msg.int_header() {
            Some(header) => header,
            None => return Ok(ImportFees::default())
        };
        let mut fees = ImportFees::default();
        match self {
            InMsg::External(_) => {
                //println!("InMsg::External");
            }
            InMsg::IHR(_) =>  {
                //println!("InMsg::IHR");
                fees.fees_collected = header.ihr_fee.clone();

                fees.value_imported = header.value.clone();
                fees.value_imported.grams.add(&header.ihr_fee)?;
            }
            InMsg::Immediatelly(_) => {
                //println!("InMsg::Immediatelly");
                fees.fees_collected = header.fwd_fee.clone();
            }
            InMsg::Final(ref x) => {
                //println!("InMsg::Final");
                let env = x.read_message()?;
                if env.fwd_fee_remaining() != x.fwd_fee() {
                    fail!("fwd_fee_remaining not equal to fwd_fee")
                }
                fees.fees_collected = env.fwd_fee_remaining().clone();

                fees.value_imported = header.value.clone();
                fees.value_imported.grams.add(env.fwd_fee_remaining())?;
                fees.value_imported.grams.add(&header.ihr_fee)?;
            }
            InMsg::Transit(ref x) => {
                //println!("InMsg::Transit");
                let env = x.read_in_message()?;
                if env.fwd_fee_remaining() < x.transit_fee() {
                    fail!("fwd_fee_remaining less than transit_fee")
                }

                fees.fees_collected = x.transit_fee().clone();

                fees.value_imported = header.value.clone();
                fees.value_imported.grams.add(&header.ihr_fee)?;
                fees.value_imported.grams.add(env.fwd_fee_remaining())?;
            }
            InMsg::DiscardedFinal(_) => {
                //println!("InMsg::DiscardedFinal");
                fees.fees_collected = header.fwd_fee.clone();

                fees.value_imported.grams = header.fwd_fee.clone();
            }
            InMsg::DiscardedTransit(_) => {
                //println!("InMsg::DiscardedTransit");
                fees.fees_collected = header.fwd_fee.clone();

                fees.value_imported.grams = header.fwd_fee.clone();
            }
            InMsg::None => fail!("wrong InMsg type")
        }
        Ok(fees)
    }
}

impl Serializable for InMsg {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        match self {
            InMsg::External(ref x) => x.write_to(write_ctor_tag!(cell, MSG_IMPORT_EXT)),
            InMsg::IHR(ref x) => x.write_to(write_ctor_tag!(cell, MSG_IMPORT_IHR)),
            InMsg::Immediatelly(ref x) => x.write_to(write_ctor_tag!(cell, MSG_IMPORT_IMM)),
            InMsg::Final(ref x) => x.write_to(write_ctor_tag!(cell, MSG_IMPORT_FIN)),
            InMsg::Transit(ref x) => x.write_to(write_ctor_tag!(cell, MSG_IMPORT_TR)),
            InMsg::DiscardedFinal(ref x) => x.write_to(write_ctor_tag!(cell, MSG_DISCARD_FIN)),
            InMsg::DiscardedTransit(ref x) => x.write_to(write_ctor_tag!(cell, MSG_DISCARD_TR)),
            InMsg::None => Ok(()), // Due to ChildCell it is need sometimes to serialize default InMsg
        }
    }
}

impl Deserializable for InMsg {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag: u8 = (cell.get_next_bits(3)?[0] & 0xE0) >> 5;
        *self =  match tag {
            MSG_IMPORT_EXT => read_msg_descr!(cell, InMsgExternal, External),
            MSG_IMPORT_IHR => read_msg_descr!(cell, InMsgIHR, IHR),
            MSG_IMPORT_IMM => read_msg_descr!(cell, InMsgFinal, Immediatelly),
            MSG_IMPORT_FIN => read_msg_descr!(cell, InMsgFinal, Final),
            MSG_IMPORT_TR =>  read_msg_descr!(cell, InMsgTransit, Transit),
            MSG_DISCARD_FIN => read_msg_descr!(cell, InMsgDiscardedFinal, DiscardedFinal),
            MSG_DISCARD_TR => read_msg_descr!(cell, InMsgDiscardedTransit, DiscardedTransit),
            tag => fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: "InMsg".to_string()
                }
            )
        };        
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMsgExternal {
    msg: ChildCell<Message>,
    transaction: ChildCell<Transaction>,
}

impl InMsgExternal {
    pub fn with_params(msg: &Message, tr: &Transaction) -> Result<Self> {
        Ok(
            InMsgExternal {
                msg: ChildCell::with_struct(msg)?,
                transaction: ChildCell::with_struct(tr)?,
            }
        )
    }

    pub fn read_message(&self) -> Result<Message> {
        self.msg.read_struct()
    }

    pub fn message_cell(&self)-> Cell {
        self.msg.cell()
    }

    pub fn read_transaction(&self) -> Result<Transaction> {
        self.transaction.read_struct()
    }

    pub fn transaction_cell(&self)-> Cell {
        self.transaction.cell()
    }
}

impl Serializable for InMsgExternal {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_reference_cell(self.msg.serialize()?);
        cell.append_reference_cell(self.transaction.serialize()?);
        Ok(())
    }
}

impl Deserializable for InMsgExternal {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.msg.read_from_reference(cell)?;
        self.transaction.read_from_reference(cell)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMsgIHR {
    msg: ChildCell<Message>,
    transaction: ChildCell<Transaction>,
    ihr_fee: Grams,
    proof_created: Cell,
}


impl InMsgIHR {
    pub fn with_params(
        msg: &Message,
        tr: &Transaction,
        ihr_fee: Grams,
        proof_created: Cell) -> Result<Self> {

        Ok(
            InMsgIHR {
                msg: ChildCell::with_struct(msg)?,
                transaction: ChildCell::with_struct(tr)?,
                ihr_fee,
                proof_created
            }
        )
    }

    pub fn read_message(&self) -> Result<Message> {
        self.msg.read_struct()
    }

    pub fn message_cell(&self)-> Cell {
        self.msg.cell()
    }

    pub fn read_transaction(&self) -> Result<Transaction> {
        self.transaction.read_struct()
    }

    pub fn transaction_cell(&self)-> Cell {
        self.transaction.cell()
    }

    pub fn ihr_fee(&self) -> &Grams {
        &self.ihr_fee
    }

    pub fn proof_created(&self)-> &Cell {
        &self.proof_created
    }
}


impl Serializable for InMsgIHR {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_reference_cell(self.msg.serialize()?);
        cell.append_reference_cell(self.transaction.serialize()?);
        self.ihr_fee.write_to(cell)?;
        cell.append_reference_cell(self.proof_created.clone());
        Ok(())
    }
}

impl Deserializable for InMsgIHR {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.msg.read_from_reference(cell)?;
        self.transaction.read_from_reference(cell)?;
        self.ihr_fee.read_from(cell)?;
        self.proof_created = cell.checked_drain_reference()?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMsgFinal {
    in_msg: ChildCell<MsgEnvelope>,
    transaction: ChildCell<Transaction>,
    pub fwd_fee: Grams,
}

impl InMsgFinal {
    pub fn with_params(env: &MsgEnvelope, tr: &Transaction, fwd_fee: Grams) -> Result<Self> {
        Ok(
            InMsgFinal {
                in_msg: ChildCell::with_struct(env)?,
                transaction: ChildCell::with_struct(tr)?,
                fwd_fee,
            }
        )
    }

    pub fn read_message(&self) -> Result<MsgEnvelope> {
        self.in_msg.read_struct()
    }

    pub fn message_cell(&self)-> Cell {
        self.in_msg.cell()
    }

    pub fn read_transaction(&self) -> Result<Transaction> {
        self.transaction.read_struct()
    }

    pub fn transaction_cell(&self)-> Cell {
        self.transaction.cell()
    }

    pub fn fwd_fee(&self) -> &Grams {
        &self.fwd_fee
    }
}

impl Serializable for InMsgFinal {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_reference_cell(self.in_msg.serialize()?);
        cell.append_reference_cell(self.transaction.serialize()?);
        self.fwd_fee.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for InMsgFinal {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.in_msg.read_from_reference(cell)?;
        self.transaction.read_from_reference(cell)?;
        self.fwd_fee.read_from(cell)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMsgTransit {
    in_msg: ChildCell<MsgEnvelope>,
    out_msg: ChildCell<MsgEnvelope>,
    pub transit_fee: Grams,
}

impl InMsgTransit {
    pub fn with_params(in_msg: &MsgEnvelope, out_msg: &MsgEnvelope, fee: Grams) -> Result<Self> {
        Ok(
            InMsgTransit {
                in_msg: ChildCell::with_struct(in_msg)?,
                out_msg: ChildCell::with_struct(out_msg)?,
                transit_fee: fee,
            }
        )
    }

    pub fn read_in_message(&self) -> Result<MsgEnvelope> {
        self.in_msg.read_struct()
    }

    pub fn in_message_cell(&self)-> Cell {
        self.in_msg.cell()
    }

    pub fn read_out_message(&self) -> Result<MsgEnvelope> {
        self.out_msg.read_struct()
    }

    pub fn out_message_cell(&self)-> Cell {
        self.out_msg.cell()
    }

    pub fn transit_fee(&self) -> &Grams {
        &self.transit_fee
    }
}

impl Serializable for InMsgTransit {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_reference_cell(self.in_msg.serialize()?);
        cell.append_reference_cell(self.out_msg.serialize()?);
        self.transit_fee.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for InMsgTransit {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.in_msg.read_from_reference(cell)?;
        self.out_msg.read_from_reference(cell)?;
        self.transit_fee.read_from(cell)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMsgDiscardedFinal {
    in_msg: ChildCell<MsgEnvelope>,
    pub transaction_id: u64,
    pub fwd_fee: Grams,
}

impl InMsgDiscardedFinal {
    pub fn with_params(in_msg: &MsgEnvelope, transaction_id: u64, fee: Grams) -> Result<Self> {
        Ok(
            InMsgDiscardedFinal {
                in_msg: ChildCell::with_struct(in_msg)?,
                transaction_id,
                fwd_fee: fee,
            }
        )
    }

    pub fn read_message(&self) -> Result<MsgEnvelope> {
        self.in_msg.read_struct()
    }

    pub fn message_cell(&self)-> Cell {
        self.in_msg.cell()
    }

    pub fn transaction_id(&self) -> u64 {
        self.transaction_id
    }

    pub fn fwd_fee(&self) -> &Grams {
        &self.fwd_fee
    }
}

impl Serializable for InMsgDiscardedFinal {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_reference_cell(self.in_msg.serialize()?);
        self.transaction_id.write_to(cell)?;
        self.fwd_fee.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for InMsgDiscardedFinal {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.in_msg.read_from_reference(cell)?;
        self.transaction_id.read_from(cell)?;
        self.fwd_fee.read_from(cell)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMsgDiscardedTransit {
    in_msg: ChildCell<MsgEnvelope>,
    transaction_id: u64,
    fwd_fee: Grams,
    proof_delivered: Cell,
}

impl InMsgDiscardedTransit {
    pub fn with_params(msg: &MsgEnvelope, transaction_id: u64, fee: Grams, proof: Cell) 
    -> Result<Self> {
        Ok(
            InMsgDiscardedTransit {
                in_msg: ChildCell::with_struct(msg)?,
                transaction_id,
                fwd_fee: fee,
                proof_delivered: proof
            }
        )
    }

    pub fn read_message(&self) -> Result<MsgEnvelope> {
        self.in_msg.read_struct()
    }

    pub fn message_cell(&self)-> Cell {
        self.in_msg.cell()
    }

    pub fn transaction_id(&self) -> u64 {
        self.transaction_id
    }

    pub fn fwd_fee(&self) -> &Grams {
        &self.fwd_fee
    }

    pub fn proof_delivered(&self)-> &Cell {
        &self.proof_delivered
    }
}

impl Serializable for InMsgDiscardedTransit {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_reference_cell(self.in_msg.serialize()?);
        self.transaction_id.write_to(cell)?;
        self.fwd_fee.write_to(cell)?;
        cell.append_reference_cell(self.proof_delivered.clone());
        Ok(())
    }
}

impl Deserializable for InMsgDiscardedTransit {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.in_msg.read_from_reference(cell)?;
        self.transaction_id.read_from(cell)?;
        self.fwd_fee.read_from(cell)?;
        self.proof_delivered = cell.checked_drain_reference()?;
        Ok(())
    }
}

//3.2.8. Structure of InMsgDescr
//_ (HashmapAugE 256 InMsg ImportFees) = InMsgDescr
define_HashmapAugE!(InMsgDescr, 256, UInt256, InMsg, ImportFees);

impl InMsgDescr {
    /// insert new or replace existing, key - hash of Message
    pub fn insert_with_key(&mut self, key: UInt256, in_msg: &InMsg) -> Result<()> {
        let aug = in_msg.aug()?;
        self.set(&key, in_msg, &aug)
    }

    /// insert new or replace existing
    pub fn insert(&mut self, in_msg: &InMsg) -> Result<()> {
        self.insert_with_key(in_msg.message_cell()?.repr_hash(), in_msg)
    }

    /// insert or replace existion record
    /// use to improve speed
    pub fn insert_serialized(&mut self, key: &SliceData, msg_slice: &SliceData, fees: &ImportFees ) -> Result<()> {
        if self.set_builder_serialized(key.clone(), &BuilderData::from_slice(msg_slice), fees).is_ok() {
            Ok(())
        } else {
            fail!(BlockError::Other("Error insert serialized message".to_string()))
        }
    }

    pub fn full_import_fees(&self) -> &ImportFees {
        self.root_extra()
    }
}
