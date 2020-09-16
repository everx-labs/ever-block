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
    error::BlockError,
    messages::{InternalMessageHeader, Message, MsgAddressInt},
    shard::{AccountIdPrefixFull, ShardIdent},
    types::{AddSub, ChildCell, CurrencyCollection, Grams},
    Serializable, Deserializable,
};
use std::cmp::Ordering;
use ton_types::{
    error, fail, Result,
    BuilderData, Cell, IBitstring, SliceData, UInt256,
};

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
        Ok(IntermediateAddress::Regular(
            IntermediateAddressRegular::with_use_src_bits(use_src_bits)?
        ))
    }

    pub fn use_dest_bits(use_dest_bits: u8) -> Result<Self> {
        Ok(IntermediateAddress::Regular(
            IntermediateAddressRegular::with_use_dest_bits(use_dest_bits)?
        ))
    }

    pub fn full_src() -> Self {
        IntermediateAddress::Regular(
            IntermediateAddressRegular::with_use_dest_bits(0).unwrap()
        )
    }

    pub fn full_dest() -> Self {
        IntermediateAddress::Regular(
            IntermediateAddressRegular::with_use_src_bits(0).unwrap()
        )
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
    fn partial_cmp(&self, other: &u8) -> Option<Ordering> {
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IntermediateAddressRegular {
    use_dest_bits: u8,
}

impl Default for IntermediateAddressRegular {
    fn default() -> Self {
        IntermediateAddressRegular::with_use_src_bits(0).unwrap()
    }
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
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_addr(workchain_id: i8, addr_pfx: u64) -> Self {
        Self {
            workchain_id,
            addr_pfx,
        }
    }

    pub fn workchain_id(&self) -> i8 {
        self.workchain_id
    }

    pub fn addr_pfx(&self) -> u64 {
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
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_addr(workchain_id: i32, addr_pfx: u64) -> Self {
        Self {
            workchain_id,
            addr_pfx,
        }
    }

    pub fn workchain_id(&self) -> i32 {
        self.workchain_id
    }

    pub fn addr_pfx(&self) -> u64 {
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

// msg_envelope#4 
//   cur_addr:IntermediateAddress 
//   next_addr:IntermediateAddress
//   fwd_fee_remaining:Grams 
//   msg:^(Message Any) 
// = MsgEnvelope; 
#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct MsgEnvelope {
    cur_addr: IntermediateAddress,
    next_addr: IntermediateAddress,
    fwd_fee_remaining: Grams,
    msg: ChildCell<Message>,
}

impl MsgEnvelope {
    ///
    /// Create Envelope with message and remainig_fee
    ///
    pub fn with_message_and_fee(msg: &Message, fwd_fee_remaining: Grams) -> Result<Self> {
        Ok(MsgEnvelope {
            cur_addr: IntermediateAddress::default(),
            next_addr: IntermediateAddress::default(),
            fwd_fee_remaining,
            msg: ChildCell::with_struct(msg)?,
        })
    }

    pub fn with_routing(
        msg: Cell,
        fwd_fee_remaining: Grams,
        cur_addr: IntermediateAddress,
        next_addr: IntermediateAddress
    ) -> Self {
        MsgEnvelope {
            cur_addr,
            next_addr,
            fwd_fee_remaining,
            msg: ChildCell::with_cell(msg),
        }
    }

    ///
    /// Create Envelope with hypercube routing params
    ///
    pub fn hypercube_routing(msg: &Message, src_shard: &ShardIdent, fwd_fee_remaining: Grams) -> Result<Self> {
        let msg_cell = msg.serialize()?;
        let src = msg.src().ok_or_else(|| error!("Message {} is not internal or have bad \
            source address", msg_cell.repr_hash().to_hex_string()))?;
        let dst = msg.dst().ok_or_else(|| error!("Message {} is not internal or have bad \
            destination address", msg_cell.repr_hash().to_hex_string()))?;
        let src_prefix = AccountIdPrefixFull::prefix(&src)?;
        let dst_prefix = AccountIdPrefixFull::prefix(&dst)?;
        let ia = IntermediateAddress::full_src();
        let route_info = src_prefix.perform_hypercube_routing(&dst_prefix, src_shard, &ia)?;
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
        let src_prefix = AccountIdPrefixFull::prefix(&msg.src().unwrap_or_default())?;
        let dst_prefix = AccountIdPrefixFull::prefix(&msg.dst().unwrap_or_default())?;

        let cur_prefix  = src_prefix.interpolate_addr_intermediate(&dst_prefix, &self.cur_addr)?;
        let next_prefix = src_prefix.interpolate_addr_intermediate(&dst_prefix, &self.next_addr)?;
        Ok((cur_prefix, next_prefix))
    }

    ///
    /// Read message struct from envelope
    ///
    pub fn read_message(&self) -> Result<Message> {
        self.msg.read_struct()
    }

    ///
    /// Write message struct to envelope
    ///
    pub fn write_message(&mut self, value: &Message) -> Result<()> {
        self.msg.write_struct(value)
    }

    ///
    /// Read message cell from envelope
    ///
    pub fn message_cell(&self) -> &Cell {
        self.msg.cell()
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
        debug_assert!(msg.is_internal(), "Message with hash {} is not internal",
            self.message_cell().repr_hash().to_hex_string());
        if let (Some(src), Some(dst)) = (msg.src(), msg.dst()) {
            return Ok(src.get_workchain_id() == dst.get_workchain_id())
        }
        fail!("Message with hash {} has wrong type of src/dst address",
            self.message_cell().repr_hash().to_hex_string())
    }
}

const MSG_ENVELOPE_TAG : usize = 0x4;

impl Serializable for MsgEnvelope {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_bits(MSG_ENVELOPE_TAG, 4)?;
        self.cur_addr.write_to(cell)?;
        self.next_addr.write_to(cell)?;
        self.fwd_fee_remaining.write_to(cell)?;
        cell.append_reference(self.msg.write_to_new_cell()?);
        Ok(())
    }
}

impl Deserializable for MsgEnvelope {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()>{
        let tag = cell.get_next_int(4)? as usize;
        if tag != MSG_ENVELOPE_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: "MsgEnvelope".to_string()
                }
            )
        }
        self.cur_addr.read_from(cell)?;
        self.next_addr.read_from(cell)?;
        self.fwd_fee_remaining.read_from(cell)?;
        self.msg.read_from_reference(cell)?;
        Ok(())
    }
}

// prepare for testing purposes
pub fn prepare_test_env_message(src_prefix: u64, dst_prefix: u64, bits: u8, at: u32, lt: u64) -> Result<(Message, MsgEnvelope)> {
    let shard = ShardIdent::with_prefix_len(bits, 0, src_prefix)?;
    let src = UInt256::from(src_prefix.to_be_bytes().to_vec());
    let dst = UInt256::from(dst_prefix.to_be_bytes().to_vec());
    let src = MsgAddressInt::with_standart(None, 0, src.into())?;
    let dst = MsgAddressInt::with_standart(None, 0, dst.into())?;

    // let src_prefix = AccountIdPrefixFull::prefix(&src).unwrap();
    // let dst_prefix = AccountIdPrefixFull::prefix(&dst).unwrap();
    // let ia = IntermediateAddress::full_src();
    // let route_info = src_prefix.perform_hypercube_routing(&dst_prefix, &shard, &ia)?.unwrap();
    // let cur_prefix  = src_prefix.interpolate_addr_intermediate(&dst_prefix, &route_info.0)?;
    // let next_prefix = src_prefix.interpolate_addr_intermediate(&dst_prefix, &route_info.1)?;

    let hdr = InternalMessageHeader::with_addresses(src, dst, CurrencyCollection::with_grams(1_000_000_000));
    let mut msg = Message::with_int_header(hdr);
    msg.set_at_and_lt(at, lt);

    let env = MsgEnvelope::hypercube_routing(&msg, &shard, Grams::from(1_000_000))?;
    Ok((msg , env))
}
