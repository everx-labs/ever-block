/*
* Copyright (C) 2019-2024 EverX. All Rights Reserved.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific EVERX DEV software governing permissions and
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
    dictionary::hashmapaug::{Augmentable, Augmentation, HashmapAugType},
    messages::Message,
    common_message::CommonMessage,
    transactions::Transaction,
    types::{AddSub, ChildCell, CurrencyCollection, Grams},
    Serializable, Deserializable,
    error, fail, Result, SERDE_OPTS_EMPTY, SERDE_OPTS_COMMON_MESSAGE,
    BuilderData, Cell, IBitstring, SliceData, UInt256, hm_label,
};
use std::fmt;

#[cfg(test)]
#[path = "tests/test_in_msgs.rs"]
mod tests;

///internal helper macros for reading InMsg variants
macro_rules! read_descr {
    ($cell:expr, $msg_descr:tt, $variant:ident, $opts:ident) => {{
        let mut x = $msg_descr::default();
        x.read_from_with_opts($cell, $opts)?;
        InMsg::$variant(x)
    }}
}

///internal helper macros for writing constructor tags in InMsg variants
macro_rules! write_tag {
    ($builder:expr, $tag:ident, $bits:ident, $len:expr) => {{
        $builder.append_bits(($bits | $tag) as usize, $len).unwrap();
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
    fn calc(&mut self, other: &Self) -> Result<bool> {
        let mut result = self.fees_collected.calc(&other.fees_collected)?;
        result |= self.value_imported.calc(&other.value_imported)?;
        Ok(result)
    }
}

impl ImportFees {
    pub fn with_grams(grams: u64) -> Self {
        Self {
            fees_collected: Grams::from(grams),
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
const COMMON_MSG_TAG: u8 = 0b1000_0000;

/// 
/// Inbound message
/// blockchain spec 3.2.2. Descriptor of an inbound message.
///
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum InMsg {
    #[default]
    None,
    /// Inbound external messages
    /// msg_import_ext$000 msg:^(Message Any) transaction:^Transaction = InMsg;
    External(InMsgExternal),
    /// Internal IHR messages with destination addresses in this block
    /// msg_import_ihr$010 msg:^(Message Any) transaction:^Transaction ihr_fee:Grams proof_created:^Cell = InMsg;
    IHR(InMsgIHR),
    /// Internal messages with destinations in this block
    /// msg_import_imm$011 in_msg:^MsgEnvelope transaction:^Transaction fwd_fee:Grams = InMsg;
    Immediate(InMsgFinal),
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

impl fmt::Display for InMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg_hash = self.message_cell().unwrap_or_default().repr_hash();
        let tr_hash = self.transaction_cell().unwrap_or_default().repr_hash();
        match self {
            InMsg::External(_x) => write!(f, "InMsg msg_import_ext$000 msg: {:x} tr: {:x}",
                msg_hash, tr_hash),
            InMsg::IHR(_x) => write!(f, "InMsg msg_import_ihr$010 msg: {:x} tr: {:x}",
                msg_hash, tr_hash),
            InMsg::Immediate(x) => write!(f, "InMsg msg_import_imm$011 msg: {:x} tr: {:x} fee: {}",
                msg_hash, tr_hash, x.fwd_fee),
            InMsg::Transit(x) => write!(f, "InMsg msg_import_tr$101 in_msg: {:x} out_msg: {:x} fee: {}",
                msg_hash, x.out_msg.read_struct().unwrap_or_default().message_hash(), x.transit_fee),
            InMsg::Final(x) => write!(f, "InMsg msg_import_fin$100 msg: {:x} tr: {:x} fee: {}",
                msg_hash, tr_hash, x.fwd_fee),
            InMsg::DiscardedFinal(x) => write!(f, "InMsg msg_discard_fin$110 msg: {:x} tr: {} fee: {}",
                msg_hash, x.transaction_id, x.fwd_fee),
            InMsg::DiscardedTransit(x) => write!(f, "InMsg msg_discard_tr$111 msg: {:x} tr: {:x} fee: {} proof: {:x}",
                msg_hash, x.transaction_id, x.fwd_fee, x.proof_delivered.repr_hash()),
            InMsg::None => write!(f, "InMsg msg_unknown")
        }
    }
}

impl InMsg {
    /// Create external
    pub fn external(
        msg_cell: ChildCell<CommonMessage>,
        tr_cell: ChildCell<Transaction>,
    ) -> InMsg {
        InMsg::External(InMsgExternal::with_cells(msg_cell, tr_cell))
    }
    /// Create IHR
    pub fn ihr(
        msg_cell: ChildCell<CommonMessage>,
        tr_cell: ChildCell<Transaction>,
        ihr_fee: Grams,
        proof: Cell,
    ) -> InMsg {
        InMsg::IHR(InMsgIHR::with_cells(msg_cell, tr_cell, ihr_fee, proof))
    }
    /// Create Immediate
    pub fn immediate(
        env_cell: ChildCell<MsgEnvelope>,
        tr_cell: ChildCell<Transaction>,
        fwd_fee: Grams,
    ) -> InMsg {
        InMsg::Immediate(InMsgFinal::with_cells(env_cell, tr_cell, fwd_fee))
    }
    /// Create Final
    pub fn final_msg(
        env_cell: ChildCell<MsgEnvelope>,
        tr_cell: ChildCell<Transaction>,
        fwd_fee: Grams,
    ) -> InMsg {
        InMsg::Final(InMsgFinal::with_cells(env_cell, tr_cell, fwd_fee))
    }
    /// Create Transit
    pub fn transit(
        in_msg_cell: ChildCell<MsgEnvelope>,
        out_msg_cell: ChildCell<MsgEnvelope>,
        fwd_fee: Grams,
    ) -> InMsg {
        InMsg::Transit(InMsgTransit::with_cells(in_msg_cell, out_msg_cell, fwd_fee))
    }
    /// Create DiscardedFinal
    pub fn discarded_final(
        env_cell: ChildCell<MsgEnvelope>,
        tr_id: u64,
        fwd_fee: Grams,
    ) -> InMsg {
        InMsg::DiscardedFinal(InMsgDiscardedFinal::with_cells(env_cell, tr_id, fwd_fee))
    }
    /// Create DiscardedTransit
    pub fn discarded_transit(
        env_cell: ChildCell<MsgEnvelope>,
        tr_id: u64,
        fwd_fee: Grams,
        proof: Cell,
    ) -> InMsg {
        InMsg::DiscardedTransit(InMsgDiscardedTransit::with_cells(env_cell, tr_id, fwd_fee, proof))
    }

    /// Check if is valid message
    pub fn is_valid(&self) -> bool {
        self != &InMsg::None
    }

    pub fn tag(&self) -> u8 {
        match self {
            InMsg::External(_)             => MSG_IMPORT_EXT,
            InMsg::IHR(_)                  => MSG_IMPORT_IHR,
            InMsg::Immediate(_)            => MSG_IMPORT_IMM,
            InMsg::Final(_)                => MSG_IMPORT_FIN,
            InMsg::Transit(_)              => MSG_IMPORT_TR,
            InMsg::DiscardedFinal(_)       => MSG_DISCARD_FIN,
            InMsg::DiscardedTransit(_)     => MSG_DISCARD_TR,
            InMsg::None => 8
        }
    }

    ///
    /// Get transaction from inbound message
    /// Transaction exist only in External, IHR, Immediate and Final inbound messages.
    /// For other messages function returned None
    ///
    pub fn read_transaction(&self) -> Result<Option<Transaction>> {
        Ok(
            match self {
                InMsg::External(ref x) => Some(x.read_transaction()?),
                InMsg::IHR(ref x) => Some(x.read_transaction()?),
                InMsg::Immediate(ref x) => Some(x.read_transaction()?),
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
    /// Transaction exist only in External, IHR, Immediate and Final inbound messages.
    /// For other messages function returned None
    ///
    pub fn transaction_cell(&self) -> Option<Cell> {
        match self {
            InMsg::External(ref x) => Some(x.transaction_cell()),
            InMsg::IHR(ref x) => Some(x.transaction_cell()),
            InMsg::Immediate(ref x) => Some(x.transaction_cell()),
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
        match self {
            InMsg::External(ref x) => x.read_message(),
            InMsg::IHR(ref x) => x.read_message(),
            InMsg::Immediate(ref x) => x.read_envelope_message()?.read_message(),
            InMsg::Final(ref x) => x.read_envelope_message()?.read_message(),
            InMsg::Transit(ref x) => x.read_in_message()?.read_message(),
            InMsg::DiscardedFinal(ref x) => x.read_envelope_message()?.read_message(),
            InMsg::DiscardedTransit(ref x) => x.read_envelope_message()?.read_message(),
            InMsg::None => fail!("wrong msg type")
        }
    }

    ///
    /// Get message cell
    ///
    pub fn message_cell(&self) -> Result<Cell> {
        Ok(
            match self {
                InMsg::External(ref x) => x.message_cell(),
                InMsg::IHR(ref x) => x.message_cell(),
                InMsg::Immediate(ref x) => x.read_envelope_message()?.message_cell(),
                InMsg::Final(ref x) => x.read_envelope_message()?.message_cell(),
                InMsg::Transit(ref x) => x.read_in_message()?.message_cell(),
                InMsg::DiscardedFinal(ref x) => x.read_envelope_message()?.message_cell(),
                InMsg::DiscardedTransit(ref x) => x.read_envelope_message()?.message_cell(),
                InMsg::None => fail!("wrong message type")
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
            InMsg::Immediate(ref x) => Some(x.envelope_message_cell()),
            InMsg::Final(ref x) => Some(x.envelope_message_cell()),
            InMsg::Transit(ref x) => Some(x.in_msg.cell()),
            InMsg::DiscardedFinal(ref x) => Some(x.envelope_message_cell()),
            InMsg::DiscardedTransit(ref x) => Some(x.in_msg.cell()),
            InMsg::None => None,
        }
    }

    ///
    /// Get in envelope message
    ///
    pub fn read_in_msg_envelope(&self) -> Result<Option<MsgEnvelope>> {
        Ok(
            match self {
                InMsg::External(_) => None,
                InMsg::IHR(_) => None,
                InMsg::Immediate(ref x) => Some(x.read_envelope_message()?),
                InMsg::Final(ref x) => Some(x.read_envelope_message()?),
                InMsg::Transit(ref x) => Some(x.read_in_message()?),
                InMsg::DiscardedFinal(ref x) => Some(x.read_envelope_message()?),
                InMsg::DiscardedTransit(ref x) => Some(x.read_envelope_message()?),
                InMsg::None => fail!("wrong message type"),
            }
        )
    }

    ///
    /// Get out envelope message cell
    ///
    pub fn out_msg_envelope_cell(&self) -> Option<Cell> {
        match self {
            InMsg::External(_) => None,
            InMsg::IHR(_) => None,
            InMsg::Immediate(_) => None,
            InMsg::Final(_) => None,
            InMsg::Transit(ref x) => Some(x.out_msg.cell()),
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
            InMsg::Immediate(_) => Ok(None),
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
                fees.fees_collected = header.ihr_fee;

                fees.value_imported = header.value.clone();
                fees.value_imported.grams.add(&header.ihr_fee)?;
            }
            InMsg::Immediate(_) => {
                //println!("InMsg::Immediate");
                fees.fees_collected = header.fwd_fee;
            }
            InMsg::Final(ref x) => {
                //println!("InMsg::Final");
                let env = x.read_envelope_message()?;
                if env.fwd_fee_remaining() != x.fwd_fee() {
                    fail!("fwd_fee_remaining not equal to fwd_fee")
                }
                fees.fees_collected = *env.fwd_fee_remaining();

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

                fees.fees_collected = *x.transit_fee();

                fees.value_imported = header.value.clone();
                fees.value_imported.grams.add(&header.ihr_fee)?;
                fees.value_imported.grams.add(env.fwd_fee_remaining())?;
            }
            InMsg::DiscardedFinal(_) => {
                //println!("InMsg::DiscardedFinal");
                fees.fees_collected = header.fwd_fee;

                fees.value_imported.grams = header.fwd_fee;
            }
            InMsg::DiscardedTransit(_) => {
                //println!("InMsg::DiscardedTransit");
                fees.fees_collected = header.fwd_fee;

                fees.value_imported.grams = header.fwd_fee;
            }
            InMsg::None => fail!("wrong InMsg type")
        }
        Ok(fees)
    }
}

impl Serializable for InMsg {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.write_with_opts(cell, SERDE_OPTS_EMPTY)
    }
    fn write_with_opts(&self, cell: &mut BuilderData, opts: u8) -> Result<()> {
        let (opt_bits, len) = if opts == SERDE_OPTS_COMMON_MESSAGE { 
            (COMMON_MSG_TAG, 8)
        } else {
            (0, 3)
        };
        match self {
            InMsg::External(ref x) => 
                x.write_with_opts(write_tag!(cell, MSG_IMPORT_EXT, opt_bits, len), opts),
            InMsg::IHR(ref x) => 
                x.write_with_opts(write_tag!(cell, MSG_IMPORT_IHR, opt_bits, len), opts),
            InMsg::Immediate(ref x) => 
                x.write_with_opts(write_tag!(cell, MSG_IMPORT_IMM, opt_bits, len), opts),
            InMsg::Final(ref x) => 
                x.write_with_opts(write_tag!(cell, MSG_IMPORT_FIN, opt_bits, len), opts),
            InMsg::Transit(ref x) => 
                x.write_with_opts(write_tag!(cell, MSG_IMPORT_TR, opt_bits, len), opts),
            InMsg::DiscardedFinal(ref x) => 
                x.write_with_opts(write_tag!(cell, MSG_DISCARD_FIN, opt_bits, len), opts),
            InMsg::DiscardedTransit(ref x) => 
                x.write_with_opts(write_tag!(cell, MSG_DISCARD_TR, opt_bits, len), opts),
            InMsg::None => Ok(()), // Due to ChildCell it is need sometimes to serialize default InMsg
        }
    }
}

impl Deserializable for InMsg {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.read_from_with_opts(cell, SERDE_OPTS_EMPTY)
    }
    fn read_from_with_opts(&mut self, cell: &mut SliceData, opts: u8) -> Result<()> {
        let tag = match opts {
            SERDE_OPTS_COMMON_MESSAGE => cell.get_next_byte()? & !COMMON_MSG_TAG,
            _ => (cell.get_next_bits(3)?[0] & 0xE0) >> 5,
        };
        *self =  match tag {
            MSG_IMPORT_EXT => read_descr!(cell, InMsgExternal, External, opts),
            MSG_IMPORT_IHR => read_descr!(cell, InMsgIHR, IHR, opts),
            MSG_IMPORT_IMM => read_descr!(cell, InMsgFinal, Immediate, opts),
            MSG_IMPORT_FIN => read_descr!(cell, InMsgFinal, Final, opts),
            MSG_IMPORT_TR =>  read_descr!(cell, InMsgTransit, Transit, opts),
            MSG_DISCARD_FIN => read_descr!(cell, InMsgDiscardedFinal, DiscardedFinal, opts),
            MSG_DISCARD_TR => read_descr!(cell, InMsgDiscardedTransit, DiscardedTransit, opts),
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
    msg: ChildCell<CommonMessage>,
    transaction: ChildCell<Transaction>,
}

impl InMsgExternal {
    pub fn with_cells(msg_cell: ChildCell<CommonMessage>, tr_cell: ChildCell<Transaction>) -> Self {
        InMsgExternal {
            msg: msg_cell,
            transaction: tr_cell,
        }
    }

    pub fn read_message(&self) -> Result<Message> {
        let msg = self.msg.read_struct()?;
        match msg {
            CommonMessage::Std(m) => Ok(m),
            _ => fail!(BlockError::UnexpectedStructVariant(
                "CommonMessage::Std".to_string(),
                msg.get_type_name()
            ))
        }
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
        self.msg.write_to(cell)?;
        self.transaction.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for InMsgExternal {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        self.read_from_with_opts(slice, SERDE_OPTS_EMPTY)
    }
    fn read_from_with_opts(&mut self, slice: &mut SliceData, opts: u8) -> Result<()> {
        self.msg.read_from_with_opts(slice, opts)?;
        self.transaction.read_from_with_opts(slice, opts)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMsgIHR {
    msg: ChildCell<CommonMessage>,
    transaction: ChildCell<Transaction>,
    ihr_fee: Grams,
    proof_created: Cell,
}


impl InMsgIHR {
    pub fn with_cells(
        msg: ChildCell<CommonMessage>,
        transaction: ChildCell<Transaction>,
        ihr_fee: Grams,
        proof_created: Cell,
    ) -> Self {
        InMsgIHR {
            msg,
            transaction,
            ihr_fee,
            proof_created
        }
    }

    pub fn read_message(&self) -> Result<Message> {
        let msg = self.msg.read_struct()?;
        match msg {
            CommonMessage::Std(msg) => Ok(msg),
            _ => fail!(
                BlockError::UnexpectedStructVariant(
                    "CommonMessage::Std".to_string(),
                    msg.get_type_name()
                )
            ),
        }
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
        self.msg.write_to(cell)?;
        self.transaction.write_to(cell)?;
        self.ihr_fee.write_to(cell)?;
        self.proof_created.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for InMsgIHR {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        self.read_from_with_opts(slice, SERDE_OPTS_EMPTY)
    }
    fn read_from_with_opts(&mut self, slice: &mut SliceData, opts: u8) -> Result<()> {
        self.msg.read_from_with_opts(slice, opts)?;
        self.transaction.read_from_with_opts(slice, opts)?;
        self.ihr_fee.read_from_with_opts(slice, opts)?;
        self.proof_created.read_from(slice)?;
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
    pub fn with_cells(
        in_msg: ChildCell<MsgEnvelope>,
        transaction: ChildCell<Transaction>,
        fwd_fee: Grams,
    ) -> Self {
        InMsgFinal {
            in_msg,
            transaction,
            fwd_fee,
        }
    }

    pub fn read_envelope_message(&self) -> Result<MsgEnvelope> {
        self.in_msg.read_struct()
    }

    pub fn envelope_message_cell(&self) -> Cell {
        self.in_msg.cell()
    }

    pub fn envelope_message_hash(&self) -> UInt256 {
        self.in_msg.hash()
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
        self.in_msg.write_to(cell)?;
        self.transaction.write_to(cell)?;
        self.fwd_fee.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for InMsgFinal {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        self.read_from_with_opts(slice, SERDE_OPTS_EMPTY)
    }
    fn read_from_with_opts(&mut self, slice: &mut SliceData, opts: u8) -> Result<()> {
        self.in_msg.read_from_with_opts(slice, opts)?;
        self.transaction.read_from_with_opts(slice, opts)?;
        self.fwd_fee.read_from_with_opts(slice, opts)?;
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
    pub fn with_cells(
        in_msg: ChildCell<MsgEnvelope>,
        out_msg: ChildCell<MsgEnvelope>,
        fee: Grams,
    ) -> Self {
        InMsgTransit {
            in_msg,
            out_msg,
            transit_fee: fee,
        }
    }

    pub fn read_in_message(&self) -> Result<MsgEnvelope> {
        self.in_msg.read_struct()
    }

    pub fn read_out_message(&self) -> Result<MsgEnvelope> {
        self.out_msg.read_struct()
    }

    pub fn in_envelope_message_cell(&self)-> Cell {
        self.in_msg.cell()
    }

    pub fn in_envelope_message_hash(&self)-> UInt256 {
        self.in_msg.hash()
    }

    pub fn out_envelope_message_cell(&self)-> Cell {
        self.out_msg.cell()
    }

    pub fn transit_fee(&self) -> &Grams {
        &self.transit_fee
    }
}

impl Serializable for InMsgTransit {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.in_msg.write_to(cell)?;
        self.out_msg.write_to(cell)?;
        self.transit_fee.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for InMsgTransit {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        self.read_from_with_opts(slice, SERDE_OPTS_EMPTY)
    }
    fn read_from_with_opts(&mut self, slice: &mut SliceData, opts: u8) -> Result<()> {
        self.in_msg.read_from_with_opts(slice, opts)?;
        self.out_msg.read_from_with_opts(slice, opts)?;
        self.transit_fee.read_from_with_opts(slice, opts)?;
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
    pub fn with_cells(
        in_msg: ChildCell<MsgEnvelope>,
        transaction_id: u64,
        fee: Grams,
    ) -> Self {
        InMsgDiscardedFinal {
            in_msg,
            transaction_id,
            fwd_fee: fee,
        }
    }

    pub fn read_envelope_message(&self) -> Result<MsgEnvelope> {
        self.in_msg.read_struct()
    }

    pub fn envelope_message_cell(&self) -> Cell {
        self.in_msg.cell()
    }

    pub fn envelope_message_hash(&self) -> UInt256 {
        self.in_msg.hash()
    }

    pub fn message_cell(&self)-> Result<Cell> {
        Ok(self.read_envelope_message()?.message_cell())
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
        self.in_msg.write_to(cell)?;
        self.transaction_id.write_to(cell)?;
        self.fwd_fee.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for InMsgDiscardedFinal {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        self.read_from_with_opts(slice, SERDE_OPTS_EMPTY)
    }
    fn read_from_with_opts(&mut self, slice: &mut SliceData, opts: u8) -> Result<()> {
        self.in_msg.read_from_with_opts(slice, opts)?;
        self.transaction_id.read_from_with_opts(slice, opts)?;
        self.fwd_fee.read_from_with_opts(slice, opts)?;
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
    pub fn with_cells(
        in_msg: ChildCell<MsgEnvelope>,
        transaction_id: u64,
        fee: Grams,
        proof: Cell,
    ) -> Self {
        InMsgDiscardedTransit {
            in_msg,
            transaction_id,
            fwd_fee: fee,
            proof_delivered: proof
        }
    }

    pub fn read_envelope_message(&self) -> Result<MsgEnvelope> {
        self.in_msg.read_struct()
    }

    pub fn envelope_message_cell(&self) -> Cell {
        self.in_msg.cell()
    }

    pub fn envelope_message_hash(&self) -> UInt256 {
        self.in_msg.hash()
    }

    pub fn message_cell(&self)-> Result<Cell> {
        Ok(self.in_msg.read_struct()?.message_cell())
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
        self.in_msg.write_to(cell)?;
        self.transaction_id.write_to(cell)?;
        self.fwd_fee.write_to(cell)?;
        self.proof_delivered.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for InMsgDiscardedTransit {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        self.read_from_with_opts(slice, SERDE_OPTS_EMPTY)
    }
    fn read_from_with_opts(&mut self, slice: &mut SliceData, opts: u8) -> Result<()> {
        self.in_msg.read_from_with_opts(slice, opts)?;
        self.transaction_id.read_from_with_opts(slice, opts)?;
        self.fwd_fee.read_from_with_opts(slice, opts)?;
        self.proof_delivered.read_from(slice)?;
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
    pub fn insert_serialized(
        &mut self, 
        key: &SliceData, 
        msg_slice: &SliceData, 
        fees: &ImportFees
    ) -> Result<()> {
        if self.set_builder_serialized(key.clone(), &msg_slice.as_builder(), fees).is_ok() {
            Ok(())
        } else {
            fail!(BlockError::Other("Error insert serialized message".to_string()))
        }
    }

    pub fn full_import_fees(&self) -> &ImportFees {
        self.root_extra()
    }
}
