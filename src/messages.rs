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

use crate::GetRepresentationHash;
use crate::{
    blocks::Block,
    error::BlockError,
    hashmapaug::HashmapAugType,
    merkle_proof::MerkleProof,
    types::{AddSub, CurrencyCollection, Grams, Number5, Number9, UnixTime32},
    Deserializable, MaybeDeserialize, MaybeSerialize, Serializable,
};
use std::fmt;
use std::str::FromStr;
use ton_types::{
    BuilderData, Cell, error, fail, MAX_DATA_BITS, MAX_REFERENCES_COUNT, Result, 
    SliceData, UsageTree, IBitstring, AccountId, UInt256
};


///////////////////////////////////////////////////////////////////////////////
/// 
/// MessageAddress
/// 
///

/*
3.1.2. TL-B scheme for addresses. The serialization of source and destination addresses is defined by the following TL-B scheme:
addr_none$00 = MsgAddressExt;
addr_extern$01 len:(## 9) external_address:(len * Bit)
= MsgAddressExt;
anycast_info depth:(## 5) rewrite_pfx:(depth * Bit) = Anycast;
addr_std$10 anycast:(Maybe Anycast)
workchain_id:int8 address:uint256 = MsgAddressInt;
addr_var$11 anycast:(Maybe Anycast) addr_len:(## 9)
workchain_id:int32 address:(addr_len * Bit) = MsgAddressInt;
_ MsgAddressInt = MsgAddress;
_ MsgAddressExt = MsgAddress;
 */

impl AnycastInfo {
    pub fn with_rewrite_pfx(pfx: SliceData) -> Result<Self> {
        if pfx.remaining_bits() > Number5::get_max_len() {
            fail!(
                BlockError::InvalidArg("pfx can't be longer than 2^5-1 bits".to_string())
            )
        }
        Ok(Self {
            depth: Number5(pfx.remaining_bits() as u32),
            rewrite_pfx: pfx
        })
    }
    pub fn set_rewrite_pfx(&mut self, pfx: SliceData) -> Result<()>{
        if pfx.remaining_bits() > Number5::get_max_len() {
            fail!(
                BlockError::InvalidArg("pfx can't be longer than 2^5-1 bits".to_string())
            )
        }
        self.depth = Number5(pfx.remaining_bits() as u32);
        self.rewrite_pfx = pfx;
        Ok(())
    }
}

impl Serializable for AnycastInfo {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.depth.write_to(cell)?;                                  // write depth
        cell.checked_append_references_and_data(&self.rewrite_pfx)?; // write rewrite_pfx
        Ok(())
    } 
}

impl fmt::Display for AnycastInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "AnycastInfo[pfx {}]", self.rewrite_pfx
        )
    }
}

/*
addr_std$10 anycast:(Maybe Anycast)
workchain_id:int8 address:uint256 = MsgAddressInt;
addr_var$11 anycast:(Maybe Anycast) addr_len:(## 9)
workchain_id:int32 address:(addr_len * Bit) = MsgAddressInt;
_ MsgAddressInt = MsgAddress;
_ MsgAddressExt = MsgAddress;
 */

impl MsgAddrVar {
    pub fn with_address(anycast: Option<AnycastInfo>, workchain_id: i32, address: SliceData) -> Result<MsgAddrVar> {
        if address.remaining_bits() > Number9::get_max_len(){
            fail!(
                BlockError::InvalidArg("address can't be longer than 2^9-1 bits".to_string())
            )
        }
        let addr_len = Number9(address.remaining_bits() as u32);
        Ok(MsgAddrVar { anycast, addr_len, workchain_id, address })
    }
}

impl Serializable for MsgAddrVar {

    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.anycast.write_maybe_to(cell)?;                            // anycast
        let addr_len = Number9(self.address.remaining_bits() as u32);
        addr_len.write_to(cell)?;                                      // addr_len
        cell.append_i32(self.workchain_id)?;                           // workchain_id
        cell.checked_append_references_and_data(&self.address)?;       // address
        Ok(())
    } 
}

impl fmt::Display for MsgAddrVar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(anycast) = &self.anycast {
            write!(f, "{:x}:", anycast.rewrite_pfx)?;
        }
        if (self.workchain_id / 128 == 0) && (self.address.remaining_bits() == 256) {
            write!(f, "{}:{:x}8_", self.workchain_id, self.address)
        } else {
            write!(f, "{}:{:x}", self.workchain_id, self.address)
        }
    }
}

impl MsgAddrStd {
    pub fn with_address(anycast: Option<AnycastInfo>, workchain_id: i8, address: AccountId) -> Self {
        MsgAddrStd { anycast, workchain_id, address }
    }
}

impl Default for MsgAddrStd {
    fn default() -> Self{
        MsgAddrStd { anycast: None, workchain_id: 0, address: AccountId::from([0; 32]) }
    }
}

impl Serializable for MsgAddrStd {

    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.anycast.write_maybe_to(cell)?;  // anycast
        self.workchain_id.write_to(cell)?;   // workchain_id
        self.address.write_to(cell)?;        // address
        Ok(())
    } 
}

impl fmt::Display for MsgAddrStd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(anycast) = &self.anycast {
            write!(f, "{:x}:", anycast.rewrite_pfx)?;
        }
        write!(f, "{}:{:x}", self.workchain_id, self.address)
    }
}

impl MsgAddrExt {
    pub fn with_address(address: SliceData) -> Result<Self>{
        if address.remaining_bits() > Number9::get_max_len(){
            fail!(
                BlockError::InvalidArg("address can't be longer than 2^9-1 bits".to_string())
            )
        }
        Ok(MsgAddrExt {
            len: Number9(address.remaining_bits() as u32),
            external_address: address
        })
    }
}

impl Serializable for MsgAddrExt {

    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        let len = Number9(self.external_address.remaining_bits() as u32);
        len.write_to(cell)?;                               // write len
        cell.checked_append_references_and_data(&self.external_address)?; // write address
        Ok(())
    } 
}

impl fmt::Display for MsgAddrExt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, ":{:x}", self.external_address)
    }
}

impl MsgAddressExt {
    pub fn with_extern(address: SliceData) -> Result<Self> {
        Ok(MsgAddressExt::AddrExtern(MsgAddrExt::with_address(address)?))
    }
}

impl Default for MsgAddressExt {
    fn default() -> Self{
        MsgAddressExt::AddrNone
    }
}

impl FromStr for MsgAddressExt {
    type Err = failure::Error;
    fn from_str(string: &str) -> Result<Self> {
        match MsgAddress::from_str(string)? {
            MsgAddress::AddrNone => Ok(MsgAddressExt::AddrNone),
            MsgAddress::AddrExt(addr) => Ok(MsgAddressExt::AddrExtern(addr)),
            _ => fail!(BlockError::Other("Wrong type of address".to_string()))
        }
    }
}

impl Serializable for MsgAddressExt {

    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        
        match self {
            MsgAddressExt::AddrNone => {
                cell.append_raw(&[0x00], 2)?;     // prefix AddrNone
            },
            MsgAddressExt::AddrExtern(ext) => {
                cell.append_raw(&[0x40], 2)?;     // prefix AddrExtern
                ext.write_to(cell)?;              // MsgAddressExt
            },
        }

        Ok(())
    } 
}

impl fmt::Display for MsgAddressExt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MsgAddressExt::AddrNone => write!(f, ""),
            MsgAddressExt::AddrExtern(addr) => write!(f, "{}", addr),
        }
    }
}

impl MsgAddress {
    pub fn with_extern(address: SliceData) -> Result<Self> {
        Ok(MsgAddress::AddrExt(MsgAddrExt::with_address(address)?))
    }

    pub fn with_variant(anycast: Option<AnycastInfo>, workchain_id: i32, address: SliceData) -> Result<Self> {
        Ok(MsgAddress::AddrVar(MsgAddrVar::with_address(anycast, workchain_id, address)?))
    }

    pub fn with_standart(anycast: Option<AnycastInfo>, workchain_id: i8, address: AccountId) -> Result<Self> {
        Ok(MsgAddress::AddrStd(MsgAddrStd::with_address(anycast, workchain_id, address)))
    }

    pub fn get_address(&self) -> SliceData {
        match self {
            MsgAddress::AddrNone => SliceData::default(),
            MsgAddress::AddrExt(addr_ext) => addr_ext.external_address.clone(),
            MsgAddress::AddrStd(addr_std) => addr_std.address.clone(),
            MsgAddress::AddrVar(addr_var) => addr_var.address.clone()
        }
    }

    pub fn get_type(&self) -> u8 {
        match self {
            MsgAddress::AddrNone => 0b00,
            MsgAddress::AddrExt(_) => 0b01,
            MsgAddress::AddrStd(_) => 0b10,
            MsgAddress::AddrVar(_) => 0b11
        }
    }
}


impl FromStr for MsgAddress {
    type Err = failure::Error;
    fn from_str(string: &str) -> Result<Self> {
        let parts: Vec<&str> = string.split(':').take(4).collect();
        let len = parts.len();
        if len > 3 {
            fail!(BlockError::InvalidArg("too many components in address".to_string()))
        }
        if len == 0 {
            fail!(BlockError::InvalidArg("bad split".to_string()))
        }
        if parts[len - 1].is_empty() {
            if len == 1 {
                return Ok(MsgAddress::AddrNone)
            } else {
                fail!(BlockError::InvalidArg("wrong format".to_string()))
            }
        }
        let address = SliceData::from_string(parts[len - 1])?;
        if len == 2 && parts[0].is_empty() {
            return Ok(MsgAddress::with_extern(address)?)
        }
        let workchain_id = len.checked_sub(2)
            .map(|index| parts[index].parse::<i32>()).transpose()
            .map_err(
                |err| BlockError::InvalidArg(
                    format!("workchain_id is not correct number: {}", err)
                )
            )?
            .unwrap_or_default();
        let anycast = len.checked_sub(3)
            .map(
                |index| if parts[index].is_empty() {
                    Err(BlockError::InvalidArg("wrong format".to_string()))
                } else {
                    SliceData::from_string(parts[index])
                        .map_err(
                            |err| BlockError::InvalidArg(
                                format!("anycast is not correct: {}", err)
                            )
                        )
                }
            ).transpose()?
            .map(|value| AnycastInfo::with_rewrite_pfx(value)).transpose()
            .map_err(
                |err| BlockError::InvalidArg(
                    format!("anycast is not correct: {}", err)
                )
            )?;

        if workchain_id < 128 && workchain_id >= -128 {
            if address.remaining_bits() != 256 {
                fail!(
                    BlockError::InvalidArg(
                        format!(
                            "account address should be 256 bits long in workchain {}", 
                            workchain_id
                        )
                    )
                )
            }
            if parts[len - 1].len() == 64 {
                Ok(MsgAddress::with_standart(anycast, workchain_id as i8, address)?)
            } else {
                Ok(MsgAddress::with_variant(anycast, workchain_id, address)?)
            }
        } else {
            Ok(MsgAddress::with_variant(anycast, workchain_id, address)?)
        }
    }
}

impl Default for MsgAddress {
    fn default() -> Self {
        MsgAddress::AddrNone
    }
}

impl fmt::Display for MsgAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MsgAddress::AddrNone => write!(f, ""),
            MsgAddress::AddrExt(addr) => write!(f, "{}", addr),
            MsgAddress::AddrStd(addr) => write!(f, "{}", addr),
            MsgAddress::AddrVar(addr) => write!(f, "{}", addr),
        }
    }
}

impl Serializable for MsgAddress {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_raw(&[self.get_type() << 6], 2)?;
        match self {
            MsgAddress::AddrNone => (),
            MsgAddress::AddrExt(ext) => ext.write_to(cell)?,
            MsgAddress::AddrStd(std) => std.write_to(cell)?,
            MsgAddress::AddrVar(var) => var.write_to(cell)?,
        }
        Ok(())
    } 
}

impl Default for MsgAddressInt {
    fn default() -> Self {
        MsgAddressInt::AddrStd(MsgAddrStd::default())
    }
}

impl FromStr for MsgAddressInt {
    type Err = failure::Error;
    fn from_str(string: &str) -> Result<Self> {
        match MsgAddress::from_str(string)? {
            MsgAddress::AddrStd(addr) => Ok(MsgAddressInt::AddrStd(addr)),
            MsgAddress::AddrVar(addr) => Ok(MsgAddressInt::AddrVar(addr)),
            _ => fail!(BlockError::Other("Wrong type of address".to_string()))
        }
    }
}

impl MsgAddressInt {
    pub fn with_variant(anycast: Option<AnycastInfo>, workchain_id: i32, address: SliceData) -> Result<Self> {
        Ok(MsgAddressInt::AddrVar(MsgAddrVar::with_address(anycast, workchain_id, address)?))
    }
    pub fn with_standart(anycast: Option<AnycastInfo>, workchain_id: i8, address: AccountId) -> Result<Self> {
        Ok(MsgAddressInt::AddrStd(MsgAddrStd::with_address(anycast, workchain_id, address)))
    }
    pub fn get_address(&self) -> SliceData {
        match self {
            MsgAddressInt::AddrStd(addr_std) => addr_std.address.write_to_new_cell().unwrap().into(),
            MsgAddressInt::AddrVar(addr_var) => addr_var.address.clone()
        }
    }
    pub fn get_workchain_id(&self) -> i32 {
        match self {
            MsgAddressInt::AddrStd(addr_std) => addr_std.workchain_id as i32,
            MsgAddressInt::AddrVar(addr_var) => addr_var.workchain_id
        }
    }
    pub fn get_rewrite_pfx(&self) -> Option<AnycastInfo> {
        match self {
            MsgAddressInt::AddrStd(addr_std) => addr_std.anycast.clone(),
            MsgAddressInt::AddrVar(addr_var) => addr_var.anycast.clone()
        }
    }
    pub fn extract_std_address(&self, do_rewrite: bool) -> Result<(i32, AccountId)> {
        let (workchain_id, mut account_id, anycast_opt) = match self {
            MsgAddressInt::AddrStd(addr_std) => (addr_std.workchain_id as i32, addr_std.address.clone(), &addr_std.anycast),
            MsgAddressInt::AddrVar(addr_var) => (addr_var.workchain_id, addr_var.address.clone(), &addr_var.anycast)
        };

        if let Some(ref anycast) = anycast_opt {
            if do_rewrite {
                account_id.overwrite_prefix(&anycast.rewrite_pfx)?;
            }
        }

        Ok((workchain_id, account_id))
    }
}

impl Serializable for MsgAddressInt {

    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        match self {
            MsgAddressInt::AddrStd(std) => {
                cell.append_raw(&[0x80], 2)?;    // $10 prefix AddrStd
                std.write_to(cell)?;                                    // MsgAddrStd
            }
            MsgAddressInt::AddrVar(var) => {
                cell.append_raw(&[0xC0], 2)?;    // $11 prefix AddrVar
                var.write_to(cell)?;                                    // MsgAddressInt
            }
        }

        Ok(())
    } 
}

impl fmt::Display for MsgAddressInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MsgAddressInt::AddrStd(addr) => write!(f, "{}", addr),
            MsgAddressInt::AddrVar(addr) => write!(f, "{}", addr),
        }
    }
}


/*
This file contains definitions for internal and external message headers
as defined in Blockchain: 3.1.

In test_messages.rs and contracts/messages/contract.code there are parsers
for these formats.

Known limitations:
1. For account addreses:
    * we don't serialize the workchain id;
    * anycast is not supported (is supposed to be `nothing`);
    * only standard 256-bit addresses are supported.

2. Instead of CurrencyCollection, Grams type is used.

3. In Message X format, only the info field is parsed.

4. External address is supposed to consist of a whole number of bytes.
*/

impl fmt::Display for InternalMessageHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Internal {{src: {}, dst: {}", self.src, self.dst)?;
        if f.alternate() {
            write!(f, ", ihr_disabled: {}, bounce: {}, bounced: {}, value: {}, ihr_fee: {}, fwd_fee: {}, lt: {}, at: {}",
                self.ihr_disabled,
                self.bounce,
                self.bounced,
                self.value,
                self.ihr_fee,
                self.fwd_fee,
                self.created_lt,
                self.created_at
            )?;
        }
        write!(f, "}}")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MsgAddressIntOrNone {
    None,
    Some(MsgAddressInt)
}

impl MsgAddressIntOrNone {
    pub fn get_type(&self) -> u8 {
        match self {
            MsgAddressIntOrNone::None       => 0b00,
            MsgAddressIntOrNone::Some(addr) => 
                match addr {
                    MsgAddressInt::AddrStd(_) => 0b10,
                    MsgAddressInt::AddrVar(_) => 0b11,
                }
        }
    }
    pub fn get_rewrite_pfx(&self) -> Option<AnycastInfo> {
        match self {
            MsgAddressIntOrNone::None       => None,
            MsgAddressIntOrNone::Some(addr) => addr.get_rewrite_pfx()
        }
    }
}

impl Default for MsgAddressIntOrNone {
    fn default() -> Self {
        MsgAddressIntOrNone::None
    }
}

impl fmt::Display for MsgAddressIntOrNone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MsgAddressIntOrNone::None       => write!(f, ""),
            MsgAddressIntOrNone::Some(addr) => write!(f, "{}", addr),
        }
    }
}

impl Serializable for MsgAddressIntOrNone {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        match self {
            MsgAddressIntOrNone::None       => {
                cell.append_raw(&[0x00], 2)?;
                ()
            },
            MsgAddressIntOrNone::Some(addr) => addr.write_to(cell)?,
        }
        Ok(())
    } 
}

impl Deserializable for MsgAddressIntOrNone {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()>{
        let addr_type = cell.get_next_int(2)? as u8;
        match addr_type & 0b11 {
            0b00 => {
                *self = MsgAddressIntOrNone::None;
            },
            0b10 => {
                let mut std = MsgAddrStd::default();
                std.read_from(cell)?;
                *self = MsgAddressIntOrNone::Some(MsgAddressInt::AddrStd(std));
            },
            0b11 => {
                let mut var = MsgAddrVar::default();
                var.read_from(cell)?;
                *self = MsgAddressIntOrNone::Some(MsgAddressInt::AddrVar(var));
            },
            _ => fail!(BlockError::Other("Wrong type of address".to_string()))
        }
        Ok(())
    } 
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InternalMessageHeader {
    pub ihr_disabled: bool,
    pub bounce: bool,
    pub bounced: bool,
    pub src: MsgAddressIntOrNone,
    pub dst: MsgAddressInt,
    pub value: CurrencyCollection, 
    pub ihr_fee: Grams,
    pub fwd_fee: Grams,
    pub created_lt: u64,
    pub created_at: UnixTime32,
}

impl Default for InternalMessageHeader {
    fn default() -> Self {
        InternalMessageHeader {
            ihr_disabled: false,
            bounce: false,
            bounced: false,
            src: MsgAddressIntOrNone::None,
            dst: MsgAddressInt::default(),
            value: CurrencyCollection::default(), 
            ihr_fee: Grams::zero(),
            fwd_fee: Grams::zero(),
            created_lt: 0,
            created_at: UnixTime32::default(),
        }
    }
}

impl InternalMessageHeader {
    ///
    /// Create new instance of InternalMessageHeader
    /// with source and destination address and value
    ///
    pub fn with_addresses(
        src: MsgAddressInt, 
        dst: MsgAddressInt, 
        value: CurrencyCollection,
    ) -> Self {
        InternalMessageHeader {
            ihr_disabled: false,
            bounce: false,
            bounced: false,
            src: MsgAddressIntOrNone::Some(src),
            dst: dst,
            value: value, 
            ihr_fee: Grams::zero(),
            fwd_fee: Grams::zero(),
            created_lt: 0,  // Logical Time will be set on BlockBuilder
            created_at: UnixTime32::default(),  // UNIX time too
        }
    }

    pub fn with_addresses_and_bounce(
        src: MsgAddressInt, 
        dst: MsgAddressInt, 
        value: CurrencyCollection, 
        bounce: bool,
    ) -> Self {
        let mut hdr = Self::with_addresses(src, dst, value);
        hdr.bounce = bounce;
        hdr
    }

    ///
    /// Get value tansfered message
    ///
    pub fn value(&self) -> &CurrencyCollection {
        &self.value
    }

    ///
    /// Get IHR fee for message
    ///
    pub fn ihr_fee(&self) -> &Grams {
        &self.ihr_fee
    }

    ///
    /// Get forwarding fee for message transfer
    ///
    pub fn fwd_fee(&self) -> &Grams {
        &self.fwd_fee
    }
}

impl Serializable for InternalMessageHeader{
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        
        cell
            .append_bit_zero()?              //tag
            .append_bit_bool(self.ihr_disabled)?
            .append_bit_bool(self.bounce)?
            .append_bit_bool(self.bounced)?;

        self.src.write_to(cell)?;
        self.dst.write_to(cell)?;
        
        self.value.write_to(cell)?;         //value: CurrencyCollection

        self.ihr_fee.write_to(cell)?;       //ihr_fee
        self.fwd_fee.write_to(cell)?;       //fwd_fee

        self.created_lt.write_to(cell)?;    //created_lt
        self.created_at.write_to(cell)?;    //created_at

        Ok(())
    } 
}

impl Deserializable for InternalMessageHeader {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()>{

        // constructor tag will be readed in Message
        self.ihr_disabled = cell.get_next_bit()?;    // ihr_disabled
        self.bounce = cell.get_next_bit()?;          // bounce
        self.bounced = cell.get_next_bit()?;

        self.src.read_from(cell)?;                  // addr src
        self.dst.read_from(cell)?;                  // addr dst
        
        self.value.read_from(cell)?;                // value - balance
        
        self.ihr_fee.read_from(cell)?;              //ihr_fee
        self.fwd_fee.read_from(cell)?;              //fwd_fee

        self.created_lt.read_from(cell)?;           //created_lt
        self.created_at.read_from(cell)?;           //created_at
        Ok(())
    } 
}

impl fmt::Display for ExternalInboundMessageHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "External Inbound {{src: {}, dst: {}, fee: {}}}",
            self.src, self.dst, self.import_fee
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExternalInboundMessageHeader {
    pub src: MsgAddressExt,
    pub dst: MsgAddressInt,
    pub import_fee: Grams,
}

impl Serializable for ExternalInboundMessageHeader{
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell
            .append_bit_one()?
            .append_bit_zero()?;

        self.src.write_to(cell)?;               // addr src
        self.dst.write_to(cell)?;               // addr dst
        self.import_fee.write_to(cell)?;        //ihr_fee

        Ok(())
    } 
}

impl Deserializable for ExternalInboundMessageHeader {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()>{

        // constructor tag will be readed in Message
        self.src.read_from(cell)?;               // addr src
        self.dst.read_from(cell)?;               // addr dst
        self.import_fee.read_from(cell)?;        //ihr_fee
        Ok(())
    } 
}

impl fmt::Display for ExtOutMessageHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "External Outbound {{src: {}, dst: {}, lt: {}, at: {}}}",
            self.src, self.dst, self.created_lt, self.created_at
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExtOutMessageHeader {
    pub src: MsgAddressIntOrNone,
    pub dst: MsgAddressExt,
    pub created_lt: u64,
    pub created_at: UnixTime32,
}

impl ExtOutMessageHeader {
    pub fn with_addresses(src: MsgAddressInt, dst: MsgAddressExt) -> ExtOutMessageHeader {
        ExtOutMessageHeader {
            src: MsgAddressIntOrNone::Some(src),
            dst: dst,
            created_lt: 0, // Logical Time will be set on block builder
            created_at: UnixTime32::default(), // UNIX time too
        }
    }
}

impl Serializable for ExtOutMessageHeader{
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell
            .append_bit_one()?
            .append_bit_one()?;

        self.src.write_to(cell)?;               // addr src
        self.dst.write_to(cell)?;               // addr dst
        self.created_lt.write_to(cell)?;        //created_lt
        self.created_at.write_to(cell)?;        //created_at

        Ok(())
    } 
}

impl Deserializable for ExtOutMessageHeader {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()>{

        // constructor tag will be readed in Message
        self.src.read_from(cell)?;                  // addr src
        self.dst.read_from(cell)?;                  // addr dst
        self.created_lt.read_from(cell)?;           //created_lt
        self.created_at.read_from(cell)?;           //created_at
        Ok(())
    } 
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// 
/// int_msg_info$0 ihr_disabled:Bool bounce:Bool
/// src:MsgAddressInt dest:MsgAddressInt
/// value:CurrencyCollection ihr_fee:Grams fwd_fee:Grams
/// created_lt:uint64 created_at:uint32 = CommonMsgInfo;
/// ext_in_msg_info$10 src:MsgAddressExt dest:MsgAddressInt
/// import_fee:Grams = CommonMsgInfo;
/// ext_out_msg_info$11 src:MsgAddressInt dest:MsgAddressExt
/// created_lt:uint64 created_at:uint32 = CommonMsgInfo;
/// 

impl fmt::Display for CommonMsgInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommonMsgInfo::IntMsgInfo(hdr)    => write!(f, "{}", hdr),
            CommonMsgInfo::ExtInMsgInfo(hdr)  => write!(f, "{}", hdr),
            CommonMsgInfo::ExtOutMsgInfo(hdr) => write!(f, "{}", hdr),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommonMsgInfo{
    IntMsgInfo(InternalMessageHeader),
    ExtInMsgInfo(ExternalInboundMessageHeader),
    ExtOutMsgInfo(ExtOutMessageHeader)
}

impl CommonMsgInfo {
    ///
    /// Get destination account address
    ///
    pub fn dest_account_address(&self) -> Option<AccountId> {
        match self  {
            CommonMsgInfo::IntMsgInfo(header) => {
                match header.dst {
                    MsgAddressInt::AddrStd(ref std) => Some(std.address.clone()),
                    MsgAddressInt::AddrVar(ref _var) => unimplemented!(), // TODO 
                }
            },
            CommonMsgInfo::ExtInMsgInfo(header) => {
                match header.dst {
                    MsgAddressInt::AddrStd(ref std) => Some(std.address.clone()),
                    MsgAddressInt::AddrVar(ref _var) => unimplemented!(), // TODO 
                }
            }
            _ => None,
        }
    }

    ///
    /// Get value transmitted by the value
    /// Value can be transmitted only internal messages
    /// For other types of messages, function returned None
    ///
    pub fn get_value<'a>(&'a self) -> Option<&'a CurrencyCollection> {
        match self  {
            CommonMsgInfo::IntMsgInfo(header) => Some(&header.value),
            _ => None,
        }        
    }

    pub fn get_value_mut<'a>(&'a mut self) -> Option<&'a mut CurrencyCollection> {
        match self  {
            CommonMsgInfo::IntMsgInfo(header) => Some(&mut header.value),
            _ => None,
        }        
    }

    ///
    /// Get message header fees
    /// Fee collected only for transfer internal and external outbound messages.
    /// for other types of messages, function returned None
    ///
    pub fn fee(&self) -> Result<Option<Grams>> {
        match self  {
            CommonMsgInfo::IntMsgInfo(header) => {
                let mut result = header.ihr_fee.clone();
                result.add(&header.fwd_fee)?;
                Ok(Some(result))
            },
            CommonMsgInfo::ExtInMsgInfo(header) => {
                Ok(Some(header.import_fee.clone()))
            }
            _ => Ok(None),
        }
    }

    ///
    /// Get dest address for Intrenal and Inbound external messages
    ///
    pub fn get_dst_address(&self) -> Option<MsgAddressInt> {
        match self  {
            CommonMsgInfo::IntMsgInfo(header) => {
                Some(header.dst.clone())
            },
            CommonMsgInfo::ExtInMsgInfo(header) => {
                Some(header.dst.clone())
            }
            _ => None,        
        }
    }

}

impl Default for CommonMsgInfo {
    fn default() -> Self {
        CommonMsgInfo::IntMsgInfo(InternalMessageHeader::default())
    }
}

impl Serializable for CommonMsgInfo
{
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        match self {
            CommonMsgInfo::IntMsgInfo(header) => header.write_to(cell)?,
            CommonMsgInfo::ExtInMsgInfo(header) => header.write_to(cell)?,
            CommonMsgInfo::ExtOutMsgInfo(header) => header.write_to(cell)?,
        }
        Ok(())
    } 
}

impl Deserializable for CommonMsgInfo
{
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()>{

        *self = if !cell.get_next_bit()? {  // CommonMsgInfo::int_msg_info
            let mut int_msg = InternalMessageHeader::default();
            int_msg.read_from(cell)?;
            CommonMsgInfo::IntMsgInfo(int_msg)
        } else if !cell.get_next_bit()? {
            let mut ext_in_msg = ExternalInboundMessageHeader::default();
            ext_in_msg.read_from(cell)?;
            CommonMsgInfo::ExtInMsgInfo(ext_in_msg)
        } else {
            let mut ext_out_ms = ExtOutMessageHeader::default();
            ext_out_ms.read_from(cell)?;
            CommonMsgInfo::ExtOutMsgInfo(ext_out_ms)
        };

        Ok(())
    } 
}

pub type MessageId = UInt256;

///////////////////////////////////////////////////////////////////////////////////////////
/// 
/// message$_ {X:Type} info:CommonMsgInfo
/// init:(Maybe (Either StateInit ^StateInit))
/// body:(Either X ^X) = Message X;
///
/// 

#[derive(Debug, Default, Clone)]
pub struct Message {
    header: CommonMsgInfo,
    init: Option<StateInit>,
    body: Option<SliceData>,
    body_to_ref: Option<bool>,
    init_to_ref: Option<bool>,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Message {{header: {}", self.header)?;
        match &self.init {
            Some(init) => write!(f, ", init: {:?}", init)?,
            None => write!(f, ", init: None")?
        }
        match &self.body {
            Some(body) => write!(f, ", body: {:x}", body)?,
            None => write!(f, ", body: None")?
        }
        write!(f, "}}")
    }
}

impl PartialEq for Message {
    fn eq(&self, other: &Message) -> bool {
        self.header == other.header &&
        self.init == other.init &&
        self.body == other.body
    }
}

impl Eq for Message {}

impl Message {
    
    pub fn int_header(&self) -> Option<&InternalMessageHeader> {
        match self.header() {
            CommonMsgInfo::IntMsgInfo(header) => Some(header),
            _ => None
        }
    }

    pub fn ext_in_header(&self) -> Option<&ExternalInboundMessageHeader> {
        match self.header() {
            CommonMsgInfo::ExtInMsgInfo(header) => Some(header),
            _ => None
        }
    }

    pub fn ext_out_header(&self) -> Option<&ExtOutMessageHeader> {
        match self.header() {
            CommonMsgInfo::ExtOutMsgInfo(header) => Some(header),
            _ => None
        }
    }

    ///
    /// Create new instance internal Message with internal header
    ///
    pub fn with_int_header(h: InternalMessageHeader) -> Message {
        Message {
            header: CommonMsgInfo::IntMsgInfo(h),
            init: None,
            body: None,
            body_to_ref: None,
            init_to_ref: None,
        }
    }

    ///
    /// Create new instance of external Message with inbound header
    ///
    pub fn with_ext_in_header(h: ExternalInboundMessageHeader) -> Message{
        Message {
            header: CommonMsgInfo::ExtInMsgInfo(h),
            init: None,
            body: None,
            body_to_ref: None,
            init_to_ref: None,
        }
    }

    ///
    /// Create new instance of external Message with outbound header
    ///
    pub fn with_ext_out_header(h: ExtOutMessageHeader) -> Message{
        Message {
            header: CommonMsgInfo::ExtOutMsgInfo(h),
            init: None,
            body: None,
            body_to_ref: None,
            init_to_ref: None,
        }
    }

    pub fn header(&self) -> &CommonMsgInfo {
        &self.header
    }

    pub fn set_header(&mut self, header: CommonMsgInfo) {
        self.body_to_ref = None;
        self.init_to_ref = None;
        self.header = header;
    }

    pub fn header_mut(&mut self) -> &mut CommonMsgInfo {
        self.body_to_ref = None;
        self.init_to_ref = None;
        &mut self.header
    }

    pub fn withdraw_header(self) -> CommonMsgInfo {
        self.header
    }

    pub fn state_init(&self) -> &Option<StateInit> {
        &self.init
    }

    pub fn set_state_init(&mut self, init: StateInit) {
        self.body_to_ref = None;
        self.init_to_ref = None;
        self.init = Some(init);
    }

    pub fn state_init_mut(&mut self) -> &mut Option<StateInit> {
        self.body_to_ref = None;
        self.init_to_ref = None;
        &mut self.init
    }

    pub fn body(&self) -> Option<SliceData> {
        self.body.clone()
    }

    pub fn set_body(&mut self, body: SliceData) {
        self.body_to_ref = None;
        self.init_to_ref = None;
        self.body = Some(body);
    }

    pub fn body_mut(&mut self) -> &mut Option<SliceData> {
        self.body_to_ref = None;
        self.init_to_ref = None;
        &mut self.body
    }

    ///
    /// Get source account ID for internal message
    /// For other types of messages, function returned None
    ///
    pub fn get_int_src_account_id(&self) -> Option<AccountId> {
        let addr = match self.header {
            CommonMsgInfo::IntMsgInfo(ref header)    => &header.src,
            CommonMsgInfo::ExtOutMsgInfo(ref header) => &header.src,
            _ => &MsgAddressIntOrNone::None,
        };
        if let MsgAddressIntOrNone::Some(ref addr_int) = addr {
            if let MsgAddressInt::AddrStd(ref addr_std) = addr_int {
                return Some(addr_std.address.clone());
            }
            // TODO: What about AddrVar?
        }
        None
    }

    ///
    /// Get destination account ID for internal or inbound external message.
    /// For outbound external messages, function returns None
    ///
    pub fn int_dst_account_id(&self) -> Option<AccountId> {
        self.dst().and_then(|addr| {
            if let MsgAddressInt::AddrStd(std) = addr {
                Some(std.address)
            } else {
                None
            }
        }) 
    }

    ///
    /// Get source internal address.
    ///
    pub fn src(&self) -> Option<MsgAddressInt> {
        let addr1 = match self.header() {
            CommonMsgInfo::IntMsgInfo(ref imi)      => &imi.src,
            CommonMsgInfo::ExtOutMsgInfo(ref eimi)  => &eimi.src,
            CommonMsgInfo::ExtInMsgInfo(_)          => &MsgAddressIntOrNone::None
        };
        match addr1 {
            MsgAddressIntOrNone::None => None,
            MsgAddressIntOrNone::Some(ref addr) => Some(addr.clone())
        }
    }

    ///
    /// Get destination internal address.
    ///
    pub fn dst(&self) -> Option<MsgAddressInt> {
        match self.header {
            CommonMsgInfo::IntMsgInfo(ref header) => Some(header.dst.clone()),
            CommonMsgInfo::ExtInMsgInfo(ref header) => Some(header.dst.clone()),
            _ => None,
        }
    }
    
    ///
    /// Get value transmitted by the message
    /// Set Logical Time and UNIX time for
    /// Internal and External outbound messages
    ///
    pub fn set_at_and_lt(&mut self, at: u32, lt: u64) {
        self.body_to_ref = None;
        self.init_to_ref = None;
        match self.header {
            CommonMsgInfo::IntMsgInfo(ref mut header) => {
                header.created_at = UnixTime32(at);
                header.created_lt = lt;
            },
            CommonMsgInfo::ExtOutMsgInfo(ref mut header) => {
                header.created_at = UnixTime32(at);
                header.created_lt = lt;
            },
            _ => ()
        };
    }
    pub fn set_src(&mut self, address: MsgAddressIntOrNone) {
        match self.header {
            CommonMsgInfo::IntMsgInfo(ref mut header) => {
                header.src = address;
            }
            CommonMsgInfo::ExtOutMsgInfo(ref mut header) => {
                header.src = address;
            }
            _ => ()
        };
    }

    ///
    /// Get message's Unix time and logical time
    /// None only for internal and external outbound message
    ///
    pub fn at_and_lt(&self) -> Option<(u32, u64)> {
        match self.header {
            CommonMsgInfo::IntMsgInfo(ref header) => {
                Some((header.created_at.0, header.created_lt))
            },
            CommonMsgInfo::ExtOutMsgInfo(ref header) => {
                Some((header.created_at.0, header.created_lt))
            },
            _ => None
        }
    }

    pub fn lt(&self) -> Option<u64> {
        match self.header {
            CommonMsgInfo::IntMsgInfo(ref header) => {
                Some(header.created_lt)
            },
            CommonMsgInfo::ExtOutMsgInfo(ref header) => {
                Some(header.created_lt)
            },
            _ => None
        }
    }

    ///
    /// Get value transmitted by the message
    ///
    pub fn get_value<'a>(&'a self) -> Option<&'a CurrencyCollection> {
        self.header.get_value()
    }

    ///
    /// Get value transmitted by the message
    ///
    pub fn get_value_mut<'a>(&'a mut self) -> Option<&'a mut CurrencyCollection> {
        self.body_to_ref = None;
        self.init_to_ref = None;
        self.header.get_value_mut()
    }

    ///
    /// Get message fees
    /// Only Internal and External outbound messages has a fee
    /// If the transmittal of a message it is necessary to collect a fee. Otherwise None
    ///
    pub fn get_fee(&self) -> Result<Option<Grams>> {
        self.header.fee()
    }

    ///
    /// Is message an internal?
    /// 
    pub fn is_internal(&self) -> bool {
        if let CommonMsgInfo::IntMsgInfo(ref _header) = self.header {
            true
        } else {
            false
        }
    }

    ///
    /// Is message an external inbound?
    /// 
    pub fn is_inbound_external(&self) -> bool {
        if let CommonMsgInfo::ExtInMsgInfo(ref _header) = self.header {
            true
        } else {
            false
        }
    }

    ///
    /// is message have state init.
    ///
    pub fn have_state_init(&self) -> bool {
        self.init.is_some()
    }

    ///
    /// Get destination workchain of message
    /// 
    pub fn workchain_id(&self) -> Option<i32> {
        match &self.header {
            CommonMsgInfo::IntMsgInfo(ref imi) => {
                match imi.dst {
                    MsgAddressInt::AddrStd(ref addr) => {
                        Some(addr.workchain_id as i32)
                    }
                    MsgAddressInt::AddrVar(ref addr) => {
                        Some(addr.workchain_id)
                    }
                }
            }
            CommonMsgInfo::ExtOutMsgInfo(_) => {
                None
            }
            CommonMsgInfo::ExtInMsgInfo(ref eimi) => {
                match &eimi.dst {
                    MsgAddressInt::AddrStd(ref addr) => {
                        Some(addr.workchain_id as i32)
                    }
                    MsgAddressInt::AddrVar(ref addr) => {
                        Some(addr.workchain_id)
                    }
                }
            }
        }
    }

    ///
    /// Get source workchain of message
    /// 
    pub fn src_workchain_id(&self) -> Option<i32> {
        let addr1 = match self.header() {
            CommonMsgInfo::IntMsgInfo(ref imi)      => &imi.src,
            CommonMsgInfo::ExtOutMsgInfo(ref eimi)  => &eimi.src,
            CommonMsgInfo::ExtInMsgInfo(_)          => &MsgAddressIntOrNone::None
        };
        match addr1 {
            MsgAddressIntOrNone::None => None,
            MsgAddressIntOrNone::Some(ref addr) => {
                match addr {
                    MsgAddressInt::AddrStd(ref addr) => Some(addr.workchain_id as i32),
                    MsgAddressInt::AddrVar(ref addr) => Some(addr.workchain_id)
                }
            }
        }
    }

    pub fn prepare_proof(&self, is_inbound: bool, block_root: &Cell) -> Result<Cell> {

        // proof for message and block info in block

        let msg_hash = self.hash()?;
        let usage_tree = UsageTree::with_root(block_root.clone());
        let block: Block = Block::construct_from(&mut usage_tree.root_slice()).unwrap();

        block.read_info()?;

        if is_inbound {
            block
                .read_extra()?
                .read_in_msg_descr()?
                .get(&msg_hash)?
                .ok_or_else(|| 
                    BlockError::InvalidArg(
                        "Message isn't belonged given block's in_msg_descr".to_string()
                    )
                )?
                .read_message()?;
        } else {
            block
                .read_extra()?
                .read_out_msg_descr()?
                .get(&msg_hash)?
                .ok_or_else(|| 
                    BlockError::InvalidArg(
                        "Message isn't belonged given block's out_msg_descr".to_string()
                    )
                )?
                .read_message()?;
        }

        MerkleProof::create_by_usage_tree(block_root, &usage_tree)
            .and_then(|proof| proof.write_to_new_cell())
            .map(|cell| cell.into())
    }

}

impl Serializable for Message
{
    fn write_to(&self, builder: &mut BuilderData) -> Result<()> {

        // write header
        self.header.write_to(builder)?;

        let init_builder = if let Some(ref init) = self.init {
            init.write_to_new_cell()?
        } else {
            BuilderData::new()
        };

        let mut header_bits = builder.length_in_bits() + 2; // 2 is state_init's Maybe bit + body's Either bit
        if self.state_init().is_some() {
            header_bits += 1 // state_init's Either bit
        }
        let header_refs = builder.references_used();
        let state_bits = init_builder.length_in_bits();
        let state_refs = init_builder.references_used();
        let (body_bits, body_refs) =
            self.body.as_ref().map(|s| (s.remaining_bits(), s.remaining_references())).unwrap_or((0, 0));

        let (body_to_ref, init_to_ref) = 
        if let (Some(b), Some(i)) = (self.body_to_ref, self.init_to_ref) {
            (b, i)
        } else {
            if header_bits + state_bits + body_bits <= MAX_DATA_BITS &&
                header_refs + state_refs + body_refs <= MAX_REFERENCES_COUNT {
                // all fits into one cell
                (false, false)
            } else {
                if header_bits + state_bits <= MAX_DATA_BITS &&
                    header_refs + state_refs + 1 <= MAX_REFERENCES_COUNT { // + body cell ref
                    // header & state fit
                    (true, false)
                } else if header_bits + body_bits <= MAX_DATA_BITS &&
                    header_refs + body_refs + 1 <= MAX_REFERENCES_COUNT { // + init cell ref
                    // header & body fit
                    (false, true)
                } else {
                    // only header fits
                    (true, true)
                }
            }
        };

        // write StateInit
        match self.init {
            Some(_) => {
                if !init_to_ref {
                    builder.append_bit_one()?      //mayby bit
                        .append_bit_zero()?;    //either bit 
                    builder.append_builder(&init_builder)?;
                } else { // if not enough space in current cell - append as reference
                    builder.append_bit_one()?      //mayby bit
                        .append_bit_one()?;     //either bit 
                    builder.append_reference(init_builder);
                }
            }
            None => {
                // write may be bit
                builder.append_bit_zero()?;
            }
        }

        // write body
        match self.body {
            Some(_) => {
                if !body_to_ref {
                    builder.append_bit_zero()?;     //either bit
                    builder.checked_append_references_and_data(&self.body().unwrap())?;
                } else { // if not enough space in current cell - append as reference
                    builder.append_bit_one()?;     //either bit
                    builder.append_reference(BuilderData::from_slice(&self.body().unwrap()));
                };
            },
            None => {
                // write either be bit
                // otherwise not be able to read 
                builder.append_bit_zero()?;
            },
        }

        Ok(())
    }
}

impl Deserializable for Message {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()>{

        // read header
        self.header.read_from(cell)?;
        
        // read StateInit
        if cell.get_next_bit()? { // maybe of init
            let mut init = StateInit::default();
            if cell.get_next_bit()? { // either of init
                // read from reference
                let mut r = cell.checked_drain_reference()?.into();
                init.read_from(&mut r)?;
                self.init = Some(init);
                self.init_to_ref = Some(true);
            } else { // read from current cell
                init.read_from(cell)?;
                self.init = Some(init);
                self.init_to_ref = Some(false);
            }  
        } else {
            self.init_to_ref = Some(false);
        }

        // read body
        // A message is always serialized inside the blockchain as the last field in
        // a cell. Therefore, the blockchain software may assume that whatever bits
        // and references left unparsed after parsing the fields of a Message preceding
        // body belong to the payload body : X, without knowing anything about the
        // serialization of the type X.

        self.body = if cell.get_next_bit()? { // body in reference
            self.body_to_ref = Some(true);
            Some(cell.checked_drain_reference()?.into())
        } else {
            self.body_to_ref = Some(false);
            if cell.is_empty() { // no body
                None
            } else { // body is leftover
                Some(cell.clone())
            }
        };
        Ok(())
    } 
}

impl InternalMessageHeader {

    pub fn new() -> Self {
        InternalMessageHeader {
            ihr_disabled: false,
            bounce: false,
            bounced: false,
            src: MsgAddressIntOrNone::default(),
            dst: MsgAddressInt::default(),
            value: CurrencyCollection::default(), 
            ihr_fee: Grams::zero(),
            fwd_fee: Grams::zero(),
            created_lt: 0,
            created_at: UnixTime32::default(),
        }
    }
}

////////////////////////////////////////////////////////////////
/// 
/// 3.1.7. Message layout.
/// tick_tock$_ tick:Boolean tock:Boolean = TickTock;
/// 
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TickTock {
    pub tick: bool,
    pub tock: bool,
}

impl TickTock{
    pub fn with_values(tick: bool, tock: bool) -> Self {
        TickTock{ tick, tock }
    }

    pub fn set_tick(&mut self, tick:bool) {
        self.tick = tick;
    }

    pub fn set_tock(&mut self, tock:bool) {
        self.tock = tock;
    }
    pub fn as_usize(&self) -> usize {
        let mut result = 0;
        if self.tick {
            result += 2;
        }
        if self.tock {
            result += 1;
        }
        result
    }
}

impl Serializable for TickTock {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_bit_bool(self.tick)?;
        cell.append_bit_bool(self.tock)?;
        Ok(())
    } 
}

impl Deserializable for TickTock {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()>{
        self.tick = cell.get_next_bit()?;
        self.tock = cell.get_next_bit()?;
        Ok(())
    } 
}

impl fmt::Display for TickTock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "TickTock[Tick {}, Tock {}]", self.tick, self.tock
        )
    }
}

///////////////////////////////////////////////////////////////////////////////
///
/// 3.1.7. Message layout.
/// split_depth:(Maybe (## 5)) special:(Maybe TickTock)
/// code:(Maybe ^Cell) data:(Maybe ^Cell)
/// library:(Maybe ^Cell) = StateInit;
/// 
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StateInit {
    pub split_depth: Option<Number5>,
    pub special: Option<TickTock>,
    pub code: Option<Cell>,
    pub data: Option<Cell>,
    pub library: Option<Cell>,
}

impl StateInit {
    pub fn set_split_depth(&mut self, val: Number5)
    {
        self.split_depth = Some(val);
    }

    pub fn set_special(&mut self, val: TickTock)
    {
        self.special = Some(val);
    }

    pub fn set_code(&mut self, val: Cell)
    {
        self.code = Some(val);
    }

    pub fn set_data(&mut self, val: Cell)
    {
        self.data = Some(val);
    }

    pub fn set_library(&mut self, val: Cell)
    {
        self.library = Some(val);
    }
}

impl Serializable for StateInit {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        
        self.split_depth.write_maybe_to(cell)?;
        self.special.write_maybe_to(cell)?;
        self.code.write_maybe_to(cell)?;
        self.data.write_maybe_to(cell)?;
        self.library.write_maybe_to(cell)?;
        Ok(())
    } 
}

impl Deserializable for StateInit {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()>{
        
        self.split_depth = Number5::read_maybe_from(cell)?;
        self.special = TickTock::read_maybe_from(cell)?;
        // code:(Maybe ^Cell)
        self.code = match cell.get_next_bit()? {
            true => Some(cell.checked_drain_reference()?.clone()),
            false => None,
        };

        // data:(Maybe ^Cell)
        self.data = match cell.get_next_bit()? {
            true => Some(cell.checked_drain_reference()?.clone()),
            false => None,
        };

        // library:(Maybe ^Cell)
        self.library = match cell.get_next_bit()? {
            true => Some(cell.checked_drain_reference()?.clone()),
            false => None,
        };

        Ok(())
    } 
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum MessageProcessingStatus {
    Unknown = 0,
    Queued,
    Processing,
    Preliminary,
    Proposed,
    Finalized,
    Refused,
    Transiting,
}

impl Default for MessageProcessingStatus {
    fn default() -> Self {
        MessageProcessingStatus::Unknown
    }
}

#[allow(dead_code)]
pub fn generate_big_msg() -> Message {
    let mut msg = Message::with_int_header(InternalMessageHeader::default());

    let mut stinit = StateInit::default();
    stinit.set_split_depth(Number5(23));
    stinit.set_special(TickTock::with_values(false, true));
    let mut code = SliceData::new(vec![0x3F, 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xF4]);
    stinit.set_code(code.into_cell());
    let mut code1 = SliceData::new(vec![0xad, 0xc9, 0xba, 0xfc, 0x56, 0x94, 0x11, 0x56, 0x58, 0xfa, 0x2b, 0xdf, 0xe4, 0x65, 0x15, 0x1a, 
                                    0x32, 0x03, 0x69, 0x4a, 0xff, 0xcd, 0x00, 0x8f, 0x36, 0x8b, 0xd2, 0xcc, 0x8c, 0xc8, 0x10, 0xfb, 
                                    0x6b, 0x5b, 0x51]);
    let mut code2 = SliceData::new(vec![0xad, 0xc9, 0xba, 0xfc, 0x56, 0x94, 0x11, 0x56, 0x58, 0xfa, 0x2b, 0xdf, 0xe4, 0x65, 0x15, 0x1a, 
                                    0x32, 0x03, 0x69, 0x4a, 0xff, 0xcd, 0x00, 0x8f, 0x36, 0x8b, 0xd2, 0xcc, 0x8c, 0xc8, 0x10, 0xfb, 
                                    0x6b, 0x5b, 0x51]);
    let code3 = SliceData::new(vec![0xad, 0xc9, 0xba, 0xfc, 0x56, 0x94, 0x11, 0x57, 0x58, 0xfa, 0x2b, 0xdf, 0xe4, 0x65, 0x15, 0x1a, 
                                    0x32, 0x03, 0x69, 0x4a, 0xff, 0xcd, 0x00, 0x8f, 0x36, 0x8b, 0xd2, 0xcc, 0x8c, 0xc8, 0x10, 0xfb, 
                                    0x6b, 0x5b, 0x51]);
    code2.append_reference(code3);
    code1.append_reference(code2);
    code.append_reference(code1);

    stinit.set_code(code.into_cell());

    let data = SliceData::new(vec![0x3F, 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xF4]);
    stinit.set_data(data.into_cell());
    let library = SliceData::new(vec![0x3F, 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xF4]);
    stinit.set_library(library.into_cell());
    
    let mut body = BuilderData::from_slice(&SliceData::new(
            vec![0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
                 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
                 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
                 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
                 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
                 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
                 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
                 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
                 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
                 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
                 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
                 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
                 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
                 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0x80]));
    let mut body1 = BuilderData::from_slice(&SliceData::new(
            vec![0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,
                 0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,
                 0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,
                 0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,
                 0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,
                 0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,
                 0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,
                 0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,
                 0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,
                 0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,
                 0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,
                 0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,
                 0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,
                 0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0x80]));

    let body2 = BuilderData::from_slice(&SliceData::new(
            vec![0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,
                 0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,
                 0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,
                 0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,
                 0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,
                 0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,
                 0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,
                 0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,
                 0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,
                 0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,
                 0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,
                 0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,
                 0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,
                 0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0x80]));

    body1.append_reference(body2);
    body.append_reference(body1);

    *msg.state_init_mut() = Some(stinit);
    *msg.body_mut() = Some(body.into());

    msg
}

///////////////////////////////////////////////////////////////////////////////
/// 
/// Auto-generated code
/// 
///

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct AnycastInfo {
    pub depth: Number5,
    pub rewrite_pfx: SliceData,
}

impl Deserializable for AnycastInfo {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.depth.read_from(cell)?;
        self.rewrite_pfx = cell.get_next_slice(self.depth.0 as usize)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct MsgAddrExt {
    pub len: Number9,
    pub external_address: SliceData,
}

impl Deserializable for MsgAddrExt {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.len.read_from(cell)?;
        self.external_address = cell.get_next_slice(self.len.0 as usize)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MsgAddressExt {
    AddrNone,
    AddrExtern(MsgAddrExt),
}

impl Deserializable for MsgAddressExt {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let bits = cell.get_next_bits(2)?[0] >> 6;
        if bits == 0 {
            *self = MsgAddressExt::AddrNone;
        }
        if bits == 1 {
            let mut data = MsgAddrExt::default();
            data.read_from(cell)?;
            *self = MsgAddressExt::AddrExtern(data);
        }
        // TODO: add error checking!
        Ok(())
    }
}

// TODO: default Default is not working for MsgAddrStd
#[derive(Clone, Debug, /*Default,*/ PartialEq, Eq, Hash)]
pub struct MsgAddrStd {
    pub anycast: Option<AnycastInfo>,
    pub workchain_id: i8,
    pub address: AccountId,
}

impl Deserializable for MsgAddrStd {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.anycast = AnycastInfo::read_maybe_from(cell)?;
        self.workchain_id.read_from(cell)?;
        self.address = cell.get_next_slice(256)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct MsgAddrVar {
    pub anycast: Option<AnycastInfo>,
    pub addr_len: Number9,
    pub workchain_id: i32,
    pub address: SliceData,
}

impl Deserializable for MsgAddrVar {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.anycast = AnycastInfo::read_maybe_from(cell)?;
        self.addr_len.read_from(cell)?;
        self.workchain_id.read_from(cell)?;
        self.address = cell.get_next_slice(self.addr_len.0 as usize)?;
        Ok(())
    }
}                                                                                       	

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MsgAddressInt {
    AddrStd(MsgAddrStd),
    AddrVar(MsgAddrVar),
}

impl Deserializable for MsgAddressInt {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        *self = match cell.get_next_int(2)? {
            0b10 => MsgAddressInt::AddrStd(MsgAddrStd::construct_from(cell)?),
            0b11 => MsgAddressInt::AddrVar(MsgAddrVar::construct_from(cell)?),
            _ => fail!(BlockError::Other("Wrong type of address".to_string()))
        };
        // TODO: fix autogen for error checking!
        /*
        let bits = cell.get_next_bits(2)?[0] >> 6;
        if bits == 2 {
            let mut data = MsgAddrStd::default();
            data.read_from(cell)?;
            *self = MsgAddressInt::AddrStd(data);
        }
        if bits == 3 {
            let mut data = MsgAddrVar::default();
            data.read_from(cell)?;
            *self = MsgAddressInt::AddrVar(data);
        }
        */
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MsgAddress {
    AddrNone,
    AddrExt(MsgAddrExt),
    AddrStd(MsgAddrStd),
    AddrVar(MsgAddrVar),
}

impl Deserializable for MsgAddress {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let bits = cell.get_next_bits(2)?[0] >> 6;
        if bits == 0 {
            *self = MsgAddress::AddrNone;
        }
        if bits == 1 {
            let mut data = MsgAddrExt::default();
            data.read_from(cell)?;
            *self = MsgAddress::AddrExt(data);
        }
        if bits == 2 {
            let mut data = MsgAddrStd::default();
            data.read_from(cell)?;
            *self = MsgAddress::AddrStd(data);
        }
        if bits == 3 {
            let mut data = MsgAddrVar::default();
            data.read_from(cell)?;
            *self = MsgAddress::AddrVar(data);
        }
        Ok(())
    }
}
