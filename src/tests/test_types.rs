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

use crate::write_read_and_assert;

use super::*;
use Cell;
use num::{CheckedAdd, CheckedSub};

#[test]
fn test_get_len()
{
    let size = VarUInteger32::get_len(&0u32.into());

    assert_eq!(size, 0);

    let value = BigInt::from_slice(Sign::Plus, &[1, 2, 3, 4, 5, 6, 7, 0xFFFFFFFF]);
    let size = VarUInteger32::get_len(&value);

    assert_eq!(size, 32);

    let size = VarUInteger32::get_len(&BigInt::from_slice(Sign::Plus, &[1, 2, 3, 4, 5, 6, 7]));

    assert_eq!(size, 25);

    let size = VarUInteger32::get_len(&1u32.into());
    
    assert_eq!(size, 1);
}

#[test]
fn test_varuinteger_with_zero(){
    let vui32: VarUInteger32 = VarUInteger32::default();
    let b = vui32.serialize().unwrap();

    let mut s = SliceData::new(vec![0b00000100]);
    assert_eq!(s.cell_opt().unwrap(), &b);

    let mut v2: VarUInteger32 = VarUInteger32::default();
    v2.read_from(&mut s).unwrap();
    assert_eq!(vui32, v2);
}

#[test]
fn test_varuinteger7_from_into(){
    let mut b1: SliceData = SliceData::new(vec![0b00100000, 0b01010000]);

    println!("b1 = {}", b1);

    let mut vui7: VarUInteger7 = VarUInteger7::default();
    vui7.read_from(&mut b1).unwrap(); 
    println!("vui7 = {}", vui7);

    assert_eq!(VarUInteger7::from(2), vui7);

    let mut b2 = SliceData::new(vec![0b00100010, 0b00000100, 0b01000100, 0b00000001]);
    let mut v2 = VarUInteger7::default(); 
    v2.read_from(&mut b2).unwrap();

    let mut v3 = VarUInteger7::default();
    v3.read_from(&mut b2).unwrap();
    v2 += 1;

    assert_eq!(v2, v3);

    let mut s1: BuilderData = BuilderData::new();
    v2.write_to(& mut s1).unwrap();
    println!("s1 = {}", s1);
    println!("v2 = {}", v2);
}

#[test]
fn test_varuinteger7_serialization() {
    VarUInteger7::new(u64::MAX).expect_err("should not be contructable");
    VarUInteger7::new(0x01FF_FFFF_FFFF_FFFFu64).expect_err("should not be contructable");
    let v = VarUInteger7::new(0x00FF_FFFF_FFFF_FFFFu64).unwrap();
    v.serialize().unwrap();

    let mut v = VarUInteger7::default();
    v.read_from(&mut SliceData::new(vec![0b00100000, 0b01000001])).unwrap();
    assert_eq!(VarUInteger7::from(2), v);
    v.read_from(&mut SliceData::new(vec![0b00111111, 0b11100001])).unwrap();
    assert_eq!(VarUInteger7::from(255), v);
    v.read_from(&mut SliceData::new(vec![0b00010000])).unwrap();
    assert_eq!(VarUInteger7::from(0), v);
    v.read_from(&mut SliceData::new(vec![0b01011111, 0b11111111, 0b11100001])).unwrap();
    assert_eq!(VarUInteger7::from(65535), v);
    
    v.read_from(
            &mut SliceData::new(vec![0b11011111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11100001])).unwrap();
    assert_eq!(VarUInteger7::new(256*256*256*256*256*256-1).unwrap(), v);
}


#[test]
fn test_varuinteger32_serialization()
{
    let mut g = VarUInteger32::default();
    g.read_from( 
        &mut SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111001])).unwrap();
    assert_eq!(VarUInteger32::from_two_u128(0, 256*256*256*256*256*256*256-1).unwrap(), g); 

    let g1 = VarUInteger32::from_two_u128(0x00800000000000000000000000000000, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF).unwrap();
    write_read_and_assert(g1);

    g.read_from( 
        &mut SliceData::new(vec![0xFC,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x07,
                                 0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFC])).unwrap();
    assert_eq!(VarUInteger32::from_two_u128(0x00800000000000000000000000000000, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF).unwrap(), g); 
}

#[test]
fn test_var_uinteger_32_addiction()
{
    let mut a = VarUInteger32::from_two_u128(0, 0).unwrap();
    let b = VarUInteger32::from_two_u128(0, 1).unwrap();
    a.add(&b).unwrap();
    assert_eq!(a,b);

    let mut a = VarUInteger32::from_two_u128(123, 567).unwrap();
    let b = VarUInteger32::from_two_u128(876, 432).unwrap();
    a.add(&b).unwrap();
    assert_eq!(a,VarUInteger32::from_two_u128(999, 999).unwrap());

    let mut a = VarUInteger32::from_two_u128(0, 1).unwrap();
    let b = VarUInteger32::from_two_u128(0, 1).unwrap();
    a.sub(&b).unwrap();
    assert_eq!(a,VarUInteger32::from_two_u128(0, 0).unwrap());
}

#[test]
fn test_number5_serialization() {

    let mut v = Number5::default();
    
    v.read_from(&mut SliceData::new(vec![0b00000100])).unwrap();
    assert_eq!(Number5::new(0).unwrap(), v);
    v.read_from(&mut SliceData::new(vec![0b00001100])).unwrap();
    assert_eq!(Number5::new(1).unwrap(), v);
    v.read_from(&mut SliceData::new(vec![0b10000100])).unwrap();
    assert_eq!(Number5::new(16).unwrap(), v);
    v.read_from(&mut SliceData::new(vec![0b11111100])).unwrap();
    assert_eq!(Number5::new(31).unwrap(), v);

    v.read_from(&mut SliceData::new(vec![0b10000100])).unwrap();
    assert_eq!(Number5::new(16).unwrap(), v);

    write_read_and_assert(v);
}

#[test]
fn test_number32_serialization() {

    let mut v = Number32::default();
    
    v.read_from(&mut SliceData::new(vec![0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b10000000])).unwrap();
    assert_eq!(Number32::from(0u32), v);
    v.read_from(&mut SliceData::new(vec![0b00000000, 0b00000000, 0b00000000, 0b00000001, 0b10000000])).unwrap();
    assert_eq!(Number32::from(1u32), v);
    v.read_from(&mut SliceData::new(vec![0b00000000, 0b00000000, 0b00000000, 0b00010000, 0b10000000])).unwrap();
    assert_eq!(Number32::from(16u32), v);
    v.read_from(&mut SliceData::new(vec![0b11111111, 0b00000000, 0b00000000, 0b00000000, 0b10000000])).unwrap();
    assert_eq!(Number32::from(0xFF000000u32), v);
    v.read_from(&mut SliceData::new(vec![0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b10000000])).unwrap();
    assert_eq!(Number32::from(0xFFFFFFFFu32), v);
    
    v.read_from(&mut SliceData::new(vec![0b00000000, 0b00000000, 0b00000000, 0b00011111, 0b10000000])).unwrap();
    assert_eq!(Number32::from(31u32), v);

    write_read_and_assert(v);
}

#[test]
fn test_grams_serialization()
{
    let g = Grams::new(956_956_956_956_000_000_000u128).unwrap();
    let s = g.write_to_new_cell().unwrap();
    assert_eq!(s.data(), hex::decode("933e072122d1d2818000").unwrap());
    assert_eq!(g, Grams::construct_from_cell(s.into_cell().unwrap()).unwrap());

    let mut g = Grams::zero();
    g.read_from(&mut SliceData::new(vec![0b00010000, 0b000101000])).unwrap();
    assert_eq!(Grams::from(2), g);
    g.read_from(&mut SliceData::new(vec![0b00011111, 0b11110001])).unwrap();
    assert_eq!(Grams::from(255), g);
    g.read_from(&mut SliceData::new(vec![0b00001000])).unwrap();
    assert_eq!(Grams::zero(), g);
    g.read_from(&mut SliceData::new(vec![0b00101111, 0b11111111, 0b11110001])).unwrap();
    assert_eq!(Grams::from(65535), g);
    g.read_from( 
            &mut SliceData::new(vec![0b01111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110001])).unwrap();
    assert_eq!(Grams::new(256u128 * 256 * 256 * 256 * 256 * 256 * 256 - 1).unwrap(), g);

    let s = Grams::from(2).write_to_new_cell().unwrap();
    assert_eq!( SliceData::load_builder(s).unwrap(), SliceData::new(vec![0b00010000, 0b00101000]));

    let s = Grams::from(252).write_to_new_cell().unwrap();
    assert_eq!( SliceData::load_builder(s).unwrap(), SliceData::new(vec![0b00011111, 0b11001000]));

    let s = Grams::zero().write_to_new_cell().unwrap();
    assert_eq!( SliceData::load_builder(s).unwrap(), SliceData::new(vec![0b00001000]));

    let s = Grams::from(65534).write_to_new_cell().unwrap();
    assert_eq!( SliceData::load_builder(s).unwrap(), SliceData::new(vec![0b00101111, 0b11111111, 0b11101000]));

    let s = Grams::from(0xFFFFFFFFFFFFFE).write_to_new_cell().unwrap();
    assert_eq!( SliceData::load_builder(s).unwrap(), SliceData::new(vec![0b01111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11101000]));

    for n in 0..1000 {
        write_read_and_assert(Grams::from(n));
    }

    for n in 1000000000..1000001000 {
        write_read_and_assert(Grams::from(n));
    }
    
    for n in 1000000000000000..1000000000001000 {
        write_read_and_assert(Grams::from(n));
    }
}

define_HashmapE!{SimpleMap, 8, u8}

#[test]
fn test_filter() {
    let mut tree_1 = SimpleMap::default();

    tree_1.set(&0b11001100u8, &0).unwrap();
    tree_1.set(&0b11010000u8, &1).unwrap();
    tree_1.set(&0b11010100u8, &0).unwrap();
    tree_1.set(&0b11011000u8, &3).unwrap();
    tree_1.set(&0b11011100u8, &0).unwrap();

    tree_1.filter(|_key : &u8, value : &u8| Ok(*value != 0)).unwrap();

    let mut tree_2 = SimpleMap::default();

    tree_2.set(&0b11010000u8, &1).unwrap();
    tree_2.set(&0b11011000u8, &3).unwrap();

    let correct_dif : Vec<(SliceData, Option<u8>, Option<u8>)> = Vec::new();

    let mut diff_vec : Vec<(SliceData, Option<u8>, Option<u8>)> = Vec::new();

    tree_1.scan_diff(&tree_2, |key,value1, value2| { 
        diff_vec.push((key, value1, value2)); 
        Ok(true)
    }).unwrap();
    assert!(correct_dif == diff_vec);

}

#[test]
fn test_grams_parsing() {
    let g = Grams::from_str("0xffffffffffffffffffffffffffffffff").unwrap();
    assert_eq!(g.0, 340282366920938463463374607431768211455u128);
    Grams::from_str("0x100000000000000000000000000000000").unwrap_err();
    Grams::from_str("340282366920938463463374607431768211455").unwrap();
    assert_eq!(g.0, 340282366920938463463374607431768211455u128);
    Grams::from_str("340282366920938463463374607431768211456").unwrap_err();
}

#[test]
fn test_checked_operations() {
    let mut v = VarUInteger7::new(0x00FF_FFFF_FFFF_FFFFu64).unwrap();
    assert!(!v.add_checked(1));
    assert!(v.sub_checked(1));
    assert!(v.add_checked(1));

    let mut v = VarUInteger3::new(0x00FF_FFFFu32).unwrap();
    assert!(!v.add_checked(1));
    assert!(v.sub_checked(1));
    assert!(v.add_checked(1));

    let mut v = Grams::new(0x00FF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFFu128).unwrap();
    assert!(!v.add_checked(1));
    assert!(v.sub_checked(1));
    assert!(v.add_checked(1));
}

#[test]
fn test_math_traits() {
    let mut a = Grams::from(10);
    a *= 10;
    a *= Grams::from(10);
    a <<= 3;
    let mut b = ((a >> 2) << 1) + 5;
    b += 1;
    b -= 3;
    assert_eq!(b.as_u128(), (1000 << 3 >> 2 << 1) + 5 + 1 - 3);

    let mut a = Grams::new((1u128 << 120) - 1).unwrap();
    assert!(!a.add_checked(1), "should not fit in Grams");
    assert!(a.checked_add(&Grams::one()).is_none(), "should not fit in Grams");

    let mut a = Grams::zero();
    assert!(!a.sub_checked(1), "should not sub with negative");
    assert!(a.checked_sub(&Grams::one()).is_none(), "should not sub with negative");
}
