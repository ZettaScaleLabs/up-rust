/********************************************************************************
 * Copyright (c) 2023 Contributors to the Eclipse Foundation
 *
 * See the NOTICE file(s) distributed with this work for additional
 * information regarding copyright ownership.
 *
 * This program and the accompanying materials are made available under the
 * terms of the Apache License Version 2.0 which is available at
 * https://www.apache.org/licenses/LICENSE-2.0
 *
 * SPDX-License-Identifier: Apache-2.0
 ********************************************************************************/

use crate::types::ValidationResult;
use crate::uprotocol::{UAuthority, UUri};

/// Struct to encapsulate Uri validation logic.
pub struct UriValidator;

impl UriValidator {
    /// Validates a `UUri` to ensure that it has at least a name for the uEntity.
    ///
    /// # Arguments
    /// * `uri` - The `UUri` to validate.
    ///
    /// # Returns
    /// Returns `ValidationResult` containing a success or a failure with the error message.
    pub fn validate(uri: &UUri) -> ValidationResult {
        if Self::is_empty(uri) {
            return ValidationResult::Failure("Uri is empty".into());
        }
        if let Some(authority) = &uri.authority {
            if !Self::is_remote(authority) {
                return ValidationResult::Failure("Uri is remote missing uAuthority".into());
            }
        }
        if let Some(entity) = &uri.entity {
            if entity.name.trim().is_empty() {
                return ValidationResult::Failure("Uri is missing uSoftware Entity name".into());
            }
        }
        ValidationResult::Success
    }

    /// Validates a `UUri` that is meant to be used as an RPC method URI.
    /// Used in Request sink values and Response source values.
    ///
    /// # Arguments
    /// * `uri` - The `UUri` to validate.
    ///
    /// # Returns
    /// Returns `ValidationResult` containing a success or a failure with the error message.
    pub fn validate_rpc_method(uri: &UUri) -> ValidationResult {
        let status = Self::validate(uri);
        if status.is_failure() {
            return status;
        }
        if !Self::is_rpc_method(uri) {
            return ValidationResult::Failure("Invalid RPC method uri. Uri should be the method to be called, or method from response".into());
        }
        ValidationResult::Success
    }

    /// Validates a `UUri` that is meant to be used as an RPC response URI.
    /// This is used in Request source values and Response sink values.
    ///
    /// # Arguments
    ///
    /// * `uri` - The `UUri` instance to validate.
    ///
    /// # Returns
    ///
    /// Returns a `UStatus` containing either a success or a failure, along with the corresponding error message.
    pub fn validate_rpc_response(uri: &UUri) -> ValidationResult {
        let status = Self::validate(uri);
        if status.is_failure() {
            return status;
        }
        if Self::is_rpc_response(uri) {
            return ValidationResult::Failure("Invalid RPC response type".into());
        }
        ValidationResult::Success
    }

    /// Indicates whether this `UUri` is empty, meaning it does not contain authority, entity, and resource.
    ///
    /// # Arguments
    /// * `uri` - The `UUri` to check for emptiness.
    ///
    /// # Returns
    /// Returns `true` if this `UUri` is an empty container and has no valuable information for building uProtocol sinks or sources.
    pub fn is_empty(uri: &UUri) -> bool {
        uri.authority.is_none() || uri.entity.is_none() || uri.resource.is_none()
    }

    /// Checks if the URI contains both names and numeric representations of the names.
    ///
    /// This indicates that the `UUri` can be serialized to long or micro formats.
    ///
    /// # Arguments
    /// * `uri` - The `UUri` to check if resolved.
    ///
    /// # Returns
    /// Returns `true` if the URI contains both names and numeric representations of the names,
    /// meaning that this `UUri` can be serialized to long or micro formats.
    pub fn is_resolved(uri: &UUri) -> bool {
        !Self::is_empty(uri)
        // TODO finish this
    }

    /// Checks if the URI is of type RPC.
    ///
    /// # Arguments
    /// * `uri` - The `UUri` to check if it is of type RPC method.
    ///
    /// # Returns
    /// Returns `true` if the URI is of type RPC.
    pub fn is_rpc_method(uri: &UUri) -> bool {
        !Self::is_empty(uri)
            && uri.resource.as_ref().unwrap().name.contains("rpc")
            && (uri.resource.as_ref().unwrap().instance.is_some()
                && !uri
                    .resource
                    .as_ref()
                    .unwrap()
                    .instance
                    .as_ref()
                    .unwrap()
                    .trim()
                    .is_empty()
                || uri.resource.as_ref().unwrap().id.is_some()
                    && (uri.resource.as_ref().unwrap().id.unwrap() != 0))
    }

    /// Checks if the URI is of type RPC response.
    ///
    /// # Arguments
    /// * `uri` - The `UUri` to check if it is a response for an RPC method.
    ///
    /// # Returns
    /// Returns `true` if the URI is of type RPC response.
    pub fn is_rpc_response(uri: &UUri) -> bool {
        Self::is_rpc_method(uri)
            && ((uri.resource.as_ref().unwrap().instance.is_some()
                && uri
                    .resource
                    .as_ref()
                    .unwrap()
                    .instance
                    .as_ref()
                    .unwrap()
                    .contains("response"))
                || uri.resource.as_ref().unwrap().id.is_some()
                    && (uri.resource.as_ref().unwrap().id.unwrap() != 0))
    }

    /// Checks if a `UAuthority` is of type remote
    ///
    /// # Arguments
    /// * `authority` - The `UAuthority` to check if.
    ///
    /// # Returns
    /// Returns `true` if the `UAuthority` is of type remote.
    pub fn is_remote(authority: &UAuthority) -> bool {
        authority.remote.is_some()
    }

    /// Checks if the URI contains numbers so that it can be serialized into micro format.
    ///
    /// # Arguments
    /// * `uri` - The `UUri` to check.
    ///
    /// # Returns
    /// Returns `true` if the URI contains numbers, allowing it to be serialized into micro format.
    pub fn is_micro_form(uri: &UUri) -> bool {
        // return !isEmpty(uri) && uri.getEntity().hasId() && uri.getResource().hasId() && (!uri.hasAuthority() || uri.getAuthority().hasIp() || uri.getAuthority().hasId());
        !Self::is_empty(uri)
            && uri.entity.as_ref().unwrap().id.is_some()
            && uri.resource.as_ref().unwrap().id.is_some()
            && (uri.authority.as_ref().unwrap().remote.is_none())
    }

    /// Checks if the URI contains names so that it can be serialized into long format.
    ///
    /// # Arguments
    /// * `uri` - The `UUri` to check.
    ///
    /// # Returns
    /// Returns `true` if the URI contains names, allowing it to be serialized into long format.
    pub fn is_long_form(uri: &UUri) -> bool {
        if Self::is_empty(uri) {
            return false;
        }

        let mut aname = String::new();
        if let Some(authority) = &uri.authority {
            if let Some(crate::uprotocol::u_authority::Remote::Name(name)) = &authority.remote {
                aname = name.to_string();
            }
        }

        let mut ename = String::new();
        if let Some(entity) = &uri.entity {
            ename = entity.name.to_string();
        }

        let mut rname = String::new();
        if let Some(resource) = &uri.resource {
            rname = resource.name.to_string();
        }

        !aname.is_empty() && !ename.trim().is_empty() && !rname.trim().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Error, Value};
    use std::fs;

    use crate::{
        uprotocol::{UEntity, UResource},
        uri::serializer::{LongUriSerializer, UriSerializer},
    };

    #[test]
    fn test_validate_blank_uri() {
        let uri = LongUriSerializer::deserialize("".to_string());
        let status = UriValidator::validate(&uri);
        assert!(UriValidator::is_empty(&uri));
        assert_eq!("Uri is empty.", status.get_message());
    }

    #[test]
    fn test_validate_uri_with_get_entity() {
        let uri = LongUriSerializer::deserialize("/hartley".to_string());
        let status = UriValidator::validate(&uri);
        assert!(status.is_success());
    }

    #[test]
    fn test_validate_with_malformed_uri() {
        let uri = LongUriSerializer::deserialize("hartley".to_string());
        let status = UriValidator::validate(&uri);
        assert!(UriValidator::is_empty(&uri));
        assert_eq!("Uri is empty.", status.get_message());
    }

    #[test]
    fn test_validate_with_blank_uentity_name_uri() {
        let uri = UUri::default();
        let status = UriValidator::validate(&uri);
        assert!(status.is_failure());
        assert_eq!("Uri is empty.", status.get_message());
    }

    #[test]
    fn test_validate_rpc_method_with_valid_uri() {
        let uri = LongUriSerializer::deserialize("/hartley//rpc.echo".to_string());
        let status = UriValidator::validate_rpc_method(&uri);
        assert!(status.is_success());
    }

    #[test]
    fn test_validate_rpc_method_with_invalid_uri() {
        let uri = LongUriSerializer::deserialize("/hartley/echo".to_string());
        let status = UriValidator::validate_rpc_method(&uri);
        assert!(status.is_failure());
        assert_eq!("Uri is empty.", status.get_message());
    }

    #[test]
    fn test_validate_rpc_method_with_malformed_uri() {
        let uri = LongUriSerializer::deserialize("hartley".to_string());
        let status = UriValidator::validate_rpc_method(&uri);
        assert!(UriValidator::is_empty(&uri));
        assert!(status.is_failure());
        assert_eq!("Uri is empty.", status.get_message());
    }

    #[test]
    fn test_validate_rpc_response_with_valid_uri() {
        let uri = LongUriSerializer::deserialize("/hartley//rpc.response".to_string());
        let status = UriValidator::validate_rpc_response(&uri);
        assert!(status.is_success());
    }

    #[test]
    fn test_validate_rpc_response_with_malformed_uri() {
        let uri = LongUriSerializer::deserialize("hartley".to_string());
        let status = UriValidator::validate_rpc_response(&uri);
        assert!(UriValidator::is_empty(&uri));
        assert!(status.is_failure());
        assert_eq!("Uri is empty.", status.get_message());
    }

    #[test]
    fn test_validate_rpc_response_with_rpc_type() {
        let uri = LongUriSerializer::deserialize("/hartley//dummy.wrong".to_string());
        let status = UriValidator::validate_rpc_response(&uri);
        assert!(status.is_failure());
        assert_eq!("Invalid RPC response type.", status.get_message());
    }

    #[test]
    fn test_validate_rpc_response_with_invalid_rpc_response_type() {
        let uri = LongUriSerializer::deserialize("/hartley//rpc.wrong".to_string());
        let status = UriValidator::validate_rpc_response(&uri);
        assert!(status.is_failure());
        assert_eq!("Invalid RPC response type.", status.get_message());
    }

    #[test]
    fn test_topic_uri_with_version_when_it_is_valid_remote() {
        let uri = "//VCU.MY_CAR_VIN/body.access/1/door.front_left#Door".to_string();
        let status = UriValidator::validate(&LongUriSerializer::deserialize(uri));
        assert!(status.is_success());
    }

    #[test]
    fn test_topic_uri_no_version_when_it_is_valid_remote() {
        let uri = "//VCU.MY_CAR_VIN/body.access//door.front_left#Door".to_string();
        let status = UriValidator::validate(&LongUriSerializer::deserialize(uri));
        assert!(status.is_success());
    }

    #[test]
    fn test_topic_uri_with_version_when_it_is_valid_local() {
        let uri = "/body.access/1/door.front_left#Door".to_string();
        let status = UriValidator::validate(&LongUriSerializer::deserialize(uri));
        assert!(status.is_success());
    }

    #[test]
    fn test_topic_uri_no_version_when_it_is_valid_local() {
        let uri = "/body.access//door.front_left#Door".to_string();
        let status = UriValidator::validate(&LongUriSerializer::deserialize(uri));
        assert!(status.is_success());
    }

    #[test]
    fn test_topic_uri_invalid_when_uri_has_schema_only() {
        let uri = ":".to_string();
        let status = UriValidator::validate(&LongUriSerializer::deserialize(uri));
        assert!(status.is_failure());
    }

    #[test]
    fn test_topic_uri_invalid_when_uri_has_empty_use_name_local() {
        let uri = "/".to_string();
        let status = UriValidator::validate(&LongUriSerializer::deserialize(uri));
        assert!(status.is_failure());
    }

    #[test]
    fn test_topic_uri_invalid_when_uri_is_remote_no_authority() {
        let uri = "//".to_string();
        let status = UriValidator::validate(&LongUriSerializer::deserialize(uri));
        assert!(status.is_failure());
    }

    #[test]
    fn test_topic_uri_invalid_when_uri_is_remote_no_authority_with_use() {
        let uri = "///body.access/1/door.front_left#Door".to_string();
        let status = UriValidator::validate(&LongUriSerializer::deserialize(uri));
        assert!(status.is_failure());
    }

    #[test]
    fn test_topic_uri_invalid_when_uri_is_missing_use_remote() {
        let uri = "//VCU.myvin///door.front_left#Door".to_string();
        let status = UriValidator::validate(&LongUriSerializer::deserialize(uri));
        assert!(status.is_failure());
    }

    #[test]
    fn test_topic_uri_invalid_when_uri_is_missing_use_name_remote() {
        let uri = "/1/door.front_left#Door".to_string();
        let status = UriValidator::validate(&LongUriSerializer::deserialize(uri));
        assert!(status.is_failure());
    }

    #[test]
    fn test_topic_uri_invalid_when_uri_is_missing_use_name_local() {
        let uri = "//VCU.myvin//1".to_string();
        let status = UriValidator::validate(&LongUriSerializer::deserialize(uri));
        assert!(status.is_failure());
    }

    #[test]
    fn test_rpc_topic_uri_with_version_when_it_is_valid_remote() {
        let uri = "//bo.cloud/petapp/1/rpc.response".to_string();
        let status = UriValidator::validate_rpc_method(&LongUriSerializer::deserialize(uri));
        assert!(status.is_success());
    }

    #[test]
    fn test_rpc_topic_uri_no_version_when_it_is_valid_remote() {
        let uri = "//bo.cloud/petapp//rpc.response".to_string();
        let status = UriValidator::validate_rpc_method(&LongUriSerializer::deserialize(uri));
        assert!(status.is_success());
    }

    #[test]
    fn test_rpc_topic_uri_with_version_when_it_is_valid_local() {
        let uri = "/petapp/1/rpc.response".to_string();
        let status = UriValidator::validate_rpc_method(&LongUriSerializer::deserialize(uri));
        assert!(status.is_success());
    }

    #[test]
    fn test_rpc_topic_uri_no_version_when_it_is_valid_local() {
        let uri = "/petapp//rpc.response".to_string();
        let status = UriValidator::validate_rpc_method(&LongUriSerializer::deserialize(uri));
        assert!(status.is_success());
    }

    #[test]
    fn test_rpc_topic_uri_invalid_when_uri_has_schema_only() {
        let uri = ":".to_string();
        let status = UriValidator::validate_rpc_method(&LongUriSerializer::deserialize(uri));
        assert!(status.is_failure());
    }

    #[test]
    fn test_rpc_topic_uri_with_version_when_it_is_not_valid_missing_rpc_response_local() {
        let uri = "/petapp/1/dog".to_string();
        let status = UriValidator::validate_rpc_method(&LongUriSerializer::deserialize(uri));
        assert!(status.is_failure());
    }

    #[test]
    fn test_rpc_topic_uri_with_version_when_it_is_not_valid_missing_rpc_response_remote() {
        let uri = "//petapp/1/dog".to_string();
        let status = UriValidator::validate_rpc_method(&LongUriSerializer::deserialize(uri));
        assert!(status.is_failure());
    }

    #[test]
    fn test_rpc_topic_uri_invalid_when_uri_is_remote_no_authority() {
        let uri = "//".to_string();
        let status = UriValidator::validate_rpc_method(&LongUriSerializer::deserialize(uri));
        assert!(status.is_failure());
    }

    #[test]
    fn test_rpc_topic_uri_invalid_when_uri_is_remote_no_authority_with_use() {
        let uri = "///body.access/1".to_string();
        let status = UriValidator::validate_rpc_method(&LongUriSerializer::deserialize(uri));
        assert!(status.is_failure());
    }

    #[test]
    fn test_rpc_topic_uri_invalid_when_uri_is_missing_use() {
        let uri = "//VCU.myvin".to_string();
        let status = UriValidator::validate_rpc_method(&LongUriSerializer::deserialize(uri));
        assert!(status.is_failure());
    }

    #[test]
    fn test_rpc_topic_uri_invalid_when_uri_is_missing_use_name_remote() {
        let uri = "/1".to_string();
        let status = UriValidator::validate_rpc_method(&LongUriSerializer::deserialize(uri));
        assert!(status.is_failure());
    }

    #[test]
    fn test_rpc_topic_uri_invalid_when_uri_is_missing_use_name_local() {
        let uri = "//VCU.myvin//1".to_string();
        let status = UriValidator::validate_rpc_method(&LongUriSerializer::deserialize(uri));
        assert!(status.is_failure());
    }

    #[test]
    fn test_rpc_method_uri_with_version_when_it_is_valid_remote() {
        let uri = "//VCU.myvin/body.access/1/rpc.UpdateDoor".to_string();
        let status = UriValidator::validate_rpc_method(&LongUriSerializer::deserialize(uri));
        assert!(status.is_success());
    }

    #[test]
    fn test_rpc_method_uri_no_version_when_it_is_valid_remote() {
        let uri = "//VCU.myvin/body.access//rpc.UpdateDoor".to_string();
        let status = UriValidator::validate_rpc_method(&LongUriSerializer::deserialize(uri));
        assert!(status.is_success());
    }

    #[test]
    fn test_rpc_method_uri_with_version_when_it_is_valid_local() {
        let uri = "/body.access/1/rpc.UpdateDoor".to_string();
        let status = UriValidator::validate_rpc_method(&LongUriSerializer::deserialize(uri));
        assert!(status.is_success());
    }

    #[test]
    fn test_rpc_method_uri_no_version_when_it_is_valid_local() {
        let uri = "/body.access//rpc.UpdateDoor".to_string();
        let status = UriValidator::validate_rpc_method(&LongUriSerializer::deserialize(uri));
        assert!(status.is_success());
    }

    #[test]
    fn test_rpc_method_uri_invalid_when_uri_has_schema_only() {
        let uri = ":".to_string();
        let status = UriValidator::validate_rpc_method(&LongUriSerializer::deserialize(uri));
        assert!(status.is_failure());
    }

    #[test]
    fn test_rpc_method_uri_with_version_when_it_is_not_valid_not_rpc_method_local() {
        let uri = "/body.access//UpdateDoor".to_string();
        let status = UriValidator::validate_rpc_method(&LongUriSerializer::deserialize(uri));
        assert!(status.is_failure());
    }

    #[test]
    fn test_rpc_method_uri_with_version_when_it_is_not_valid_not_rpc_method_remote() {
        let uri = "//body.access/1/UpdateDoor".to_string();
        let status = UriValidator::validate_rpc_method(&LongUriSerializer::deserialize(uri));
        assert!(status.is_failure());
    }

    #[test]
    fn test_rpc_method_uri_invalid_when_uri_is_remote_no_authority() {
        let uri = "//".to_string();
        let status = UriValidator::validate_rpc_method(&LongUriSerializer::deserialize(uri));
        assert!(status.is_failure());
    }

    #[test]
    fn test_rpc_method_uri_invalid_when_uri_is_remote_no_authority_with_use() {
        let uri = "///body.access/1/rpc.UpdateDoor".to_string();
        let uuri = LongUriSerializer::deserialize(uri);
        let status = UriValidator::validate_rpc_method(&uuri);
        assert_eq!("", &uuri.to_string());
        assert!(status.is_failure());
    }

    #[test]
    fn test_rpc_method_uri_invalid_when_uri_is_remote_missing_authority_remotecase() {
        let entity = UEntity {
            name: "body.access".into(),
            ..Default::default()
        };
        let resource = UResource {
            name: "rpc".into(),
            instance: Some("UpdateDoor".into()),
            ..Default::default()
        };
        let uuri = UUri {
            entity: Some(entity),
            resource: Some(resource),
            authority: Some(UAuthority::default()),
        };

        let status = UriValidator::validate_rpc_method(&uuri);
        assert!(status.is_failure());
        assert_eq!("Uri is remote missing uAuthority.", status.get_message());
    }

    #[test]
    fn test_rpc_method_uri_invalid_when_uri_is_missing_use() {
        let uri = "//VCU.myvin".to_string();
        let status = UriValidator::validate_rpc_method(&LongUriSerializer::deserialize(uri));
        assert!(status.is_failure());
    }

    #[test]
    fn test_rpc_method_uri_invalid_when_uri_is_missing_use_name_local() {
        let uri = "/1/rpc.UpdateDoor".to_string();
        let status = UriValidator::validate_rpc_method(&LongUriSerializer::deserialize(uri));
        assert!(status.is_failure());
    }

    #[test]
    fn test_rpc_method_uri_invalid_when_uri_is_missing_use_name_remote() {
        let uri = "//VCU.myvin//1/rpc.UpdateDoor".to_string();
        let status = UriValidator::validate_rpc_method(&LongUriSerializer::deserialize(uri));
        assert!(status.is_failure());
    }

    #[test]
    fn test_all_valid_uris() {
        let json_object = get_json_object().expect("Failed to parse JSON");
        let valid_uris = json_object.get("validUris").unwrap().as_array().unwrap();

        for uri in valid_uris {
            let uuri = LongUriSerializer::deserialize(uri.to_string());
            let status = UriValidator::validate(&uuri);
            assert!(status.is_success());
        }
    }

    #[test]
    fn test_all_invalid_uris() {
        let json_object = get_json_object().expect("Failed to parse JSON");
        let invalid_uris = json_object.get("invalidUris").unwrap().as_array().unwrap();

        for uri_object in invalid_uris {
            let uri = uri_object.get("uri").unwrap().as_str().unwrap();
            let uuri = LongUriSerializer::deserialize(uri.into());
            let status = UriValidator::validate(&uuri);
            assert!(status.is_failure());
            assert_eq!(
                status.get_message(),
                uri_object.get("status_message").unwrap().as_str().unwrap()
            );
        }
    }

    #[test]
    fn test_all_valid_rpc_uris() {
        let json_object = get_json_object().expect("Failed to parse JSON");
        let valid_rpc_uris = json_object.get("validRpcUris").unwrap().as_array().unwrap();

        for uri in valid_rpc_uris {
            let uuri = LongUriSerializer::deserialize(uri.to_string());
            let status = UriValidator::validate_rpc_method(&uuri);
            assert!(status.is_success());
        }
    }

    #[test]
    fn test_all_invalid_rpc_uris() {
        let json_object = get_json_object().expect("Failed to parse JSON");
        let invalid_rpc_uris = json_object
            .get("invalidRpcUris")
            .unwrap()
            .as_array()
            .unwrap();

        for uri_object in invalid_rpc_uris {
            let uri = uri_object.get("uri").unwrap().as_str().unwrap();
            let uuri = LongUriSerializer::deserialize(uri.to_string());
            let status = UriValidator::validate_rpc_method(&uuri);
            assert!(status.is_failure());
            assert_eq!(
                status.get_message(),
                uri_object.get("status_message").unwrap().as_str().unwrap()
            );
        }
    }

    #[test]
    fn test_all_valid_rpc_response_uris() {
        let json_object = get_json_object().expect("Failed to parse JSON");
        let valid_rpc_response_uris = json_object
            .get("validRpcResponseUris")
            .unwrap()
            .as_array()
            .unwrap();

        for uri in valid_rpc_response_uris {
            let uuri = LongUriSerializer::deserialize(uri.to_string());
            let status = UriValidator::validate_rpc_response(&uuri);
            assert!(UriValidator::is_rpc_response(&uuri));
            assert!(status.is_success());
        }
    }

    #[test]
    fn test_valid_rpc_response_uri() {
        let entity = UEntity {
            name: "hartley".into(),
            ..Default::default()
        };
        let resource = UResource {
            name: "rpc".into(),
            id: Some(19999),
            ..Default::default()
        };
        let uuri = UUri {
            entity: Some(entity),
            resource: Some(resource),
            authority: None,
        };

        let status = UriValidator::validate_rpc_response(&uuri);
        assert!(UriValidator::is_rpc_response(&uuri));
        assert!(status.is_success());
    }

    #[test]
    fn test_all_invalid_rpc_response_uris() {
        let json_object = get_json_object().expect("Failed to parse JSON");
        let invalid_rpc_response_uris = json_object
            .get("invalidRpcResponseUris")
            .unwrap()
            .as_array()
            .unwrap();

        for uri in invalid_rpc_response_uris {
            let uuri = LongUriSerializer::deserialize(uri.to_string());
            let status = UriValidator::validate_rpc_response(&uuri);
            assert!(status.is_failure());
        }
    }

    fn get_json_object() -> Result<Value, Error> {
        let current_directory = std::env::current_dir().expect("Failed to get current directory");
        let json_path = current_directory.join("test").join("uris.json");

        let json_string = fs::read_to_string(json_path).expect("Failed to read the JSON file");
        serde_json::from_str(&json_string)
    }
}
