use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_json_binary, Binary, IbcChannel};
use cw_storage_plus::Item;

/// This is set for the verifier to prevent the presentation from being too large
pub type Channel = Item<IbcChannel>;
pub const CHANNEL: Channel = Item::new("mpl");

#[cw_serde]
#[serde(rename_all = "camelCase")]
pub struct ResourceReqPacket {
    pub resource_id: String,
    pub collection_id: String,
}

impl std::fmt::Display for ResourceReqPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.collection_id, self.resource_id)
    }
}

#[cw_serde]
#[serde(rename_all = "camelCase")]
/// This is the same type as the one stored on cheqd
pub struct ResourceWithMetadata {
    pub linked_resource: LinkedResource,
    pub linked_resource_metadata: LinkedResourceMetadata,
}

// https://github.com/cheqd/cheqd-node/blob/f8d41e68f437b9aa5cf10ff275fbd6365b6541ec/x/resource/types/resource.pb.go#L31
#[cw_serde]
pub struct LinkedResource {
    pub data: Binary,
}

// Metadata stores the metadata of a DID-Linked Resource
//type Metadata struct {
//	// collection_id is the ID of the collection that the Resource belongs to. Defined client-side.
//	// This field is the unique identifier of the DID linked to this Resource
//	// Format: <unique-identifier>
//	//
//	// Examples:
//	// - c82f2b02-bdab-4dd7-b833-3e143745d612
//	// - wGHEXrZvJxR8vw5P3UWH1j
//	CollectionId string `protobuf:"bytes,1,opt,name=collection_id,json=collectionId,proto3" json:"resourceCollectionId"`
//	// id is the ID of the Resource. Defined client-side.
//	// This field is a unique identifier for this specific version of the Resource.
//	// Format: <uuid>
//	Id string `protobuf:"bytes,2,opt,name=id,proto3" json:"resourceId"`
//	// name is a human-readable name for the Resource. Defined client-side.
//	// Does not change between different versions.
//	// Example: PassportSchema, EducationTrustRegistry
//	Name string `protobuf:"bytes,3,opt,name=name,proto3" json:"resourceName"`
//	// version is a human-readable semantic version for the Resource. Defined client-side.
//	// Stored as a string. OPTIONAL.
//	// Example: 1.0.0, v2.1.0
//	Version string `protobuf:"bytes,4,opt,name=version,proto3" json:"resourceVersion"`
//	// resource_type is a Resource type that identifies what the Resource is. Defined client-side.
//	// This is NOT the same as the resource's media type.
//	// Example: AnonCredsSchema, StatusList2021
//	ResourceType string `protobuf:"bytes,5,opt,name=resource_type,json=resourceType,proto3" json:"resourceType"`
//	// List of alternative URIs for the SAME Resource.
//	AlsoKnownAs []*AlternativeUri `protobuf:"bytes,6,rep,name=also_known_as,json=alsoKnownAs,proto3" json:"resourceAlternativeUri"`
//	// media_type is IANA media type of the Resource. Defined ledger-side.
//	// Example: application/json, image/png
//	MediaType string `protobuf:"bytes,7,opt,name=media_type,json=mediaType,proto3" json:"media_type,omitempty"`
//	// created is the time at which the Resource was created. Defined ledger-side.
//	// Format: RFC3339
//	// Example: 2021-01-01T00:00:00Z
//	Created time.Time `protobuf:"bytes,8,opt,name=created,proto3,stdtime" json:"created"`
//	// checksum is a SHA-256 checksum hash of the Resource. Defined ledger-side.
//	// Example: d14a028c2a3a2bc9476102bb288234c415a2b01f828ea62ac5b3e42f
//	Checksum string `protobuf:"bytes,9,opt,name=checksum,proto3" json:"checksum,omitempty"`
//	// previous_version_id is the ID of the previous version of the Resource. Defined ledger-side.
//	// This is based on the Resource's name and Resource type to determine whether it's the same Resource.
//	// Format: <uuid>
//	PreviousVersionId string `protobuf:"bytes,10,opt,name=previous_version_id,json=previousVersionId,proto3" json:"previous_version_id,omitempty"`
//	// next_version_id is the ID of the next version of the Resource. Defined ledger-side.
//	// This is based on the Resource's name and Resource type to determine whether it's the same Resource.
//	// Format: <uuid>
//	NextVersionId string `protobuf:"bytes,11,opt,name=next_version_id,json=nextVersionId,proto3" json:"next_version_id,omitempty"`
//}
// https://github.com/cheqd/cheqd-node/blob/f8d41e68f437b9aa5cf10ff275fbd6365b6541ec/x/resource/types/resource.pb.go#L77
#[cw_serde]
#[serde(rename_all = "camelCase")]
pub struct LinkedResourceMetadata {
    pub resource_collection_id: String,
    pub resource_id: String,
    pub resource_name: String,
    pub resource_version: String,
    pub resource_type: String,
    pub resource_alternative_uri: Vec<AlternativeUri>,
    #[serde(rename = "media_type")]
    pub media_type: Option<String>,
    pub created: String,
    pub checksum: Option<String>,
    #[serde(rename = "previous_version_id")]
    pub previous_version_id: Option<String>,
    #[serde(rename = "next_version_id")]
    pub next_version_id: Option<String>,
}

#[cw_serde]
pub struct AlternativeUri {
    pub uri: Option<String>,
    pub description: Option<String>,
}

/// This is a generic ICS acknowledgement format.
/// Proto defined here: https://github.com/cosmos/cosmos-sdk/blob/v0.42.0/proto/ibc/core/channel/v1/channel.proto#L141-L147
/// If ibc_receive_packet returns Err(), then x/wasm runtime will rollback the state and return an error message in this format
#[cw_serde]
pub enum StdAck {
    Result(Binary),
    Error(String),
}

impl StdAck {
    // create a serialized error message
    pub fn fail(err: String) -> Binary {
        StdAck::Error(err).ack()
    }

    pub fn ack(&self) -> Binary {
        to_json_binary(self).unwrap()
    }

    #[cfg(test)]
    pub fn unwrap(self) -> Binary {
        match self {
            StdAck::Result(data) => data,
            StdAck::Error(err) => panic!("{}", err),
        }
    }
}
