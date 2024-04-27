/*
* Copyright (C) 2019-2023 EverX Rights Reserved.
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
    base64_decode_to_slice,
    sha256_digest,
    define_HashmapE,
    error::BlockError,
    Augmentable,
    HashmapE, HashmapType, Cell, CellType, BuilderData, SliceData,
    IBitstring,
    Serializable, Deserializable,
    SERDE_OPTS_EMPTY
};

use num::{BigInt, bigint::Sign, FromPrimitive, One, Zero};
use std::{
    cmp, convert::TryInto, fmt::{self, LowerHex, UpperHex, Display, Formatter},
    str::{self, FromStr}, ops::{Deref, DerefMut}, marker::PhantomData,
    sync::Arc, time::{SystemTime, UNIX_EPOCH},
};
use smallvec::SmallVec;

pub type Error = failure::Error;
pub type Result<T> = std::result::Result<T, Error>;
pub type Failure = Option<Error>;
pub type Status = Result<()>;

#[macro_export]
macro_rules! error {
    ($error:literal) => {
        failure::err_msg(format!("{} {}:{}", $error, file!(), line!()))
    };
    ($error:expr) => {
        failure::Error::from($error)
    };
    ($fmt:expr, $($arg:tt)+) => {
        failure::err_msg(format!("{} {}:{}", format!($fmt, $($arg)*), file!(), line!()))
    };
}

#[macro_export]
macro_rules! fail {
    ($error:literal) => {
        return Err(failure::err_msg(format!("{} {}:{}", $error, file!(), line!())))
    };
    // uncomment to explicit panic for any ExceptionCode
    // (ExceptionCode::CellUnderflow) => {
    //     panic!("{}", error!(ExceptionCode::CellUnderflow))
    // };
    ($error:expr) => {
        return Err(crate::error!($error))
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err(failure::err_msg(format!("{} {}:{}", format!($fmt, $($arg)*), file!(), line!())))
    };
}

#[derive(Clone, Default, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct UInt256([u8; 32]);

impl PartialEq<SliceData> for UInt256 {
    fn eq(&self, other: &SliceData) -> bool {
        if other.remaining_bits() == 256 {
            return self.0 == other.get_bytestring(0).as_slice()
        }
        false
    }
}

impl PartialEq<SliceData> for &UInt256 {
    fn eq(&self, other: &SliceData) -> bool {
        if other.remaining_bits() == 256 {
            return self.0 == other.get_bytestring(0).as_slice()
        }
        false
    }
}

impl UInt256 {

    pub const fn default() -> Self { Self::new() }
    pub const fn new() -> Self {
        Self::ZERO
    }
    pub const fn with_array(data: [u8; 32]) -> Self {
        Self(data)
    }

    pub fn is_zero(&self) -> bool {
        for b in &self.0 {
            if b != &0 {
                return false
            }
        }
        true
    }

    pub const fn as_array(&self) -> &[u8; 32] {
        &self.0
    }

    pub const fn as_slice(&self) -> &[u8; 32] {
        &self.0
    }

    // Returns solid string like this: a80b23bfe4d301497f3ce11e753f23e8dec32368945ee279d044dbc1f91ace2a
    pub fn as_hex_string(&self) -> String {
        hex::encode(self.0)
    }

    // TODO: usage should be changed to as_hex_string
    #[allow(clippy::wrong_self_convention)]
    pub fn to_hex_string(&self) -> String { 
        self.as_hex_string()
    }

    pub fn calc_file_hash(bytes: &[u8]) -> Self { 
        Self::calc_sha256(bytes)
    }

    pub fn calc_sha256(bytes: &[u8]) -> Self {
        Self(sha256_digest(bytes))
    }

    pub fn first_u64(&self) -> u64 {
        u64::from_be_bytes(self.0[0..8].try_into().unwrap())
    }

    pub fn from_raw(data: Vec<u8>, length: usize) -> Self {
        assert_eq!(length, 256);
        let hash: [u8; 32] = data.try_into().unwrap();
        Self(hash)
    }

    pub fn from_slice(value: &[u8]) -> Self {
        match value.try_into() {
            Ok(hash) => Self(hash),
            Err(_) => Self::from_le_bytes(value)
        }
    }

    pub fn from_be_bytes(value: &[u8]) -> Self {
        let mut data = [0; 32];
        let len = cmp::min(value.len(), 32);
        let offset = 32 - len;
        (0..len).for_each(|i| data[i + offset] = value[i]);
        Self(data)
    }

    pub fn from_le_bytes(value: &[u8]) -> Self {
        let mut data = [0; 32];
        let len = cmp::min(value.len(), 32);
        (0..len).for_each(|i| data[i] = value[i]);
        Self(data)
    }

    pub const fn max() -> Self {
        UInt256::MAX
    }

    pub fn rand() -> Self {
        Self((0..32).map(|_| { rand::random::<u8>() }).collect::<Vec<u8>>().try_into().unwrap())
    }

    pub fn inner(self) -> [u8; 32] {
        self.0
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub const ZERO: UInt256 = UInt256([0; 32]);
    pub const MIN: UInt256 = UInt256([0; 32]);
    pub const MAX: UInt256 = UInt256([0xFF; 32]);
    // hash of default cell 0x96a296d224f285c67bee93c30f8a309157f0daa35dc5b87e410b78630a09cfc7;
    pub const DEFAULT_CELL_HASH: UInt256 = UInt256([150, 162, 150, 210, 36, 242, 133, 198, 123, 238, 147,
        195, 15, 138, 48, 145, 87, 240, 218, 163, 93, 197, 184, 126, 65, 11, 120, 99, 10, 9, 207, 199]);
}

impl From<[u8; 32]> for UInt256 {
    fn from(data: [u8; 32]) -> Self {
        UInt256(data)
    }
}

impl From<&[u8; 32]> for UInt256 {
    fn from(data: &[u8; 32]) -> Self {
        UInt256(*data)
    }
}

impl From<&[u8]> for UInt256 {
    fn from(value: &[u8]) -> Self { Self::from_slice(value) }
}

impl From<Vec<u8>> for UInt256 {
    fn from(value: Vec<u8>) -> Self {
        match value.try_into() {
            Ok(hash) => Self(hash),
            Err(value) => UInt256::from_le_bytes(value.as_slice())
        }
    }
}

impl FromStr for UInt256 {
    type Err = Error;
    fn from_str(value: &str) -> Result<Self> {
        let mut result = Self::default();
        match value.len() {
            64 => hex::decode_to_slice(value, &mut result.0)?,
            66 => hex::decode_to_slice(&value[2..], &mut result.0)?,
            44 => base64_decode_to_slice(value, &mut result.0)?,
            _ => fail!("invalid account ID string (32 bytes expected), but got string {}", value)
        }
        Ok(result)
    }
}

impl fmt::Debug for UInt256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        LowerHex::fmt(self, f)
    }
}

impl fmt::Display for UInt256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "UInt256[{:X?}]", self.as_slice()
        )
    }    
}

impl LowerHex for UInt256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(f, "0x{}", hex::encode(self.0))
        } else {
            write!(f, "{}", hex::encode(self.0))
            // write!(f, "{}...{}", hex::encode(&self.0[..2]), hex::encode(&self.0[30..32]))
        }
    }
}

impl UpperHex for UInt256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(f, "0x")?;
        }
        write!(f, "{}", hex::encode_upper(self.0))
    }
}

impl AsRef<[u8; 32]> for UInt256 {
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

impl AsRef<[u8]> for UInt256 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

pub type AccountId = SliceData;

impl From<[u8; 32]> for AccountId {
    fn from(data: [u8; 32]) -> AccountId {
        SliceData::load_builder(BuilderData::with_raw(SmallVec::from_slice(&data), 256).unwrap()).unwrap()
    }
}

impl From<UInt256> for AccountId {
    fn from(data: UInt256) -> AccountId {
        SliceData::load_builder(BuilderData::with_raw(SmallVec::from_slice(&data.0), 256).unwrap()).unwrap()
    }
}

impl From<&UInt256> for AccountId {
    fn from(data: &UInt256) -> AccountId {
        SliceData::load_builder(BuilderData::with_raw(SmallVec::from_slice(&data.0), 256).unwrap()).unwrap()
    }
}

impl FromStr for AccountId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let uint: UInt256 = FromStr::from_str(s)?;
        Ok(AccountId::from(uint.0))
    }
}

// Exceptions *****************************************************************

#[derive(Clone, Copy, Debug, num_derive::FromPrimitive, PartialEq, Eq, failure::Fail)]
pub enum ExceptionCode {
    #[fail(display = "normal termination")]
    NormalTermination = 0,
    #[fail(display = "alternative termination")]
    AlternativeTermination = 1,
    #[fail(display = "stack underflow")]
    StackUnderflow = 2,
    #[fail(display = "stack overflow")]
    StackOverflow = 3,
    #[fail(display = "integer overflow")]
    IntegerOverflow = 4,
    #[fail(display = "range check error")]
    RangeCheckError = 5,
    #[fail(display = "invalid opcode")]
    InvalidOpcode = 6,
    #[fail(display = "type check error")]
    TypeCheckError = 7,
    #[fail(display = "cell overflow")]
    CellOverflow = 8,
    #[fail(display = "cell underflow")]
    CellUnderflow = 9,
    #[fail(display = "dictionaty error")]
    DictionaryError = 10,
    #[fail(display = "unknown error")]
    UnknownError = 11,
    #[fail(display = "fatal error")]
    FatalError = 12,
    #[fail(display = "out of gas")]
    OutOfGas = 13,
    #[fail(display = "illegal instruction")]
    IllegalInstruction = 14,
    #[fail(display = "pruned cell")]
    PrunedCellAccess = 15,
    #[fail(display = "big cell")]
    BigCellAccess = 16
}

/*
impl fmt::Display for ExceptionCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}
*/

#[rustfmt::skip]
impl ExceptionCode {
/*
    pub fn message(&self) -> &'static str {
        match self {
            ExceptionCode::NormalTermination        => "normal termination",
            ExceptionCode::AlternativeTermination   => "alternative termination",
            ExceptionCode::StackUnderflow           => "stack underflow",
            ExceptionCode::StackOverflow            => "stack overflow",
            ExceptionCode::IntegerOverflow          => "integer overflow",
            ExceptionCode::RangeCheckError          => "range check error",
            ExceptionCode::InvalidOpcode            => "invalid opcode",
            ExceptionCode::TypeCheckError           => "type check error",
            ExceptionCode::CellOverflow             => "cell overflow",
            ExceptionCode::CellUnderflow            => "cell underflow",
            ExceptionCode::DictionaryError          => "dictionary error",
            ExceptionCode::UnknownError             => "unknown error",
            ExceptionCode::FatalError               => "fatal error",
            ExceptionCode::OutOfGas                 => "out of gas error"
        }
    }
*/
    pub fn from_usize(number: usize) -> Option<ExceptionCode> {
        FromPrimitive::from_usize(number)
    }
}

pub trait ByteOrderRead {
    fn read_be_uint(&mut self, bytes: usize) -> std::io::Result<u64>;
    fn read_le_uint(&mut self, bytes: usize) -> std::io::Result<u64>;
    fn read_byte(&mut self) -> std::io::Result<u8>;
    fn read_be_u16(&mut self) -> std::io::Result<u16>;
    fn read_be_u32(&mut self) -> std::io::Result<u32>;
    fn read_be_u64(&mut self) -> std::io::Result<u64>;
    fn read_le_u16(&mut self) -> std::io::Result<u16>;
    fn read_le_u32(&mut self) -> std::io::Result<u32>;
    fn read_le_u64(&mut self) -> std::io::Result<u64>;
    fn read_u256(&mut self) -> std::io::Result<[u8; 32]>;
}

impl<T: std::io::Read> ByteOrderRead for T {
    fn read_be_uint(&mut self, bytes: usize) -> std::io::Result<u64> {
        read_uint(self, bytes, false)
    }

    fn read_le_uint(&mut self, bytes: usize) -> std::io::Result<u64> {
        read_uint(self, bytes, true)
    }

    fn read_byte(&mut self) -> std::io::Result<u8> {
        self.read_be_uint(1).map(|value| value as u8)
    }

    fn read_be_u16(&mut self) -> std::io::Result<u16> {
        self.read_be_uint(2).map(|value| value as u16)
    }

    fn read_be_u32(&mut self) -> std::io::Result<u32> {
        self.read_be_uint(4).map(|value| value as u32)
    }

    fn read_be_u64(&mut self) -> std::io::Result<u64> {
        self.read_be_uint(8)
    }

    fn read_le_u16(&mut self) -> std::io::Result<u16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    fn read_le_u32(&mut self) -> std::io::Result<u32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    fn read_le_u64(&mut self) -> std::io::Result<u64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    fn read_u256(&mut self) -> std::io::Result<[u8; 32]> {
        let mut buf = [0; 32];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }
}

fn read_uint<T: std::io::Read>(src: &mut T, bytes: usize, le: bool) -> std::io::Result<u64> {
    match bytes {
        1 => {
            let mut buf = [0];
            src.read_exact(&mut buf)?;
            Ok(buf[0] as u64)
        }
        2 => {
            let mut buf = [0; 2];
            src.read_exact(&mut buf)?;
            if le {
                Ok(u16::from_le_bytes(buf) as u64)
            } else {
                Ok(u16::from_be_bytes(buf) as u64)
            }
        }
        3..=4 => {
            let mut buf = [0; 4];
            if le {
                src.read_exact(&mut buf[0..bytes])?;
                Ok(u32::from_le_bytes(buf) as u64)
            } else {
                src.read_exact(&mut buf[4 - bytes..])?;
                Ok(u32::from_be_bytes(buf) as u64)
            }
        },
        5..=8 => {
            let mut buf = [0; 8];
            if le {
                src.read_exact(&mut buf[0..bytes])?;
                Ok(u64::from_le_bytes(buf))
            } else {
                src.read_exact(&mut buf[8 - bytes..])?;
                Ok(u64::from_be_bytes(buf))
            }
        },
        n => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("too many bytes ({}) to read in u64", n),
        )),
    }
}

pub type Bitmask = u8;


///
/// var_uint$_ {n:#} len:(#< n) value:(uint (len * 8)) = VarUInteger n;
/// 

/// var_int$_ {n:#} len:(#< n) value:(int (len * 8)) = VarInteger n;
/// nanograms$_ amount:(VarUInteger 16) = Grams;
/// 
/// If one wants to represent x nanograms, one selects an integer l < 16 such
/// that x < 2^8*l, and serializes first l as an unsigned 4-bit integer, then x itself
/// as an unsigned 8`-bit integer. Notice that four zero bits represent a zero
/// amount of Grams.

macro_rules! define_VarIntegerN {
    ( $varname:ident, $N:expr, BigInt ) => {
        #[derive( Eq, Clone, Debug)]
        pub struct $varname(BigInt);

        #[allow(dead_code)]
        impl $varname {

            fn get_len(value: &BigInt) -> usize {
                (value.bits() as usize + 7) >> 3
            }

            pub fn value(&self) -> &BigInt {
                &self.0
            }

            pub fn value_mut(&mut self) -> &mut BigInt {
                &mut self.0
            }

            pub fn zero() -> Self {
                $varname(Zero::zero())
            }

            pub fn one() -> Self {
                $varname(One::one())
            }

            pub fn sgn(&self) -> bool {
                self.0.sign() != Sign::NoSign
            }

            pub fn from_two_u128(hi: u128, lo: u128) -> Result<Self> {
                let val = (BigInt::from(hi) << 128) | BigInt::from(lo);
                Self::check_overflow(&val)?;
                Ok($varname(val))
            }

            pub fn is_zero(&self) -> bool {
                self.0.is_zero()
            }

            fn check_overflow(value: &BigInt) -> Result<()> {
                match Self::get_len(&value) > $N {
                    true => fail!("value {} is bigger than {} bytes", value, $N),
                    false => Ok(())
                }
            }

            // determine the size of the len field, using the formula from 3.3.4 VM 
            fn get_len_len() -> usize {
                let max_bits = ($N - 1) as f64;
                max_bits.log2() as usize + 1
            }

            // Interface to write value with type rule
            fn write_to_cell(value: &BigInt) -> Result<BuilderData> {
                let len = Self::get_len(value);
                if len >= $N {
                    fail!("serialization of {} error {} >= {}", stringify!($varname), len, $N)
                }

                let mut cell = BuilderData::default();
                cell.append_bits(len, Self::get_len_len())?;
                let value = value.to_bytes_be().1;
                cell.append_raw(&value, len * 8)?;
                Ok(cell)
            }

            fn read_from_cell(cell: &mut SliceData) -> Result<BigInt> {
                let len = cell.get_next_int(Self::get_len_len())? as usize;
                if len >= $N {
                    fail!("deserialization of {} error {} >= {}", stringify!($varname), len, $N)
                }
                Ok(BigInt::from_bytes_be(Sign::Plus, &cell.get_next_bytes(len)?))
            }

        }

        impl<T: Into<BigInt>> From<T> for $varname {
            fn from(value: T) -> Self {
                let val = BigInt::from(value.into());
                Self::check_overflow(&val).expect("Integer overflow");
                $varname(val)
            }
        }

        impl FromStr for $varname {
            type Err = crate::Error;

            fn from_str(string: &str) -> Result<Self> {
                let result = if let Some(stripped) = string.strip_prefix("0x") {
                    BigInt::parse_bytes(stripped.as_bytes(), 16)
                } else {
                    BigInt::parse_bytes(string.as_bytes(), 10)
                };
                match result {
                    Some(val) => {
                        Self::check_overflow(&val)?;
                        Ok(Self(val))
                    }
                    None => fail!("cannot parse {} for {}", stringify!($varname), string)
                }
            }
        }

        impl AddSub for $varname {
            fn add(&mut self, other: &Self) -> Result<bool> {
                if let Some(result) = self.0.checked_add(&other.0) {
                    if let Err(err) = Self::check_overflow(&result) {
                        log::warn!("{} + {} overflow: {:?}", self, other, err);
                        Ok(false)
                    } else {
                        self.0 = result;
                        Ok(true)
                    }
                } else {
                    Ok(false)
                }
            }
            fn sub(&mut self, other: &Self) -> Result<bool> {
                if let Some(result) = self.0.checked_sub(&other.0) {
                    self.0 = result;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }

        impl Ord for $varname {
            fn cmp(&self, other: &$varname) -> std::cmp::Ordering {
                Ord::cmp(&self.0, &other.0)
            }
        }

        impl PartialOrd for $varname {
            fn partial_cmp(&self, other: &$varname) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl PartialEq for $varname {
            fn eq(&self, other: &$varname) -> bool {
                self.cmp(other) == std::cmp::Ordering::Equal
            }
        }


        impl Default for $varname {
            fn default() -> Self {
                $varname(BigInt::default())
            }
        }

        impl Serializable for $varname {
            fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
                let data = Self::write_to_cell(&self.0)?;
                cell.append_builder(&data)?;
                Ok(())
            }
        }

        impl Deserializable for $varname {
            fn read_from(&mut self, cell: &mut SliceData) -> Result<()>{
                self.0 = Self::read_from_cell(cell)?;
                Ok(())
            }
        }

        impl fmt::Display for $varname {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", &self.0)
            }
        }
    };
    ( $varname:ident, $N:expr, $tt:ty ) => {
        #[derive( Eq, Copy, Clone, Debug, Default, Ord, PartialEq, PartialOrd)]
        pub struct $varname($tt);

        impl $varname {
            pub fn new(value: $tt) -> Result<Self> {
                Self::check_overflow(&value)?;
                Ok(Self(value))
            }
            pub const fn zero() -> Self { Self(0) }
            pub const fn one() -> Self { Self(1) }
            pub const fn sgn(&self) -> bool { false }
            pub const fn is_zero(&self) -> bool { self.0 == 0 }
            pub fn add_checked(&mut self, other: $tt) -> bool {
                if let Some(result) = self.0.checked_add(other) {
                    if let Err(err) = Self::check_overflow(&result) {
                        log::warn!("{} + {} overflow: {:?}", self, other, err);
                        false
                    } else {
                        self.0 = result;
                        true
                    }
                } else {
                    false
                }
            }
            pub fn sub_checked(&mut self, other: $tt) -> bool {
                if let Some(result) = self.0.checked_sub(other) {
                    self.0 = result;
                    true
                } else {
                    false
                }
            }
            fn check_overflow(value: &$tt) -> Result<()> {
                let bytes = ((0 as $tt).leading_zeros() / 8 - value.leading_zeros() / 8) as usize;
                match bytes > $N {
                    true => fail!("value {} is bigger than {} bytes", value, $N),
                    false => Ok(())
                }
            }
            pub fn get_len(&self) -> usize {
                let bits = 8 - ($N as u8).leading_zeros();
                let bytes = ((0 as $tt).leading_zeros() / 8 - self.0.leading_zeros() / 8) as usize;
                bits as usize + bytes * 8
            }
            pub const fn inner(&self) -> $tt { self.0 }
        }

        impl Serializable for $varname {
            fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
                let bits = 8 - ($N as u8).leading_zeros();
                let bytes = ((0 as $tt).leading_zeros() / 8 - self.0.leading_zeros() / 8) as usize;
                if bytes > $N {
                    fail!("cannot store {} {}, required {} bytes", self, stringify!($varname), bytes)
                }
                cell.append_bits(bytes, bits as usize)?;
                let be_bytes = self.0.to_be_bytes();
                cell.append_raw(&be_bytes[be_bytes.len() - bytes..], bytes * 8)?;
                Ok(())
            }
        }

        impl Deserializable for $varname {
            fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
                let bits = 8 - ($N as u8).leading_zeros();
                let bytes = slice.get_next_int(bits as usize)?;
                let max = std::mem::size_of::<$tt>();
                let mut bytes = slice.get_next_bytes(bytes as usize)?;
                bytes.reverse();
                bytes.resize_with(max, || 0);
                self.0 = <$tt>::from_le_bytes(bytes.as_slice().try_into()?);
                Ok(())
            }
        }

        impl AddSub for $varname {
            fn add(&mut self, other: &Self) -> Result<bool> {
                Ok(self.add_checked(other.0))
            }
            fn sub(&mut self, other: &Self) -> Result<bool> {
                Ok(self.sub_checked(other.0))
            }
        }

        #[cfg(not(test))]
        impl std::convert::TryFrom<$tt> for $varname {
            type Error = crate::Error;
            fn try_from(value: $tt) -> Result<Self> {
                Self::check_overflow(&value)?;
                Ok(Self(value))
            }
        }

        impl fmt::Display for $varname {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", &self.0)
            }
        }

        impl std::ops::Mul<$tt> for $varname {
            type Output = Self;
            fn mul(mut self, rhs: $tt) -> Self::Output {
                self.0 *= rhs;
                self
            }
        }

        impl std::ops::MulAssign<$tt> for $varname {
            fn mul_assign(&mut self, rhs: $tt) {
                self.0 *= rhs;
                
            }
        }

        impl std::ops::Mul for $varname {
            type Output = Self;
            fn mul(mut self, rhs: Self) -> Self::Output {
                self.0 *= rhs.0;
                self
            }
        }

        impl std::ops::MulAssign for $varname {
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }

        impl std::ops::Div<$tt> for $varname {
            type Output = Self;
            fn div(mut self, rhs: $tt) -> Self::Output {
                self.0 /= rhs;
                self
            }
        }

        impl std::ops::DivAssign<$tt> for $varname {
            fn div_assign(&mut self, rhs: $tt) {
                self.0 /= rhs;
                
            }
        }

        impl std::ops::Div for $varname {
            type Output = Self;
            fn div(mut self, rhs: Self) -> Self::Output {
                self.0 /= rhs.0;
                self
            }
        }

        impl std::ops::DivAssign for $varname {
            fn div_assign(&mut self, rhs: Self) {
                self.0 /= rhs.0;
            }
        }

        impl std::ops::Shr<u8> for $varname {
            type Output = Self;
            fn shr(mut self, rhs: u8) -> Self::Output {
                self.0 >>= rhs;
                self
            }
        }

        impl std::ops::ShrAssign<u8> for $varname {
            fn shr_assign(&mut self, rhs: u8) {
                self.0 >>= rhs;
            }
        }

        impl std::ops::Shl<u8> for $varname {
            type Output = Self;
            fn shl(mut self, rhs: u8) -> Self{
                self.0 <<= rhs;
                self
            }
        }

        impl std::ops::ShlAssign<u8> for $varname {
            fn shl_assign(&mut self, rhs: u8) {
                self.0 <<= rhs;
            }
        }

        impl num::CheckedAdd for $varname {
            fn checked_add(&self, rhs: &Self) -> Option<Self> {
                if let Some(result) = self.0.checked_add(rhs.0) {
                    if Self::check_overflow(&result).is_ok() {
                        return Some(Self(result))
                    }
                }
                None
            }
        }

        impl std::ops::Add<$tt> for $varname {
            type Output = Self;
            fn add(mut self, rhs: $tt) -> Self{
                self.0 += rhs;
                self
            }
        }

        impl std::ops::AddAssign<$tt> for $varname {
            fn add_assign(&mut self, rhs: $tt) {
                self.0 += rhs;
            }
        }

        impl std::ops::Add for $varname {
            type Output = Self;
            fn add(mut self, rhs: Self) -> Self{
                self.0 += rhs.0;
                self
            }
        }

        impl std::ops::AddAssign for $varname {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl num::CheckedSub for $varname {
            fn checked_sub(&self, rhs: &Self) -> Option<Self> {
                Some(Self(self.0.checked_sub(rhs.0)?))
            }
        }

        impl std::ops::Sub<$tt> for $varname {
            type Output = Self;
            fn sub(mut self, rhs: $tt) -> Self{
                self.0 -= rhs;
                self
            }
        }

        impl std::ops::SubAssign<$tt> for $varname {
            fn sub_assign(&mut self, rhs: $tt) {
                self.0 -= rhs;
            }
        }

        impl std::ops::Sub for $varname {
            type Output = Self;
            fn sub(mut self, rhs: Self) -> Self{
                self.0 -= rhs.0;
                self
            }
        }

        impl std::ops::SubAssign for $varname {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl PartialEq<$tt> for $varname {
            fn eq(&self, other: &$tt) -> bool {
                self.0.cmp(other) == std::cmp::Ordering::Equal
            }
        }

        impl PartialOrd<$tt> for $varname {
            fn partial_cmp(&self, other: &$tt) -> Option<std::cmp::Ordering> {
                Some(self.0.cmp(other))
            }
        }
    }
}
define_VarIntegerN!(Grams, 15, u128);
define_VarIntegerN!(VarUInteger32, 32, BigInt);
define_VarIntegerN!(VarUInteger3, 3, u32);
define_VarIntegerN!(VarUInteger7, 7, u64);

impl Augmentable for Grams {
    fn calc(&mut self, other: &Self) -> Result<bool> {
        self.add(other)
    }
}

// it cannot produce problem
impl From<u64> for Grams {
    fn from(value: u64) -> Self {
        Self(value as u128)
    }
}

// it cannot produce problem
impl From<u16> for VarUInteger3 {
    fn from(value: u16) -> Self {
        Self(value as u32)
    }
}

// it cannot produce problem
impl From<u32> for VarUInteger7 {
    fn from(value: u32) -> Self {
        Self(value as u64)
    }
}

impl VarUInteger7 {
    pub const fn as_u64(&self) -> u64 { self.0 }
}

impl VarUInteger3 {
    pub const fn as_u32(&self) -> u32 { self.0 }
}

impl Grams {
    pub const fn as_u128(&self) -> u128 { self.0 }
    pub const fn as_u64(&self) -> Option<u64> {
        if self.0 <= u64::MAX as u128 {
            Some(self.0 as u64)
        } else {
            None
        }
    }
}

impl FromStr for Grams {
    type Err = crate::Error;

    fn from_str(string: &str) -> Result<Self> {
        if let Some(stripped) = string.strip_prefix("0x") {
            Ok(Self(u128::from_str_radix(stripped, 16)?))
        } else {
            Ok(Self(string.parse::<u128>()?))
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
///
/// number ## N
/// n<=X
///
macro_rules! define_NumberN_up32bit {
    ( $varname:ident, $N:expr ) => {
        #[derive(PartialEq, Eq, Hash, Clone, Debug, Default, PartialOrd, Ord)]
        pub struct $varname(u32);

        #[allow(dead_code)]
        impl $varname {
            pub fn new_checked(value: u32, max_value: u32) -> Result<Self> {
                if value > max_value {
                    fail!(BlockError::InvalidArg(
                        format!("value: {} must be <= {}", value, max_value) 
                    ))
                }
                Ok($varname(value))
            }

            pub fn new(value: u32) -> Result<Self> {
                let max_value = Self::get_max_value();
                Self::new_checked(value, max_value)
            }

            pub fn as_u8(&self) -> u8 {
                self.0 as u8
            }

            pub fn as_u16(&self) -> u16 {
                self.0 as u16
            }

            pub fn as_u32(&self) -> u32 {
                self.0
            }

            pub fn as_usize(&self) -> usize {
                self.0 as usize
            }

            pub fn get_max_len() -> usize {
                (((1 as u64) << $N) - 1) as usize
            }

            pub fn get_max_value() -> u32 {
                (((1 as u64) << $N) - 1) as u32
            }
        }

        impl Serializable for $varname {
            fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
                cell.append_bits(self.0 as usize, $N)?;
                Ok(())
            }
        }

        impl Deserializable for $varname {
            fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
                self.0 = cell.get_next_int($N)? as u32;
                Ok(())
            }
        }

        impl fmt::Display for $varname {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    f,
                    "vui{}[value = {}]", $N, self.0
                )
            }
        }

        impl PartialEq<u32> for $varname {
            fn eq(&self, other: &u32) -> bool {
                &self.0 == other
            }
        }

        impl PartialOrd<u32> for $varname {
            fn partial_cmp(&self, other: &u32) -> Option<std::cmp::Ordering> {
                Some(self.0.cmp(other))
            }
        }
    };
}
define_NumberN_up32bit!(Number5, 5);
define_NumberN_up32bit!(Number8, 8);
define_NumberN_up32bit!(Number9, 9);
define_NumberN_up32bit!(Number12, 12);
define_NumberN_up32bit!(Number13, 13);
define_NumberN_up32bit!(Number16, 16);
define_NumberN_up32bit!(Number32, 32);

define_HashmapE!{ExtraCurrencyCollection, 32, VarUInteger32}

impl From<HashmapE> for ExtraCurrencyCollection {
    fn from(other: HashmapE) -> Self {
        Self::with_hashmap(other.data().cloned(), SERDE_OPTS_EMPTY)
    }
}

impl From<u8> for Number8 {
    fn from(value: u8) -> Self {
        Self(value as u32)
    }
}

impl From<u8> for Number9 {
    fn from(value: u8) -> Self {
        Self(value as u32)
    }
}

impl From<u8> for Number12 {
    fn from(value: u8) -> Self {
        Self(value as u32)
    }
}

impl From<u8> for Number13 {
    fn from(value: u8) -> Self {
        Self(value as u32)
    }
}

impl From<u16> for Number16 {
    fn from(value: u16) -> Self {
        Self(value as u32)
    }
}

impl From<u32> for Number32 {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl std::convert::TryFrom<u32> for Number5 {
    type Error = crate::Error;
    fn try_from(value: u32) -> crate::Result<Self> {
        Self::new(value)
    }
}

impl std::convert::TryFrom<u32> for Number8 {
    type Error = crate::Error;
    fn try_from(value: u32) -> crate::Result<Self> {
        Self::new(value)
    }
}

impl std::convert::TryFrom<u32> for Number9 {
    type Error = crate::Error;
    fn try_from(value: u32) -> crate::Result<Self> {
        Self::new(value)
    }
}

impl std::convert::TryFrom<u32> for Number12 {
    type Error = crate::Error;
    fn try_from(value: u32) -> crate::Result<Self> {
        Self::new(value)
    }
}

impl std::convert::TryFrom<u32> for Number13 {
    type Error = crate::Error;
    fn try_from(value: u32) -> crate::Result<Self> {
        Self::new(value)
    }
}

impl std::convert::TryFrom<u32> for Number16 {
    type Error = crate::Error;
    fn try_from(value: u32) -> crate::Result<Self> {
        Self::new(value)
    }
}

/*
extra_currencies$_
    dict:(HashMapE 32 (VarUInteger 32))
= ExtraCurrencyCollection;

currencies$_
    grams: Grams
    other:ExtraCurrencyCollection
= CurrencyCollection;
*/
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CurrencyCollection {
    pub grams: Grams,
    pub other: ExtraCurrencyCollection,
}

impl Augmentable for CurrencyCollection {
    fn calc(&mut self, other: &Self) -> Result<bool> {
        self.add(other)
    }
}

impl CurrencyCollection {
    pub fn get_other(&self, key: u32) -> Result<Option<VarUInteger32>> {
        self.other.get(&key)
    }

    pub fn set_other(&mut self, key: u32, other: u128) -> Result<()> {
        self.set_other_ex(key, &VarUInteger32::from_two_u128(0, other)?)?;
        Ok(())
    }

    pub fn set_other_ex(&mut self, key: u32, other: &VarUInteger32) -> Result<()> {
        self.other.set(&key, other)?;
        Ok(())
    }

    pub fn other_as_hashmap(&self) -> HashmapE {
        self.other.as_hashmap()
    }

    pub fn with_grams(grams: u64) -> Self {
        Self::from_grams(Grams::from(grams))
    }

    pub fn from_grams(grams: Grams) -> Self {
        CurrencyCollection {
            grams,
            other: ExtraCurrencyCollection::default()
        }
    }

    pub fn is_zero(&self) -> Result<bool> {
        if !self.grams.is_zero() {
            return Ok(false)
        }
        self.other.iterate(|value| Ok(value.is_zero()))
    }
}

impl Serializable for CurrencyCollection {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.grams.write_to(cell)?;
        self.other.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for CurrencyCollection {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()>{
        self.grams.read_from(cell)?;
        self.other.read_from(cell)?;
        Ok(())
    }
}

pub trait AddSub {
    fn sub(&mut self, other: &Self) -> Result<bool>;
    fn add(&mut self, other: &Self) -> Result<bool>;
}

impl AddSub for CurrencyCollection {
    fn sub(&mut self, other: &Self) -> Result<bool> {
        if !self.grams.sub(&other.grams)? {
            return Ok(false)
        }
        other.other.iterate_with_keys(|key: u32, b| -> Result<bool> {
            if let Some(mut a) = self.other.get(&key)? {
                if a >= b {
                    a.sub(&b)?;
                    self.other.set(&key, &a)?;
                    return Ok(true)
                }
            }
            Ok(false) // coin not found in mine or amount is smaller - cannot subtract
        })
    }
    fn add(&mut self, other: &Self) -> Result<bool> {
        self.grams.add(&other.grams)?;
        let mut result = self.other.clone();
        other.other.iterate_with_keys(|key: u32, b| -> Result<bool> {
            match self.other.get(&key)? {
                Some(mut a) => {
                    a.add(&b)?;
                    result.set(&key, &a)?;
                }
                None => {
                    result.set(&key, &b)?;
                }
            }
            Ok(true)
        })?;
        self.other = result;
        Ok(true)
    }
}

impl fmt::Display for CurrencyCollection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.grams)?;
        if !self.other.is_empty() {
            let mut len = 0;
            write!(f, ", other: {{")?;
            self.other.iterate_with_keys(|key: u32, value| {
                len += 1;
                write!(f, " {} => {},", key, value.0)?;
                Ok(true)
            }).ok();
            write!(f, " count: {} }}", len)?;
        }
        Ok(())
    }
}

impl From<u64> for CurrencyCollection {
    fn from(value: u64) -> Self {
        Self::with_grams(value)
    }
}

impl From<u32> for CurrencyCollection {
    fn from(value: u32) -> Self {
        Self::with_grams(value as u64)
    }
}

impl Serializable for u64 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u64(*self)?;
        Ok(())
    }
}

impl Deserializable for u64 {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        slice.get_next_u64()
    }
}

impl Serializable for u8 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(*self)?;
        Ok(())
    }
}

impl Deserializable for u8 {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        slice.get_next_byte()
    }
}

impl Serializable for i32 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_i32(*self)?;
        Ok(())
    }
}

impl Deserializable for u32 {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        slice.get_next_u32()
    }
}

impl Serializable for u32 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u32(*self)?;
        Ok(())
    }
}

impl Serializable for u128 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u128(*self)?;
        Ok(())
    }
}

impl Deserializable for i32 {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        slice.get_next_i32()
    }
}

impl Serializable for i8 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_i8(*self)?;
        Ok(())
    }
}

impl Deserializable for i8 {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        slice.get_next_byte().map(|v| v as i8)
    }
}

impl Serializable for i16 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_i16(*self)?;
        Ok(())
    }
}

impl Deserializable for i16 {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        slice.get_next_i16()
    }
}

impl Serializable for u16 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u16(*self)?;
        Ok(())
    }
}

impl Deserializable for u16 {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        slice.get_next_u16()
    }
}

impl Serializable for bool {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_bit_bool(*self)?;
        Ok(())
    }
}

impl Deserializable for bool {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        slice.get_next_bit()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InRefValue<X: Deserializable + Serializable>(pub X);

impl<X: Deserializable + Serializable> InRefValue<X> {
    pub fn new(inner: X) -> InRefValue<X> {
        InRefValue(inner)
    }
    pub fn inner(self) -> X {
        self.0
    }
}

impl<X: Deserializable + Serializable> AsRef<X> for InRefValue<X> {
    fn as_ref(&self) -> &X {
        &self.0
    }
}

impl<X: Deserializable + Serializable> Deref for InRefValue<X> {
    type Target = X;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<X: Deserializable + Serializable> DerefMut for InRefValue<X> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<X: Deserializable + Serializable> Deserializable for InRefValue<X> {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        Ok(Self(X::construct_from_reference(slice)?))
    }
    fn construct_from_with_opts(slice: &mut SliceData, opts: u8) -> Result<Self> {
        Ok(Self(X::construct_from_cell_with_opts(slice.checked_drain_reference()?, opts)?))
    }
}

impl<X: Deserializable + Serializable> Serializable for InRefValue<X> {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.0.serialize()?.write_to(cell)
    }
    fn write_with_opts(&self, cell: &mut BuilderData, opts: u8) -> Result<()> {
        cell.checked_append_reference(self.0.serialize_with_opts(opts)?)?;
        Ok(())
    }
}

impl<X: Deserializable> Deserializable for Arc<X> {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        Ok(Arc::new(X::construct_from(slice)?))
    }
}

impl<X: Serializable> Serializable for Arc<X> {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.deref().write_to(cell)
    }
}

#[derive(PartialEq, Copy, Clone, Debug, Eq, Default, Hash)]
pub struct UnixTime32(u32);

impl UnixTime32 {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
    pub const fn as_u32(&self) -> u32 {
        self.0
    }
    pub fn now() -> Self {
        UnixTime32( SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32 )
    }
}

impl From<u32> for UnixTime32 {
    fn from(value: u32) -> Self {
        UnixTime32(value)
    }
}

impl From<UnixTime32> for u32 {
    fn from(value: UnixTime32) -> Self {
        value.0
    }
}

impl Serializable for UnixTime32 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()>{
        self.0.write_to(cell)
    } 
}

impl Deserializable for UnixTime32 {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        Ok(Self(slice.get_next_u32()?))
    }
}

impl Display for UnixTime32 {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

#[derive(Debug, Default, Clone, Eq)]
pub struct ChildCell<T: Serializable + Deserializable> {
    cell: Option<Cell>,
    opts: u8,
    phantom: PhantomData<T>,
}

impl<T: Serializable + Deserializable> ChildCell<T> {
    pub fn with_serde_opts(opts: u8) -> Self {
        Self {
            opts,
            ..Default::default()
        }
    }
    pub fn with_cell(cell: Cell) -> Self {
        Self::with_cell_and_opts(cell, crate::SERDE_OPTS_EMPTY)
    }
    pub fn with_cell_and_opts(cell: Cell, opts: u8) -> Self {
        Self {
            cell: Some(cell),
            opts,
            phantom: PhantomData,
        }
    }
    pub fn with_struct(s: &T) -> Result<Self> {
        Ok(Self::with_cell_and_opts(s.serialize()?, crate::SERDE_OPTS_EMPTY))
    }

    pub fn with_struct_and_opts(s: &T, opts: u8) -> Result<Self> {
        Ok(Self::with_cell_and_opts(s.serialize_with_opts(opts)?, opts))
    }

    pub fn write_struct(&mut self, s: &T) -> Result<()> {
        self.cell = Some(s.serialize_with_opts(self.opts)?);
        Ok(())
    }

    pub fn read_struct(&self) -> Result<T> {
        match self.cell.clone() {
            Some(cell) => {
                if cell.cell_type() == CellType::PrunedBranch {
                    fail!(
                        BlockError::PrunedCellAccess(std::any::type_name::<T>().into())
                    )
                }
                T::construct_from_cell_with_opts(cell, self.opts)
            }
            None => Ok(T::default())
        }
    }

    pub fn cell(&self)-> Cell {
        match self.cell.as_ref() {
            Some(cell) => cell.clone(),
            None => T::default().serialize_with_opts(self.opts).unwrap_or_default()
        }
    }

    pub fn serde_opts(&self) -> u8 {
        self.opts
    }

    pub fn set_cell(&mut self, cell: Cell) {
        self.cell = Some(cell);
    }

    pub fn set_options(&mut self, opts: u8) {
        self.opts = opts;
    }

    pub fn hash(&self) -> UInt256 {
        match self.cell.as_ref() {
            Some(cell) => cell.repr_hash(),
            None => T::default().serialize_with_opts(self.opts).unwrap_or_default().repr_hash()
        }
    }

    pub fn empty(&self) -> bool {
        self.cell.is_none()
    }
}

impl<T: Default + Serializable + Deserializable> PartialEq for ChildCell<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.cell == other.cell {
            return true
        }
        match (self.cell.as_ref(), other.cell.as_ref()) {
            (Some(cell), Some(other)) => cell.eq(other),
            (None, Some(cell)) |
            (Some(cell), None) => cell.eq(
                &T::default().serialize_with_opts(self.opts).unwrap_or_default()
            ),
            (None, None) => true
        }
    }
}

impl<T: Serializable + Deserializable> Serializable for ChildCell<T> {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        if let Some(child_cell) = &self.cell {
            child_cell.write_to(cell)?;
        } else {
            T::default().serialize_with_opts(self.opts)?.write_to(cell)?;
        }
        Ok(())
    }
}

impl<T: Serializable + Deserializable> Deserializable for ChildCell<T> {
    fn construct_from_with_opts(slice: &mut SliceData, opts: u8) -> Result<Self> {
        Ok(Self::with_cell_and_opts(slice.checked_drain_reference()?, opts))
    }
    fn read_from_with_opts(&mut self, slice: &mut SliceData, opts: u8) -> Result<()> {
        *self = Self::construct_from_with_opts(slice, opts)?;
        Ok(())
    }
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        *self = Self::construct_from_with_opts(slice, SERDE_OPTS_EMPTY)?;
        Ok(())
    }
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        Self::construct_from_with_opts(slice, SERDE_OPTS_EMPTY)
    }
}

#[cfg(test)]
#[path = "tests/test_types.rs"]
mod tests;