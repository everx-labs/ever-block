# Release Notes

All notable changes to this project will be documented in this file.

## Version 1.9.139

- HashmapAugE returns depth of tree on insert new item

## Version 1.9.125

- added MerkleUpdate::apply_for_with_cells_factory

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
