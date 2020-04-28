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
    error::BlockError,
    messages::Message,
    types::{AddSub, ChildCell, Grams},
    Serializable, Deserializable,
};
use std::cmp::Ordering;
use ton_types::{
    error, fail, Result,
    BuilderData, Cell, IBitstring, SliceData,
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
/// interm_addr_regular$0 use_src_bits:(#<= 96) = IntermediateAddress;
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
        IntermediateAddress::Regular(
            IntermediateAddressRegular{
                use_dest_bits:0
            })
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
        IntermediateAddressRegular {
            use_dest_bits: 0
        }
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
    pub fn with_message_and_fee(msg: &Message, fee_remainig: Grams) -> Result<Self> {
        Ok(
            MsgEnvelope {
                cur_addr: IntermediateAddress::default(),
                next_addr: IntermediateAddress::default(),
                fwd_fee_remaining: fee_remainig,
                msg: ChildCell::with_struct(msg)?,
            }
        )
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
}

const MSG_ENVELOPE_TAG : usize = 0x4;

impl Serializable for MsgEnvelope{
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_bits(MSG_ENVELOPE_TAG, 4)?;
        self.cur_addr.write_to(cell)?;
        self.next_addr.write_to(cell)?;
        self.fwd_fee_remaining.write_to(cell)?;
        cell.append_reference(self.msg.write_to_new_cell()?);
        Ok(())
    }
}

impl Deserializable for MsgEnvelope{
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
        self.msg.read_from(&mut cell.checked_drain_reference()?.into())?;
        Ok(())
    }
}
