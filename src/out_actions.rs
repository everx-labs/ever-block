/*
* Copyright (C) 2019-2021 TON Labs. All Rights Reserved.
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
    error::BlockError, messages::Message, types::CurrencyCollection, Deserializable, Serializable,
};
use std::collections::LinkedList;
use ton_types::{
    error, fail, AccountId, BuilderData, Cell, IBitstring, Result, SliceData, UInt256,
};

pub const ACTION_SEND_MSG:   u32 = 0x0ec3c86d;
pub const ACTION_SET_CODE:   u32 = 0xad4de08e;
pub const ACTION_RESERVE:    u32 = 0x36e6b809;
pub const ACTION_CHANGE_LIB: u32 = 0x26fa1dd4;
pub const ACTION_COPYLEFT:   u32 = 0x24486f7a;

#[cfg(test)]
#[path = "tests/test_out_actions.rs"]
mod tests;

/*
out_list_empty$_ = OutList 0;
out_list$_ {n:#} prev:^(OutList n) action:OutAction = OutList (n+1);
action_reserve#ad4de08e = OutAction;
action_send_msg#0ec3c86d out_msg:^Message = OutAction;
action_set_code#ad4de08e new_code:^Cell = OutAction;
*/


///
/// List of output actions
///
pub type OutActions = LinkedList<OutAction>;


///
/// Implementation of Serializable for OutActions
///
impl Serializable for OutActions {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {

        let mut builder = BuilderData::new();

        for action in self.iter() {
            let mut next_builder = BuilderData::new();

            next_builder.append_reference_cell(builder.into_cell()?);
            action.write_to(&mut next_builder)?;

            builder = next_builder;
        }

        cell.append_builder(&builder)?;
        Ok(())
    }
}


///
/// Implementation of Deserializable for OutActions
///
impl Deserializable for OutActions {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let mut cell = cell.clone();
        while cell.remaining_references() != 0 {
            let prev_cell = cell.checked_drain_reference()?;
            let action = OutAction::construct_from(&mut cell)?;
            self.push_front(action);
            cell = prev_cell.into();
        }
        if !cell.is_empty() {
            fail!(BlockError::Other("cell is not empty".to_string()))
        }
        Ok(())
    }
}



///
/// Enum OutAction
///
#[derive(Clone, Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum OutAction {

    ///
    /// Action for send message
    ///
    SendMsg {
        mode: u8,
        out_msg: Message,
    },

    ///
    /// Action for set new code of smart-contract
    ///
    SetCode {
        new_code: Cell,
    },

    ///
    /// Action for reserving some account balance.
    /// It is roughly equivalent to creating an output
    /// message carrying x nanograms to oneself,so that
    /// the subsequent output actions would not be able
    /// to spend more money than the remainder.
    ///
    ReserveCurrency {
        mode: u8,
        value: CurrencyCollection,
    },

    ///
    /// Action for change library.
    ///
    ChangeLibrary {
        mode: u8,
        code: Option<Cell>,
        hash: Option<UInt256>,
    },

    ///
    /// Action for revert reward for code to code creater.
    ///
    CopyLeft {
        license: u8,
        address: AccountId,
    },

    None
}

impl Default for OutAction {
    fn default() -> Self {
        OutAction::None
    }
}

/// Flags of SendMsg action
pub const SENDMSG_ORDINARY: u8 = 0;
pub const SENDMSG_PAY_FEE_SEPARATELY: u8 = 1;
pub const SENDMSG_IGNORE_ERROR: u8 = 2;
pub const SENDMSG_DELETE_IF_EMPTY: u8 = 32;
pub const SENDMSG_REMAINING_MSG_BALANCE: u8 = 64;
pub const SENDMSG_ALL_BALANCE: u8 = 128;
//mask for cheking valid flags
pub const SENDMSG_VALID_FLAGS: u8 =
    SENDMSG_ORDINARY
    | SENDMSG_PAY_FEE_SEPARATELY
    | SENDMSG_IGNORE_ERROR
    | SENDMSG_DELETE_IF_EMPTY
    | SENDMSG_REMAINING_MSG_BALANCE
    | SENDMSG_ALL_BALANCE;

/// variants of reserve action
pub const RESERVE_EXACTLY: u8 = 0;
pub const RESERVE_ALL_BUT: u8 = 1;
pub const RESERVE_IGNORE_ERROR: u8 = 2;
pub const RESERVE_PLUS_ORIG: u8 = 4;
pub const RESERVE_REVERSE: u8 = 8;
pub const RESERVE_VALID_MODES: u8 =
    RESERVE_EXACTLY
    | RESERVE_ALL_BUT
    | RESERVE_IGNORE_ERROR
    | RESERVE_PLUS_ORIG
    | RESERVE_REVERSE;

pub const CHANGE_LIB_REMOVE: u8 = 0;
pub const SET_LIB_CODE_REMOVE: u8 = 1;
pub const SET_LIB_CODE_ADD_PRIVATE: u8 = 2 + 1;
pub const SET_LIB_CODE_ADD_PUBLIC: u8 = 2 * 2 + 1;

///
/// Implementation of Output Actions
///
impl OutAction {

    ///
    /// Create new instance OutAction::ActionSend
    ///
    pub fn new_send(mode: u8, out_msg: Message) -> Self {
        OutAction::SendMsg { mode, out_msg }
    }

    ///
    /// Create new instance OutAction::ActionCode
    ///
    pub fn new_set(new_code: Cell) -> Self {
        OutAction::SetCode { new_code }
    }

    ///
    /// Create new instance OutAction::ReserveCurrency
    ///
    pub fn new_reserve(mode: u8, value: CurrencyCollection) -> Self {
        OutAction::ReserveCurrency { mode, value }
    }

    ///
    /// Create new instance OutAction::ChangeLibrary
    ///
    pub fn new_change_library(mode: u8, code: Option<Cell>, hash: Option<UInt256>) -> Self {
        debug_assert!(match mode {
            CHANGE_LIB_REMOVE => code.is_none() && hash.is_some(),
            SET_LIB_CODE_REMOVE |
            SET_LIB_CODE_ADD_PRIVATE |
            SET_LIB_CODE_ADD_PUBLIC => code.is_some() && hash.is_none(),
            _ => false
        });
        OutAction::ChangeLibrary { mode, code, hash }
    }

    ///
    /// Create new instance OutAction::Copyleft
    ///
    pub fn new_copyleft(license: u8, address: AccountId) -> Self {
        OutAction::CopyLeft { license, address }
    }
}

impl Serializable for OutAction {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        match self {
            OutAction::SendMsg{ref mode, ref out_msg} => {
                ACTION_SEND_MSG.write_to(cell)?; // tag
                mode.write_to(cell)?;
                cell.append_reference_cell(out_msg.serialize()?);
            }
            OutAction::SetCode{ref new_code} => {
                ACTION_SET_CODE.write_to(cell)?; //tag
                cell.append_reference_cell(new_code.clone());
            }
            OutAction::ReserveCurrency{ref mode, ref value} => {
                ACTION_RESERVE.write_to(cell)?; // tag
                mode.write_to(cell)?;
                value.write_to(cell)?;
            }
            OutAction::ChangeLibrary{ref mode, ref code, ref hash} => {
                ACTION_CHANGE_LIB.write_to(cell)?; // tag
                mode.write_to(cell)?;
                if let Some(value) = hash {
                    value.write_to(cell)?;
                }
                if let Some(value) = code {
                    cell.append_reference_cell(value.clone());
                }
            }
            OutAction::CopyLeft{ref license, ref address} => {
                ACTION_COPYLEFT.write_to(cell)?; // tag
                license.write_to(cell)?;
                address.write_to(cell)?;
            }
            OutAction::None => fail!(
                BlockError::InvalidOperation("self is None".to_string())
            )
        }
        Ok(())
    }
}

impl Deserializable for OutAction {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        if cell.remaining_bits() < std::mem::size_of::<u32>() * 8 {
            fail!(
                BlockError::InvalidArg("cell can't be shorter than 32 bits".to_string())
            )
        }
        let tag = cell.get_next_u32()?;
        match tag {
            ACTION_SEND_MSG => {
                let mode = cell.get_next_byte()?;
                let msg = Message::construct_from_reference(cell)?;
                *self = OutAction::new_send(mode, msg);
            }
            ACTION_SET_CODE => {
                *self = OutAction::new_set(cell.checked_drain_reference()?)
            }
            ACTION_RESERVE => {
                let mut mode = 0u8;
                let mut value = CurrencyCollection::default();
                mode.read_from(cell)?;
                value.read_from(cell)?;
                *self = OutAction::new_reserve(mode, value);
            }
            ACTION_CHANGE_LIB => {
                let mut mode = 0u8;
                mode.read_from(cell)?;
                match mode & 1 {
                    0 => {
                        let hash = cell.get_next_hash()?;
                        *self = OutAction::new_change_library(mode, None, Some(hash));
                    }
                    _ => {
                        let code = cell.checked_drain_reference()?;
                        *self = OutAction::new_change_library(mode, Some(code), None);
                    }
                }
            }
            ACTION_COPYLEFT => {
                let license = cell.get_next_byte()?;
                let mut address = AccountId::default();
                address.read_from(cell)?;
                *self = OutAction::new_copyleft(license, address);
            }
            tag => fail!(
                BlockError::InvalidConstructorTag {
                    t: tag,
                    s: "OutAction".to_string()
                }
            )
        }
        Ok(())
    }
}
