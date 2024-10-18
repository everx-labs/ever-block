# Release Notes

All notable changes to this project will be documented in this file.

## Version 1.11.20

- Added method `BocReader::read_root` for fast reading root cell data from BOC
- Added method `Message::read_header_fast` for fast reading message header from BOC

## Version 1.11.19

- Deleted deprecated method CellImpl::cell_data

## Version 1.11.18

- Fixed bug in the find_validators function when the number of validators in the new set was increased.

## Version 1.11.17

- Added methods McBlockExtra::validators_stat, McBlockExtra::validators_stat_mut and McBlockExtra::set_validators_stat

## Version 1.11.16

- Deleted unnecessary debug print

## Version 1.11.15

- Optimized Merkle proof creation ( less cell loading)
- Cell refactoring

## Version 1.11.14

- Fixed ser/deserialization of `AccountBlock` with mesh transactions

## Version 1.11.13

- SMFT configuration parameters

## Version 1.11.12

- Fixed partial change of mempool nodes. There was possible to get collator and mempool node with same number.

## Version 1.11.11

- Add round to MsgPackProcessingInfo

## Version 1.11.10

- Add construct_from_bitstring adapter

## Version 1.11.9

- SMFT config parameters

## Version 1.11.8

- Add filtering interfaces to hashmaps for encapsulation of serde_opts

## Version 1.11.7

- Changed serialization of ShardStateUnsplit: now it is possible to serialize/deserialize pack info without SERDE_OPTS_COMMON_MESSAGE enabled.

## Version 1.11.6

- To function find_validators added the ability to change mempool partially. Added related parameter to FastFinalityConfig

## Version 1.11.5

- Add adnl_addr interface for ValidatorDescr

## Version 1.11.4

- Addons for `MsgPack` and related data types

## Version 1.11.3

- Fix for external messages dictionary

## Version 1.11.2

- Added `MsgPack` and related data types

## Version 1.11.1

- Added data types for new fast finality roles mechanism

## Version 1.11.0

- Use modern crates anyhow and thiserror instead of failure

## Version 1.10.4

- Fix build for WASM target

## Version 1.10.3

- New CommonMessage structure as a container of old `Message` and new types of messages.
- New serialization option for ser/de operations with other structs.

## Version 1.10.2

- Enhanced HashMap interface
- Some interfaces were refactored due to merging repos

## Version 1.10.1

- Added GlobalCapabilities::CapDuePaymentFix which disables due payments on credit phase and adds payed dues to storage fee in TVM

## Version 1.10.0

- `ton-types` repository was merged with this repository
- crate was renamed from `ton_block` to `ever_block`

## Version 1.9.143

- Added GlobalCapabilities::CapTvmV20 which enables BLS instructions in TVM

## Version 1.9.140

- Capability to avoid deletion of frozen accounts

## Version 1.9.139

- HashmapAugE returns depth of tree on insert new item

## Version 1.9.125

- Added MerkleUpdate::apply_for_with_cells_factory

## Version 1.9.121

- Removed extra crates base64, ed25519, sha2
- Minor refactoring

## Version 1.9.120

- Add new capability for disabling split of out queues during shard split

## Version 1.9.119

- HashMapAug and OutMsgQueue insertion API extended

## Version 1.9.118

- SMFT capability added

## Version 1.9.117

- Fix cells serialization format

## Version 1.9.110

- BLS data structures have been added

## Version 1.9.107

- Add new capability constant for TVM improvements

## Version 1.9.105

- Remove set_level_mask calls (levels are set at finalization now)

## Version 1.9.104

- Fixed `Block::set_gen_utime_ms` method

## Version 1.9.101

- Fixed deserialization of BlockInfo. Added method BlockInfo::read_from_ex()

## Version 1.9.98

- Removed 'fast_finality' feature

## Version 1.9.93

- Use new functions for hashmap manipulation

## Version 1.9.90

- Added new method MerkleUdate::apply_for_with_metrics()

## Version 1.9.89

- Remove compiler warning

## Version 1.9.84

- Fix the build for fast finality

## Version 1.9.77

- Added GlobalCapabilities::CapOptimisticConsensus

## Version 1.9.76

-  Deleted unused field CollatorRange.unexpected_finish

## Version 1.9.75

- Store `original_shard` in `ProcessedUpto` for optimistic consensus

## Version 1.9.74

- Added CollatorRange:: updated_at

## Version 1.9.73

- Added ref_shard_blocks to ShardStateUnsplit
- Added new parameter 'collators' to ShardHashes::add_workchain
- Add milliseconds to state
- Added end_lt to ShardBlockRef

## Version 1.9.68

- Added BlockInfo::gen_utime_ms (#1)
- Open library tests
- Fix compiler warnings
- Increase package version

## Version 1.9.67

- Minor fixes for optimistic consensus

## Version 1.9.63

- Added "collator" and "ref shard blocks" fields for optimistic consensus

## Version 1.9.47

- Added config param 44 `SuspendedAddresses` and `ComputeSkipReason::Suspended`

## Version 1.9.45

- Added capability flag for big cells `GlobalCapabilities::CapBigCells = 0x4000_0000`

## Version 1.9.40

- Supported ever-types version 2.0

## Version: 1.9.38

### New

- Add capability for fees in Gas units

## Version: 1.9.12

### New

- Fixed appending references to cells

## Version: 1.9.30

### New

- Add capability and feature for groth

## Version: 1.9.7

### New

- Add common as submodule

## Version: 1.9.0

### New

- New version of `Block` struct with out messages queue updates for foreign workchains
- Added `GlobalCapabilities::CapWc2WcQueueUpdates`
- Added new functions related with out messages queue updates

## Version: 1.8.19

### Fix

- Merkle proof pruned cell access fix

## Version: 1.8.0

### New

- Deprecated functions removed
- Refactor in message types naming
- Switched to Rust 2021 edition

## Version: 1.7.52

### New

- Performance issue in ValidatorDescr - removed ed25519_dalek::PublicKey using for holding public_key data
- Removed unused SignedBlock structure
- Bumped crc version to 3.0


# ton-types repository changelog
## Version 2.0.40

- Added wrappers for BLS arithmetic

## Version 2.0.39

- Remove debug prints for BLS

## Version 2.0.38

- Minor optimization for fn SliceData::get_bytestring

## Version 2.0.37

- Renamed `TON` into `ever` in comments

## Version 2.0.36

- Added `BocReader::set_allow_big_cells` method
- Supported big cell data serialization in `CellData::serialize` & `CellData::deserialize` 
  (the functions are not used in boc, they need in node)

## Version 2.0.35

- The code related to cell counting was covered by the `cell_counter` feature.
  Disable counting allows to increase cell's synthetic performance, 
  but the counting is very needed for memory usage diagnostics.

## Version 2.0.34

- Generate BLS key based on key material

## Version 2.0.33

- Added new method `Cell::cell_impl(&self)`

## Version 2.0.32

- Add ability to get root of changed subtree during hashmap filter

## Version 2.0.31

- Add BLS KeyOption from ever-crypto

## Version 2.0.30

- Fixed performance issue with fake 16MB big cell

## Version 2.0.29

- Remove BuilderData::level_mask (compute it at finalization)

## Version 2.0.28

- Fixed persistant state save

## Version 2.0.27

- Make SliceData::with_bitstring() public

## Version 2.0.26

- Optimize hashmap labels

## Version 2.0.25

- Optimize put_to_fork_with_mode()

## Version 2.0.24

- Add BocWriterStack for faster boc saving

## Version 2.0.23

- Add crypto wrappers

## Version 2.0.22

- Transform BocWriter::traverse() into iterative algo

## Version 2.0.21

- Refactor hashmap functions to optimize perfomance

## Version 2.0.20

- Add hashmap benchmark with profiling instructions

## Version 2.0.19

- Upgraded to ed25519-dalek 2.0

## Version 2.0.18

- Fixed big cells counting in read_inmem

## Version 2.0.17

- Additional checks for big cells count while BOC reading

## Version 2.0.16

- Removal of dead code

## Version 2.0.15

- BocWriter::write_ex(): do not compute CRC if not requested

## Version 2.0.14

- Fixed cells lifetime while BOC traverse

## Version 2.0.13

- Added UsageTree::build_visited_set

## Version 2.0.12

- Refactor LabelReader for perfomance
- Use load_bitstring for performance

## Version 2.0.11

- Use SliceData as bitstring for hashmap key manipulation

## Version 2.0.10


- Fixed BocReader::read_inmem for big bocs (> 4Gb)

## Version 2.0.9

- Optimize Cell::default()

## Version 2.0.8

- Fixed bug in hashmap_filter function

## Version 2.0.7

- Fixed panics after fuzzing

## Version 2.0.6

- Enhanced hashmap filter split

## Version 2.0.5

- Enhanced hashmap split by prefix
- Enhanced hashmap merge in any place
- Implemented hashmap filter split in one pass like two hashmap filters

## Version 2.0.4

- Fixed bug in x25519_shared_secret

## Version 2.0.3

- Added interface base64_encode_url_safe
- Minor refactoring

## Version 2.0.2

- Moved all crypto crates to wrappers

## Version 2.0.1

- Added crypto functions from crypto-repo
- Added wrappers for sha256, sha512, base64
- Bumped version of crc crate to 3.0
- Fix for clippy

## Version 2.0.0
- Added big cell. Call `create_big_cell` to create one.
- BOC routines: supported big cells and refactoring. 
  Created two basic structs for in-depth working with BOC: `BocWriter` and `BocReader`.
  Additionally three convinient wrappers: `write_boc`, `read_boc` and `read_single_root_boc`, that you'll probably want to use.

## Version: 1.12.2
- Fix for clippy

## Version: 1.12.1
- Add common as submodule

## Version: 1.12.0
- Remove bad types conversion

## Version: 1.11.11

### Bugfixes
- Loading cells with checking cell type

## Version: 1.11.3

### Bugfixes

- Fixed bug in 'deserialize_cells_tree_inmem_with_abort' - deleted unneded check with error (all needed checks performs in 'precheck_cells_tree_len'). Error appeared when BOC contained CRC.
- Fixed bug in 'deserialize_cells_tree_inmem_with_abort' - CRC calculated using wrong offsets.