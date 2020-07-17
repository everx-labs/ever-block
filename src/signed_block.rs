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

use crate::{
	blocks::Block,
	error::BlockError,
	Deserializable, Serializable,
};
use ed25519::signature::{Signature, Signer, Verifier};
use sha2::Digest;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::{Write, Read, Cursor};
use ton_types::{
    BuilderData, Cell, error, fail, Result, SliceData,
    BagOfCells, deserialize_tree_of_cells,
    ExceptionCode, UInt256
};


#[allow(dead_code)]
const SHA256_SIZE: usize = 32;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockSignature(ed25519::Signature);

// ed25519_signature#5
// 	R:uint256
// 	s:uint256
// = CryptoSignature;
impl Serializable for BlockSignature {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_raw(&self.0.to_bytes(), ed25519_dalek::SIGNATURE_LENGTH * 8)?;
        Ok(())
    }
}

impl Deserializable for BlockSignature {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.0 = ed25519::Signature::from_bytes(
            &cell.get_next_bytes(ed25519_dalek::SIGNATURE_LENGTH)?
        )?;
	Ok(())
    }
}

impl Default for BlockSignature {
    fn default() -> Self {
        BlockSignature(
            ed25519::Signature::from_bytes(
                &vec!(0; ed25519_dalek::SIGNATURE_LENGTH)
            ).unwrap()
        )
    }
}

#[derive(Clone, Debug, Default, Eq)]
pub struct SignedBlock {
	block: Block,
	block_repr_hash: UInt256, // Block is absent cell. We have to store block hash instead.
	block_serialize_hash: UInt256,
	combined_hash: UInt256,
	serialized_block: Vec<u8>,
	signatures: HashMap<u64, BlockSignature>
}

impl Ord for SignedBlock {
    fn cmp(&self, other: &SignedBlock) -> Ordering {
        self.block.read_info().unwrap().seq_no().cmp(&other.block.read_info().unwrap().seq_no())
    }
}

impl PartialOrd for SignedBlock {
    fn partial_cmp(&self, other: &SignedBlock) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for SignedBlock {
    fn eq(&self, other: &SignedBlock) -> bool {
        self.block.read_info().unwrap().seq_no() == other.block.read_info().unwrap().seq_no()
    }
}

#[allow(dead_code)]
impl SignedBlock {

        pub fn with_block_and_key(block: Block, key: &ed25519_dalek::Keypair) -> Result<Self> {

		// block serialization 
		let block_root: Cell = block.write_to_new_cell()?.into();
		let bag = BagOfCells::with_root(&block_root);
		let mut serialized_block = Vec::<u8>::new();
		bag.write_to(&mut serialized_block, true)?;

		// hashes calculation
		let serlz_hash = Self::calc_merkle_hash(&serialized_block)?;
		let mut combined_data = Vec::<u8>::with_capacity(SHA256_SIZE * 2);
		combined_data.write(block_root.repr_hash().as_slice()).unwrap();
		combined_data.write(&serlz_hash).unwrap();

		let mut hasher = sha2::Sha256::new();
		hasher.input(combined_data.as_slice());
		let combined_hash = hasher.result().to_vec().into();
		
		let mut result = SignedBlock {
			block: block,
			block_repr_hash: block_root.repr_hash(),
			block_serialize_hash: serlz_hash.into(),
			combined_hash: combined_hash,
			serialized_block: serialized_block,
			signatures: HashMap::<u64, BlockSignature>::new() };

		result.add_signature(key);

		Ok(result)
	}
	
	pub fn block(&self) -> &Block {
		&self.block
	}

	pub fn hash(&self) -> &UInt256 {
		&self.combined_hash
	}

	/// Get representation hash of block
	pub fn block_hash(&self) -> &UInt256 {
		&self.block_repr_hash
	}

	pub fn signatures(&self) -> &HashMap<u64, BlockSignature> {
		&self.signatures
	}
	
	pub fn add_signature(self: &mut Self, key: &ed25519_dalek::Keypair) {
		let signature = key.sign(self.combined_hash.as_slice());
		let key = super::id_from_key(&key.public);
		self.signatures.insert(key, BlockSignature {0: signature});
	}

	pub fn verify_signature(self: &Self, key: &ed25519_dalek::PublicKey) -> Result<bool> {
		let key_id = super::id_from_key(key);
		let signature = match self.signatures.get(&key_id) {
			Some(s) => &s.0,
			_ => fail!(BlockError::NotFound("signature".to_string()))
		};
		Ok(key.verify(self.combined_hash.as_slice(), signature).is_ok())
	}

	pub fn write_to<T: Write>(self: &Self, dest: &mut T) -> Result<()> {
		// Transform signed block into tree of cells
		//
		// signed_block
		// 	block:^Block
		// 	blk_serialize_hash:uint256
		//  signatures:(HashmapE 64 CryptoSignature)
		// = SignedBlock;

		let block_absent_cell = self.block_repr_hash.write_to_new_cell()?;

		let mut cell = self.block_serialize_hash.write_to_new_cell()?;
		cell.append_reference(block_absent_cell.clone());
		cell.append_reference(self.signatures.write_to_new_cell()?);

		// Transfom tree into bag 
		let bag = BagOfCells::with_roots_and_absent(vec![&cell.into()], vec![&block_absent_cell.into()]);

		// Write signed block's bytes and then unsigned block's bytes
		bag.write_to(dest, true)?;
		dest.write(&self.serialized_block)?;

		Ok(())
	}

	pub fn read_from<T>(src: &mut T) -> Result<Self> where T: Read {
		// first - signed block's bytes (with block hash instead block bytes)
		let cell = deserialize_tree_of_cells(src)?;

		let repr_hash_cell = cell.reference(0)?;
		if (repr_hash_cell.bit_length() < SHA256_SIZE * 8) || (cell.bit_length() < SHA256_SIZE * 8) {
		    fail!(ExceptionCode::CellUnderflow)
		}
		let block_repr_hash = repr_hash_cell.data()[..SHA256_SIZE].to_vec();
		let serlz_hash = cell.data()[..SHA256_SIZE].to_vec();
		let mut signatures = HashMap::<u64, BlockSignature>::new();
		signatures.read_from(&mut cell.reference(1)?.into())?;

		// second - block's bytes
		let mut serialized_block = Vec::new();
		src.read_to_end(&mut serialized_block)?;
				
		let mut serialized_block_cur = Cursor::new(serialized_block);
		let cell = deserialize_tree_of_cells(&mut serialized_block_cur)?;
		let mut block = Block::default();
		block.read_from(&mut cell.clone().into())?;

		// check block repr hash
		if &block_repr_hash != cell.repr_hash().as_slice() {
		    fail!(BlockError::WrongHash);
		}

		let mut hasher = sha2::Sha256::new();
		hasher.input(block_repr_hash.as_slice());
		hasher.input(serlz_hash.as_slice());
		let combined_hash = hasher.result().to_vec().into();

		Ok(SignedBlock {
			block: block,
			block_repr_hash: block_repr_hash.into(), 
			block_serialize_hash: serlz_hash.into(),
			combined_hash: combined_hash,
			serialized_block: serialized_block_cur.into_inner(),
			signatures: signatures })
	}

	fn calc_merkle_hash(data: &[u8]) -> Result<Vec<u8>> {
		let l = data.len();
		if l <= 256 {
			let mut hasher = sha2::Sha256::new();
			hasher.input(data);
			Ok(hasher.result().to_vec())
		} else {
			let n = Self::largest_power_of_two_less_than(l);
			let data1_hash = Self::calc_merkle_hash(&data[..n])?;
			let data2_hash = Self::calc_merkle_hash(&data[n..])?;
			let mut data_for_hash = [0 as u8; 8 + SHA256_SIZE * 2];
			data_for_hash[..8].copy_from_slice(&(l as u64).to_be_bytes());
			data_for_hash[8..8 + SHA256_SIZE].copy_from_slice(&data1_hash);
			data_for_hash[8 + SHA256_SIZE..].copy_from_slice(&data2_hash);
			let mut hasher = sha2::Sha256::new();
			hasher.input(&data_for_hash[..]);
			Ok(hasher.result().to_vec())
		}
	}

	fn largest_power_of_two_less_than(l: usize) -> usize {
		let mut n = 1;
		let mut l1 = l;
		
		while l1 != 1 {
			l1 >>= 1;
			n <<= 1;
		}
			
		if n == l {
			n / 2
		} else {
			n
		}		
	}
}
