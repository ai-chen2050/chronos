// This file is @generated by prost-build.
/// business data
/// ZMessage.type = Z_TYPE_CHAT
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ZChat {
    #[prost(bytes = "vec", tag = "1")]
    pub message_data: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub clock: ::core::option::Option<super::vlc::ClockInfo>,
}
/// ZMessage.type = Z_TYPE_HPOINTS
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HPoints {
    #[prost(bytes = "vec", tag = "1")]
    pub op_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub event_id: ::prost::alloc::vec::Vec<u8>,
    /// new points
    #[prost(uint64, tag = "3")]
    pub points: u64,
}
/// ZMessage.type = Z_TYPE_GATEWAY
/// Gateway just only needs read api
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ZGateway {
    #[prost(enumeration = "GatewayType", tag = "1")]
    pub r#type: i32,
    #[prost(enumeration = "QueryMethod", tag = "2")]
    pub method: i32,
    #[prost(bytes = "vec", tag = "3")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
/// ZGateway.type = GATEWAY_TYPE_CLOCK_NODE
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClockNode {
    #[prost(message, optional, tag = "1")]
    pub clock: ::core::option::Option<super::vlc::Clock>,
    #[prost(bytes = "vec", tag = "2")]
    pub id: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    pub message_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "4")]
    pub count: u64,
    #[prost(uint64, tag = "5")]
    pub create_at: u64,
    #[prost(bytes = "vec", tag = "6")]
    pub raw_message: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClockNodes {
    #[prost(message, repeated, tag = "1")]
    pub clock_nodes: ::prost::alloc::vec::Vec<ClockNode>,
}
/// ZGateway.type = GATEWAY_TYPE_NODE_INFO
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodeInfo {
    #[prost(string, repeated, tag = "1")]
    pub node_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub reason: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "3")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
/// ZGateway.method = QUERY_BY_MSGID
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryByMsgId {
    #[prost(string, tag = "1")]
    pub msg_id: ::prost::alloc::string::String,
}
/// ZGateway.method = QUERY_BY_TABLE_KEYID
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryByTableKeyId {
    #[prost(uint64, tag = "1")]
    pub last_pos: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum GatewayType {
    ClockNode = 0,
    /// ref merge log
    MergeLog = 1,
    /// heartbeat or node info
    NodeInfo = 2,
    /// p2p message
    ZMessage = 3,
}
impl GatewayType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            GatewayType::ClockNode => "GATEWAY_TYPE_CLOCK_NODE",
            GatewayType::MergeLog => "GATEWAY_TYPE_MERGE_LOG",
            GatewayType::NodeInfo => "GATEWAY_TYPE_NODE_INFO",
            GatewayType::ZMessage => "GATEWAY_TYPE_Z_MESSAGE",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "GATEWAY_TYPE_CLOCK_NODE" => Some(Self::ClockNode),
            "GATEWAY_TYPE_MERGE_LOG" => Some(Self::MergeLog),
            "GATEWAY_TYPE_NODE_INFO" => Some(Self::NodeInfo),
            "GATEWAY_TYPE_Z_MESSAGE" => Some(Self::ZMessage),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum QueryMethod {
    QueryByMsgid = 0,
    QueryByTableKeyid = 1,
}
impl QueryMethod {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            QueryMethod::QueryByMsgid => "QUERY_BY_MSGID",
            QueryMethod::QueryByTableKeyid => "QUERY_BY_TABLE_KEYID",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "QUERY_BY_MSGID" => Some(Self::QueryByMsgid),
            "QUERY_BY_TABLE_KEYID" => Some(Self::QueryByTableKeyid),
            _ => None,
        }
    }
}
