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

use crate::{ed25519_generate_private_key, read_single_root_boc, write_read_and_assert, Ed25519KeyOption, SigPubKey, ED25519_SIGNATURE_LENGTH };

use super::*;

#[test]
fn test_serialize_many_times(){
    let mut msg = Message::with_int_header(InternalMessageHeader::default());

    let mut stinit = StateInit::default();
    stinit.set_split_depth(Number5::new(23).unwrap());
    stinit.set_special(TickTock::with_values(false, true));
    let code = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    stinit.set_code(code.into_cell());
    let data = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    stinit.set_data(data.into_cell());
    let library = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    stinit.set_library_code(library.into_cell(), true).unwrap();
    
    let body = SliceData::new(vec![0x3F, 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xF4]);

    msg.init = Some(stinit);
    msg.set_body(body);

    let b1 = msg.write_to_new_cell().unwrap();
    let b2 = msg.write_to_new_cell().unwrap();
    let b3 = msg.write_to_new_cell().unwrap();

    assert_eq!(b1, b2);
    assert_eq!(b1, b3);
}

#[test]
fn test_serialize_simple_messages(){
    let msg = Message::with_int_header(InternalMessageHeader::default());
    write_read_and_assert(msg);

    let msg = Message::with_ext_in_header(ExternalInboundMessageHeader::default());
    write_read_and_assert(msg);

    let msg = Message::with_ext_out_header(ExtOutMessageHeader::default());
    write_read_and_assert(msg);
}

#[test]
fn test_serialize_msg_with_state_init() {

    let mut msg = Message::with_int_header(InternalMessageHeader::default());

    let mut stinit = StateInit::default();
    stinit.set_split_depth(Number5::new(23).unwrap());
    stinit.set_special(TickTock::with_values(false, true));
    let code = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    stinit.set_code(code.into_cell());
    let data = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    stinit.set_data(data.into_cell());
    let library = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    stinit.set_library_code(library.into_cell(), false).unwrap();
    
    msg.init = Some(stinit);
    write_read_and_assert(msg);
}

#[test]
fn test_save_external_serialization_order() {

    let mut msg = Message::with_int_header(InternalMessageHeader::default());

    let body = SliceData::new(vec![0x55; 64]);
    msg.set_body(body);

    msg.set_serialization_params(Some(true), Some(false));
    let b = msg.serialize().unwrap();

    let m1 = Message::construct_from_cell(b).unwrap();

    println!("{:?}", m1.serialization_params());

    assert_eq!(m1.serialization_params(), (Some(true), Some(false)));
    assert_eq!(msg, m1);
}

#[test]
fn test_serialize_msg_with_state_init_code_and_small_body() {

    let mut msg = Message::with_int_header(InternalMessageHeader::default());

    let mut stinit = StateInit::default();
    let code = SliceData::new(vec![0x3F, 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xF4]);
    stinit.set_code(code.into_cell());
    let body = SliceData::new(vec![0x55; 64]);

    msg.init = Some(stinit);
    msg.set_body(body);

    write_read_and_assert(msg);
}


#[test]
fn test_serialize_msg_with_state_init_and_body() {

    let mut msg = Message::with_int_header(InternalMessageHeader::default());

    let mut stinit = StateInit::default();
    stinit.set_split_depth(Number5::new(23).unwrap());
    stinit.set_special(TickTock::with_values(false, true));
    let code = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    stinit.set_code(code.into_cell());
    let data = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    stinit.set_data(data.into_cell());
    let library = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    stinit.set_library_code(library.into_cell(), true).unwrap();
    
    let body = SliceData::new(vec![0x3F, 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xF4]);

    msg.init = Some(stinit);
    msg.set_body(body);

    write_read_and_assert(msg);
}

#[test]
fn test_serialize_msg_with_state_init_and_big_body() {

    let mut msg = Message::with_int_header(InternalMessageHeader::default());

    let mut stinit = StateInit::default();
    stinit.set_split_depth(Number5::new(23).unwrap());
    stinit.set_special(TickTock::with_values(false, true));
    let code = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    stinit.set_code(code.into_cell());
    let data = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    stinit.set_data(data.into_cell());
    let library = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    stinit.set_library_code(library.into_cell(), true).unwrap();
    
    let body = SliceData::new(
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
                 0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0x80]);

    msg.set_state_init(stinit);
    msg.set_body(body);

    write_read_and_assert(msg);
}

#[test]
fn test_serialize_msg_with_state_init_with_refs_and_big_body_with_refs() {

    let mut msg = Message::with_int_header(InternalMessageHeader::default());

    let mut stinit = StateInit::default();
    stinit.set_split_depth(Number5::new(23).unwrap());
    stinit.set_special(TickTock::with_values(false, true));
    let mut code = SliceData::new(vec![0x3F, 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xF4]);
    stinit.set_code(code.clone().into_cell());
    let mut code1 = SliceData::new(vec![0xad, 0xc9, 0xba, 0xfc, 0x56, 0x94, 0x11, 0x56, 0x58, 0xfa, 0x2b, 0xdf, 0xe4, 0x65, 0x15, 0x1a, 
                                    0x32, 0x03, 0x69, 0x4a, 0xff, 0xcd, 0x00, 0x8f, 0x36, 0x8b, 0xd2, 0xcc, 0x8c, 0xc8, 0x10, 0xfb, 
                                    0x6b, 0x5b, 0x51]);
    let mut code2 = SliceData::new(vec![0xad, 0xc9, 0xba, 0xfc, 0x56, 0x94, 0x11, 0x56, 0x58, 0xfa, 0x2b, 0xdf, 0xe4, 0x65, 0x15, 0x1a, 
                                    0x32, 0x03, 0x69, 0x4a, 0xff, 0xcd, 0x00, 0x8f, 0x36, 0x8b, 0xd2, 0xcc, 0x8c, 0xc8, 0x10, 0xfb, 
                                    0x6b, 0x5b, 0x51]);
    let code3 = SliceData::new(vec![0xad, 0xc9, 0xba, 0xfc, 0x56, 0x94, 0x11, 0x56, 0x58, 0xfa, 0x2b, 0xdf, 0xe4, 0x65, 0x15, 0x1a, 
                                    0x32, 0x03, 0x69, 0x4a, 0xff, 0xcd, 0x00, 0x8f, 0x36, 0x8b, 0xd2, 0xcc, 0x8c, 0xc8, 0x10, 0xfb, 
                                    0x6b, 0x5b, 0x51]);
    code2.append_reference(code3);
    code1.append_reference(code2);
    code.append_reference(code1);

    stinit.set_code(code.clone().into_cell());

    let data = SliceData::new(vec![0x3F, 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xF4]);
    stinit.set_data(data.into_cell());
    let library = SliceData::new(vec![0x3F, 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xF4]);
    stinit.set_library_code(library.into_cell(), true).unwrap();
    
    let mut body = BuilderData::with_bitstring(
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
                 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0x80]).unwrap();
    let mut body1 = BuilderData::with_bitstring(
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
                 0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0x80]).unwrap();

    let body2 = BuilderData::with_bitstring(
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
                 0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0x80]).unwrap();

    body1.checked_append_reference(body2.into_cell().unwrap()).unwrap();
    body.checked_append_reference(body1.into_cell().unwrap()).unwrap();

    msg.set_state_init(stinit);
    msg.set_body(SliceData::load_builder(body).unwrap());

    write_read_and_assert(msg);
}

#[test]
fn test_check_message_output() {
    let mut msg = Message::with_int_header(InternalMessageHeader::with_addresses_and_bounce(
        MsgAddressInt::with_variant(None, -1, SliceData::new(vec![12, 13, 17])).unwrap(),
        MsgAddressInt::with_standart(Some(AnycastInfo::with_rewrite_pfx(SliceData::new(vec![0xC4])).unwrap()), 5, [55; 32].into()).unwrap(),
        CurrencyCollection::with_grams(79),
        false
    ));
    let mut stinit = StateInit::default();
    stinit.set_split_depth(Number5::new(23).unwrap());
    stinit.set_special(TickTock::with_values(false, true));
    let code = SliceData::new(vec![0x3F, 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xF4]);
    stinit.set_code(code.into_cell());
    let library = SliceData::new(vec![0x3F, 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xF4]);
    stinit.set_library_code(library.into_cell(), false).unwrap();
    msg.init = Some(stinit);
    msg.set_body(SliceData::new(vec![0x55, 0x55, 0x80]));
    pretty_assertions::assert_eq!(format!("{}", msg), "Message {header: Internal {src: -1:0c0d11_, \
        dst: c4_:5:3737373737373737373737373737373737373737373737373737373737373737}, \
        init: StateInit { split_depth: Some(Number5(23)), special: Some(TickTock { tick: false, tock: true }), \
        code: Some(7a0b957a15e93cca3ce96ccb4aecf275a3718a263c8aeca2ab14fe6e1e62172c), data: None, \
        library: StateInitLib(HashmapE { bit_len: 256, data: Some(c39760fbba54774b6c7fa76bfd46d6fb89d1fe0b19570bef3c4d08decc8b4566) }, 0) }, \
        body: 5555}");
}

#[test]
fn test_check_json_address() {
    let addresses = [
        " ", "11:", "q:22", ":-33", ":44 ", ":0:55", " :66", " :77 ",
        "2147483648:33", "-2147483649:44", ":0:66", "66 ", " 66", " 66 ", "12345678:0:66",
        "0:555555555555555555555555555555555555555555555555555555555555555",
        "-1:555555555555555555555555555555555555555555555555555555555555555555"];

    addresses.iter().for_each(|addr| {
        let err = MsgAddressInt::from_str(addr).err();
        println!("{:?}", err);
        assert!(err.is_some());
    });

    let anycast = AnycastInfo::with_rewrite_pfx(SliceData::new(vec![0x77, 0x80])).unwrap();
    let addresses_int = [
        ("255:55_", MsgAddressInt::with_variant(None, 255, SliceData::new(vec![0x55])).unwrap()),
        ("77:-129:55_",
            MsgAddressInt::with_variant(Some(anycast.clone()), -129, SliceData::new(vec![0x55])).unwrap()),
        ("0:5555555555555555555555555555555555555555555555555555555555555555", 
            MsgAddressInt::with_standart(None, 0, AccountId::from([0x55; 32])).unwrap()),
        ("1:5555555555555555555555555555555555555555555555555555555555555555", 
            MsgAddressInt::with_standart(None, 1, AccountId::from([0x55; 32])).unwrap()),
        ("77:1:5555555555555555555555555555555555555555555555555555555555555555", 
            MsgAddressInt::with_standart(Some(anycast), 1, AccountId::from([0x55; 32])).unwrap()),
        ("128:5555555555555555555555555555555555555555555555555555555555555555", 
            MsgAddressInt::with_variant(None, 128, AccountId::from([0x55; 32])).unwrap()),
        ("1:55555555555555555555555555555555555555555555555555555555555555558_", 
            MsgAddressInt::with_variant(None, 1, AccountId::from([0x55; 32])).unwrap()),
        ("0:55555555555555555555555555555555555555555555555555555555555555558_",
            MsgAddressInt::with_variant(None, 0, AccountId::from([0x55; 32])).unwrap()),

        ("1111:8888", MsgAddressInt::with_variant(None, 1111, SliceData::new(vec![0x88, 0x88, 0x80])).unwrap()),
        ("1111:777",
            MsgAddressInt::with_variant(None, 1111, SliceData::new(vec![0x77, 0x78])).unwrap()),
        ("1111:abc_",
            MsgAddressInt::with_variant(None, 1111, SliceData::new(vec![0xAB, 0xC0])).unwrap()),
    ];
    addresses_int.iter().for_each(|(addr, check)| {
        let real = MsgAddressInt::from_str(addr).unwrap();
        println!("{}", real);
        assert_eq!(&real, check);
        assert_eq!(&format!("{}", real), addr);
    });

    let addresses_ext = [
        ("", MsgAddressExt::AddrNone),
        (":55_", MsgAddressExt::with_extern(SliceData::new(vec![0x55])).unwrap()),
        (":5555555555555555555555555555555555555555555555555555555555555555",
            MsgAddressExt::with_extern(AccountId::from([0x55; 32])).unwrap()),
    ];
    addresses_ext.iter().for_each(|(addr, check)| {
        let real = MsgAddressExt::from_str(addr).unwrap();
        println!("{}", real);
        assert_eq!(&real, check);
        assert_eq!(&format!("{}", real), addr);
    });
}

#[test]
fn test_message_addr_external_err_1() {
    let err = MsgAddressExt::with_extern(SliceData::from_raw(vec!(1; 65), 513)).err();
    assert!(err.is_some());
}

#[test]
fn test_message_addr_external_err_2() {
    let err = MsgAddressExt::from_str(":2323232323232323232323232323232323232323232323232323232323232323232323232323232323232323232323232323232323232323232323232323232333").err();
    assert!(err.is_some());
}

#[test]
fn test_msg_address_int_or_none() {
    let addr1 = MsgAddressIntOrNone::default();
    let addr2str = "-1:5555555555555555555555555555555555555555555555555555555555555555";
    let addr2 = MsgAddressIntOrNone::Some(MsgAddressInt::from_str(addr2str).unwrap());
    let addr3 = MsgAddressExt::with_extern(SliceData::new(vec![0x55])).unwrap();
    let mut b = BuilderData::new();
    addr1.write_to(&mut b).unwrap();
    addr2.write_to(&mut b).unwrap();
    addr3.write_to(&mut b).unwrap();
    let mut s = SliceData::load_builder(b).unwrap();
    println!("{:x}", s);
    let mut addr1_ = MsgAddressIntOrNone::default();
    addr1_.read_from(&mut s).unwrap();
    assert_eq!(addr1, addr1_);
    let mut addr2_ = MsgAddressIntOrNone::default();
    addr2_.read_from(&mut s).unwrap();
    assert_eq!(addr2, addr2_);
    let mut addr3_ = MsgAddressIntOrNone::default();
    let err = addr3_.read_from(&mut s).err();
    assert!(err.is_some());
}

#[test]
fn test_msg_address_int_invalid() {
    let addr1 = MsgAddressIntOrNone::default();
    let b = addr1.write_to_new_cell().unwrap();
    let mut s = SliceData::load_builder(b).unwrap();
    println!("{:x}", s);
    MsgAddressInt::construct_from(&mut s)
        .expect_err("MsgAddressInt should not be deserialized from None");
}

fn create_rnd_external_message() -> (UInt256, CommonMessage) {

    let mut data: Vec<u8> = (0..32).map(|_| { rand::random::<u8>() }).collect::<Vec<u8>>();
    data.push(0x80);
    let src = MsgAddressExt::with_extern(SliceData::new(data)).unwrap();
    let dst = MsgAddressInt::with_standart(None, -1, AccountId::from(UInt256::rand())).unwrap();
    let mut hdr = ExternalInboundMessageHeader::new(src, dst);
    hdr.import_fee = 10.into();
    let msg = Message::with_ext_in_header(hdr);
    (msg.hash().unwrap(), CommonMessage::Std(msg))
}

#[test]
fn test_msg_pack() -> Result<()> {
    let mut msg_pack = MsgPack::default();
    msg_pack.info.seqno = 123;
    msg_pack.info.shard = ShardIdent::with_tagged_prefix(0, 0x4000_0000_0000_0000)?;
    msg_pack.info.round = 182943412343;
    msg_pack.info.gen_utime_ms = 1234567890;
    msg_pack.info.prev = UInt256::rand();
    msg_pack.info.prev_2 = Some(UInt256::rand());
    msg_pack.info.mc_block = 1234;
    for _ in 0..128 {
        let (hash, msg) = create_rnd_external_message();
        msg_pack.messages.set(&hash, &msg)?;
    }
    write_read_and_assert(msg_pack);

    let mut msg_pack = MsgPack::default();
    msg_pack.info.seqno = 123;
    msg_pack.info.shard = ShardIdent::with_tagged_prefix(0, 0x4000_0000_0000_0000)?;
    msg_pack.info.round = 18294345784573;
    msg_pack.info.gen_utime_ms = 1234567890;
    msg_pack.info.prev = UInt256::rand();
    msg_pack.info.mc_block = 1234;
    let msg_pack = write_read_and_assert(msg_pack);
    let msg_pack_boc = msg_pack.write_to_bytes()?;

    let mut signatures = MsgPackSignatures::default();
    signatures.set(&1_u16, &CryptoSignature::from_bytes(&[1; ED25519_SIGNATURE_LENGTH])?)?;
    signatures.set(&5_u16, &CryptoSignature::from_bytes(&[5; ED25519_SIGNATURE_LENGTH])?)?;
    signatures.set(&25_u16, &CryptoSignature::from_bytes(&[25; ED25519_SIGNATURE_LENGTH])?)?;

    let msg_pack_root = read_single_root_boc(msg_pack_boc)?;

    let proof = MsgPackProof::new(&msg_pack_root, signatures)?;

    let proof = write_read_and_assert(proof);

    assert_eq!(proof.virtualize()?.info, msg_pack.info);

    Ok(())
}

#[test]
fn test_msg_pack_proof() -> Result<()> {
    let mut msg_pack = MsgPack::default();
    msg_pack.info.seqno = 123;
    msg_pack.info.shard = ShardIdent::with_tagged_prefix(0, 0x4000_0000_0000_0000)?;
    msg_pack.info.round = 182943412343;
    msg_pack.info.gen_utime_ms = 1234567890;
    msg_pack.info.prev = UInt256::rand();
    msg_pack.info.prev_2 = Some(UInt256::rand());
    msg_pack.info.mc_block = 1234;
    for _ in 0..128 {
        let (hash, msg) = create_rnd_external_message();
        msg_pack.messages.set(&hash, &msg)?;
    }


    let mut validators = vec!();
    let mut secret_keys = vec!();
    for _ in 0..20 {
        let secret_key = ed25519_generate_private_key()?;
        let public_key = Ed25519KeyOption::from_private_key(secret_key.as_bytes()).unwrap();
        secret_keys.push(secret_key);
        let vd = ValidatorDescr {
            public_key: SigPubKey::from_bytes(public_key.pub_key()?)?, 
            ..Default::default()
        };
        validators.push(vd);
    }

    let msg_pack_root = msg_pack.write_to_new_cell()?.into_cell()?;
    let mut signatures = MsgPackSignatures::default();
    for i in [1_u16, 5, 15] {
        signatures.set(
            &i, 
            &CryptoSignature::from_bytes(
                &secret_keys[i as usize].sign(msg_pack_root.repr_hash().as_slice())
            )?
        )?;
    }

    let proof = MsgPackProof::new(&msg_pack_root, signatures)?;

    let proof = write_read_and_assert(proof);

    proof.check(123, &msg_pack_root.repr_hash(), &validators)?;

    Ok(())
}