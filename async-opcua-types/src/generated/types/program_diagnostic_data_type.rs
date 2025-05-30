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
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ProgramDiagnosticDataType {
    pub create_session_id: opcua::types::node_id::NodeId,
    pub create_client_name: opcua::types::string::UAString,
    pub invocation_creation_time: opcua::types::data_types::UtcTime,
    pub last_transition_time: opcua::types::data_types::UtcTime,
    pub last_method_call: opcua::types::string::UAString,
    pub last_method_session_id: opcua::types::node_id::NodeId,
    pub last_method_input_arguments: Option<Vec<crate::argument::Argument>>,
    pub last_method_output_arguments: Option<Vec<crate::argument::Argument>>,
    pub last_method_call_time: opcua::types::data_types::UtcTime,
    pub last_method_return_status: super::status_result::StatusResult,
}
impl opcua::types::MessageInfo for ProgramDiagnosticDataType {
    fn type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::ProgramDiagnosticDataType_Encoding_DefaultBinary
    }
    fn json_type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::ProgramDiagnosticDataType_Encoding_DefaultJson
    }
    fn xml_type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::ProgramDiagnosticDataType_Encoding_DefaultXml
    }
    fn data_type_id(&self) -> opcua::types::DataTypeId {
        opcua::types::DataTypeId::ProgramDiagnosticDataType
    }
}
