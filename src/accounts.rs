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
    error::BlockError,
    dictionary::hashmapaug::{HashmapAugType, Augmentation},
    merkle_proof::MerkleProof,
    messages::{AnycastInfo, Message, MsgAddressInt, SimpleLib, StateInit, StateInitLib, TickTock},
    types::{AddSub, ChildCell, CurrencyCollection, Grams, Number5, VarUInteger7},
    shard::{ShardIdent, ShardStateUnsplit},
    shard_accounts::DepthBalanceInfo,
    GetRepresentationHash, Serializable, Deserializable, ConfigParams,
    error, fail, Result,
    UInt256, AccountId, BuilderData, Cell, IBitstring, SliceData, UsageTree, HashmapType,
};
use std::{collections::HashSet, fmt};

#[cfg(test)]
#[path = "tests/test_accounts.rs"]
mod tests;

///////////////////////////////////////////////////////////////////////////////
///
/// 4.1.5. Storage profile of an account.
///
/// storage_used$_ cells:(VarUInteger 7) bits:(VarUInteger 7)
/// ext_refs:(VarUInteger 7) int_refs:(VarUInteger 7)
/// public_cells:(VarUInteger 7) = StorageUsed;
///
/// storage_info$_ used:StorageUsed last_paid:uint32
/// due_payment:(Maybe Grams) = StorageInfo;
///
/// 4.1.6. Account description.
///
/// original format
/// account_none$0 = Account;
/// account$1 addr:MsgAddressInt storage_stat:StorageInfo
/// storage:AccountStorage = Account;
///
/// account_storage$_ last_trans_lt:uint64
/// balance:CurrencyCollection state:AccountState
/// = AccountStorage;
///
/// new format 1
/// account_none$0 = Account;
/// account#1 stuff:AccountStuff = Account;
/// addr:MsgAddressInt storage_stat:StorageInfo
/// storage:AccountStorage = AccountStuff;
///
/// account_storage$_ last_trans_lt:uint64
/// balance:CurrencyCollection state:AccountState
/// init_code_hash:(Maybe uint256)
/// = AccountStorage;
///
/// account_uninit$00 = AccountState;
/// account_active$1 _:StateInit = AccountState;
/// account_frozen$01 state_hash:uint256 = AccountState;
///
/// acc_state_uninit$00 = AccountStatus;
/// acc_state_frozen$01 = AccountStatus;
/// acc_state_active$10 = AccountStatus;
/// acc_state_nonexist$11 = AccountStatus;
///
/// tick_tock$_ tick:Boolean tock:Boolean = TickTock;
/// _ split_depth:(Maybe (## 5)) special:(Maybe TickTock)
/// code:(Maybe ^Cell) data:(Maybe ^Cell)
/// library:(Maybe ^Cell) = StateInit;

///////////////////////////////////////////////////////////////////////////////
///
/// 4.1.5. Storage profile of an account.
///
/// storage_used$_ cells:(VarUInteger 7) bits:(VarUInteger 7)
/// ext_refs:(VarUInteger 7) int_refs:(VarUInteger 7)
/// public_cells:(VarUInteger 7) = StorageUsed;
///

#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord, Default)]
pub struct StorageUsed {
    cells: VarUInteger7,
    bits: VarUInteger7,
    public_cells: VarUInteger7,
}

impl StorageUsed {
    pub fn new() -> Self { Self::default() }
    pub const fn bits(&self) -> u64 { self.bits.as_u64() }
    pub const fn cells(&self) -> u64 { self.cells.as_u64() }
    pub const fn public_cells(&self) -> u64 { self.public_cells.as_u64() }

    pub fn with_values_checked(cells: u64, bits: u64, public_cells: u64) -> Result<Self> {
        Ok(Self {
            cells: VarUInteger7::new(cells)?,
            bits: VarUInteger7::new(bits)?,
            public_cells: VarUInteger7::new(public_cells)?,
        })
    }

    pub fn calculate_for_struct<T: Serializable>(value: &T) -> Result<StorageUsed> {
        let root_cell = value.serialize()?;
        let mut used = Self::default();
        used.calculate_for_cell(&mut HashSet::new(), &root_cell);
        Ok(used)
    }

    fn calculate_for_cell(&mut self, hashes: &mut HashSet<UInt256>, cell: &Cell) {
        if hashes.insert(cell.repr_hash()) {
            self.cells.add_checked(1);
            self.bits.add_checked(cell.bit_length() as u64);
            for i in 0..cell.references_count() {
                self.calculate_for_cell(hashes, &cell.reference(i).unwrap())
            }
        }
    }
}

impl Serializable for StorageUsed {
    fn write_to(&self, output: &mut BuilderData) -> Result<()> {
        self.cells.write_to(output)?; //cells:(VarUInteger 7)
        self.bits.write_to(output)?; //bits:(VarUInteger 7)
        self.public_cells.write_to(output)?; //public_cells:(VarUInteger 7)
        Ok(())
    }
}

impl Deserializable for StorageUsed {
    fn read_from(&mut self, data: &mut SliceData) -> Result<()> {
        self.cells.read_from(data)?; //cells:(VarUInteger 7)
        self.bits.read_from(data)?; //bits:(VarUInteger 7)
        self.public_cells.read_from(data)?; //public_cells:(VarUInteger 7)
        Ok(())
    }
}

impl fmt::Display for StorageUsed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "StorageUsed[cells = {}, bits = {}, public_cells = {}]",
            self.cells, self.bits, self.public_cells
        )
    }
}

/*
storage_used_short$_
    cells:(VarUInteger 7)
  bits:(VarUInteger 7)
= StorageUsedShort;
*/
///
/// StorageUsedShort struct
///
#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord, Default)]
pub struct StorageUsedShort {
    cells: VarUInteger7,
    bits: VarUInteger7,
}

impl StorageUsedShort {
    pub fn new() -> Self { Self::default() }
    pub const fn bits(&self) -> u64 { self.bits.as_u64() }
    pub const fn cells(&self) -> u64 { self.cells.as_u64() }

    pub fn with_values_checked(cells: u64, bits: u64) -> Result<Self> {
        Ok(Self {
            cells: VarUInteger7::new(cells)?,
            bits: VarUInteger7::new(bits)?,
        })
    }

    pub fn calculate_for_struct<T: Serializable>(value: &T) -> Result<StorageUsedShort> {
        let root_cell = value.serialize()?;
        let mut used = Self::default();
        used.calculate_for_cell(&mut HashSet::new(), &root_cell);
        Ok(used)
    }

    fn calculate_for_cell(&mut self, hashes: &mut HashSet<UInt256>, cell: &Cell) {
        if hashes.insert(cell.repr_hash()) {
            self.cells.add_checked(1);
            self.bits.add_checked(cell.bit_length() as u64);
            for i in 0..cell.references_count() {
                self.calculate_for_cell(hashes, &cell.reference(i).unwrap())
            }
        }
    }

    /// append cell and bits count into
    pub fn append(&mut self, root_cell: &Cell) {
        Self::calculate_for_cell(self, &mut HashSet::new(), root_cell);
    }
}

impl Serializable for StorageUsedShort {
    fn write_to(&self, output: &mut BuilderData) -> Result<()> {
        self.cells.write_to(output)?; //cells:(VarUInteger 7)
        self.bits.write_to(output)?; //cells:(VarUInteger 7)
        Ok(())
    }
}

impl Deserializable for StorageUsedShort {
    fn read_from(&mut self, data: &mut SliceData) -> Result<()> {
        self.cells.read_from(data)?; //cells:(VarUInteger 7)
        self.bits.read_from(data)?; //cells:(VarUInteger 7)
        Ok(())
    }
}

impl fmt::Display for StorageUsedShort {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "StorageUsed[cells = {}, bits = {}]",
            self.cells, self.bits
        )
    }
}

///////////////////////////////////////////////////////////////////////////////
///
/// 4.1.5. Storage profile of an account.
/// storage_info$_ used:StorageUsed last_paid:uint32
/// due_payment:(Maybe Grams) = StorageInfo;

#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord, Default)]
pub struct StorageInfo {
    used: StorageUsed,
    last_paid: u32,
    due_payment: Option<Grams>,
}

impl StorageInfo {
    pub fn with_values(last_paid: u32, due_payment: Option<Grams>) -> Self {
        StorageInfo {
            used: StorageUsed::default(),
            last_paid,
            due_payment,
        }
    }
    pub const fn used(&self) -> &StorageUsed { &self.used }
    pub const fn last_paid(&self) -> u32 { self.last_paid }
    pub const fn due_payment(&self) -> Option<&Grams> { self.due_payment.as_ref() }
}

impl Serializable for StorageInfo {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.used.write_to(cell)?;
        cell.append_u32(self.last_paid)?;
        self.due_payment.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for StorageInfo {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.used.read_from(cell)?;
        self.last_paid = cell.get_next_u32()?;
        self.due_payment.read_from(cell)?;
        Ok(())
    }
}

impl fmt::Display for StorageInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "StorageInfo[\r\nlast_paid = {}, \r\ndue_payment = {:?}]",
            self.last_paid, self.due_payment
        )
    }
}

///////////////////////////////////////////////////////////////////////////////
///
/// 4.1.6. Account description.
///
/// acc_state_uninit$00 = AccountStatus;
/// acc_state_frozen$01 = AccountStatus;
/// acc_state_active$10 = AccountStatus;
/// acc_state_nonexist$11 = AccountStatus;
///

#[derive(Default, PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub enum AccountStatus {
    #[default]
    AccStateUninit,
    AccStateFrozen,
    AccStateActive,
    AccStateNonexist,
}

/// serialize AccountStatus
impl Serializable for AccountStatus {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        // write to cell only prefix
        match self {
            AccountStatus::AccStateUninit => cell.append_bits(0b00, 2)?,
            AccountStatus::AccStateFrozen => cell.append_bits(0b01, 2)?,
            AccountStatus::AccStateActive => cell.append_bits(0b10, 2)?,
            AccountStatus::AccStateNonexist => cell.append_bits(0b11, 2)?,
        };
        Ok(())
    }
}

// deserialize AccountStatus
impl Deserializable for AccountStatus {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        // read value of AccountStatus from cell
        let flags = cell.get_next_bits(2)?;
        *self = match flags[0] & 0xC0 {
            0x00 => AccountStatus::AccStateUninit,
            0x80 => AccountStatus::AccStateActive,
            0x40 => AccountStatus::AccStateFrozen,
            0xC0 => AccountStatus::AccStateNonexist,
            _ => fail!(BlockError::Other("unreachable".to_string()))
        };
        Ok(())
    }
}

///////////////////////////////////////////////////////////////////////////////
///
/// 4.1.6. Account description.
///
/// account_storage$_ last_trans_lt:uint64
/// balance:CurrencyCollection state:AccountState
/// = AccountStorage;
///

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AccountStorage {
    last_trans_lt: u64,
    balance: CurrencyCollection,
    state: AccountState,
    init_code_hash: Option<UInt256>, 
}

impl AccountStorage {
    /// Construct storage for uninit account
    pub fn unint(balance: CurrencyCollection) -> Self {
        Self {
            balance,
            ..Self::default()
        }
    }

    /// Construct storage for active account
    pub fn active_by_init_code_hash(
        last_trans_lt: u64, 
        balance: CurrencyCollection, 
        state_init: StateInit, 
        init_code_hash: bool
    ) -> Self {
        let init_code_hash = match init_code_hash {
            true => state_init.code().map(|code| code.repr_hash()),
            false => None
        };
        Self {
            last_trans_lt,
            balance,
            state: AccountState::AccountActive { state_init },
            init_code_hash,
        }
    }

    /// Construct storage for frozen account
    pub fn frozen(
        last_trans_lt: u64, 
        balance: CurrencyCollection, 
        state_init_hash: UInt256
    ) -> Self {
        Self {
            last_trans_lt,
            balance,
            state: AccountState::AccountFrozen { state_init_hash },
            ..Self::default()
        }
    }

    /// Construct storage for uninit account with balance
    pub fn with_balance(balance: CurrencyCollection) -> Self { Self::unint(balance) }

    const fn state(&self) -> &AccountState {
        &self.state
    }
}

impl Serializable for AccountStorage {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.last_trans_lt.write_to(cell)?; //last_trans_lt:uint64
        self.balance.write_to(cell)?; //balance:CurrencyCollection
        self.state.write_to(cell)?; //state:AccountState
        if self.init_code_hash.is_some() {
            self.init_code_hash.write_to(cell)?;
        }
        Ok(())
    }
}

impl fmt::Display for AccountStorage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "AccountStorage[last_trans_lt {}, balance {}, account state {:?}]",
            self.last_trans_lt, self.balance, self.state
        )
    }
}

///////////////////////////////////////////////////////////////////////////////
///
/// 4.1.6. Account description.
///
/// account_uninit$00 = AccountState;
/// account_active$1 _:StateInit = AccountState;
/// account_frozen$01 state_hash:uint256 = AccountState;
///

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
enum AccountState {
    #[default]
    AccountUninit,
    AccountActive {
        state_init: StateInit
    },
    AccountFrozen {
        state_init_hash: UInt256
    }
}

impl Serializable for AccountState {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        match self {
            AccountState::AccountUninit => {
                cell.append_bits(0b00, 2)?; // prefix AccountUninit
            }
            AccountState::AccountFrozen { state_init_hash } => {
                cell.append_bits(0b01, 2)?; // prefix AccountFrozen
                state_init_hash.write_to(cell)?;
            }
            AccountState::AccountActive { state_init } => {
                cell.append_bits(0b1, 1)?; // prefix AccountActive
                state_init.write_to(cell)?; // StateInit
            }
        }
        Ok(())
    }
}

impl Deserializable for AccountState {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        let ret = if slice.get_next_bit()? {
            let state_init = StateInit::construct_from(slice)?;
            AccountState::AccountActive { state_init }
        } else if slice.get_next_bit()? {
            let state_init_hash = slice.get_next_hash()?;
            AccountState::AccountFrozen { state_init_hash }
        } else {
            AccountState::AccountUninit
        };
        Ok(ret) 
    }
}

impl fmt::Display for AccountState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AccountStorage[{:?}]", self)
    }
}

#[derive(Debug, Clone, Default)]
struct AccountStuff {
    addr: MsgAddressInt,
    storage_stat: StorageInfo,
    storage: AccountStorage,
}

impl AccountStuff {
    pub fn storage_stat(&self) -> &StorageInfo {
        &self.storage_stat
    }
    pub fn state_init_mut(&mut self) -> Option<&mut StateInit> {
        match self.storage.state {
            AccountState::AccountActive { 
                ref mut state_init 
            } => Some(state_init),
            _ => None
        }
    }
    fn update_storage_stat(&mut self) -> Result<()> {
        self.storage_stat.used = StorageUsed::calculate_for_struct(&self.storage)?;
        Ok(())
    }
    fn update_storage_stat_fast(&mut self) -> Result<()> {
        let cell = self.storage.serialize()?;
        self.storage_stat.used.bits = VarUInteger7::new(cell.tree_bits_count())?;
        self.storage_stat.used.cells = VarUInteger7::new(cell.tree_cell_count())?;
        self.storage_stat.used.public_cells = VarUInteger7::default();
        Ok(())
    }
}

impl Serializable for AccountStuff {
    fn write_to(&self, builder: &mut BuilderData) -> Result<()> {
        self.addr.write_to(builder)?;
        self.storage_stat.write_to(builder)?;
        self.storage.write_to(builder)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct Account {
    stuff: Option<AccountStuff>
}

impl PartialEq for Account {
    fn eq(&self, other: &Account) -> bool {
        match (self.stuff(), other.stuff()) {
            (Some(stuff1), Some(stuff2)) => {
                stuff1.addr == stuff2.addr
                    && stuff1.storage_stat == stuff2.storage_stat
                    && stuff1.storage == stuff2.storage
            }
            (None, None) => true,
            _ => false
        }
    }
}

impl Eq for Account {}

impl Account {
    pub fn new() -> Self { Self::default() }
    const fn with_stuff(stuff: AccountStuff) -> Self {
        Self {
            stuff: Some(stuff)
        }
    }

    pub fn active_by_init_code_hash(
        addr: MsgAddressInt,
        balance: CurrencyCollection,
        last_paid: u32,
        state_init: StateInit,
        init_code_hash: bool,
    ) -> Result<Self> {
        let mut account = Account::with_stuff(AccountStuff {
            addr,
            storage_stat: StorageInfo::with_values(last_paid, None),
            storage: AccountStorage::active_by_init_code_hash(0, balance, state_init, init_code_hash),
        });
        account.update_storage_stat()?;
        Ok(account)
    }

    ///
    /// create unintialized account, only with address and balance
    ///
    pub fn with_address_and_ballance(addr: &MsgAddressInt, balance: &CurrencyCollection) -> Self {
        Account::with_stuff(AccountStuff {
            addr: addr.clone(),
            storage_stat: StorageInfo::default(),
            storage: AccountStorage::with_balance(balance.clone()),
        })
    }

    ///
    /// Create unintialize account with zero balance
    ///
    pub fn with_address(addr: MsgAddressInt) -> Self {
        Account::with_stuff(AccountStuff {
            addr,
            storage_stat: StorageInfo::default(),
            storage: AccountStorage::default(),
        })
    }

    ///
    /// Create initialized account from "constructor internal message"
    ///
    pub fn from_message_by_init_code_hash(msg: &Message, init_code_hash: bool) -> Option<Self> {
        let hdr = msg.int_header()?;
        if hdr.value().grams.is_zero() {
            return None
        }
        let mut storage = AccountStorage {
            balance: hdr.value().clone(),
            ..Default::default()
        };
        if let Some(init) = msg.state_init() {
            init.code()?;
            storage.init_code_hash = match init_code_hash {
                true => Some(init.code()?.repr_hash()),
                false => None
            };
            storage.state = AccountState::AccountActive {
                state_init: init.clone() 
            };
        } else if hdr.bounce {
            return None
        }
        let mut account = Account::with_stuff(AccountStuff {
            addr: hdr.dst.clone(),
            storage_stat: StorageInfo::default(),
            storage
        });
        account.update_storage_stat().ok()?;
        Some(account)
    }

    // freeze active account
    pub fn try_freeze(&mut self) -> Result<()> {
        if let Some(state) = self.state_mut() {
            if let AccountState::AccountActive {
                state_init
            } = state {
                *state = AccountState::AccountFrozen { 
                    state_init_hash: state_init.hash()? 
                }
            }
        }
        Ok(())
    }

    // uninit active account
    pub fn uninit_account(&mut self) {
        if let Some(stuff) = self.stuff_mut() {
            if let AccountState::AccountActive { 
                state_init: _ 
            } = &stuff.storage.state {
                stuff.storage.state = AccountState::AccountUninit
            }
        }
    }

    /// obsolete - use try_freeze
    pub fn freeze_account(&mut self) { self.try_freeze().unwrap() }

    /// create frozen account - for test purposes
    pub fn frozen(
        addr: MsgAddressInt,
        last_trans_lt: u64,
        last_paid: u32,
        state_hash: UInt256,
        due_payment: Option<Grams>,
        balance: CurrencyCollection
    ) -> Self {
        let storage = AccountStorage::frozen(last_trans_lt, balance, state_hash);
        let used = StorageUsed::calculate_for_struct(&storage).unwrap();
        let storage_stat = StorageInfo {
            used,
            last_paid,
            due_payment,
        };
        let stuff = AccountStuff {
            addr,
            storage_stat,
            storage,
        };
        Account::with_stuff(stuff)
    }

    /// create uninit account - for test purposes
    pub fn uninit(
        addr: MsgAddressInt,
        last_trans_lt: u64,
        last_paid: u32,
        balance: CurrencyCollection
    ) -> Self {
        let storage = AccountStorage {
            last_trans_lt, 
            balance, 
            state: AccountState::AccountUninit,
            ..AccountStorage::default()
        };
        let bits = storage.write_to_new_cell().unwrap().length_in_bits();
        let storage_stat = StorageInfo {
            used: StorageUsed::with_values_checked(1, bits as u64, 0).unwrap(),
            last_paid,
            due_payment: None,
        };
        let stuff = AccountStuff {
            addr,
            storage_stat,
            storage,
        };
        Account::with_stuff(stuff)
    }

    // constructor only same tests
    pub fn with_storage(
        addr: &MsgAddressInt,
        storage_stat: &StorageInfo,
        storage: &AccountStorage,
    ) -> Self {
        Account::with_stuff(AccountStuff {
            addr: addr.clone(),
            storage_stat: storage_stat.clone(),
            storage: storage.clone()
        })
    }

    pub fn system(
        address: MsgAddressInt,
        balance: u64,
        mut state_init: StateInit,
    ) -> Result<Account> {
        state_init.special = Some(TickTock::with_values(true, true));
        let balance = CurrencyCollection::with_grams(balance);
        Account::active_by_init_code_hash(
            address,
            balance,
            0,
            state_init,
            true
        )
    }

    pub fn is_none(&self) -> bool {
        self.stuff().is_none()
    }

    pub fn frozen_hash(&self) -> Option<&UInt256> {
        match self.state() {
            Some(
                AccountState::AccountFrozen { 
                    state_init_hash 
                }
            ) => Some(state_init_hash),
            _ => None
        }
    }

    pub fn init_code_hash(&self) -> Option<&UInt256> {
        self.stuff()?.storage.init_code_hash.as_ref()
    }

    pub fn belongs_to_shard(&self, shard: &ShardIdent) -> Result<bool> {
        match self.get_addr() {
            Some(addr) => Ok(
                addr.get_workchain_id() == shard.workchain_id() && 
                shard.contains_account(addr.get_address())?
            ),
            None => fail!("Account is None")
        }
    }

    fn stuff(&self) -> Option<&AccountStuff> {
        self.stuff.as_ref()
    }

    fn stuff_mut(&mut self) -> Option<&mut AccountStuff> {
        self.stuff.as_mut()
    }

    pub fn update_storage_stat(&mut self) -> Result<()> {
        match self.stuff_mut() {
            Some(stuff) => stuff.update_storage_stat(),
            None => Ok(())
        }
    }

    pub fn update_storage_stat_fast(&mut self) -> Result<()> {
        match self.stuff_mut() {
            Some(stuff) => stuff.update_storage_stat_fast(),
            None => Ok(())
        }
    }

    #[cfg(test)]
    /// getting statistic using storage for calculate storage/transfer fee
    fn get_storage_stat(&self) -> Result<StorageUsed> {
        if let Some(stuff) = self.stuff() {
            StorageUsed::calculate_for_struct(&stuff.storage)
        } else {
            Ok(StorageUsed::default())
        }
    }

    /// Getting account ID
    pub fn get_id(&self) -> Option<AccountId> {
        Some(self.get_addr()?.address())
    }

    pub fn get_addr(&self) -> Option<&MsgAddressInt> {
        self.stuff().map(|s| &s.addr)
    }

    /// Get ref to account's AccountState.
    /// Return None if account is empty (AccountNone)
    fn state(&self) -> Option<&AccountState> {
        self.stuff().map(|s| &s.storage.state)
    }

    fn state_mut(&mut self) -> Option<&mut AccountState> {
        self.stuff_mut().map(|s| &mut s.storage.state)
    }

    pub fn state_init(&self) -> Option<&StateInit> {
        match self.state() {
            Some(
                AccountState::AccountActive { 
                    state_init 
                }
            ) => Some(state_init),
            _ => None
        }
    }

    pub fn state_init_mut(&mut self) -> Option<&mut StateInit> {
        self.stuff_mut().and_then(|stuff| stuff.state_init_mut())
    }

    pub fn get_tick_tock(&self) -> Option<&TickTock> {
        self.state_init().and_then(|s| s.special.as_ref())
    }

    /// Get ref to account's storage information.
    /// Return None if account is empty (AccountNone)
    pub fn storage_info(&self) -> Option<&StorageInfo> {
        self.stuff().map(|s| s.storage_stat())
    }

    /// getting the root of the cell with Code of Smart Contract
    pub fn code(&self) -> Option<&Cell> {
        self.state_init()?.code.as_ref()
    }

    /// getting the root of the cell with Code of Smart Contract
    pub fn get_code(&self) -> Option<Cell> {
        self.code().cloned()
    }

    /// getting the hash of the root of the cell with Code of Smart Contract
    pub fn get_code_hash(&self) -> Option<UInt256> {
        Some(self.state_init()?.code.as_ref()?.repr_hash())
    }

    /// getting the root of the cell with persistent Data of Smart Contract
    pub fn data(&self) -> Option<&Cell> {
        self.state_init()?.data.as_ref()
    }

    /// getting the root of the cell with persistent Data of Smart Contract
    pub fn get_data(&self) -> Option<Cell> { self.data().cloned() }

    /// getting hash of the root of the cell with persistent Data of Smart Contract
    pub fn get_data_hash(&self) -> Option<UInt256> {
        Some(self.state_init()?.data.as_ref()?.repr_hash())
    }

    /// save persistent data of smart contract 
    /// (for example, after execute code of smart contract into transaction)
    pub fn set_data(&mut self, new_data: Cell) -> bool {
        if let Some(state_init) = self.state_init_mut() {
            state_init.set_data(new_data);
            return true
        }
        false
    }

    /// set new code of smart contract
    pub fn set_code(&mut self, new_code: Cell) -> bool {
        if let Some(state_init) = self.state_init_mut() {
            state_init.set_code(new_code);
            return true
        }
        false
    }

    /// set new library code
    pub fn set_library(&mut self, code: Cell, public: bool) -> bool {
        if let Some(state_init) = self.state_init_mut() {
            return state_init.library.set(&code.repr_hash(), &SimpleLib::new(code, public)).is_ok()
        }
        false
    }

    /// change library code public flag
    pub fn set_library_flag(&mut self, hash: &UInt256, public: bool) -> bool {
        if let Some(state_init) = self.state_init_mut() {
            match state_init.library.get(hash) {
                Ok(Some(ref mut lib)) => if lib.is_public_library() == public {
                    return true
                } else {
                    lib.public = public;
                    return state_init.library.set(hash, lib).is_ok()
                }
                _ => return false
            }
        }
        false
    }

    /// delete library code
    pub fn delete_library(&mut self, hash: &UInt256) -> bool {
        if let Some(state_init) = self.state_init_mut() {
            return state_init.library.remove(hash).is_ok()
        }
        false
    }

    /// Try to activate account with new StateInit
    pub fn try_activate_by_init_code_hash(
        &mut self, 
        state_init: &StateInit, 
        init_code_hash: bool
    ) -> Result<()> {
        let mut init_code_hash_opt = None;
        if let Some(stuff) = self.stuff_mut() {
            let new_state = match &stuff.storage.state {
                AccountState::AccountUninit => {
                    if state_init.hash()? == stuff.addr.get_address() {
                        init_code_hash_opt = match init_code_hash {
                            true => state_init.code().map(|code| code.repr_hash()),
                            false => None
                        };
                        AccountState::AccountActive {
                            state_init: state_init.clone()
                        }
                    } else {
                        fail!("StateInit doesn't correspond to uninit account address")
                    }
                }
                AccountState::AccountFrozen { state_init_hash } => {
                    if state_init_hash == &state_init.hash()? {
                        AccountState::AccountActive { 
                            state_init: state_init.clone() 
                        }
                    } else {
                        fail!("StateInit doesn't correspond to frozen hash")
                    }
                }
                _ => stuff.storage.state.clone()
            };
            stuff.storage.state = new_state;
            stuff.storage.init_code_hash = init_code_hash_opt;
            Ok(())
        } else {
            fail!("Cannot activate not existing account")
        }
    }

    /// getting to the root of the cell with library
    pub fn libraries(&self) -> StateInitLib {
        match self.state_init() {
            Some(state_init) => state_init.libraries(),
            None => StateInitLib::default()
        }
    }

    /// Get enum variant indicating current state of account
    pub fn status(&self) -> AccountStatus {
        if let Some(stuff) = self.stuff() {
            match stuff.storage.state() {
                AccountState::AccountUninit => AccountStatus::AccStateUninit,
                AccountState::AccountFrozen { 
                    state_init_hash: _ 
                } => AccountStatus::AccStateFrozen,
                AccountState::AccountActive { 
                    state_init: _ 
                } => AccountStatus::AccStateActive,
            }
        } else {
            AccountStatus::AccStateNonexist
        }
    }

    pub fn last_paid(&self) -> u32 {
        match self.stuff() {
            Some(stuff) => stuff.storage_stat.last_paid,
            None => 0
        }
    }

    /// calculate storage fee and sub funds, freeze if not enough
    pub fn set_last_paid(&mut self, last_paid: u32) {
        if let Some(stuff) = self.stuff_mut() {
            stuff.storage_stat.last_paid = last_paid;
        }
    }

    /// getting due payment
    pub fn due_payment(&self) -> Option<&Grams> {
        self.stuff().and_then(|s| s.storage_stat.due_payment.as_ref())
    }

    /// setting due payment
    pub fn set_due_payment(&mut self, due_payment: Option<Grams>) {
        if let Some(stuff) = self.stuff_mut() {
            stuff.storage_stat.due_payment = due_payment
        }
    }

    /// getting balance of the account
    pub fn balance(&self) -> Option<&CurrencyCollection> {
        self.stuff().map(|s| &s.storage.balance)
    }

    /// deprecated: getting balance of the account
    pub fn get_balance(&self) -> Option<&CurrencyCollection> { self.balance() }

    /// getting balance of the account or empty balance
    pub fn balance_checked(&self) -> CurrencyCollection {
        match self.stuff() {
            Some(s) => s.storage.balance.clone(),
            None => CurrencyCollection::default()
        }
    }

    /// setting balance of the account
    pub fn set_balance(&mut self, balance: CurrencyCollection) {
        if let Some(stuff) = self.stuff_mut() {
            stuff.storage.balance = balance
        }
    }

    /// adding funds to account (for example, for credit phase transaction)
    pub fn add_funds(&mut self, funds_to_add: &CurrencyCollection) -> Result<()> {
        if let Some(stuff) = self.stuff_mut() {
            stuff.storage.balance.add(funds_to_add)?;
        }
        Ok(())
    }

    /// subtraction funds from account (for example, rollback transaction)
    pub fn sub_funds(&mut self, funds_to_sub: &CurrencyCollection) -> Result<bool> {
        if let Some(stuff) = self.stuff_mut() {
            stuff.storage.balance.sub(funds_to_sub)
        } else {
            Ok(false)
        }
    }

    pub fn split_depth(&self) -> Option<Number5> {
        self.state_init().and_then(|s| s.split_depth.clone())
    }

    pub fn last_tr_time(&self) -> Option<u64> {
        self.stuff().map(|stuff| stuff.storage.last_trans_lt)
    }

    pub fn set_last_tr_time(&mut self, tr_lt: u64) {
        if let Some(stuff) = self.stuff_mut() {
            stuff.storage.last_trans_lt = tr_lt;
        }
    }

    pub fn prepare_proof(&self, state_root: &Cell) -> Result<Cell> {
        match self.get_id() {
            Some(addr) => {
                // proof for account in shard state

                let usage_tree = UsageTree::with_root(state_root.clone());
                let ss = ShardStateUnsplit::construct_from_cell(usage_tree.root_cell())?;

                ss
                    .read_accounts()?
                    .get_serialized(addr)?
                    .ok_or_else(|| 
                        error!(
                            BlockError::InvalidArg(
                                "Account doesn't belong to given shard state".to_string()
                            )
                        )
                    )?
                    .read_account()?;

                MerkleProof::create_by_usage_tree(state_root, usage_tree)
                    .and_then(|proof| proof.serialize())
            }
            None => fail!(BlockError::InvalidData("Account cannot be None".to_string()))
        }
    }
    
    pub fn write_original_format(&self, builder: &mut BuilderData) -> Result<()> {
        if let Some(stuff) = self.stuff() {
            builder.append_bit_one()?;
            stuff.addr.write_to(builder)?;
            stuff.storage_stat.write_to(builder)?;
            stuff.storage.last_trans_lt.write_to(builder)?; //last_trans_lt:uint64
            stuff.storage.balance.write_to(builder)?; //balance:CurrencyCollection
            stuff.storage.state.write_to(builder)?; //state:AccountState
        } else {
            builder.append_bit_zero()?;
        }
        Ok(())
    }
    
    fn read_original_format(slice: &mut SliceData) -> Result<Self> {
        let addr = Deserializable::construct_from(slice)?;
        let storage_stat = Deserializable::construct_from(slice)?;
        let last_trans_lt = Deserializable::construct_from(slice)?; //last_trans_lt:uint64
        let balance = Deserializable::construct_from(slice)?; //balance:CurrencyCollection
        let state = Deserializable::construct_from(slice)?; //state:AccountState
        let storage = AccountStorage {
            last_trans_lt,
            balance,
            state,
            ..AccountStorage::default()
        };
        Ok(Account::with_stuff(AccountStuff {addr, storage_stat, storage}))
    }
    
    fn read_version(slice: &mut SliceData, _version: u32) -> Result<Self> {
        let addr = Deserializable::construct_from(slice)?;
        let storage_stat = Deserializable::construct_from(slice)?;
        let last_trans_lt = Deserializable::construct_from(slice)?; //last_trans_lt:uint64
        let balance = CurrencyCollection::construct_from(slice)?; //balance:CurrencyCollection
        let state = Deserializable::construct_from(slice)?; //state:AccountState
        let init_code_hash = Deserializable::construct_from(slice)?;
        let storage = AccountStorage {
            last_trans_lt,
            balance,
            state,
            init_code_hash,
        };
        let stuff = AccountStuff {
            addr,
            storage_stat,
            storage,
        };
        Ok(Account::with_stuff(stuff))
    }
}

// functions for testing purposes
impl Account {
    pub fn set_addr(&mut self, addr: MsgAddressInt) {
        if let Some(s) = self.stuff_mut() {
            s.addr = addr;
        }
    }

    pub fn set_init_code_hash(&mut self, init_code_hash: UInt256) {
        if let Some(s) = self.stuff_mut() {
            s.storage.init_code_hash = Some(init_code_hash);
        }
    }

    pub fn update_config_smc(&mut self, config: &ConfigParams) -> Result<()> {
        let data = self.get_data()
            .ok_or_else(|| error!("config SMC doesn't contain data"))?;
        let mut data = SliceData::load_cell(data)?;
        data.checked_drain_reference()
            .map_err(|_| error!("config SMC data doesn't contain reference with old config"))?;
        let mut builder = data.into_builder();
        let cell = config.config_params.data()
            .ok_or_else(|| error!("configs musn't be empty"))?;
        builder.checked_prepend_reference(cell.clone())?;
        self.set_data(builder.into_cell()?);
        Ok(())
    }
}

impl Augmentation<DepthBalanceInfo> for Account {
    fn aug(&self) -> Result<DepthBalanceInfo> {
        let mut info = DepthBalanceInfo::default();
        if let Some(balance) = self.balance() {
            info.set_balance(balance.clone());
        }
        if let Some(split_depth) = self.state_init().and_then(|s| s.split_depth.clone()) {
            info.set_split_depth(split_depth);
        }
        Ok(info)
    }
}

impl Serializable for Account {
    fn write_to(&self, builder: &mut BuilderData) -> Result<()> {
        if let Some(stuff) = self.stuff() {
            if stuff.storage.init_code_hash.is_some() {
                builder.append_bits(1, 4)?;
                return stuff.write_to(builder)
            }
        }
        Self::write_original_format(self, builder)
    }
}

impl Deserializable for Account {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        if slice.get_next_bit()? {
            Self::read_original_format(slice)
        } else if slice.remaining_bits() == 0 {
            Ok(Account::default())
        } else {
            let tag = slice.get_next_int(3)? as u32;
            match tag {
                0 => Ok(Account::default()),
                1 => {
                    match Account::read_version(slice, tag) {
                        Ok(account) => Ok(account),
                        Err(err) => fail!("cannot deserialize account with tag {}, err {}", tag, err)
                    }
                }
                t => {
                    let s = format!("wrong tag {} deserializing account", tag);
                    fail!(BlockError::InvalidConstructorTag{ t, s })
                }
            }
        }
    }
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Account[{:?}]", self)
    }
}

/*
account_descr$_ account:^Account last_trans_hash:bits256
  last_trans_lt:uint64 = ShardAccount;
*/

/// struct ShardAccount
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ShardAccount {
    account: ChildCell<Account>,
    last_trans_hash: UInt256,
    last_trans_lt: u64
}

impl ShardAccount {

    pub fn with_account_root(
        account_root: Cell, 
        last_trans_hash: UInt256, 
        last_trans_lt: u64
    ) -> Self {
        ShardAccount {
            account: ChildCell::with_cell(account_root),
            last_trans_hash,
            last_trans_lt,
        }
    }

    pub fn with_params(
        account: &Account, 
        last_trans_hash: UInt256, 
        last_trans_lt: u64
    ) -> Result<Self> {
        Ok(ShardAccount {
            account: ChildCell::with_struct(account)?,
            last_trans_hash,
            last_trans_lt,
        })
    }

    pub fn read_account(&self) -> Result<Account> {
        self.account.read_struct()
    }

    pub fn write_account(&mut self, value: &Account) -> Result<()> {
        self.account.write_struct(value)
    }

    pub fn last_trans_hash(&self) -> &UInt256 {
        &self.last_trans_hash
    }

    pub fn set_last_trans_hash(&mut self, hash: UInt256) {
        self.last_trans_hash = hash
    }

    pub fn last_trans_lt(&self) -> u64 {
        self.last_trans_lt
    }

    pub fn set_last_trans_lt(&mut self, lt: u64) {
        self.last_trans_lt = lt
    }

    pub fn last_trans_hash_mut(&mut self) -> &mut UInt256 {
        &mut self.last_trans_hash
    }

    pub fn last_trans_lt_mut(&mut self) -> &mut u64 {
        &mut self.last_trans_lt
    }

    pub fn account_cell(&self) -> Cell {
        self.account.cell()
    }

    pub fn set_account_cell(&mut self, cell: Cell) {
        self.account.set_cell(cell);
    }
}

impl Serializable for ShardAccount {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.account.write_to(cell)?;
        self.last_trans_hash.write_to(cell)?;
        self.last_trans_lt.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for ShardAccount {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.account.read_from(cell)?;
        self.last_trans_hash.read_from(cell)?;
        self.last_trans_lt.read_from(cell)?;
        Ok(())
    }
}

#[allow(dead_code)]
pub fn generate_test_account_by_init_code_hash(init_code_hash: bool) -> Account {
    let mut anc = AnycastInfo::default();
    anc.set_rewrite_pfx(SliceData::new(vec![0x98,0x32,0x17,0x80])).unwrap();

    let acc_id = AccountId::from(
        [0x00,0x01,0x02,0x03,0x04,0x05,0x06,0x07,0x08,0x09,0x0A,0x0B,0x0C,0x0D,0x0E,0x0F,
         0x10,0x11,0x12,0x13,0x14,0x15,0x16,0x17,0x18,0x19,0x1A,0x1B,0x1C,0x1D,0x1E,0x1F]
    );

    //let st_used = StorageUsed::with_values(1,2,3,4,5);
    let g = Some(111.into());
    let st_info = StorageInfo::with_values(123456789, g);
    
    let mut stinit = StateInit::default();
    
    stinit.set_split_depth(Number5::new(23).unwrap());
    stinit.set_special(TickTock::with_values(false, true));
    
    let mut code = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    let mut subcode1 = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    let mut subcode2 = SliceData::new(vec![0b00111111, 0b111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    let mut subcode3 = SliceData::new(vec![0b01111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    let subcode4 = SliceData::new(vec![0b0111111, 0b11111111,0b111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    subcode3.append_reference(subcode4);
    subcode2.append_reference(subcode3);
    subcode1.append_reference(subcode2);
    code.append_reference(subcode1);
    stinit.set_code(code.into_cell());

    let data = SliceData::new(vec![0b00111111, 0b11111111,0b11111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    stinit.set_data(data.into_cell());
    let library = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    stinit.set_library_code(library.into_cell(), true).unwrap();

    let mut balance = CurrencyCollection::with_grams(100000000000);
    balance.set_other(1, 100).unwrap();
    balance.set_other(2, 200).unwrap();
    balance.set_other(3, 300).unwrap();
    balance.set_other(4, 400).unwrap();
    balance.set_other(5, 500).unwrap();
    balance.set_other(6, 600).unwrap();
    balance.set_other(7, 10000100).unwrap();

    let acc_st = AccountStorage::active_by_init_code_hash(0, balance, stinit, init_code_hash);
    let addr = MsgAddressInt::with_standart(Some(anc), 0, acc_id).unwrap();
    let mut account = Account::with_storage(&addr, &st_info, &acc_st);
    account.update_storage_stat().unwrap();
    account
}
