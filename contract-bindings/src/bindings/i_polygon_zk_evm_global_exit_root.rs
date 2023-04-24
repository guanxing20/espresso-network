pub use i_polygon_zk_evm_global_exit_root::*;
/// This module was auto-generated with ethers-rs Abigen.
/// More information at: <https://github.com/gakonst/ethers-rs>
#[allow(
    clippy::enum_variant_names,
    clippy::too_many_arguments,
    clippy::upper_case_acronyms,
    clippy::type_complexity,
    dead_code,
    non_camel_case_types
)]
pub mod i_polygon_zk_evm_global_exit_root {
    #[rustfmt::skip]
    const __ABI: &str = "[{\"inputs\":[],\"type\":\"error\",\"name\":\"OnlyAllowedContracts\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getLastGlobalExitRoot\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"globalExitRootNum\",\"type\":\"bytes32\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"globalExitRootMap\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"newRollupExitRoot\",\"type\":\"bytes32\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"updateExitRoot\",\"outputs\":[]}]";
    ///The parsed JSON ABI of the contract.
    pub static IPOLYGONZKEVMGLOBALEXITROOT_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> =
        ::ethers::contract::Lazy::new(|| {
            ::ethers::core::utils::__serde_json::from_str(__ABI).expect("ABI is always valid")
        });
    pub struct IPolygonZkEVMGlobalExitRoot<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for IPolygonZkEVMGlobalExitRoot<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for IPolygonZkEVMGlobalExitRoot<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for IPolygonZkEVMGlobalExitRoot<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for IPolygonZkEVMGlobalExitRoot<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(stringify!(IPolygonZkEVMGlobalExitRoot))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> IPolygonZkEVMGlobalExitRoot<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(::ethers::contract::Contract::new(
                address.into(),
                IPOLYGONZKEVMGLOBALEXITROOT_ABI.clone(),
                client,
            ))
        }
        ///Calls the contract's `getLastGlobalExitRoot` (0x3ed691ef) function
        pub fn get_last_global_exit_root(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([62, 214, 145, 239], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `globalExitRootMap` (0x257b3632) function
        pub fn global_exit_root_map(
            &self,
            global_exit_root_num: [u8; 32],
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([37, 123, 54, 50], global_exit_root_num)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `updateExitRoot` (0x33d6247d) function
        pub fn update_exit_root(
            &self,
            new_rollup_exit_root: [u8; 32],
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([51, 214, 36, 125], new_rollup_exit_root)
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
        for IPolygonZkEVMGlobalExitRoot<M>
    {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Custom Error type `OnlyAllowedContracts` with signature `OnlyAllowedContracts()` and selector `0xb49365dd`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[etherror(name = "OnlyAllowedContracts", abi = "OnlyAllowedContracts()")]
    pub struct OnlyAllowedContracts;
    ///Container type for all input parameters for the `getLastGlobalExitRoot` function with signature `getLastGlobalExitRoot()` and selector `0x3ed691ef`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(name = "getLastGlobalExitRoot", abi = "getLastGlobalExitRoot()")]
    pub struct GetLastGlobalExitRootCall;
    ///Container type for all input parameters for the `globalExitRootMap` function with signature `globalExitRootMap(bytes32)` and selector `0x257b3632`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(name = "globalExitRootMap", abi = "globalExitRootMap(bytes32)")]
    pub struct GlobalExitRootMapCall {
        pub global_exit_root_num: [u8; 32],
    }
    ///Container type for all input parameters for the `updateExitRoot` function with signature `updateExitRoot(bytes32)` and selector `0x33d6247d`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(name = "updateExitRoot", abi = "updateExitRoot(bytes32)")]
    pub struct UpdateExitRootCall {
        pub new_rollup_exit_root: [u8; 32],
    }
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum IPolygonZkEVMGlobalExitRootCalls {
        GetLastGlobalExitRoot(GetLastGlobalExitRootCall),
        GlobalExitRootMap(GlobalExitRootMapCall),
        UpdateExitRoot(UpdateExitRootCall),
    }
    impl ::ethers::core::abi::AbiDecode for IPolygonZkEVMGlobalExitRootCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) =
                <GetLastGlobalExitRootCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetLastGlobalExitRoot(decoded));
            }
            if let Ok(decoded) =
                <GlobalExitRootMapCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GlobalExitRootMap(decoded));
            }
            if let Ok(decoded) =
                <UpdateExitRootCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::UpdateExitRoot(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for IPolygonZkEVMGlobalExitRootCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::GetLastGlobalExitRoot(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GlobalExitRootMap(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::UpdateExitRoot(element) => ::ethers::core::abi::AbiEncode::encode(element),
            }
        }
    }
    impl ::core::fmt::Display for IPolygonZkEVMGlobalExitRootCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::GetLastGlobalExitRoot(element) => ::core::fmt::Display::fmt(element, f),
                Self::GlobalExitRootMap(element) => ::core::fmt::Display::fmt(element, f),
                Self::UpdateExitRoot(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<GetLastGlobalExitRootCall> for IPolygonZkEVMGlobalExitRootCalls {
        fn from(value: GetLastGlobalExitRootCall) -> Self {
            Self::GetLastGlobalExitRoot(value)
        }
    }
    impl ::core::convert::From<GlobalExitRootMapCall> for IPolygonZkEVMGlobalExitRootCalls {
        fn from(value: GlobalExitRootMapCall) -> Self {
            Self::GlobalExitRootMap(value)
        }
    }
    impl ::core::convert::From<UpdateExitRootCall> for IPolygonZkEVMGlobalExitRootCalls {
        fn from(value: UpdateExitRootCall) -> Self {
            Self::UpdateExitRoot(value)
        }
    }
    ///Container type for all return fields from the `getLastGlobalExitRoot` function with signature `getLastGlobalExitRoot()` and selector `0x3ed691ef`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct GetLastGlobalExitRootReturn(pub [u8; 32]);
    ///Container type for all return fields from the `globalExitRootMap` function with signature `globalExitRootMap(bytes32)` and selector `0x257b3632`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct GlobalExitRootMapReturn(pub ::ethers::core::types::U256);
}
