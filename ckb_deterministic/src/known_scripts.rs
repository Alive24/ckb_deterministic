//! Registry of well-known CKB scripts.
//! 
//! This module provides a comprehensive registry of commonly used scripts
//! in the CKB ecosystem, including their code hashes and deployment information
//! for both mainnet and testnet.
//! 
//! # Supported Scripts
//! 
//! - **Lock Scripts**: Secp256k1, Multisig, AnyoneCanPay, OmniLock, JoyID, etc.
//! - **Type Scripts**: xUDT, Spore, NervosDAO, TypeID, etc.
//! - **Utility Scripts**: AlwaysSuccess (testing), Proxy locks
//! 
//! # Usage
//! 
//! ```no_run
//! use ckb_deterministic::known_scripts::{KnownScript, Network, get_script_info};
//! 
//! // Get xUDT script information for mainnet
//! if let Some(info) = get_script_info(KnownScript::XUdt, Network::Mainnet) {
//!     let code_hash = info.code_hash;
//!     let cell_deps = info.cell_deps;
//! }
//! ```

use crate::cell_classifier::CellClass;
use crate::errors::Error;
extern crate alloc;
use alloc::vec::Vec;
use alloc::vec;
use alloc::ffi::CString;
use ckb_std::high_level::decode_hex;

/// Known script types in the CKB ecosystem.
/// 
/// Each variant represents a well-known script with established
/// deployment addresses and usage patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnownScript {
    /// Nervos DAO type script
    NervosDao,
    /// Secp256k1 Blake160 lock script (most common lock)
    Secp256k1Blake160,
    /// Secp256k1 Multisig lock script
    Secp256k1Multisig,
    /// Secp256k1 Multisig V2 lock script
    Secp256k1MultisigV2,
    /// Anyone Can Pay lock script
    AnyoneCanPay,
    /// JoyID lock script
    JoyId,
    /// OmniLock script
    OmniLock,
    /// Nostr lock script
    NostrLock,
    /// Type ID type script
    TypeId,
    /// xUDT (User Defined Token) type script
    XUdt,
    /// Unique type script
    UniqueType,
    /// Spore (NFT) type script
    Spore,
    /// COTA (Compact On-chain Token Aggregator) type script
    Cota,
    /// PW Lock script
    PWLock,
    /// Always Success script (for testing)
    AlwaysSuccess,
    /// Input Type Proxy Lock
    InputTypeProxyLock,
    /// Output Type Proxy Lock
    OutputTypeProxyLock,
    /// Lock Proxy Lock
    LockProxyLock,
    /// Single Use Lock
    SingleUseLock,
    /// Type Burn Lock
    TypeBurnLock,
    /// Easy To Discover Type
    EasyToDiscoverType,
    /// Time Lock
    TimeLock,
}

impl KnownScript {
    /// Get the identifier string for this known script
    pub fn identifier(&self) -> &'static str {
        match self {
            KnownScript::NervosDao => "nervos_dao",
            KnownScript::Secp256k1Blake160 => "secp256k1_blake160",
            KnownScript::Secp256k1Multisig => "secp256k1_multisig",
            KnownScript::Secp256k1MultisigV2 => "secp256k1_multisig_v2",
            KnownScript::AnyoneCanPay => "anyone_can_pay",
            KnownScript::JoyId => "joy_id",
            KnownScript::OmniLock => "omni_lock",
            KnownScript::NostrLock => "nostr_lock",
            KnownScript::TypeId => "type_id",
            KnownScript::XUdt => "xudt",
            KnownScript::UniqueType => "unique_type",
            KnownScript::Spore => "spore",
            KnownScript::Cota => "cota",
            KnownScript::PWLock => "pw_lock",
            KnownScript::AlwaysSuccess => "always_success",
            KnownScript::InputTypeProxyLock => "input_type_proxy_lock",
            KnownScript::OutputTypeProxyLock => "output_type_proxy_lock",
            KnownScript::LockProxyLock => "lock_proxy_lock",
            KnownScript::SingleUseLock => "single_use_lock",
            KnownScript::TypeBurnLock => "type_burn_lock",
            KnownScript::EasyToDiscoverType => "easy_to_discover_type",
            KnownScript::TimeLock => "time_lock",
        }
    }

    /// Get the code hash for this known script
    /// Returns the slice representation of the code hash for network-agnostic comparison
    pub fn code_hash(&self) -> Result<[u8; 32], Error> {
        // Use mainnet as default for code hash lookup
        match get_script_info(*self, Network::Mainnet) {
            Some(script_info) => script_info.code_hash_in_slice(),
            None => Err(Error::InvalidCodeHash),
        }
    }
    
    /// Get the cell class for this known script
    pub fn cell_class(&self) -> CellClass {
        CellClass::known(self.identifier())
    }
    
    /// Check if this is a lock script type
    pub fn is_lock_script(&self) -> bool {
        matches!(self, 
            KnownScript::Secp256k1Blake160 |
            KnownScript::Secp256k1Multisig |
            KnownScript::Secp256k1MultisigV2 |
            KnownScript::AnyoneCanPay |
            KnownScript::JoyId |
            KnownScript::OmniLock |
            KnownScript::NostrLock |
            KnownScript::PWLock |
            KnownScript::AlwaysSuccess |
            KnownScript::InputTypeProxyLock |
            KnownScript::OutputTypeProxyLock |
            KnownScript::LockProxyLock |
            KnownScript::SingleUseLock |
            KnownScript::TypeBurnLock |
            KnownScript::TimeLock
        )
    }
    
    /// Check if this is a type script type
    pub fn is_type_script(&self) -> bool {
        matches!(self,
            KnownScript::NervosDao |
            KnownScript::TypeId |
            KnownScript::XUdt |
            KnownScript::UniqueType |
            KnownScript::Spore |
            KnownScript::Cota |
            KnownScript::EasyToDiscoverType
        )
    }
    
    /// Get all known script types
    pub fn all() -> Vec<KnownScript> {
        vec![
            KnownScript::NervosDao,
            KnownScript::Secp256k1Blake160,
            KnownScript::Secp256k1Multisig,
            KnownScript::Secp256k1MultisigV2,
            KnownScript::AnyoneCanPay,
            KnownScript::JoyId,
            KnownScript::OmniLock,
            KnownScript::NostrLock,
            KnownScript::TypeId,
            KnownScript::XUdt,
            KnownScript::UniqueType,
            KnownScript::Spore,
            KnownScript::Cota,
            KnownScript::PWLock,
            KnownScript::AlwaysSuccess,
            KnownScript::InputTypeProxyLock,
            KnownScript::OutputTypeProxyLock,
            KnownScript::LockProxyLock,
            KnownScript::SingleUseLock,
            KnownScript::TypeBurnLock,
            KnownScript::EasyToDiscoverType,
            KnownScript::TimeLock,
        ]
    }
}


/// Script information with code hash and cell dependencies
#[derive(Debug, Clone)]
pub struct ScriptInfo {
    pub code_hash: &'static str,
    pub hash_type: u8, // 0: data, 1: type, 2: data1
    pub cell_deps: Vec<(
        &'static str, // tx_hash
        u32,          // index
        u8,           // dep_type: 0 = code, 1 = depGroup
    )>,
}

impl ScriptInfo {
    pub fn code_hash_in_slice(&self) -> Result<[u8; 32], Error> {
        // Handle hex strings with or without 0x prefix
        let hex_str = if self.code_hash.starts_with("0x") || self.code_hash.starts_with("0X") {
            &self.code_hash[2..]
        } else {
            self.code_hash
        };
        
        decode_hex(&CString::new(hex_str).map_err(|_| Error::Encoding)?)
            .map_err(|_| Error::Encoding)?
            .try_into()
            .map_err(|_| Error::InvalidCodeHash)
    }
}

/// Network type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Testnet,
}

/// Get script information for a known script on a specific network
pub fn get_script_info(script: KnownScript, network: Network) -> Option<ScriptInfo> {
    match network {
        Network::Mainnet => get_mainnet_script_info(script),
        Network::Testnet => get_testnet_script_info(script),
    }
}

/// Get mainnet script information
fn get_mainnet_script_info(script: KnownScript) -> Option<ScriptInfo> {
    match script {
        KnownScript::NervosDao => Some(ScriptInfo {
                        code_hash: "0x82d76d1b75fe2fd9a27dfbaa65a039221a380d76c926f378d3f81cf3e7e13f2e",
                        hash_type: 1, // type
                        cell_deps: vec![
                            (
                                "0xe2fb199810d49a4d8beec56718ba2593b665db9d52299a0f9e6e75416d73ff5c",
                                2,
                                0, // code
                            )
                        ],
            }),
        KnownScript::Secp256k1Blake160 => Some(ScriptInfo {
                code_hash: "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
                hash_type: 1, // type
                cell_deps: vec![
                    (
                        "0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c",
                        0,
                        1, // depGroup
                    )
                ],
            }),
        KnownScript::Secp256k1Multisig => Some(ScriptInfo {
                code_hash: "0x5c5069eb0857efc65e1bca0c07df34c31663b3622fd3876c876320fc9634e2a8",
                hash_type: 1, // type
                cell_deps: vec![
                    (
                        "0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c",
                        1,
                        1, // depGroup
                    )
                ],
            }),
        KnownScript::Secp256k1MultisigV2 => Some(ScriptInfo {
                code_hash: "0x36c971b8d41fbd94aabca77dc75e826729ac98447b46f91e00796155dddb0d29",
                hash_type: 2, // data1
                cell_deps: vec![
                    (
                        "0x6888aa39ab30c570c2c30d9d5684d3769bf77265a7973211a3c087fe8efbf738",
                        0,
                        1, // depGroup
                    )
                ],
            }),
        KnownScript::AnyoneCanPay => Some(ScriptInfo {
                code_hash: "0xd369597ff47f29fbc0d47d2e3775370d1250b85140c670e4718af712983a2354",
                hash_type: 1, // type
                cell_deps: vec![
                    (
                        "0x4153a2014952d7cac45f285ce9a7c5c0c0e1b21f2d378b82ac1433cb11c25c4d",
                        0,
                        1, // depGroup
                    )
                ],
            }),
        KnownScript::TypeId => Some(ScriptInfo {
                code_hash: "0x00000000000000000000000000000000000000000000000000545950455f4944",
                hash_type: 1, // type
                cell_deps: vec![],
            }),
        KnownScript::XUdt => Some(ScriptInfo {
                code_hash: "0x50bd8d6680b8b9cf98b73f3c08faf8b2a21914311954118ad6609be6e78a1b95",
                hash_type: 2, // data1
                cell_deps: vec![
                    (
                        "0xc07844ce21b38e4b071dd0e1ee3b0e27afd8d7532491327f39b786343f558ab7",
                        0,
                        0, // code
                    )
                ],
            }),
        KnownScript::JoyId => Some(ScriptInfo {
                code_hash: "0xd00c84f0ec8fd441c38bc3f87a371f547190f2fcff88e642bc5bf54b9e318323",
                hash_type: 1, // type
                cell_deps: vec![
                    (
                        "0xf05188e5f3a6767fc4687faf45ba5f1a6e25d3ada6129dae8722cb282f262493",
                        0,
                        1, // depGroup
                    )
                ],
            }),
        KnownScript::Cota => Some(ScriptInfo {
                code_hash: "0x1122a4fb54697cf2e6e3a96c9d80fd398a936559b90954c6e88eb7ba0cf652df",
                hash_type: 1, // type
                cell_deps: vec![
                    (
                        "0xabaa25237554f0d6c586dc010e7e85e6870bcfd9fb8773257ecacfbe1fd738a0",
                        0,
                        1, // depGroup
                    )
                ],
            }),
        KnownScript::OmniLock => Some(ScriptInfo {
                code_hash: "0x9b819793a64463aed77c615d6cb226eea5487ccfc0783043a587254cda2b6f26",
                hash_type: 1, // type
                cell_deps: vec![
                    (
                        "0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c",
                        0,
                        1, // depGroup
                    ),
                    (
                        "0xc76edf469816aa22f416503c38d0b533d2a018e253e379f134c3985b3472c842",
                        0,
                        0, // code
                    )
                ],
            }),
        KnownScript::AlwaysSuccess => Some(ScriptInfo {
                code_hash: "0x3b521cc4b552f109d092d8cc468a8048acb53c5952dbe769d2b2f9cf6e47f7f1",
                hash_type: 2, // data1
                cell_deps: vec![
                    (
                        "0x10d63a996157d32c01078058000052674ca58d15f921bec7f1dcdac2160eb66b",
                        0,
                        0, // code
                    )
                ],
            }),
        KnownScript::PWLock => Some(ScriptInfo {
                code_hash: "0xbf43c3602455798c1a61a596e0d95278864c552fafe231c063b3fabf97a8febc",
                hash_type: 1, // type
                cell_deps: vec![
                    (
                        "0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c",
                        0,
                        1, // depGroup
                    ),
                    (
                        "0x1d60cb8f4666e039f418ea94730b1a8c5aa0bf2f7781474406387462924d15d4",
                        0,
                        0, // code
                    )
                ],
            }),
        KnownScript::NostrLock => Some(ScriptInfo {
                code_hash: "0x641a89ad2f77721b803cd50d01351c1f308444072d5fa20088567196c0574c68",
                hash_type: 1, // type
                cell_deps: vec![
                    (
                        "0x1911208b136957d5f7c1708a8835edfe8ae1d02700d5cb2c3a6aacf4d5906306",
                        0,
                        0, // code
                    )
                ],
            }),
        KnownScript::UniqueType => Some(ScriptInfo {
                code_hash: "0x2c8c11c985da60b0a330c61a85507416d6382c130ba67f0c47ab071e00aec628",
                hash_type: 2, // data1
                cell_deps: vec![
                    (
                        "0x67524c01c0cb5492e499c7c7e406f2f9d823e162d6b0cf432eacde0c9808c2ad",
                        0,
                        0, // code
                    )
                ],
            }),
        KnownScript::InputTypeProxyLock => Some(ScriptInfo {
                code_hash: "0x5123908965c711b0ffd8aec642f1ede329649bda1ebdca6bd24124d3796f768a",
                hash_type: 2, // data1
                cell_deps: vec![
                    (
                        "0x10d63a996157d32c01078058000052674ca58d15f921bec7f1dcdac2160eb66b",
                        1,
                        0, // code
                    )
                ],
            }),
        KnownScript::OutputTypeProxyLock => Some(ScriptInfo {
                code_hash: "0x2df53b592db3ae3685b7787adcfef0332a611edb83ca3feca435809964c3aff2",
                hash_type: 2, // data1
                cell_deps: vec![
                    (
                        "0x10d63a996157d32c01078058000052674ca58d15f921bec7f1dcdac2160eb66b",
                        2,
                        0, // code
                    )
                ],
            }),
        KnownScript::LockProxyLock => Some(ScriptInfo {
                code_hash: "0x5d41e32e224c15f152b7e6529100ebeac83b162f5f692a5365774dad2c1a1d02",
                hash_type: 2, // data1
                cell_deps: vec![
                    (
                        "0x10d63a996157d32c01078058000052674ca58d15f921bec7f1dcdac2160eb66b",
                        3,
                        0, // code
                    )
                ],
            }),
        KnownScript::SingleUseLock => Some(ScriptInfo {
                code_hash: "0x8290467a512e5b9a6b816469b0edabba1f4ac474e28ffdd604c2a7c76446bbaf",
                hash_type: 2, // data1
                cell_deps: vec![
                    (
                        "0x10d63a996157d32c01078058000052674ca58d15f921bec7f1dcdac2160eb66b",
                        4,
                        0, // code
                    )
                ],
            }),
        KnownScript::TypeBurnLock => Some(ScriptInfo {
                code_hash: "0xff78bae0abf17d7a404c0be0f9ad9c9185b3f88dcc60403453d5ba8e1f22f53a",
                hash_type: 2, // data1
                cell_deps: vec![
                    (
                        "0x10d63a996157d32c01078058000052674ca58d15f921bec7f1dcdac2160eb66b",
                        5,
                        0, // code
                    )
                ],
            }),
        KnownScript::EasyToDiscoverType => Some(ScriptInfo {
                code_hash: "0xaba4430cc7110d699007095430a1faa72973edf2322ddbfd4d1d219cacf237af",
                hash_type: 2, // data1
                cell_deps: vec![
                    (
                        "0xb0ed754fb27d67fd8388c97fed914fb7998eceaa01f3e6f967e498de1ba0ac9b",
                        0,
                        0, // code
                    )
                ],
            }),
        KnownScript::TimeLock => Some(ScriptInfo {
                code_hash: "0x6fac4b2e89360a1e692efcddcb3a28656d8446549fb83da6d896db8b714f4451",
                hash_type: 2, // data1
                cell_deps: vec![
                    (
                        "0xb0ed754fb27d67fd8388c97fed914fb7998eceaa01f3e6f967e498de1ba0ac9b",
                        1,
                        0, // code
                    )
                ],
            }),
        // TODO: There are many versions of Spore, we need to add all of them later.
        // Check https://github.com/sporeprotocol/spore-sdk/blob/main/packages/core/src/config/predefined.ts
        KnownScript::Spore => Some(ScriptInfo {
            code_hash: "0x598d793defef36e2eeba54a9b45130e4ca92822e1d193671f490950c3b856080",
            hash_type: 2, // data1
            cell_deps: vec![
                (
                    "0xb0ed754fb27d67fd8388c97fed914fb7998eceaa01f3e6f967e498de1ba0ac9b",
                    1,
                    0, // code
                )
            ],
        }),
    }
}

/// Get testnet script information
fn get_testnet_script_info(script: KnownScript) -> Option<ScriptInfo> {
    match script {
        KnownScript::NervosDao => Some(ScriptInfo {
            code_hash: "0x82d76d1b75fe2fd9a27dfbaa65a039221a380d76c926f378d3f81cf3e7e13f2e",
            hash_type: 1, // type
            cell_deps: vec![
                (
                    "0x8f8c79eb6671709633fe6a46de93c0fedc9c1b8a6527a18d3983879542635c9f",
                    2,
                    0, // code
                )
            ],
        }),
        KnownScript::Secp256k1Blake160 => Some(ScriptInfo {
            code_hash: "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
            hash_type: 1, // type
            cell_deps: vec![
                (
                    "0xf8de3bb47d055cdf460d93a2a6e1b05f7432f9777c8c474abf4eec1d4aee5d37",
                    0,
                    1, // depGroup
                )
            ],
        }),
        KnownScript::Secp256k1Multisig => Some(ScriptInfo {
            code_hash: "0x5c5069eb0857efc65e1bca0c07df34c31663b3622fd3876c876320fc9634e2a8",
            hash_type: 1, // type
            cell_deps: vec![
                (
                    "0xf8de3bb47d055cdf460d93a2a6e1b05f7432f9777c8c474abf4eec1d4aee5d37",
                    1,
                    1, // depGroup
                )
            ],
        }),
        KnownScript::Secp256k1MultisigV2 => Some(ScriptInfo {
            code_hash: "0x36c971b8d41fbd94aabca77dc75e826729ac98447b46f91e00796155dddb0d29",
            hash_type: 2, // data1
            cell_deps: vec![
                (
                    "0x2eefdeb21f3a3edf697c28a52601b4419806ed60bb427420455cc29a090b26d5",
                    0,
                    1, // depGroup
                )
            ],
        }),
        KnownScript::AnyoneCanPay => Some(ScriptInfo {
            code_hash: "0x3419a1c09eb2567f6552ee7a8ecffd64155cffe0f1796e6e61ec088d740c1356",
            hash_type: 1, // type
            cell_deps: vec![
                (
                    "0xec26b0f85ed839ece5f11c4c4e837ec359f5adc4420410f6453b1f6b60fb96a6",
                    0,
                    1, // depGroup
                )
            ],
        }),
        KnownScript::TypeId => Some(ScriptInfo {
            code_hash: "0x00000000000000000000000000000000000000000000000000545950455f4944",
            hash_type: 1, // type
            cell_deps: vec![],
        }),
        KnownScript::XUdt => Some(ScriptInfo {
            code_hash: "0x25c29dc317811a6f6f3985a7a9ebc4838bd388d19d0feeecf0bcd60f6c0975bb",
            hash_type: 1, // type
            cell_deps: vec![
                (
                    "0xbf6fb538763efec2a70a6a3dcb7242787087e1030c4e7d86585bc63a9d337f5f",
                    0,
                    0, // code
                )
            ],
        }),
        KnownScript::JoyId => Some(ScriptInfo {
            code_hash: "0xd23761b364210735c19c60561d213fb3beae2fd6172743719eff6920e020baac",
            hash_type: 1, // type
            cell_deps: vec![
                (
                    "0x4dcf3f3b09efac8995d6cbee87c5345e812d310094651e0c3d9a730f32dc9263",
                    0,
                    1, // depGroup
                )
            ],
        }),
        KnownScript::Cota => Some(ScriptInfo {
            code_hash: "0x89cd8003a0eaf8e65e0c31525b7d1d5c1becefd2ea75bb4cff87810ae37764d8",
            hash_type: 1, // type
            cell_deps: vec![
                (
                    "0x636a786001f87cb615acfcf408be0f9a1f077001f0bbc75ca54eadfe7e221713",
                    0,
                    1, // depGroup
                )
            ],
        }),
        KnownScript::OmniLock => Some(ScriptInfo {
            code_hash: "0xf329effd1c475a2978453c8600e1eaf0bc2087ee093c3ee64cc96ec6847752cb",
            hash_type: 1, // type
            cell_deps: vec![
                (
                    "0xf8de3bb47d055cdf460d93a2a6e1b05f7432f9777c8c474abf4eec1d4aee5d37",
                    0,
                    1, // depGroup
                ),
                (
                    "0xec18bf0d857c981c3d1f4e17999b9b90c484b303378e94de1a57b0872f5d4602",
                    0,
                    0, // code
                )
            ],
        }),
        KnownScript::AlwaysSuccess => Some(ScriptInfo {
            code_hash: "0x3b521cc4b552f109d092d8cc468a8048acb53c5952dbe769d2b2f9cf6e47f7f1",
            hash_type: 2, // data1
            cell_deps: vec![
                (
                    "0xb4f171c9c9caf7401f54a8e56225ae21d95032150a87a4678eac3f66a3137b93",
                    0,
                    0, // code
                )
            ],
        }),
        KnownScript::PWLock => Some(ScriptInfo {
            code_hash: "0x58c5f491aba6d61678b7cf7edf4910b1f5e00ec0cde2f42e0abb4fd9aff25a63",
            hash_type: 1, // type
            cell_deps: vec![
                (
                    "0xf8de3bb47d055cdf460d93a2a6e1b05f7432f9777c8c474abf4eec1d4aee5d37",
                    0,
                    1, // depGroup
                ),
                (
                    "0x57a62003daeab9d54aa29b944fc3b451213a5ebdf2e232216a3cfed0dde61b38",
                    0,
                    0, // code
                )
            ],
        }),
        KnownScript::NostrLock => Some(ScriptInfo {
            code_hash: "0x6ae5ee0cb887b2df5a9a18137315b9bdc55be8d52637b2de0624092d5f0c91d5",
            hash_type: 1, // type
            cell_deps: vec![
                (
                    "0xa2a434dcdbe280b9ed75bb7d6c7d68186a842456aba0fc506657dc5ed7c01d68",
                    0,
                    0, // code
                )
            ],
        }),
        KnownScript::UniqueType => Some(ScriptInfo {
            code_hash: "0x8e341bcfec6393dcd41e635733ff2dca00a6af546949f70c57a706c0f344df8b",
            hash_type: 1, // type
            cell_deps: vec![
                (
                    "0xff91b063c78ed06f10a1ed436122bd7d671f9a72ef5f5fa28d05252c17cf4cef",
                    0,
                    0, // code
                )
            ],
        }),
        KnownScript::InputTypeProxyLock => Some(ScriptInfo {
            code_hash: "0x5123908965c711b0ffd8aec642f1ede329649bda1ebdca6bd24124d3796f768a",
            hash_type: 2, // data1
            cell_deps: vec![
                (
                    "0xb4f171c9c9caf7401f54a8e56225ae21d95032150a87a4678eac3f66a3137b93",
                    1,
                    0, // code
                )
            ],
        }),
        KnownScript::OutputTypeProxyLock => Some(ScriptInfo {
            code_hash: "0x2df53b592db3ae3685b7787adcfef0332a611edb83ca3feca435809964c3aff2",
            hash_type: 2, // data1
            cell_deps: vec![
                (
                    "0xb4f171c9c9caf7401f54a8e56225ae21d95032150a87a4678eac3f66a3137b93",
                    2,
                    0, // code
                )
            ],
        }),
        KnownScript::LockProxyLock => Some(ScriptInfo {
            code_hash: "0x5d41e32e224c15f152b7e6529100ebeac83b162f5f692a5365774dad2c1a1d02",
            hash_type: 2, // data1
            cell_deps: vec![
                (
                    "0xb4f171c9c9caf7401f54a8e56225ae21d95032150a87a4678eac3f66a3137b93",
                    3,
                    0, // code
                )
            ],
        }),
        KnownScript::SingleUseLock => Some(ScriptInfo {
            code_hash: "0x8290467a512e5b9a6b816469b0edabba1f4ac474e28ffdd604c2a7c76446bbaf",
            hash_type: 2, // data1
            cell_deps: vec![
                (
                    "0xb4f171c9c9caf7401f54a8e56225ae21d95032150a87a4678eac3f66a3137b93",
                    4,
                    0, // code
                )
            ],
        }),
        KnownScript::TypeBurnLock => Some(ScriptInfo {
            code_hash: "0xff78bae0abf17d7a404c0be0f9ad9c9185b3f88dcc60403453d5ba8e1f22f53a",
            hash_type: 2, // data1
            cell_deps: vec![
                (
                    "0xb4f171c9c9caf7401f54a8e56225ae21d95032150a87a4678eac3f66a3137b93",
                    5,
                    0, // code
                )
            ],
        }),
        KnownScript::EasyToDiscoverType => Some(ScriptInfo {
            code_hash: "0xaba4430cc7110d699007095430a1faa72973edf2322ddbfd4d1d219cacf237af",
            hash_type: 2, // data1
            cell_deps: vec![
                (
                    "0x1b4ffcad55ecd36ffb2715b6816b83da73851f1a24fe594f263c4f34dad90792",
                    0,
                    0, // code
                )
            ],
        }),
        KnownScript::TimeLock => Some(ScriptInfo {
            code_hash: "0x6fac4b2e89360a1e692efcddcb3a28656d8446549fb83da6d896db8b714f4451",
            hash_type: 2, // data1
            cell_deps: vec![
                (
                    "0x1b4ffcad55ecd36ffb2715b6816b83da73851f1a24fe594f263c4f34dad90792",
                    1,
                    0, // code
                )
            ],
        }),
        // TODO: There are many versions of Spore, we need to add all of them later.
        // Check https://github.com/sporeprotocol/spore-sdk/blob/main/packages/core/src/config/predefined.ts
        KnownScript::Spore => Some(ScriptInfo {
            code_hash: "0x4a4dce1df3dffff7f8b2cd7dff7303df3b6150c9788cb75dcf6747247132b9f5",
            hash_type: 2, // data1
            cell_deps: vec![
                (
                    "0x96b198fb5ddbd1eed57ed667068f1f1e55d07907b4c0dbd38675a69ea1b69824",
                    1,
                    0, // code
                )
            ],
        }),
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell_classifier::{CellClass, CellClassifier, CellInfo, ClassificationRule, RuleBasedClassifier};
    use ckb_std::ckb_constants::Source;
    use ckb_std::ckb_types::packed::Script;

    #[test]
    fn test_known_script_identifiers() {
        assert_eq!(KnownScript::XUdt.identifier(), "xudt");
        assert_eq!(KnownScript::Spore.identifier(), "spore");
        assert_eq!(KnownScript::Secp256k1Blake160.identifier(), "secp256k1_blake160");
        assert_eq!(KnownScript::NervosDao.identifier(), "nervos_dao");
    }
    
    #[test]
    fn test_script_type_classification() {
        assert!(KnownScript::XUdt.is_type_script());
        assert!(!KnownScript::XUdt.is_lock_script());
        
        assert!(KnownScript::Secp256k1Blake160.is_lock_script());
        assert!(!KnownScript::Secp256k1Blake160.is_type_script());
    }
    
    #[test]
    fn test_simple_ckb_cell_class() {
        let class = CellClass::SimpleCKB;
        assert!(class.is_simple_ckb());
    }
    
    #[test]
    fn test_rule_based_classifier_defaults() {
        let classifier = RuleBasedClassifier::new("test");
        
        // Test simple CKB cell (no type script)
        let simple_cell = CellInfo {
            source: Source::Input,
            index: 0,
            data: Vec::new(),
            lock: Script::default(),
            lock_hash: [0u8; 32],
            type_script: None,
            type_hash: None,
        };
        
        let result = classifier.classify(&simple_cell).unwrap();
        assert!(result.is_simple_ckb());
        
        // Test cell with type script (should be unidentified without additional config)
        let typed_cell = CellInfo {
            source: Source::Input,
            index: 0,
            data: Vec::new(),
            lock: Script::default(),
            lock_hash: [0u8; 32],
            type_script: Some(Script::default()),
            type_hash: Some([1u8; 32]),
        };
        
        let result2 = classifier.classify(&typed_cell).unwrap();
        assert!(result2.is_unidentified());
    }
    
    #[test]
    fn test_known_script_with_rule_based_classifier() {
        let xudt_code_hash = [1u8; 32];
        let spore_code_hash = [2u8; 32];
        
        let classifier = RuleBasedClassifier::new("test")
            .add_rule(ClassificationRule::TypeCodeHash {
                code_hash: xudt_code_hash,
                class: KnownScript::XUdt.cell_class(),
            })
            .add_rule(ClassificationRule::TypeCodeHash {
                code_hash: spore_code_hash,
                class: KnownScript::Spore.cell_class(),
            });
        
        // Test that the classifier was built successfully
        assert_eq!(classifier.name(), "test");
    }
    
    #[test]
    fn test_all_known_scripts() {
        let all_scripts = KnownScript::all();
        assert_eq!(all_scripts.len(), 22);
        assert!(all_scripts.contains(&KnownScript::XUdt));
        assert!(all_scripts.contains(&KnownScript::Spore));
        assert!(all_scripts.contains(&KnownScript::Secp256k1Blake160));
    }
}