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
    merkle_proof::MerkleProof,
    messages::{AnycastInfo, CommonMsgInfo, Message, MsgAddressInt, StateInit, TickTock},
    types::{AddSub, ChildCell, CurrencyCollection, Grams, Number5, VarUInteger7},
    shard::{ShardIdent, ShardStateUnsplit},
    GetRepresentationHash, Serializable, Deserializable, MaybeSerialize, MaybeDeserialize,
};
use std::fmt;
use ton_types::{
    error, fail, Result,
    BagOfCells,
    UInt256, AccountId, BuilderData, Cell, IBitstring, SliceData, UsageTree,
};


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
/// account_none$0 = Account;
/// account$1 addr:MsgAddressInt storage_stat:StorageInfo
/// storage:AccountStorage = Account;
///
/// account_storage$_ last_trans_lt:uint64
/// balance:CurrencyCollection state:AccountState
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
    pub cells: VarUInteger7,
    pub bits: VarUInteger7,
    pub public_cells: VarUInteger7,
}

impl StorageUsed {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_values(cells: u64, bits: u64, public_cells: u64) -> Self {
        StorageUsed {
            cells: VarUInteger7(cells),
            bits: VarUInteger7(bits),
            public_cells: VarUInteger7(public_cells),
        }
    }

    pub fn calculate_for_struct<T: Serializable>(value: &T) -> Result<StorageUsed> {
        let root_cell = value.write_to_new_cell()?.into();
        Ok(Self::calculate_for_cell(&root_cell))
    }

    pub fn calculate_for_cell(root_cell: &Cell) -> StorageUsed {

        let boc = BagOfCells::with_root(root_cell);
        let mut cells: u64 = 0;
        let mut bits: u64 = 0;
        let mut _public_cells: u64 = 0; // TODO

        for (_, cell) in boc.cells().iter() {
            cells += 1;
            bits += cell.bit_length() as u64;
        }

        StorageUsed::with_values(cells, bits, _public_cells)
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
    pub cells: VarUInteger7,
    pub bits: VarUInteger7,
}

impl StorageUsedShort {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_values(cells: u64, bits: u64) -> Self {
        StorageUsedShort {
            cells: cells.into(),
            bits: bits.into(),
        }
    }

    pub fn calculate_for_struct<T: Serializable>(value: &T) -> Result<StorageUsedShort> {
        let root_cell = value.write_to_new_cell()?.into();
        Ok(Self::calculate_for_cell(&root_cell))
    }

    pub fn calculate_for_cell(root_cell: &Cell) -> StorageUsedShort {

        let boc = BagOfCells::with_root(root_cell);
        let mut cells: u64 = 0;
        let mut bits: u64 = 0;

        for (_, cell) in boc.cells().iter() {
            cells += 1;
            bits += cell.bit_length() as u64;
        }

        StorageUsedShort::with_values(cells, bits)
    }

    /// append cell and bits count into
    pub fn append(&mut self, root_cell: &Cell) {
        let addition = Self::calculate_for_cell(root_cell);
        self.cells.0 += addition.cells.0;
        self.bits.0 += addition.bits.0;
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
    pub used: StorageUsed,
    pub last_paid: u32,
    pub due_payment: Option<Grams>,
}

impl StorageInfo {
    pub fn with_values(last_paid: u32, due_payment: Option<Grams>) -> Self {
        StorageInfo {
            used: StorageUsed::default(),
            last_paid,
            due_payment,
        }
    }
}

impl Serializable for StorageInfo {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.used.write_to(cell)?;
        cell.append_u32(self.last_paid)?;
        self.due_payment.write_maybe_to(cell)?;
        Ok(())
    }
}

impl Deserializable for StorageInfo {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.used.read_from(cell)?;
        self.last_paid = cell.get_next_u32()?;
        self.due_payment = Grams::read_maybe_from(cell)?;
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

#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub enum AccountStatus {
    AccStateUninit,
    AccStateFrozen,
    AccStateActive,
    AccStateNonexist,
}

impl Default for AccountStatus {
    fn default() -> Self {
        AccountStatus::AccStateUninit
    }
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
    pub last_trans_lt: u64,
    pub balance: CurrencyCollection,
    pub state: AccountState,
}

impl Serializable for AccountStorage {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.last_trans_lt.write_to(cell)?; //last_trans_lt:uint64
        self.balance.write_to(cell)?; //balance:CurrencyCollection
        self.state.write_to(cell)?; //state:AccountState

        Ok(())
    }
}

impl Deserializable for AccountStorage {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.last_trans_lt.read_from(cell)?; //last_trans_lt:uint64
        self.balance.read_from(cell)?; //balance:CurrencyCollection
        self.state.read_from(cell)?; //state:AccountState

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccountState {
    AccountUninit,
    AccountActive(StateInit),
    AccountFrozen(UInt256),
}

impl AccountState {
    pub fn with_hash(hash: UInt256) -> Self {
        AccountState::AccountFrozen(hash)
    }

    pub fn with_state(state_init: StateInit) -> Self {
        AccountState::AccountActive(state_init)
    }
}

impl Default for AccountState {
    fn default() -> Self {
        AccountState::AccountUninit
    }
}

impl AccountState {
    pub fn freeze_account(&self) -> AccountState {
        match self {
            AccountState::AccountActive(state_init) => {
                AccountState::AccountFrozen(state_init.hash().unwrap())
            }
            AccountState::AccountUninit => AccountState::AccountUninit,
            AccountState::AccountFrozen(x) => AccountState::AccountFrozen(x.clone()),
        }
    }
}

impl Serializable for AccountState {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        match self {
            AccountState::AccountUninit => {
                cell.append_bits(0b00, 2)?; // prefix AccountUninit
            }
            AccountState::AccountFrozen(hash) => {
                cell.append_bits(0b01, 2)?; // prefix AccountFrozen
                cell.append_raw(hash.as_slice(), 256)?; // hash
            }
            AccountState::AccountActive(state) => {
                cell.append_bits(0b1, 1)?; // prefix AccountActive
                state.write_to(cell)?; // StateInit
            }
        }
        Ok(())
    }
}

impl Deserializable for AccountState {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        if cell.get_next_bit()? {
            // if state Active
            let mut state = StateInit::default();
            state.read_from(cell)?; // StateInit
            *self = AccountState::with_state(state);
        } else if cell.get_next_bit()? {
            // if state frozen
            let mut hash = [0; 32];
            let sha256 = cell.get_next_bytes(32)?; // hash
            hash.copy_from_slice(&sha256[0..32]);
            *self = AccountState::with_hash(UInt256::from(hash));
        } else {
            // uninit
            *self = AccountState::AccountUninit; // else state Uninit
        }
        Ok(())
    }
}

impl fmt::Display for AccountState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AccountStorage[{:?}]", self)
    }
}

#[derive(Debug, Clone, Default)]
pub struct AccountStuff {
    pub addr: MsgAddressInt,
    pub storage_stat: StorageInfo,
    pub storage: AccountStorage,
}

impl Serializable for AccountStuff {
    fn write_to(&self, builder: &mut BuilderData) -> Result<()> {
        let mut storage_stat = self.storage_stat.clone();
        storage_stat.used = StorageUsed::calculate_for_struct(&self.storage)?;

        self.addr.write_to(builder)?;
        storage_stat.write_to(builder)?;
        self.storage.write_to(builder)?;

        Ok(())
    }
}

impl Deserializable for AccountStuff {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.addr.read_from(cell)?;
        self.storage_stat.read_from(cell)?;
        self.storage.read_from(cell)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Account {
    AccountNone,
    Account(AccountStuff),
}

impl PartialEq for Account {
    fn eq(&self, other: &Account) -> bool {
        if let Some(stuff1) = self.stuff() {
            if let Some(stuff2) = other.stuff() {
                return stuff1.addr == stuff2.addr
                    && stuff1.storage_stat == stuff2.storage_stat
                    && stuff1.storage == stuff2.storage;
            }
        }
        self.is_none() && other.is_none()
    }
}

impl Eq for Account {}

impl Account {
    ///
    /// Create new empty instance of account
    ///
    pub fn new() -> Self {
        Account::AccountNone
    }

    ///
    /// create unintialize account, only with address and balance
    ///
    pub fn with_address_and_ballance(addr: &MsgAddressInt, balance: &CurrencyCollection) -> Self {
        let mut storage = AccountStorage::default();
        storage.balance = balance.clone();
        //storage.last_trans_lt = current_unix_time(); //the time of account creation or last transaction time

        let account = Account::Account(AccountStuff {
            addr: addr.clone(),
            storage_stat: StorageInfo::default(),
            storage: storage
        });
        account
    }

    ///
    /// Create unintialize account with zero balance
    ///
    pub fn with_address(addr: &MsgAddressInt) -> Self {
        let account = Account::Account(AccountStuff {
            addr: addr.clone(),
            storage_stat: StorageInfo::default(),
            storage: AccountStorage::default()
        });
        account
    }

    ///
    /// Create initialized account from "constructor message"
    ///
    pub fn with_message(msg: &Message) -> Result<Self> {
        let mut storage = AccountStorage::default();
        let mut address = MsgAddressInt::default();

        match msg.header() {
            CommonMsgInfo::IntMsgInfo(ref header) => {
                storage.balance = header.value.clone();
                address = header.dst.clone();
            },
            CommonMsgInfo::ExtInMsgInfo(ref header) => {
                address = header.dst.clone();
            },
            _ => (),
        }
        if let Some(init) = msg.state_init() {
            //code must present in constructor message
            match init.code {
                Some(_) => storage.state = AccountState::AccountActive(init.clone()),
                None => fail!(
                    BlockError::InvalidData(
                        "code field must present in StateInit in constructor message  \
                         while creating account".to_string()
                    )
                ),
            };
        } else {
            fail!(
                BlockError::InvalidData(
                    "stateInit must present in constructor message \
                     while creating account".to_string()
                )
            )
        }

        let account = Account::Account(AccountStuff {
            addr: address,
            storage_stat: StorageInfo::default(),
            storage: storage
        });
        Ok(account)
    }

    #[warn(unused_variables)]
    // freeze account from active
    pub fn freeze_account(&mut self) {
        if let Some(stuff) = self.stuff_mut() {
            let state = if let AccountState::AccountActive(ref state_init) = stuff.storage.state {
                Some(AccountState::AccountFrozen(state_init.hash().unwrap()))
            } else {
                None
            };
            if state.is_some() {
                stuff.storage.state = state.unwrap();
            }
        }
    }

    // constructor only same tests
    pub fn with_storage(
        addr: &MsgAddressInt,
        storage_stat: &StorageInfo,
        storage: &AccountStorage,
    ) -> Self {
        Account::Account(AccountStuff {
            addr: addr.clone(),
            storage_stat: storage_stat.clone(),
            storage: storage.clone()
        })
    }

    pub fn is_none(&self) -> bool {
        self.stuff().is_none()
    }

    pub fn belongs_to_shard(&self, shard: &ShardIdent) -> Result<bool> {
        match self.get_addr() {
            Some(addr) => Ok(addr.get_workchain_id() == shard.workchain_id() && shard.contains_account(addr.get_address())?),
            None => fail!("Account is None")
        }
    }

    pub fn stuff(&self) -> Option<&AccountStuff> {
        if let Account::Account(stuff) = self {
            Some(stuff)
        } else {
            None
        }
    }

    fn stuff_mut(&mut self) -> Option<&mut AccountStuff> {
        if let Account::Account(ref mut stuff) = self {
            Some(stuff)
        } else {
            None
        }
    }

    pub fn update_storage_stat(&mut self) -> Result<()> {
        if let Some(stuff) = self.stuff_mut() {
            stuff.storage_stat.used = StorageUsed::calculate_for_struct(&stuff.storage)?;
        }
        Ok(())
    }

    /// getting statistic using storage for calculate storage/transfer fee
    pub fn get_storage_stat(&self) -> Result<StorageUsed> {
        if let Some(stuff) = self.stuff() {
            Ok(StorageUsed::calculate_for_struct(&stuff.storage)?)
        } else {
            Ok(StorageUsed::new())
        }
    }

    /// getting statistic using storage short for calculate storage/transfer fee
    pub fn get_storage_stat_short(&self) -> Result<StorageUsedShort> {
        if let Some(stuff) = self.stuff() {
            Ok(StorageUsedShort::calculate_for_struct(&stuff.storage)?)
        } else {
            Ok(StorageUsedShort::new())
        }
    }

    /// Getting account ID
    pub fn get_id(&self) -> Option<AccountId> {
        if let MsgAddressInt::AddrStd(addr) = self.get_addr()? {
            Some(addr.address.clone())
        } else {
            None
        }
    }

    pub fn get_addr(&self) -> Option<&MsgAddressInt> {
        self.stuff().map(|s| &s.addr)
    }
    /// Get copy of account's AccountState.
    /// Return None if account is empty (AccountNone)
    pub fn state(&self) -> Option<&AccountState> {
        self.stuff().map(|s| &s.storage.state)
    }

    pub fn get_tick_tock(&self) -> Option<&TickTock> {
        match self.state() {
            Some(AccountState::AccountActive(state_init)) => state_init.special.as_ref(),
            _ => None
        }
    }

    /// Get copy of account's storage information.
    /// Return None if account is empty (AccountNone)
    pub fn storage_info(&self) -> Option<&StorageInfo> {
        self.stuff().map(|s| &s.storage_stat)
    }

    /// getting to the root of the cell with Code of Smart Contract
    pub fn get_code(&self) -> Option<Cell> {
        if let Some(stuff) = self.stuff() {
            if let AccountState::AccountActive(ref state_init) = stuff.storage.state {
                if let Some(ref code) = (*state_init).code {
                    return Some(code.clone());
                }
            }
        }
        None
    }

    /// getting to the root of the cell with persistent Data of Smart Contract
    pub fn get_data(&self) -> Option<Cell> {
        if let Some(stuff) = self.stuff() {
            if let AccountState::AccountActive(ref state_init) = stuff.storage.state {
                if let Some(ref data) = (*state_init).data {
                    return Some(data.clone());
                }
            }
        }
        None
    }

    /// save persistent data of smart contract (for example, after execute code of smart contract into transaction)
    pub fn set_data(&mut self, new_data: Cell) -> bool {
        if let Some(stuff) = self.stuff_mut() {
            if let AccountState::AccountActive(ref mut state_init) = stuff.storage.state {
                if let Some(ref mut data) = (*state_init).data {
                    *data = new_data;
                    return true;
                }
            }
        }
        false
    }

    /// set new code of smart contract
    pub fn set_code(&mut self, new_code: Cell) -> bool {
        if let Some(stuff) = self.stuff_mut() {
            if let AccountState::AccountActive(ref mut state_init) = stuff.storage.state {
                if let Some(ref mut code) = state_init.code {
                    *code = new_code;
                    return true;
                }
            }
        }
        false
    }

    /// set new library code
    pub fn set_library(&mut self, _new_code: Cell) -> bool {
        unimplemented!()
    }

    /// delete library code
    pub fn delete_library(&mut self, _hash: UInt256) -> bool {
        unimplemented!()
    }

    /// Activate account with new StateInit
    pub fn activate(&mut self, state: StateInit) {
        if let Some(stuff) = self.stuff_mut() {
            let new_state = match &stuff.storage.state {
                AccountState::AccountUninit => AccountState::AccountActive(state),
                AccountState::AccountFrozen(ref _acc_address) => {
                    //TODO: calc address from code and data in 'state',
                    //if it's equal to 'acc_address' then ok - apply new state.
                    unimplemented!()
                }
                AccountState::AccountActive(_) => stuff.storage.state.clone(),
            };
            stuff.storage.state = new_state;
        }
    }

    /// getting to the root of the cell with library
    pub fn get_library(&self) -> Option<Cell> {
        if let Some(stuff) = self.stuff() {
            if let AccountState::AccountActive(ref state_init) = stuff.storage.state {
                if let Some(ref library) = (*state_init).library {
                    return Some(library.clone());
                }
            }
        }
        None
    }

    /// Get enum variant indicating current state of account
    pub fn status(&self) -> AccountStatus {
        if let Some(stuff) = self.stuff() {
            match stuff.storage.state {
                AccountState::AccountUninit => AccountStatus::AccStateUninit,
                AccountState::AccountFrozen(_) => AccountStatus::AccStateFrozen,
                AccountState::AccountActive(_) => AccountStatus::AccStateActive,
            }
        } else {
            AccountStatus::AccStateNonexist
        }
    }
    /// getting balance of the account
    pub fn get_balance(&self) -> Option<&CurrencyCollection> {
        self.stuff().map(|s| &s.storage.balance)
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

    pub fn last_tr_time(&mut self) -> Option<u64> {
        self.stuff().map(|stuff| stuff.storage.last_trans_lt)
    }

    pub fn set_last_tr_time(&mut self, tr_lt: u64) {
        if let Some(stuff) = self.stuff_mut() {
            stuff.storage.last_trans_lt = tr_lt;
        }
    }

    pub fn prepare_proof(&self, state_root: &Cell) -> Result<Cell> {
        if self.is_none() {
            fail!(BlockError::InvalidData("Account cannot be None".to_string()))
        } else {
            // proof for account in shard state

            let usage_tree = UsageTree::with_root(state_root.clone());
            let ss: ShardStateUnsplit = 
                ShardStateUnsplit::construct_from(&mut usage_tree.root_slice())?;

            ss
                .read_accounts()?
                .get(&self.get_addr().unwrap().get_address())?
                .ok_or(
                    error!(
                        BlockError::InvalidArg(
                            "Account doesn't belong to given shard state".to_string()
                        )
                    )
                )?
                .read_account()?;

            MerkleProof::create_by_usage_tree(state_root, &usage_tree)
                .and_then(|proof| proof.write_to_new_cell())
                .map(|cell| cell.into())
        }
    }
}

impl Default for Account {
    fn default() -> Self {
        Account::AccountNone
    }
}

impl Serializable for Account {
    fn write_to(&self, builder: &mut BuilderData) -> Result<()> {
        if let Some(stuff) = self.stuff() {
            builder.append_bit_one()?;
            stuff.write_to(builder)?;
        } else {
            builder.append_bit_zero()?;
        }
        Ok(())
    }
}

impl Deserializable for Account {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        *self = if cell.get_next_bit()? {
            Account::Account(AccountStuff::construct_from(cell)?)
        } else {
            Account::AccountNone
        };
        Ok(())
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
    pub fn with_params(account: Account, last_trans_hash: UInt256, last_trans_lt: u64) -> Result<Self> {
        Ok(ShardAccount {
            account: ChildCell::with_struct(&account)?,
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

    pub fn account_cell(&self) -> &Cell {
        self.account.cell()
    }

    pub fn set_account_cell(&mut self, cell: Cell) {
        self.account.set_cell(cell);
    }
}

impl Serializable for ShardAccount {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_reference(self.account.write_to_new_cell()?);
        self.last_trans_hash.write_to(cell)?;
        self.last_trans_lt.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for ShardAccount {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.account
            .read_from(&mut cell.checked_drain_reference()?.into())?;
        self.last_trans_hash.read_from(cell)?;
        self.last_trans_lt.read_from(cell)?;
        Ok(())
    }
}

#[allow(dead_code)]
pub fn generate_test_account() -> Account {
    let mut anc = AnycastInfo::default();
    anc.set_rewrite_pfx(SliceData::new(vec![0x98,0x32,0x17,0x80])).unwrap();

    let acc_id = AccountId::from_raw(vec![0x00,0x01,0x02,0x03,0x04,0x05,0x06,0x07,0x08,0x09,0x0A,0x0B,0x0C,0x0D,0x0E,0x0F,
                                      0x10,0x11,0x12,0x13,0x14,0x15,0x16,0x17,0x18,0x19,0x1A,0x1B,0x1C,0x1D,0x1E,0x1F], 256);

    //let st_used = StorageUsed::with_values(1,2,3,4,5);
    let g = Some(Grams(111u32.into()));
    let st_info = StorageInfo::with_values(123456789, g);
    
    let mut stinit = StateInit::default();
    
    stinit.set_split_depth(Number5(23));
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
    stinit.set_library(library.into_cell());

    let mut balance = CurrencyCollection::new();
    balance.grams = Grams(100000000000u64.into());
    balance.set_other(1, 100).unwrap();
    balance.set_other(2, 200).unwrap();
    balance.set_other(3, 300).unwrap();
    balance.set_other(4, 400).unwrap();
    balance.set_other(5, 500).unwrap();
    balance.set_other(6, 600).unwrap();
    balance.set_other(7, 10000100).unwrap();

    let acc_st = AccountStorage {
        last_trans_lt: 0,
        balance: balance,
        state: AccountState::with_state(stinit),
    };
    let addr = MsgAddressInt::with_standart(Some(anc), 0, acc_id).unwrap();
    let mut account = Account::with_storage(&addr, &st_info, &acc_st);
    account.update_storage_stat().unwrap();
    account
}
