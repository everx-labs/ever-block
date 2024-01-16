/*
* Copyright (C) 2019-2024 EverX. All Rights Reserved.
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

use super::*;
use std::fs::File;
use ton_types::BocReader;
use crate::{MsgAddressExt, write_read_and_assert};

#[test]
fn test_serialize_storage_used()
{
    let st_used = StorageUsed::with_values_checked(1,2,3).unwrap();
    let mut s = BuilderData::default();
    st_used.write_to(&mut s).unwrap();
    let st_used1 = StorageUsed::with_values_checked(1,256,3).unwrap();
    st_used1.write_to(&mut s).unwrap();

    let mut s = SliceData::load_builder(s).unwrap();

    let mut st_used2 = StorageUsed::default();
    st_used2.read_from(&mut s).unwrap();
    let mut st_used3 = StorageUsed::default();
    st_used3.read_from(&mut s).unwrap();

    assert_eq!(st_used, st_used2);
    assert_eq!(st_used1, st_used3);

}

#[test]
fn test_storage_used_short() {
    let stu1 = StorageUsedShort::default();
    let stu2 = StorageUsedShort::default();
    
    assert_eq!(stu1, stu2);
    write_read_and_assert(stu1);

    let stu1 = StorageUsedShort::with_values_checked(1234231, 233232345634).unwrap();
    let stu2 = StorageUsedShort::with_values_checked(1234232, 233232345633).unwrap();

    assert_ne!(stu1, stu2);
    write_read_and_assert(stu1);
    write_read_and_assert(stu2);
}


#[test]
fn test_serialize_storage_info()
{
    let g = Some(111.into());
    let g_none: Option<Grams> = None;
    let st_info = StorageInfo::with_values(123456789, g);
    let st_info1 = StorageInfo::with_values(123456789, g_none);

    let mut s = BuilderData::new();

    st_info.write_to(&mut s).unwrap();

    st_info1.write_to(&mut s).unwrap();

    st_info.write_to(&mut s).unwrap();


    let mut s1 = StorageInfo::default();
    let mut s2 = StorageInfo::default();
    let mut s3 = StorageInfo::default();

    let mut s = SliceData::load_builder(s).unwrap();
    s1.read_from(&mut s).unwrap();
    s2.read_from(&mut s).unwrap();
    s3.read_from(&mut s).unwrap();

    assert_eq!(s1, st_info);
    assert_eq!(s1, s3);
    assert_eq!(st_info1, s2);

}

/*
pub struct StateInit {
    split_depth: Option<u8>,
    special: Option<TickTock>,
    code: Option<Cell>,
    data: Option<Cell>,
    library: Option<Cell>,
}
*/
#[test]
fn test_state_init(){
    let stinit = StateInit::default();
    write_read_and_assert(stinit);
}

#[test]
fn test_state_init1(){
    let mut stinit = StateInit::default();
    
    stinit.set_split_depth(Number5::new(23).unwrap());

    write_read_and_assert(stinit);
}

#[test]
fn test_state_init2(){
    let mut stinit = StateInit::default();
    
    stinit.set_split_depth(Number5::new(23).unwrap());
    stinit.set_special(TickTock::with_values(false, true));

    write_read_and_assert(stinit);
}

fn prepare_library_code() -> Cell {
    SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]).into_cell()
}

fn prepare_library(public: bool) -> StateInitLib {
    let mut lib = StateInitLib::default();
    let code = prepare_library_code();
    lib.set(&code.repr_hash(), &SimpleLib::new(code, public)).unwrap();
    lib
}

#[test]
fn test_state_init3(){
    let mut stinit = StateInit::default();
    
    stinit.set_split_depth(Number5::new(23).unwrap());
    stinit.set_special(TickTock::with_values(false, true));
    let code = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    stinit.set_code(code.into_cell());
    let data = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    stinit.set_data(data.into_cell());
    stinit.set_library_code(prepare_library_code(), false).unwrap();

    write_read_and_assert(stinit);
}

#[test]
fn test_state_init4(){
    let mut stinit = StateInit::default();
    
    stinit.set_split_depth(Number5::new(31).unwrap());
    let code = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b0,0b11111111,0b0,0b11111111,0b11110100]);
    stinit.set_code(code.into_cell());
    stinit.set_library_code(prepare_library_code(), false).unwrap();

    write_read_and_assert(stinit);
}

#[test]
fn test_account_state_uninit()
{
    let acc_state = AccountState::default();
    write_read_and_assert(acc_state);
}

#[test]
fn test_account_state_active() {
    let mut stinit = StateInit::default();
    stinit.set_split_depth(Number5::new(31).unwrap());
    stinit.set_special(TickTock::with_values(false, true));
    let code = SliceData::new(vec![0b00111111, 0b0,0b11111111,0b11111111,0b0,0b11111111,0b11111111,0b11110100]);
    stinit.set_code(code.into_cell());
    let data = SliceData::new(vec![0b00111111, 0b11111111,0b0,0b11111111,0b11111111,0b0,0b11111111,0b11110100]);
    stinit.set_data(data.into_cell());
    stinit.set_library_code(prepare_library_code(), false).unwrap();

    let state_init = stinit;

    write_read_and_assert(AccountState::AccountActive{ state_init });
}

#[test]
fn test_account_state_frozen() {
    let mut stinit = StateInit::default();
    stinit.set_split_depth(Number5::new(31).unwrap());
    stinit.set_special(TickTock::with_values(false, true));
    let code = SliceData::new(vec![0b00111111, 0b0,0b11111111,0b11111111,0b0,0b11111111,0b11111111,0b11110100]);
    stinit.set_code(code.into_cell());
    let data = SliceData::new(vec![0b00111111, 0b11111111,0b0,0b11111111,0b11111111,0b0,0b11111111,0b11110100]);
    stinit.set_data(data.into_cell());
    stinit.set_library_code(prepare_library_code(), false).unwrap();

    let state_init_hash = stinit.hash().unwrap();

    write_read_and_assert(AccountState::AccountFrozen{ state_init_hash });
}


/*
pub struct AnycastInfo{
    depth: u8,                      // ##5
    pub rewrite_pfx: SliceData,         // depth length
}
*/

#[test]
fn test_anycastinfo_exception()
{
    let mut anc = AnycastInfo::default();
    anc.set_rewrite_pfx(SliceData::new(vec![0x34,0x35,0x36,0x37,0x80])).expect_err("pfx can't be longer than 2^5-1 bits");
}

#[test]
fn test_anycastinfo()
{
    let mut anc = AnycastInfo::default();
    anc.set_rewrite_pfx(SliceData::new(vec![0x34,0x35,0x36,0x80])).unwrap();
    write_read_and_assert(anc);
}

/*
pub struct MsgAddrStd {
    pub anycast: Option<AnycastInfo>,
    addr_len: Number9,                  
    workchain_id: i8,
    address: SliceData,
}
*/

#[test]
fn test_msg_addr_std_empty()
{
    let addr = MsgAddressExt::with_extern(SliceData::default()).unwrap();
    write_read_and_assert(addr);
}

#[test]
fn test_msg_addr_std()
{
    let mut anc = AnycastInfo::default();
    anc.set_rewrite_pfx(SliceData::new(vec![0x34,0x35,0x36,0x80])).unwrap();

    let addr = MsgAddressInt::with_variant(Some(anc), 0, SliceData::new(vec![0x01,0x02,0x03,0x04,0x05,0x80])).unwrap();
    write_read_and_assert(addr);
}

/*
pub struct MsgAddressInt {
    anycast: Option<AnycastInfo>,
    workchain_id: i8,
    address: AccountId,
}
*/


#[test]
fn test_msg_addr_int_empty()
{
    let addr = MsgAddressInt::default();
    write_read_and_assert(addr);
}


#[test]
fn test_msg_addr_int()
{

    let mut anc = AnycastInfo::default();
    anc.set_rewrite_pfx(SliceData::new(vec![0x98,0x32,0x17,0x80])).unwrap();

    let acc_id = AccountId::from([0x00,0x01,0x02,0x03,0x04,0x05,0x06,0x07,0x08,0x09,0x0A,0x0B,0x0C,0x0D,0x0E,0x0F,
                                      0x10,0x11,0x12,0x13,0x14,0x15,0x16,0x17,0x18,0x19,0x1A,0x1B,0x1C,0x1D,0x1E,0x1F]);

    let addr = MsgAddressInt::with_standart(Some(anc), 0, acc_id).unwrap();
    write_read_and_assert(addr);
}


/*
pub struct MsgAddressExt {
    len: u8, // ## 8
    external_address: SliceData, // len length
}
*/

#[test]
fn test_msg_addr_1ext_exception() {
    MsgAddressExt::with_extern(SliceData::from_raw(vec![0; 64], 512)).unwrap_err();
}

#[test]
fn test_msg_addr_ext() {
    let addr = MsgAddressExt::with_extern(SliceData::new(vec![
        0x00,0x01,0x02,0x03,0x04,0x05,0x06,0x07,0x08,0x09,0x0A,0x0B,0x0C,0x0D,0x0E,0x0F,
        0x00,0x01,0x02,0x03,0x04,0x05,0x06,0x07,0x08,0x09,0x0A,0x0B,0x0C,0x0D,0x0E,0x0F,
        0x00,0x01,0x02,0x03,0x04,0x05,0x06,0x07,0x08,0x09,0x0A,0x0B,0x0C,0x0D,0x0E,0x0F,
        0x10,0x11,0x12,0x13,0x14,0x15,0x16,0x17,0x18,0x19,0x1A,0x1B,0x1C,0x1D,0x1E,0x1F]
    )).unwrap();
    write_read_and_assert(addr);
}

/*
pub enum MsgAddress{
    AddrNone,
    AddrExtern(MsgAddressExt),
    AddrStd(MsgAddrStd),
    AddrVar(MsgAddressInt),
}
*/

#[test]
fn test_msg_addr_empty() {
    let addr = MsgAddressInt::default();
    write_read_and_assert(addr);
}

#[test]
fn test_msg_addr_standart() {
    let mut anc = AnycastInfo::default();
    anc.set_rewrite_pfx(SliceData::new(vec![0x34,0x35,0x36,0x80])).unwrap();
    let addr = MsgAddressInt::with_variant(Some(anc), 0, SliceData::new(vec![0x01,0x02,0x03,0x04,0x05,0x80])).unwrap();
    write_read_and_assert(addr);
}

#[test]
fn test_msg_addr_var() {
    let mut anc = AnycastInfo::default();
    anc.set_rewrite_pfx(SliceData::new(vec![0x98,0x32,0x17,0x80])).unwrap();

    let acc_id = AccountId::from([0x00,0x01,0x02,0x03,0x04,0x05,0x06,0x07,0x08,0x09,0x0A,0x0B,0x0C,0x0D,0x0E,0x0F,
                                 0x10,0x11,0x12,0x13,0x14,0x15,0x16,0x17,0x18,0x19,0x1A,0x1B,0x1C,0x1D,0x1E,0x1F]);
    let addr = MsgAddressInt::with_standart(Some(anc), 0, acc_id).unwrap();
    write_read_and_assert(addr);
}


#[test]
fn test_currency_collection_addiction()
{
    let c1 = CurrencyCollection::with_grams(1000);

    let c2 = CurrencyCollection::with_grams(2000);

    let mut c_res = c1; c_res.add(&c2).unwrap();

    let c3 = CurrencyCollection::with_grams(3000);

    assert_eq!(c_res, c3);

    let mut c1 = CurrencyCollection::with_grams(1000);
    c1.set_other(1, 100).unwrap();
    c1.set_other(2, 200).unwrap();

    let mut c2 = CurrencyCollection::with_grams(2000);
    c2.set_other(2, 300).unwrap();
    c2.set_other(3, 300).unwrap();

    let mut c_res = c1; c_res.add(&c2).unwrap();

    let mut c3 = CurrencyCollection::with_grams(3000);
    c3.set_other(1, 100).unwrap();
    c3.set_other(2, 500).unwrap();
    c3.set_other(3, 300).unwrap();

    assert_eq!(c_res, c3);
}

/*
pub enum Account{
    AccountNone,
    Account{
        addr: MsgAddressInt,
        storage_stat: StorageInfo,
        storage: AccountStorage,
    },
}*/

#[test]
fn test_account_none(){
    let acc = Account::default();
    write_read_and_assert(acc);
}

#[test]
fn test_account_account(){
    
    let mut anc = AnycastInfo::default();
    anc.set_rewrite_pfx(SliceData::new(vec![0x98,0x32,0x17,0x80])).unwrap();

    let acc_id = AccountId::from([0x00,0x01,0x02,0x03,0x04,0x05,0x06,0x07,0x08,0x09,0x0A,0x0B,0x0C,0x0D,0x0E,0x0F,
                                      0x10,0x11,0x12,0x13,0x14,0x15,0x16,0x17,0x18,0x19,0x1A,0x1B,0x1C,0x1D,0x1E,0x1F]);

    //let st_used = StorageUsed::with_values(1,2,3,4,5);
    let g = Some(111.into());
    let st_info = StorageInfo::with_values(123456789, g);  
    
    let mut stinit = StateInit::default();
    
    stinit.set_split_depth(Number5::new(23).unwrap());
    stinit.set_special(TickTock::with_values(false, true));
    let code = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    stinit.set_code(code.into_cell());
    let data = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    stinit.set_data(data.into_cell());
    stinit.set_library_code(prepare_library_code(), false).unwrap();

    let acc_st = AccountStorage::active_by_init_code_hash(
        0, CurrencyCollection::default(), stinit, false
    );

    let addr = MsgAddressInt::with_standart(Some(anc), 0, acc_id).unwrap();
    let mut acc = Account::with_storage(&addr, &st_info, &acc_st);
    acc.update_storage_stat().unwrap();

    write_read_and_assert(acc);
}

#[test]
fn test_account_account2(){
    
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
    let subcode2 = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    subcode1.append_reference(subcode2);
    code.append_reference(subcode1);
    stinit.set_code(code.clone().into_cell());
    let data = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    stinit.set_data(data.clone().into_cell());
    stinit.set_library_code(prepare_library_code(), false).unwrap();

    let mut balance = CurrencyCollection::with_grams(100000000000u64);
    balance.set_other(1, 100).unwrap();
    balance.set_other(2, 200).unwrap();
    balance.set_other(3, 300).unwrap();
    balance.set_other(4, 400).unwrap();
    balance.set_other(5, 500).unwrap();
    balance.set_other(6, 600).unwrap();
    balance.set_other(7, 10000100).unwrap();

    let acc_st = AccountStorage::active_by_init_code_hash(0, balance, stinit, false);

    let addr = MsgAddressInt::with_standart(Some(anc), 0, acc_id).unwrap();
    let mut acc = Account::with_storage(&addr, &st_info, &acc_st);
    acc.update_storage_stat().unwrap();

    println!("acc before update {}", acc);
    let su1 = acc.get_storage_stat().unwrap();
    println!("StorageUsed before {}", su1);

    let mut acc = write_read_and_assert(acc);

    let su2 = acc.get_storage_stat().unwrap();
    println!("StorageUsed after {}", su2);
    assert_eq!(su1, su2);

    if let Some(acc_code) = acc.get_code() {
        assert_eq!(code, SliceData::load_cell(acc_code).unwrap());
    }
    
    if let Some(acc_data) = acc.get_data() {
        assert_eq!(data, SliceData::load_cell(acc_data).unwrap());
    }
    
    assert_eq!(prepare_library(false), acc.libraries());

    let mut f_to_add = CurrencyCollection::with_grams(12);
    f_to_add.set_other(3, 1005000).unwrap();

    acc.add_funds(&f_to_add).unwrap();

    let mut result_f = CurrencyCollection::with_grams(100000000012u64);
    result_f.set_other(1, 100).unwrap();
    result_f.set_other(2, 200).unwrap();
    result_f.set_other(3, 1005300).unwrap();
    result_f.set_other(4, 400).unwrap();
    result_f.set_other(5, 500).unwrap();
    result_f.set_other(6, 600).unwrap();
    result_f.set_other(7, 10000100).unwrap();
    
    assert_eq!(*acc.get_balance().unwrap(), result_f);
}

#[test]
fn test_freeze_account() {
    let mut acc = generate_test_account_by_init_code_hash(false);
    acc.try_freeze().unwrap();
    assert!(acc.status() == AccountStatus::AccStateFrozen, "Account isnt in frozen state!");
}

#[test]
fn test_compare_currency_collections() {
    let c1 = CurrencyCollection::with_grams(10);

    let c2 = CurrencyCollection::with_grams(20);

    let c3 = CurrencyCollection::with_grams(20);

    assert!(c1 != c2);
    assert!(c2 == c3);
    assert!(c2 != c1);

    let mut c1 = CurrencyCollection::with_grams(10);
    c1.set_other(1, 100).unwrap();

    let c2 = CurrencyCollection::with_grams(20);

    let c3 = CurrencyCollection::with_grams(20);

    assert!(c1 != c2);
    assert!(c2 == c3);
    assert!(c2 != c1);

    let mut c1 = CurrencyCollection::with_grams(10);
    c1.set_other(1, 100).unwrap();

    let mut c2 = CurrencyCollection::with_grams(20);
    c2.set_other(2, 200).unwrap();

    let mut c3 = CurrencyCollection::with_grams(20);
    c3.set_other(2, 200).unwrap();

    assert!(c1 != c2);
    assert!(c2 == c3);
    assert!(c2 != c1);

    let mut c1 = CurrencyCollection::with_grams(10);
    c1.set_other(1, 100).unwrap();

    let mut c2 = CurrencyCollection::with_grams(20);
    c2.set_other(1, 200).unwrap();

    let mut c3 = CurrencyCollection::with_grams(20);
    c3.set_other(2, 200).unwrap();

    assert!(c1 != c2);
    assert!(c2 != c3);
    assert!(c2 != c1);

    let mut c1 = CurrencyCollection::with_grams(10);
    c1.set_other(1, 100).unwrap();
    c1.set_other(2, 200).unwrap();
    c1.set_other(3, 300).unwrap();
    
    let mut c2 = CurrencyCollection::with_grams(20);
    c2.set_other(1, 200).unwrap();
    c2.set_other(2, 400).unwrap();
    c2.set_other(3, 600).unwrap();

    let mut c3 = CurrencyCollection::with_grams(20);
    c3.set_other(2, 200).unwrap();

    assert!(c1 != c2);
    assert!(c2 != c3);
    assert!(c2 != c1);
}

/*
pub enum AccountStatus{
    AccStateUninit,
    AccStateFrozen,
    AccStateActive,
    AccStateNonexist,
}
*/

#[test]
fn test_account_status_serialization()
{
    let as_orig = AccountStatus::AccStateUninit;
    write_read_and_assert(as_orig);

    let as_orig = AccountStatus::AccStateFrozen;
    write_read_and_assert(as_orig);

    let as_orig = AccountStatus::AccStateActive;
    write_read_and_assert(as_orig);

    let as_orig = AccountStatus::AccStateNonexist;
    write_read_and_assert(as_orig);
}

fn get_real_ton_state(filename: &str) -> (ShardStateUnsplit, Cell) {
    let root = BocReader::new().read(&mut File::open(filename).expect("Error open boc file"))
        .expect("Error deserializing boc file")
        .withdraw_single_root().expect("Error deserializing boc - expact one root");
    let state = ShardStateUnsplit::construct_from_cell(root.clone())
        .expect("error deserializing state");

    (state, root)
}

#[test]
fn test_real_account_serde() {
    let state_files = [
        "src/tests/data/7992DD77CEB677577A7D5A8B6F388CDA76B4D0DDE16FF5004C87215E6ADF84DD.boc",
    ];

    for state_file in state_files {
        println!("state file: {}", state_file);

        let (state, _) = get_real_ton_state(state_file);

        state
            .read_accounts()
            .unwrap()
            .iterate_objects(|sa| {


                let acc_cell = sa.account_cell();
                let acc = sa.read_account().unwrap();

                let cell = acc.serialize().unwrap();
                let acc2 = Account::construct_from_cell(cell.clone()).unwrap();

                println!("orig:\n{:#.1}\n\n", acc_cell);
                println!("our:\n{:#.1}\n\n", cell);

                assert_eq!(acc, acc2);

                assert_eq!(acc_cell.repr_hash(), cell.repr_hash());

                Ok(true)
            })
            .unwrap();
    }
}

#[test]
fn test_account_modify_state() {
    let mut stinit = StateInit::default();
    let code = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    stinit.set_code(code.into_cell());
    let data = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    stinit.set_data(data.into_cell());

    let mut fakestinit = StateInit::default();
    let code = SliceData::new(vec![0b00111111, 0b11111111]);
    fakestinit.set_code(code.into_cell());
    let data = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111]);
    fakestinit.set_data(data.into_cell());

    let hash = stinit.hash().unwrap();

    let now = 1600000000;
    let addr = MsgAddressInt::with_standart(None, 0, AccountId::from(hash)).unwrap();
    let mut acc = Account::uninit(addr, 100, now, CurrencyCollection::with_grams(10000000));
    assert_eq!(acc.state_init(), None);
    assert_eq!(acc.status(), AccountStatus::AccStateUninit);

    acc.try_activate_by_init_code_hash(&fakestinit, false).expect_err("should not be activated with wrong StateInit");
    assert_eq!(acc.state_init(), None);
    assert_eq!(acc.status(), AccountStatus::AccStateUninit);

    acc.try_activate_by_init_code_hash(&stinit, false).unwrap();
    assert_eq!(acc.state_init(), Some(&stinit));
    assert_eq!(acc.status(), AccountStatus::AccStateActive);

    acc.try_freeze().unwrap();
    assert_eq!(acc.state_init(), None);
    assert_eq!(acc.status(), AccountStatus::AccStateFrozen);

    acc.try_activate_by_init_code_hash(&fakestinit, false).expect_err("should not be unfreezed with wrong StateInit");
    assert_eq!(acc.state_init(), None);
    assert_eq!(acc.status(), AccountStatus::AccStateFrozen);

    acc.try_activate_by_init_code_hash(&stinit, false).unwrap();
    assert_eq!(acc.state_init(), Some(&stinit));
    assert_eq!(acc.status(), AccountStatus::AccStateActive);
}

#[test]
fn test_account_from_message() {
    let src = MsgAddressInt::with_standart(None, 0, [0x11; 32].into()).unwrap();
    let dst = MsgAddressInt::with_standart(None, 0, [0x22; 32].into()).unwrap();
    let ext = MsgAddressExt::with_extern([0x99; 32].into()).unwrap();

    // external inbound message
    let hdr = crate::ExternalInboundMessageHeader::new(ext.clone(), dst.clone());
    let msg = Message::with_ext_in_header(hdr);
    assert!(Account::from_message_by_init_code_hash(&msg, false).is_none(), "account mustn't be constructed using external message");

    // external outbound message
    let hdr = crate::ExtOutMessageHeader::with_addresses(src.clone(), ext);
    let msg = Message::with_ext_out_header(hdr);
    assert!(Account::from_message_by_init_code_hash(&msg, false).is_none(), "account mustn't be constructed using external message");

    // message without StateInit and with bounce
    let value = CurrencyCollection::with_grams(100);
    let hdr = crate::InternalMessageHeader::with_addresses_and_bounce(src.clone(), dst.clone(), value, true);
    let msg = Message::with_int_header(hdr);
    assert!(Account::from_message_by_init_code_hash(&msg, false).is_none(), "account mustn't be constructed without StateInit and with bounce");

    // message without code
    let value = CurrencyCollection::with_grams(100);
    let hdr = crate::InternalMessageHeader::with_addresses_and_bounce(src.clone(), dst.clone(), value, true);
    let mut msg = Message::with_int_header(hdr);
    let init = StateInit::default();
    msg.set_state_init(init);
    assert!(Account::from_message_by_init_code_hash(&msg, false).is_none(), "account mustn't be constructed without code");

    // message without balance
    let hdr = crate::InternalMessageHeader::with_addresses_and_bounce(src.clone(), dst.clone(), Default::default(), true);
    let mut msg = Message::with_int_header(hdr);
    let mut init = StateInit::default();
    init.set_code(SliceData::new(vec![0x71, 0x80]).into_cell());
    msg.set_state_init(init);
    assert!(Account::from_message_by_init_code_hash(&msg, false).is_none(), "account mustn't be constructed without balance");

    // message without StateInit and without bounce
    let value = CurrencyCollection::with_grams(100);
    let hdr = crate::InternalMessageHeader::with_addresses_and_bounce(src.clone(), dst.clone(), value, false);
    let msg = Message::with_int_header(hdr);
    assert!(Account::from_message_by_init_code_hash(&msg, false).is_some(), "account must be constructed without StateInit and without bounce");

    // message with code and without bounce
    let value = CurrencyCollection::with_grams(100);
    let hdr = crate::InternalMessageHeader::with_addresses_and_bounce(src.clone(), dst.clone(), value, false);
    let mut msg = Message::with_int_header(hdr);
    let mut init = StateInit::default();
    init.set_code(BuilderData::with_bitstring(vec![0x71, 0x80]).unwrap().into_cell().unwrap());
    msg.set_state_init(init);
    assert!(Account::from_message_by_init_code_hash(&msg, false).is_some(), "account must be constructed with code and without bounce");

    // message with code and with bounce
    let value = CurrencyCollection::with_grams(100);
    let hdr = crate::InternalMessageHeader::with_addresses_and_bounce(src, dst, value, true);
    let mut msg = Message::with_int_header(hdr);
    let mut init = StateInit::default();
    init.set_code(BuilderData::with_bitstring(vec![0x71, 0x80]).unwrap().into_cell().unwrap());
    msg.set_state_init(init);
    assert!(Account::from_message_by_init_code_hash(&msg, false).is_some(), "account must be constructed with code and with bounce");
}

#[test]
fn test_generate_account_and_update() {
    let mut account = generate_test_account_by_init_code_hash(false);
    account.set_code(Cell::default()); // set code does not update storage stat
    let cell = account.serialize().unwrap(); // serialization doesn't update storage stat
    let account2 = Account::construct_from_cell(cell).unwrap();
    assert_eq!(account, account2);
    account.update_storage_stat().unwrap();
    assert_ne!(account, account2);
}

#[test]
fn test_account_formats() {
    // init_code_hash - yes
    let account1 = generate_test_account_by_init_code_hash(true);
    let cell = account1.serialize().unwrap();
    let account2 = Account::construct_from_cell(cell).unwrap();
    pretty_assertions::assert_eq!(account1, account2);

    let mut builder = BuilderData::default();
    account1.write_original_format(&mut builder).unwrap();
    let cell = builder.into_cell().unwrap();
    let account2 = Account::construct_from_cell(cell).unwrap();
    assert_ne!(account1, account2, "we must loose additional information in old format");

    assert!(account1.init_code_hash().is_some());
    assert!(account2.init_code_hash().is_none());

    // init_code_hash - no
    let account1 = generate_test_account_by_init_code_hash(false);
    let mut builder = BuilderData::default();
    account1.write_original_format(&mut builder).unwrap();
    let cell = builder.into_cell().unwrap();
    let account2 = Account::construct_from_cell(cell).unwrap();
    assert_eq!(account1, account2);

    assert!(account1.init_code_hash().is_none());
    assert!(account2.init_code_hash().is_none());
}
