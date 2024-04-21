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

use super::*;

#[test]
fn test_out_action_create() {
    let msg = Message::default();
    let action_send = OutAction::new_send(0, msg.clone());
    assert_eq!(action_send, OutAction::SendMsg{mode: 0, out_msg: msg});
    let new_code = Cell::default();
    let action_set = OutAction::new_set(new_code.clone());
    assert_eq!(action_set, OutAction::SetCode{ new_code });
}


fn test_action_serde_equality(action: OutAction) {
    let action_cell = action.serialize().unwrap();
    let deser_action = OutAction::construct_from_cell(action_cell).unwrap();
    assert_eq!(action, deser_action);
}

#[test]
fn test_sendmsg_action_serde() {
    test_action_serde_equality(OutAction::new_send(SENDMSG_ORDINARY, Message::default()));
    test_action_serde_equality(OutAction::new_send(SENDMSG_PAY_FEE_SEPARATELY, Message::default()));
    test_action_serde_equality(OutAction::new_send(SENDMSG_ALL_BALANCE, Message::default()));
}

#[test]
fn test_setcode_action_serde() {
    let code = Cell::default();
    test_action_serde_equality(OutAction::new_set(code));
}

#[test]
fn test_reserve_action_serde() {
    test_action_serde_equality(OutAction::new_reserve(RESERVE_EXACTLY, CurrencyCollection::with_grams(12345)));
    test_action_serde_equality(OutAction::new_reserve(RESERVE_EXACTLY | RESERVE_IGNORE_ERROR, CurrencyCollection::with_grams(54321)));
}

#[test]
fn test_copyleft_action_serde() {
    let acc_id = AccountId::from([0x11; 32]);
    test_action_serde_equality(OutAction::new_copyleft(5, acc_id));

    let acc_id = AccountId::from([0x22; 32]);
    test_action_serde_equality(OutAction::new_copyleft(0, acc_id));
}

fn get_out_actions() -> OutActions {
    let code = SliceData::new(vec![0x71, 0x80]).into_cell();
    let msg = Message::default();
    let mut oa = OutActions::new();
    oa.push_back(OutAction::new_send(SENDMSG_ORDINARY, msg.clone()));
    oa.push_back(OutAction::new_send(SENDMSG_ALL_BALANCE, msg.clone()));
    oa.push_back(OutAction::new_send(SENDMSG_IGNORE_ERROR, msg));
    oa.push_back(OutAction::new_set(Cell::default()));
    oa.push_back(OutAction::new_set(Cell::default()));
    oa.push_back(OutAction::new_set(Cell::default()));
    oa.push_back(OutAction::new_reserve(RESERVE_EXACTLY, CurrencyCollection::with_grams(12345678)));
    oa.push_back(OutAction::new_reserve(RESERVE_ALL_BUT, CurrencyCollection::with_grams(87654321)));
    oa.push_back(OutAction::new_change_library(CHANGE_LIB_REMOVE, None, Some(code.repr_hash())));
    oa.push_back(OutAction::new_change_library(SET_LIB_CODE_REMOVE, Some(code), None));
    let acc_id = AccountId::from([0x11; 32]);
    oa.push_back(OutAction::new_copyleft(0, acc_id));
    oa
}

#[test]
fn test_outactions() {
    let oa = get_out_actions();
    assert_eq!(oa.len(), 11);

    for a in oa.iter() {
        println!("action {:?}", a);
    }
}

#[test]
fn test_outactions_serialization() {    
    let oa = get_out_actions();    
    let b = oa.serialize().unwrap();
    let mut s = SliceData::load_cell(b).unwrap();

    println!("action send slice: {}", s);

    let mut oa_restored = OutActions::new();
    oa_restored.read_from(&mut s).unwrap();
    
    for a in oa_restored.iter() {
        println!("action {:?}", a);
    }
    assert_eq!(oa, oa_restored);
}

// TODO: move to anythere
// #[test]
// fn test_tvm_serialize_currency_collection() {
//     let grams = 1u64<<63;
//     let grams1 = int!(grams).as_grams().unwrap();
//     let grams1 = serialize_currency_collection(grams1, None).unwrap();
//     let grams1: CurrencyCollection = CurrencyCollection::construct_from(&mut grams1.into()).unwrap();
//     let grams2 = CurrencyCollection::with_grams(grams);
//     assert_eq!(grams1, grams2);

//     assert_eq!(int!(1u128<<120).as_grams().expect_err("Expect range check error").code,
//         ExceptionCode::RangeCheckError);
// }
