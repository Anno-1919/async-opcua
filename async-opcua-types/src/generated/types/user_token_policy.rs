// This file was autogenerated from schemas/1.05/Opc.Ua.NodeSet2.Services.xml by async-opcua-codegen
//
// DO NOT EDIT THIS FILE

// OPCUA for Rust
// SPDX-License-Identifier: MPL-2.0
// Copyright (C) 2017-2024 Adam Lock, Einar Omang
#[allow(unused)]
mod opcua {
    pub(super) use crate as types;
}
#[opcua::types::ua_encodable]
///https://reference.opcfoundation.org/v105/Core/docs/Part4/7.42
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UserTokenPolicy {
    pub policy_id: opcua::types::string::UAString,
    pub token_type: super::enums::UserTokenType,
    pub issued_token_type: opcua::types::string::UAString,
    pub issuer_endpoint_url: opcua::types::string::UAString,
    pub security_policy_uri: opcua::types::string::UAString,
}
impl opcua::types::MessageInfo for UserTokenPolicy {
    fn type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::UserTokenPolicy_Encoding_DefaultBinary
    }
    fn json_type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::UserTokenPolicy_Encoding_DefaultJson
    }
    fn xml_type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::UserTokenPolicy_Encoding_DefaultXml
    }
    fn data_type_id(&self) -> opcua::types::DataTypeId {
        opcua::types::DataTypeId::UserTokenPolicy
    }
}
