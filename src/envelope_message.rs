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

use crate::{
    common_message::CommonMessage,
    error::BlockError,
    shard::{AccountIdPrefixFull, ShardIdent},
    messages::Message,
    types::{AddSub, ChildCell, Grams},
    Serializable, Deserializable,
    error, fail, Result, SERDE_OPTS_EMPTY, SERDE_OPTS_COMMON_MESSAGE,
    BuilderData, Cell, IBitstring, SliceData, UInt256,
};

#[cfg(test)]
#[path = "tests/test_envelope_message.rs"]
mod tests;

/*

3.1.15. Enveloped messages. Message envelopes are used for attaching
routing information, such as the current (transit) address and the next-hop
address, to inbound, transit, and outbound messages (cf. 2.1.16). The message
itself is kept in a separate cell and referred to from the message envelope
by a cell reference.

*/

/////////////////////////////////////////////////////////////////////
/// 
/// interm_addr_regular$0 use_dest_bits:(#<= 96) = IntermediateAddress;
/// interm_addr_simple$10 workchain_id:int8 addr_pfx:(64 * Bit) = IntermediateAddress;
/// interm_addr_ext$11 workchain_id:int32 addr_pfx:(64 * Bit) = IntermediateAddress;
/// 

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum IntermediateAddress {
    Regular(IntermediateAddressRegular),
    Simple(IntermediateAddressSimple),
    Ext(IntermediateAddressExt),
}

impl IntermediateAddress {
    pub fn use_src_bits(use_src_bits: u8) -> Result<Self> {
        let ia = IntermediateAddressRegular::with_use_src_bits(use_src_bits)?;
        Ok(IntermediateAddress::Regular(ia))
    }

    pub fn use_dest_bits(use_dest_bits: u8) -> Result<Self> {
        let ia = IntermediateAddressRegular::with_use_dest_bits(use_dest_bits)?;
        Ok(IntermediateAddress::Regular(ia))
    }

    pub fn full_src() -> Self {
        let src = IntermediateAddressRegular::with_use_dest_bits(0).unwrap();
        IntermediateAddress::Regular(src)
    }

    pub fn full_dest() -> Self {
        let dest = IntermediateAddressRegular::with_use_src_bits(0).unwrap();
        IntermediateAddress::Regular(dest)
    }
    pub fn any_masterchain() -> Self {
        let master = IntermediateAddressSimple::with_addr(-1, 0x8000000000000000);
        IntermediateAddress::Simple(master)
    }
    ///
    /// Get workchain_id
    ///
    pub fn workchain_id(&self) -> Result<i32> {
        match self {
            IntermediateAddress::Simple(simple) => Ok(simple.workchain_id() as i32),
            IntermediateAddress::Ext(ext) => Ok(ext.workchain_id()),
            _ => fail!("Unsupported address type")
        }
    }

    ///
    /// Get prefix
    ///
    pub fn prefix(&self) -> Result<u64> {
        match self {
            IntermediateAddress::Simple(simple) => Ok(simple.addr_pfx()),
            IntermediateAddress::Ext(ext) => Ok(ext.addr_pfx()),
            _ => fail!("Unsupported address type")
        }
    }
}

impl Default for IntermediateAddress{
    fn default() -> Self{
        IntermediateAddress::full_src()
    }
}

impl PartialOrd<u8> for IntermediateAddress {
    fn partial_cmp(&self, other: &u8) -> Option<std::cmp::Ordering> {
        match self {
            IntermediateAddress::Regular(ia) => Some(ia.use_dest_bits.cmp(other)),
            _ => None
        }
    }
}

impl PartialEq<u8> for IntermediateAddress {
    fn eq(&self, other: &u8) -> bool {
        match self {
            IntermediateAddress::Regular(ia) => &ia.use_dest_bits == other,
            _ => false
        }
    }
}

impl Serializable for IntermediateAddress{
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        match self {
            IntermediateAddress::Regular(addr) => {
                cell.append_raw(&[0b00000000], 1)?;       // tag = $0
                addr.write_to(cell)?;
            },
            IntermediateAddress::Simple(addr) => {
                cell.append_raw(&[0b10000000], 2)?;    // tag = $10
                addr.write_to(cell)?;
            },
            IntermediateAddress::Ext(addr) => {
                cell.append_raw(&[0b11000000], 2)?;    // tag = $11
                addr.write_to(cell)?;
            }
        };
        Ok(())
    }
}

impl Deserializable for IntermediateAddress{
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()>{
        *self =
            if cell.get_next_bit()? {
                if cell.get_next_bit()? { // tag = 11
                    let mut addr = IntermediateAddressExt::default();
                    addr.read_from(cell)?;
                    IntermediateAddress::Ext(addr)
                } else { // tag = $10
                    let mut addr = IntermediateAddressSimple::default();
                    addr.read_from(cell)?;
                    IntermediateAddress::Simple(addr)
                }
            } else { // tag = $0
                let mut addr = IntermediateAddressRegular::default();
                addr.read_from(cell)?;
                IntermediateAddress::Regular(addr)
            };

        Ok(())
    }
}

/////////////////////////////////////////////////////////////////
/// 
/// interm_addr_regular$0 use_dest_bits:(#<= 96) = IntermediateAddress;
/// 

#[derive(Clone, Default, Debug, PartialEq, Eq, Hash)]
pub struct IntermediateAddressRegular {
    use_dest_bits: u8,
}

pub static FULL_BITS: u8 = 96;

impl IntermediateAddressRegular {
    pub fn with_use_src_bits(use_src_bits: u8) -> Result<Self> {
        if use_src_bits > FULL_BITS {
            fail!(BlockError::InvalidArg(format!("use_src_bits must be <= {}", FULL_BITS)))
        }
        Ok(IntermediateAddressRegular {
            use_dest_bits: FULL_BITS - use_src_bits
        })
    }

    pub fn with_use_dest_bits(use_dest_bits: u8) -> Result<Self> {
        if use_dest_bits > FULL_BITS {
            fail!(BlockError::InvalidArg(format!("use_dest_bits must be <= {}", FULL_BITS)))
        }
        Ok(IntermediateAddressRegular {
            use_dest_bits
        })
    }

    pub fn use_src_bits(&self) -> u8 {
        FULL_BITS - self.use_dest_bits
    }

    pub fn use_dest_bits(&self) -> u8 {
        self.use_dest_bits
    }

    pub fn set_use_src_bits(&mut self, use_src_bits: u8) -> Result<()>{
        if use_src_bits > FULL_BITS {
            fail!(BlockError::InvalidArg(format!("use_src_bits must be <= {}", FULL_BITS)))
        }
        self.use_dest_bits = FULL_BITS - use_src_bits;
        Ok(())
    }
}

impl Serializable for IntermediateAddressRegular{
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        // write 7-bit from use_dest_bits
        cell.append_raw(&[self.use_dest_bits << 1], 7)?;
        Ok(())
    }
}

impl Deserializable for IntermediateAddressRegular{
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()>{
        self.use_dest_bits = cell.get_next_bits(7)?[0] >> 1;    // read 7 bit into use_dest_bits
        if self.use_dest_bits > FULL_BITS {
            fail!(BlockError::InvalidArg(format!("use_dest_bits must be <= {}", FULL_BITS)))
        }
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////
/// 
/// interm_addr_simple$10 workchain_id:int8 addr_pfx:(64 * Bit) = IntermediateAddress;
/// 


#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct IntermediateAddressSimple{
    pub workchain_id: i8,
    pub addr_pfx: u64,
}

impl IntermediateAddressSimple {
    pub const fn with_addr(workchain_id: i8, addr_pfx: u64) -> Self {
        Self {
            workchain_id,
            addr_pfx,
        }
    }

    pub const fn workchain_id(&self) -> i8 {
        self.workchain_id
    }

    pub const fn addr_pfx(&self) -> u64 {
        self.addr_pfx
    }

    pub fn set_workchain_id(&mut self, workchain_id: i8) {
        self.workchain_id = workchain_id;
    }

    pub fn set_addr_pfx(&mut self, addr_pfx: u64){
        self.addr_pfx = addr_pfx;
    }
}

impl Serializable for IntermediateAddressSimple{
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.workchain_id.write_to(cell)?;
        self.addr_pfx.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for IntermediateAddressSimple{
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()>{
        self.workchain_id.read_from(cell)?;
        self.addr_pfx.read_from(cell)?;
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////
/// 
/// interm_addr_ext$11 workchain_id:int32 addr_pfx:(64 * Bit) = IntermediateAddress;
/// 


#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct IntermediateAddressExt{
    pub workchain_id: i32,
    pub addr_pfx: u64,
}

impl IntermediateAddressExt {
    pub const fn with_addr(workchain_id: i32, addr_pfx: u64) -> Self {
        Self {
            workchain_id,
            addr_pfx,
        }
    }

    pub const fn workchain_id(&self) -> i32 {
        self.workchain_id
    }

    pub const fn addr_pfx(&self) -> u64 {
        self.addr_pfx
    }

    pub fn set_workchain_id(&mut self, workchain_id: i32) {
        self.workchain_id = workchain_id;
    }

    pub fn set_addr_pfx(&mut self, addr_pfx: u64) {
        self.addr_pfx = addr_pfx;
    }
}

impl Serializable for IntermediateAddressExt {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.workchain_id.write_to(cell)?;
        self.addr_pfx.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for IntermediateAddressExt {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()>{
        self.workchain_id.read_from(cell)?;
        self.addr_pfx.read_from(cell)?;
        Ok(())
    }
}

const MSG_ENVELOPE_TAG: usize = 0x4;
const MSG_ENVELOPE_TAG_2: usize = 0x5;

// msg_envelope#4
//   cur_addr:IntnveloMsgEnvelope; 
#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct MsgEnvelope {
    cur_addr: IntermediateAddress,
    next_addr: IntermediateAddress,
    fwd_fee_remaining: Grams,
    msg: ChildCell<CommonMessage>,
}

impl MsgEnvelope {
    ///
    /// Create Envelope with message and remainig_fee
    ///
    pub fn with_message_and_fee(msg: &Message, fwd_fee_remaining: Grams) -> Result<Self> {
        if !msg.is_internal() {
            fail!("MsgEnvelope can be made only for internal messages");
        }
        let opts = SERDE_OPTS_EMPTY;
        Ok(Self::with_routing(
            ChildCell::with_struct_and_opts(&CommonMessage::Std(msg.clone()), opts)?,
            fwd_fee_remaining,
            IntermediateAddress::full_dest(),
            IntermediateAddress::full_dest(),
        ))
    }

    pub fn with_common_msg_support(msg: &CommonMessage, fwd_fee_remaining: Grams) -> Result<Self> {
        if let Ok(std_msg)= msg.get_std() {
            if !std_msg.is_internal() {
                fail!("MsgEnvelope can be made only for internal messages");
            }
        }
        let opts = SERDE_OPTS_COMMON_MESSAGE;
        Ok(Self::with_routing(
            ChildCell::with_struct_and_opts(msg, opts)?,
            fwd_fee_remaining,
            IntermediateAddress::full_dest(),
            IntermediateAddress::full_dest(),
        ))
    }

    ///
    /// Create Envelope with message cell and remainig_fee
    /// TODO should be marked as deprecated and removed if possible
    pub fn with_message_cell_and_fee(msg_cell: Cell, fwd_fee_remaining: Grams) -> Self {
        Self::with_routing(
            ChildCell::with_cell_and_opts(msg_cell, SERDE_OPTS_EMPTY),
            fwd_fee_remaining,
            IntermediateAddress::full_dest(),
            IntermediateAddress::full_dest(),
        )
    }

    ///
    /// Create Envelope with message and remainig_fee and routing settings
    ///
    pub fn with_routing(
        msg: ChildCell<CommonMessage>,
        fwd_fee_remaining: Grams,
        cur_addr: IntermediateAddress,
        next_addr: IntermediateAddress,
    ) -> Self {
        MsgEnvelope {
            cur_addr,
            next_addr,
            fwd_fee_remaining,
            msg,
        }
    }

    ///
    /// Create Envelope with hypercube routing params
    /// TBD
    ///
    #[allow(dead_code)]
    pub(crate) fn hypercube_routing(msg: &Message, src_shard: &ShardIdent, fwd_fee_remaining: Grams) -> Result<Self> {
        let msg_cell = msg.serialize()?;
        let src = msg.src_ref().ok_or_else(|| error!("source address of message {:x} is invalid", msg_cell.repr_hash()))?;
        let src_prefix = AccountIdPrefixFull::prefix(src)?;
        let dst = msg.dst_ref().ok_or_else(|| error!("destination address of message {:x} is invalid", msg_cell.repr_hash()))?;
        let dst_prefix = AccountIdPrefixFull::prefix(dst)?;
        let ia = IntermediateAddress::default();
        let route_info = src_prefix.perform_hypercube_routing(&dst_prefix, src_shard, ia)?;
        Ok(MsgEnvelope {
            cur_addr: route_info.0,
            next_addr: route_info.1,
            fwd_fee_remaining,
            msg: ChildCell::with_cell(msg_cell),
        })
    }

    /// calc prefixes with routing info
    pub fn calc_cur_next_prefix(&self) -> Result<(AccountIdPrefixFull, AccountIdPrefixFull)> {
        let msg = self.read_message()?;
        let src = msg.src_ref().ok_or_else(|| error!("source address of message {:x} is invalid", self.message_hash()))?;
        let src_prefix = AccountIdPrefixFull::prefix(src)?;
        let dst = msg.dst_ref().ok_or_else(|| error!("destination address of message {:x} is invalid", self.message_hash()))?;
        let dst_prefix = AccountIdPrefixFull::prefix(dst)?;

        let cur_prefix  = src_prefix.interpolate_addr_intermediate(&dst_prefix, &self.cur_addr)?;
        let next_prefix = src_prefix.interpolate_addr_intermediate(&dst_prefix, &self.next_addr)?;
        Ok((cur_prefix, next_prefix))
    }

    ///
    /// Read message struct from envelope
    ///
    pub fn read_message(&self) -> Result<Message> {
        let msg = self.msg.read_struct()?;
        match msg {
            CommonMessage::Std(msg) => Ok(msg),
            _ => fail!(BlockError::UnexpectedStructVariant(
                "CommonMessage::Std".to_string(), 
                msg.get_type_name()
            )),
        }
    }

    pub fn read_common_message(&self) -> Result<CommonMessage> {
        self.msg.read_struct()
    }

    ///
    /// Write message struct to envelope
    ///
    pub fn write_message(&mut self, value: &Message) -> Result<()> {
        self.msg.write_struct(&CommonMessage::Std(value.clone()))
    }

    ///
    /// Return message cell from envelope
    ///
    pub fn message_cell(&self)-> Cell {
        self.msg.cell()
    }

    pub fn msg_cell(&self)-> ChildCell<CommonMessage> {
        self.msg.clone()
    }

    ///
    /// Return message hash from envelope
    ///
    pub fn message_hash(&self)-> UInt256 {
        self.msg.cell().repr_hash()
    }

    ///
    /// Get remaining fee of envelope
    ///
    pub fn fwd_fee_remaining(&self) -> &Grams {
        &self.fwd_fee_remaining
    }

    ///
    /// Collect transfer fee from envelope
    ///
    pub fn collect_fee(&mut self, fee: Grams) -> bool {
        self.fwd_fee_remaining.sub(&fee).unwrap() // no excpetion here
    }

    ///
    /// Set current address of envelope
    ///
    pub fn set_cur_addr(&mut self, addr: IntermediateAddress) -> &mut Self{
        self.cur_addr = addr;
        self
    }

    ///
    /// Set next address of envelope
    ///
    pub fn set_next_addr(&mut self, addr: IntermediateAddress) -> &mut Self{
        self.next_addr = addr;
        self
    }

    ///
    /// Get current address
    ///
    pub fn cur_addr(&self) -> &IntermediateAddress{
        &self.cur_addr
    }

    ///
    /// Get next address
    ///
    pub fn next_addr(&self) -> &IntermediateAddress{
        &self.next_addr
    }

    /// is message route in one workchain
    pub fn same_workchain(&self) -> Result<bool> {
        let msg = self.read_message()?;
        debug_assert!(msg.is_internal(), "Message with hash {:x} is not internal",
            self.message_cell().repr_hash());
        if let (Some(src), Some(dst)) = (msg.src_ref(), msg.dst_ref()) {
            return Ok(src.get_workchain_id() == dst.get_workchain_id())
        }
        fail!("Message with hash {:x} has wrong type of src/dst address",
            self.message_cell().repr_hash())
    }

    pub fn serde_opts(&self) -> u8 {
        self.msg.serde_opts()
    }
}

fn serialize_msgenvelope(x: &MsgEnvelope, cell: &mut BuilderData, opts: u8) -> Result<()> {
    let tag = if opts & SERDE_OPTS_COMMON_MESSAGE != 0 {
        MSG_ENVELOPE_TAG_2
    } else {
        MSG_ENVELOPE_TAG
    };
    cell.append_bits(tag, 4)?;
    x.cur_addr.write_with_opts(cell, opts)?;
    x.next_addr.write_with_opts(cell, opts)?;
    x.fwd_fee_remaining.write_with_opts(cell, opts)?;
    cell.checked_append_reference(x.msg.cell())?;
    Ok(())
}

impl Serializable for MsgEnvelope {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        serialize_msgenvelope(self, cell, SERDE_OPTS_EMPTY)
    }
    fn write_with_opts(&self, cell: &mut BuilderData, opts: u8) -> Result<()> {
        serialize_msgenvelope(self, cell, opts)
    }
}

impl Deserializable for MsgEnvelope {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()>{
        let tag = cell.get_next_int(4)? as usize;
        if tag != MSG_ENVELOPE_TAG && tag != MSG_ENVELOPE_TAG_2 {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: "MsgEnvelope".to_string()
                }
            )
        }
        let opts = match tag {
            MSG_ENVELOPE_TAG_2 => SERDE_OPTS_COMMON_MESSAGE,
            _ => SERDE_OPTS_EMPTY,
        };
        self.cur_addr.read_from_with_opts(cell, opts)?;
        self.next_addr.read_from_with_opts(cell, opts)?;
        self.fwd_fee_remaining.read_from_with_opts(cell, opts)?;
        let msg_cell = cell.checked_drain_reference()?;
        self.msg = ChildCell::with_cell_and_opts(msg_cell, opts);
        Ok(())
    }
}
