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

use cloudevents::event::SpecVersion;
use cloudevents::{AttributesReader, Event};

use crate::cloudevent::builder::UCloudEventUtils;
use crate::transport::datamodel::{UCode, UStatus};
use crate::types::ValidationResult;
use crate::uprotocol::{UMessageType, UUri};
use crate::uri::serializer::{LongUriSerializer, UriSerializer};
use crate::uri::validator::UriValidator;

/// Validates a CloudEvent
pub trait CloudEventValidator: std::fmt::Display {
    /// Validates the `CloudEvent`. A `CloudEventValidator` instance is obtained according to
    /// the `type` attribute on the `CloudEvent`.
    ///
    /// # Arguments
    ///
    /// * `cloud_event` - The `CloudEvent` to validate.
    ///
    /// # Returns
    ///
    /// Returns a `UStatus` with success, or a `UStatus` with failure containing all the
    /// errors that were found.
    fn validate(&self, cloud_event: &Event) -> UStatus {
        let error_messages: Vec<String> = vec![
            self.validate_version(cloud_event),
            self.validate_id(cloud_event),
            self.validate_source(cloud_event),
            self.validate_type(cloud_event),
            self.validate_sink(cloud_event),
        ]
        .into_iter()
        .filter(|status| status.is_failure())
        .map(|status| status.get_message())
        .collect();

        let error_message = error_messages.join(" ");
        if error_message.is_empty() {
            UStatus::ok()
        } else {
            UStatus::fail_with_msg_and_reason(&error_message, UCode::InvalidArgument)
        }
    }

    /// Validates the version attribute of a `CloudEvent`.
    ///
    /// # Arguments
    ///
    /// * `cloud_event` - The cloud event containing the version to validate.
    ///
    /// # Returns
    ///
    /// Returns a `ValidationResult` containing either a success or a failure with the accompanying error message.
    fn validate_version(&self, cloud_event: &Event) -> ValidationResult {
        let version = cloud_event.specversion();

        if version == SpecVersion::V10 {
            ValidationResult::success()
        } else {
            ValidationResult::failure(&format!(
                "Invalid CloudEvent version [{}]. CloudEvent version must be 1.0.",
                version
            ))
        }
    }

    /// Validates the ID attribute of a `CloudEvent`.
    ///
    /// # Arguments
    ///
    /// * `cloud_event` - The cloud event containing the ID to validate.
    ///
    /// # Returns
    ///
    /// Returns a `ValidationResult` containing either a success or a failure with the accompanying error message.
    fn validate_id(&self, cloud_event: &Event) -> ValidationResult {
        if UCloudEventUtils::is_cloud_event_id(cloud_event) {
            ValidationResult::success()
        } else {
            ValidationResult::failure(&format!(
                "Invalid CloudEvent Id [{}]. CloudEvent Id must be of type UUIDv8.",
                cloud_event.id()
            ))
        }
    }

    /// Validates the source value of a `CloudEvent`.
    ///
    /// # Arguments
    ///
    /// * `cloud_event` - The `CloudEvent` containing the source to validate.
    ///
    /// # Returns
    ///
    /// Returns a `ValidationResult` containing a success or a failure with the error message.
    fn validate_source(&self, cloud_event: &Event) -> ValidationResult;

    /// Validates the type attribute of a `CloudEvent`.
    ///
    /// # Arguments
    ///
    /// * `cloud_event` - The cloud event containing the type to validate.
    ///
    /// # Returns
    ///
    /// Returns a `ValidationResult` containing either a success or a failure with the accompanying error message.
    fn validate_type(&self, cloud_event: &Event) -> ValidationResult;

    /// Validates the sink value of a `CloudEvent` in the default scenario where the sink attribute is optional.
    ///
    /// # Arguments
    ///
    /// * `cloud_event` - The `CloudEvent` containing the sink to validate.
    ///
    /// # Returns
    ///
    /// Returns a `ValidationResult` containing a success or a failure with the error message.
    fn validate_sink(&self, cloud_event: &Event) -> ValidationResult {
        if let Some(sink) = UCloudEventUtils::get_sink(cloud_event) {
            let uri = LongUriSerializer::deserialize(sink.clone());

            let result = self.validate_entity_uri(&uri);
            if result.is_failure() {
                return ValidationResult::failure(&format!(
                    "Invalid CloudEvent sink [{}]. {}",
                    sink,
                    result.get_message()
                ));
            }
        }
        ValidationResult::Success
    }

    /// Validates an `UriPart` for a `Software Entity`. This must have an authority in the case of
    /// a microRemote URI and must also contain the name of the USE (Unified Software Entity).
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI string to validate.
    ///
    /// # Returns
    ///
    /// Returns a `ValidationResult` containing a success or a failure with the error message.
    fn validate_entity_uri(&self, uri: &UUri) -> ValidationResult {
        UriValidator::validate(uri)
    }

    /// Validates a `UriPart` that is to be used as a topic in publish scenarios for events such as
    /// "publish", "file", and "notification".
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI string (or part) to validate.
    ///
    /// # Returns
    ///
    /// Returns a `ValidationResult` containing a success or a failure with the error message.
    fn validate_topic_uri(&self, uri: &UUri) -> ValidationResult {
        let result = self.validate_entity_uri(uri);
        if result.is_failure() {
            return result;
        }

        if let Some(resource) = &uri.resource {
            if resource.name.trim().is_empty() {
                return ValidationResult::failure("UriPart is missing uResource name.");
            }
            if resource.message.is_none() {
                return ValidationResult::failure("UriPart is missing Message information.");
            }
        }
        ValidationResult::Success
    }

    /// Validates a `UriPart` that is meant to be used as the application response topic for RPC calls.
    ///
    /// Used in Request source values and Response sink values.
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI string (or part) to validate.
    ///
    /// # Returns
    ///
    /// Returns a `ValidationResult` containing a success or a failure with the error message.
    fn validate_rpc_topic_uri(&self, uri: &UUri) -> ValidationResult {
        let result = self.validate_entity_uri(uri);
        if result.is_failure() {
            return ValidationResult::failure(&format!(
                "Invalid RPC uri application response topic. {}",
                result.get_message()
            ));
        }

        if let Some(resource) = &uri.resource {
            if resource.name == "rpc"
                && resource.instance.as_ref().is_some()
                && resource.instance.as_ref().unwrap() == "response"
            {
                return ValidationResult::Success;
            } else {
                return ValidationResult::failure(
                    "Invalid RPC uri application response topic. UriPart is missing rpc.response.",
                );
            }
        }
        ValidationResult::Success
    }

    /// Validates a `UriPart` that is intended to be used as an RPC method URI.
    ///
    /// This is typically used in Request sink values and Response source values.
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI string (or part) to validate.
    ///
    /// # Returns
    ///
    /// Returns a `ValidationResult` containing either a success or a failure with the accompanying error message.
    fn validate_rpc_method(&self, uri: &UUri) -> ValidationResult {
        let result = self.validate_entity_uri(uri);
        if result.is_failure() {
            return ValidationResult::Failure(format!(
                "Invalid RPC method uri. {}",
                result.get_message()
            ));
        }
        if !UriValidator::is_rpc_method(uri) {
            return ValidationResult::Failure("Invalid RPC method uri. UriPart should be the method to be called, or method from response.".to_string());
        }
        ValidationResult::Success
    }
}

/// Enum that hold the implementations of CloudEventValidator according to type.
pub enum CloudEventValidators {
    Response,
    Request,
    Publish,
    Notification,
}

impl CloudEventValidators {
    pub fn validator(&self) -> Box<dyn CloudEventValidator> {
        match self {
            CloudEventValidators::Response => Box::new(ResponseValidator),
            CloudEventValidators::Request => Box::new(RequestValidator),
            CloudEventValidators::Publish => Box::new(PublishValidator),
            CloudEventValidators::Notification => Box::new(NotificationValidator),
        }
    }

    /// Obtains a `CloudEventValidator` according to the `type` attribute in the `CloudEvent`.
    ///
    /// # Arguments
    ///
    /// * `cloud_event` - The `CloudEvent` with the `type` attribute.
    ///
    /// # Returns
    ///
    /// Returns a `CloudEventValidator` according to the `type` attribute in the `CloudEvent`.
    pub fn get_validator(cloud_event: &Event) -> Box<dyn CloudEventValidator> {
        if let Some(message_type) = UMessageType::from_str_name(cloud_event.ty()) {
            match message_type {
                UMessageType::UmessageTypeResponse => return Box::new(ResponseValidator),
                UMessageType::UmessageTypeRequest => return Box::new(RequestValidator),
                _ => return Box::new(PublishValidator),
            }
        }
        Box::new(PublishValidator)
    }
}

/// Implements Validations for a CloudEvent of type Publish.
struct PublishValidator;
impl CloudEventValidator for PublishValidator {
    fn validate_source(&self, cloud_event: &Event) -> ValidationResult {
        let source = LongUriSerializer::deserialize(cloud_event.source().to_string());
        let result = self.validate_topic_uri(&source);
        if result.is_failure() {
            return ValidationResult::failure(&format!(
                "Invalid Publish type CloudEvent source [{}]. {}",
                source,
                result.get_message()
            ));
        }
        ValidationResult::Success
    }

    fn validate_type(&self, cloud_event: &Event) -> ValidationResult {
        if let Some(event_type) = UMessageType::from_str_name(cloud_event.ty()) {
            if event_type.eq(&UMessageType::UmessageTypePublish) {
                return ValidationResult::Success;
            }
        }
        ValidationResult::failure(&format!(
            "Invalid CloudEvent type [{}]. CloudEvent of type Publish must have a type of 'pub.v1'",
            cloud_event.ty(),
        ))
    }
}

impl std::fmt::Display for PublishValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "CloudEventValidator.Publish")
    }
}

/// Implements Validations for a CloudEvent of type Publish that behaves as a Notification, meaning it must have a sink.
struct NotificationValidator;
impl CloudEventValidator for NotificationValidator {
    fn validate_source(&self, cloud_event: &Event) -> ValidationResult {
        PublishValidator.validate_source(cloud_event)
    }

    fn validate_type(&self, cloud_event: &Event) -> ValidationResult {
        PublishValidator.validate_type(cloud_event)
    }

    fn validate_sink(&self, cloud_event: &Event) -> ValidationResult {
        if let Some(sink) = UCloudEventUtils::get_sink(cloud_event) {
            let uri = LongUriSerializer::deserialize(sink.clone());
            let result = self.validate_entity_uri(&uri);
            if result.is_failure() {
                return ValidationResult::failure(&format!(
                    "Invalid Notification type CloudEvent sink [{}]. {}",
                    sink,
                    result.get_message()
                ));
            }
        } else {
            return ValidationResult::failure(
                "Invalid CloudEvent sink. Notification CloudEvent sink must be an uri.",
            );
        }

        ValidationResult::Success
    }
}

impl std::fmt::Display for NotificationValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "CloudEventValidator.Notification")
    }
}

/// Implements Validations for a CloudEvent for RPC Request.
struct RequestValidator;
impl CloudEventValidator for RequestValidator {
    fn validate_source(&self, cloud_event: &Event) -> ValidationResult {
        let source = cloud_event.source();
        let uri = LongUriSerializer::deserialize(source.clone());
        let result = self.validate_rpc_topic_uri(&uri);
        if result.is_failure() {
            return ValidationResult::failure(&format!(
                "Invalid RPC Request CloudEvent source [{}]. {}",
                source,
                result.get_message()
            ));
        }
        ValidationResult::Success
    }

    fn validate_sink(&self, cloud_event: &Event) -> ValidationResult {
        if let Some(sink) = UCloudEventUtils::get_sink(cloud_event) {
            let uri = LongUriSerializer::deserialize(sink.clone());
            let result = self.validate_rpc_method(&uri);
            if result.is_failure() {
                return ValidationResult::failure(&format!(
                    "Invalid RPC Request CloudEvent sink [{}]. {}",
                    sink,
                    result.get_message()
                ));
            }
        } else {
            return ValidationResult::failure(
                "Invalid RPC Request CloudEvent sink. Request CloudEvent sink must be uri for the method to be called.",
            );
        }

        ValidationResult::Success
    }

    fn validate_type(&self, cloud_event: &Event) -> ValidationResult {
        if let Some(event_type) = UMessageType::from_str_name(cloud_event.ty()) {
            if event_type.eq(&UMessageType::UmessageTypeRequest) {
                return ValidationResult::Success;
            }
        }
        ValidationResult::failure(&format!(
            "Invalid CloudEvent type [{}]. CloudEvent of type Request must have a type of 'req.v1'",
            cloud_event.ty(),
        ))
    }
}

impl std::fmt::Display for RequestValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "CloudEventValidator.Request")
    }
}

/// Implements Validations for a CloudEvent for RPC Response.
struct ResponseValidator;
impl CloudEventValidator for ResponseValidator {
    fn validate_source(&self, cloud_event: &Event) -> ValidationResult {
        let source = cloud_event.source();
        let uri = LongUriSerializer::deserialize(source.clone());
        let result = self.validate_rpc_method(&uri);
        if result.is_failure() {
            return ValidationResult::failure(&format!(
                "Invalid RPC Response CloudEvent source [{}]. {}",
                source,
                result.get_message()
            ));
        }
        ValidationResult::Success
    }

    fn validate_sink(&self, cloud_event: &Event) -> ValidationResult {
        if let Some(sink) = UCloudEventUtils::get_sink(cloud_event) {
            let uri = LongUriSerializer::deserialize(sink.clone());
            let result = self.validate_rpc_topic_uri(&uri);
            if result.is_failure() {
                return ValidationResult::failure(&format!(
                    "Invalid RPC Response CloudEvent sink [{}]. {}",
                    sink,
                    result.get_message()
                ));
            }
        } else {
            return ValidationResult::failure(
                "Invalid RPC Response CloudEvent sink. Response CloudEvent sink must be uri of the destination of the response.",
            );
        }
        ValidationResult::Success
    }

    fn validate_type(&self, cloud_event: &Event) -> ValidationResult {
        if let Some(event_type) = UMessageType::from_str_name(cloud_event.ty()) {
            if event_type.eq(&UMessageType::UmessageTypeResponse) {
                return ValidationResult::Success;
            }
        }
        ValidationResult::failure(&format!(
            "Invalid CloudEvent type [{}]. CloudEvent of type Response must have a type of 'res.v1'",
            cloud_event.ty(),
        ))
    }
}

impl std::fmt::Display for ResponseValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "CloudEventValidator.Response")
    }
}

#[cfg(test)]
mod tests {
    use crate::cloudevent::builder::UCloudEventBuilder;
    use crate::cloudevent::datamodel::UCloudEventAttributesBuilder;
    use crate::uprotocol::{UAuthority, UEntity, UPriority, UResource};
    use crate::uuid::builder::UUIDv8Builder;

    use super::*;

    use cloudevents::{Data, EventBuilder, EventBuilderV03, EventBuilderV10};
    use prost::Message;
    use prost_types::Any;
    use uuid::Uuid;

    #[test]
    fn test_get_a_publish_cloud_event_validator() {
        let cloud_event = build_base_cloud_event_for_test();
        let validator: Box<dyn CloudEventValidator> =
            CloudEventValidators::get_validator(&cloud_event);
        let status = validator.validate_type(&cloud_event);

        assert_eq!(status, ValidationResult::Success);
        assert_eq!("CloudEventValidator.Publish", validator.to_string());
    }

    #[test]
    fn test_get_a_notification_cloud_event_validator() {
        let mut cloud_event = build_base_cloud_event_for_test();
        cloud_event.set_extension("sink", "//bo.cloud/petapp");

        let validator: Box<dyn CloudEventValidator> =
            CloudEventValidators::Notification.validator();
        let status = validator.validate_type(&cloud_event);

        assert_eq!(status, ValidationResult::Success);
        assert_eq!("CloudEventValidator.Notification", validator.to_string());
    }

    #[test]
    fn test_publish_cloud_event_type() {
        let builder = build_base_cloud_event_builder_for_test();
        let event = builder
            .ty(UMessageType::UmessageTypeResponse)
            .build()
            .unwrap();

        let validator: Box<dyn CloudEventValidator> = CloudEventValidators::Publish.validator();
        let status = validator.validate_type(&event).to_status();

        assert_eq!(UCode::InvalidArgument, UCode::from(status.code));
        assert_eq!(
            "Invalid CloudEvent type [res.v1]. CloudEvent of type Publish must have a type of 'pub.v1'",
            status.message
        );
    }

    #[test]
    fn test_notification_cloud_event_type() {
        let builder = build_base_cloud_event_builder_for_test();
        let event = builder
            .ty(UMessageType::UmessageTypeResponse)
            .build()
            .unwrap();

        let validator: Box<dyn CloudEventValidator> =
            CloudEventValidators::Notification.validator();
        let status = validator.validate_type(&event).to_status();

        assert_eq!(UCode::InvalidArgument, UCode::from(status.code));
        assert_eq!(
        "Invalid CloudEvent type [res.v1]. CloudEvent of type Publish must have a type of 'pub.v1'",
        status.message
    );
    }

    #[test]
    fn test_get_a_request_cloud_event_validator() {
        let builder = build_base_cloud_event_builder_for_test();
        let event = builder
            .ty(UMessageType::UmessageTypeRequest)
            .build()
            .unwrap();

        let validator: Box<dyn CloudEventValidator> = CloudEventValidators::get_validator(&event);
        let status = validator.validate_type(&event);

        assert_eq!(ValidationResult::Success, status);
        assert_eq!("CloudEventValidator.Request", &validator.to_string());
    }

    #[test]
    fn test_request_cloud_event_type() {
        let builder = build_base_cloud_event_builder_for_test();
        let event = builder
            .ty(UMessageType::UmessageTypePublish)
            .build()
            .unwrap();

        let validator: Box<dyn CloudEventValidator> = CloudEventValidators::Request.validator();
        let status = validator.validate_type(&event).to_status();

        assert_eq!(UCode::InvalidArgument, UCode::from(status.code));
        assert_eq!(
        "Invalid CloudEvent type [pub.v1]. CloudEvent of type Request must have a type of 'req.v1'",
        status.message
    );
    }

    #[test]
    fn test_get_a_response_cloud_event_validator() {
        let builder = build_base_cloud_event_builder_for_test();
        let event = builder
            .ty(UMessageType::UmessageTypeResponse)
            .build()
            .unwrap();

        let validator: Box<dyn CloudEventValidator> = CloudEventValidators::get_validator(&event);
        let status = validator.validate_type(&event);

        assert_eq!(ValidationResult::success(), status);
        assert_eq!("CloudEventValidator.Response", &validator.to_string());
    }

    #[test]
    fn test_response_cloud_event_type() {
        let builder = build_base_cloud_event_builder_for_test();
        let event = builder
            .ty(UMessageType::UmessageTypePublish)
            .build()
            .unwrap();

        let validator: Box<dyn CloudEventValidator> = CloudEventValidators::Response.validator();
        let status = validator.validate_type(&event).to_status();

        assert_eq!(UCode::InvalidArgument, UCode::from(status.code));
        assert_eq!(
            "Invalid CloudEvent type [pub.v1]. CloudEvent of type Response must have a type of 'res.v1'",
            status.message
        );
    }

    #[test]
    fn test_get_a_publish_cloud_event_validator_when_cloud_event_type_is_unknown() {
        let builder = build_base_cloud_event_builder_for_test();
        let event = builder.ty("lala.v1".to_string()).build().unwrap();

        let validator: Box<dyn CloudEventValidator> = CloudEventValidators::get_validator(&event);
        assert_eq!("CloudEventValidator.Publish", &validator.to_string());
    }

    #[test]
    fn validate_cloud_event_version_when_valid() {
        let uuid = UUIDv8Builder::new().build();
        let builder = build_base_cloud_event_builder_for_test()
            .ty(UMessageType::UmessageTypePublish)
            .id(uuid.to_string());
        let event = builder.build().unwrap();

        let validator = CloudEventValidators::get_validator(&event);
        let status = validator.validate_version(&event);
        assert_eq!(ValidationResult::Success, status);
    }

    #[test]
    fn validate_cloud_event_version_when_not_valid() {
        let builder = EventBuilderV03::new()
            .id("id".to_string())
            .ty(UMessageType::UmessageTypePublish)
            .source("/body.access".to_string());

        let event = builder.build().unwrap();
        let validator = CloudEventValidators::get_validator(&event);
        let status = validator.validate_version(&event).to_status();

        assert_eq!(UCode::InvalidArgument, UCode::from(status.code));
        assert_eq!(
            "Invalid CloudEvent version [0.3]. CloudEvent version must be 1.0.",
            status.message
        );
    }

    #[test]
    fn validate_cloud_event_id_when_valid() {
        let uuid = UUIDv8Builder::new().build();
        let builder = build_base_cloud_event_builder_for_test()
            .ty(UMessageType::UmessageTypePublish)
            .id(uuid.to_string());
        let event = builder.build().unwrap();

        let validator = CloudEventValidators::get_validator(&event);
        let status = validator.validate_id(&event);
        assert_eq!(ValidationResult::Success, status);
    }

    #[test]
    fn validate_cloud_event_id_when_not_uuidv6_type_id() {
        let uuid = Uuid::new_v4();

        let builder = build_base_cloud_event_builder_for_test()
            .ty(UMessageType::UmessageTypePublish)
            .id(uuid.to_string());
        let event = builder.build().unwrap();

        let validator = CloudEventValidators::get_validator(&event);
        let status = validator.validate_id(&event).to_status();

        assert_eq!(UCode::InvalidArgument, UCode::from(status.code));
        assert_eq!(
            format!(
                "Invalid CloudEvent Id [{}]. CloudEvent Id must be of type UUIDv8.",
                uuid
            ),
            status.message
        );
    }

    #[test]
    fn validate_cloud_event_id_when_not_valid() {
        let event = build_base_cloud_event_for_test();
        let status = CloudEventValidators::get_validator(&event)
            .validate_id(&event)
            .to_status();

        assert_eq!(UCode::InvalidArgument, UCode::from(status.code));
        assert_eq!(
            "Invalid CloudEvent Id [testme]. CloudEvent Id must be of type UUIDv8.",
            status.message
        );
    }

    #[test]
    fn test_publish_type_cloudevent_is_valid_when_everything_is_valid_local() {
        let uuid = UUIDv8Builder::new().build();
        let event = build_base_cloud_event_builder_for_test()
            .id(uuid.to_string())
            .source("/body.access/1/door.front_left#Door".to_string())
            .ty(UMessageType::UmessageTypePublish)
            .build()
            .unwrap();

        let validator = CloudEventValidators::Publish.validator();
        let status = validator.validate(&event);
        assert_eq!(UStatus::ok(), status);
    }

    #[test]
    fn test_publish_type_cloudevent_is_valid_when_everything_is_valid_remote() {
        let uuid = UUIDv8Builder::new().build();
        let uri = LongUriSerializer::deserialize(
            ", //VCU.myvin/body.access/1/door.front_left#Door".to_string(),
        );
        let event = build_base_cloud_event_builder_for_test()
            .id(uuid.to_string())
            .source(uri.to_string())
            .ty(UMessageType::UmessageTypePublish)
            .build()
            .unwrap();

        let validator = CloudEventValidators::get_validator(&event);
        let status = validator.validate(&event);

        assert_eq!(UStatus::ok(), status);
    }

    #[test]
    fn test_publish_type_cloudevent_is_valid_when_everything_is_valid_remote_with_a_sink() {
        let uuid = UUIDv8Builder::new().build();
        let uri = LongUriSerializer::deserialize(
            "//VCU.myvin/body.access/1/door.front_left#Door".to_string(),
        );
        let sink = LongUriSerializer::deserialize("//bo.cloud/petapp".to_string());
        let event = build_base_cloud_event_builder_for_test()
            .id(uuid.to_string())
            .source(uri.to_string())
            .extension("sink", sink.to_string())
            .ty(UMessageType::UmessageTypePublish)
            .build()
            .unwrap();

        let validator = CloudEventValidators::get_validator(&event);
        let status = validator.validate(&event);

        assert_eq!(UStatus::ok(), status);
    }

    #[test]
    fn test_publish_type_cloudevent_is_not_valid_when_remote_with_invalid_sink() {
        let uuid = UUIDv8Builder::new().build();
        let uri = LongUriSerializer::deserialize(
            "//VCU.myvin/body.access/1/door.front_left#Door".to_string(),
        );
        let sink = LongUriSerializer::deserialize("//bo.cloud".to_string());
        let event = build_base_cloud_event_builder_for_test()
            .id(uuid.to_string())
            .source(uri.to_string())
            .extension("sink", sink.to_string())
            .ty(UMessageType::UmessageTypePublish)
            .build()
            .unwrap();

        let validator = CloudEventValidators::get_validator(&event);
        let status = validator.validate(&event);

        assert_eq!(UCode::InvalidArgument, status.code());
        assert_eq!(
            "Invalid CloudEvent sink [//bo.cloud/]. UriPart is missing uSoftware Entity name.",
            status.message()
        );
    }

    #[test]
    fn test_publish_type_cloudevent_is_not_valid_when_source_is_empty() {
        let uuid = UUIDv8Builder::new().build();
        let event = build_base_cloud_event_builder_for_test()
            .id(uuid.to_string())
            .source("/".to_string())
            .ty(UMessageType::UmessageTypePublish)
            .build()
            .unwrap();

        let validator = CloudEventValidators::get_validator(&event);
        let status = validator.validate(&event);

        assert_eq!(UCode::InvalidArgument, status.code());
        assert_eq!(
            "Invalid Publish type CloudEvent source []. UriPart is missing uSoftware Entity name.",
            status.message()
        );
    }

    #[test]
    fn test_publish_type_cloudevent_is_not_valid_when_source_is_missing_authority() {
        let uri = LongUriSerializer::deserialize("/body.access".to_string());

        let event = build_base_cloud_event_builder_for_test()
            .id("testme".to_string())
            .source(uri.to_string())
            .ty(UMessageType::UmessageTypePublish)
            .build()
            .unwrap();

        let validator = CloudEventValidators::get_validator(&event);
        let status = validator.validate(&event);

        assert_eq!(UCode::InvalidArgument, status.code());
        assert_eq!(
            "Invalid CloudEvent Id [testme]. CloudEvent Id must be of type UUIDv8. \
        Invalid Publish type CloudEvent source [/body.access]. UriPart is missing uResource name.",
            status.message()
        );
    }

    #[test]
    fn test_publish_type_cloudevent_is_not_valid_when_source_is_missing_message_info() {
        let uri = LongUriSerializer::deserialize("/body.access/1/door.front_left".to_string());

        let event = build_base_cloud_event_builder_for_test()
            .id("testme".to_string())
            .source(uri.to_string())
            .ty(UMessageType::UmessageTypePublish)
            .build()
            .unwrap();

        let validator = CloudEventValidators::get_validator(&event);
        let status = validator.validate(&event);

        assert_eq!(UCode::InvalidArgument, status.code());
        assert_eq!(
            "Invalid CloudEvent Id [testme]. CloudEvent Id must be of type UUIDv8. \
        Invalid Publish type CloudEvent source [/body.access/1/door.front_left]. UriPart is missing uResource name.",
            status.message()
        );
    }

    #[test]
    fn test_notification_type_cloudevent_is_valid_when_everything_is_valid() {
        let uuid = UUIDv8Builder::new().build();
        let uri = LongUriSerializer::deserialize("/body.access/1/door.front_left#Door".to_string());
        let sink = LongUriSerializer::deserialize("//bo.cloud/petapp".to_string());
        let event = build_base_cloud_event_builder_for_test()
            .id(uuid.to_string())
            .source(uri.to_string())
            .extension("sink", sink.to_string())
            .ty(UMessageType::UmessageTypePublish)
            .build()
            .unwrap();

        let validator = CloudEventValidators::validator(&CloudEventValidators::Notification);
        let status = validator.validate(&event);

        assert_eq!(UStatus::ok(), status);
    }

    #[test]
    fn test_notification_type_cloudevent_is_not_valid_missing_sink() {
        let uuid = UUIDv8Builder::new().build();
        let uri = LongUriSerializer::deserialize("/body.access/1/door.front_left#Door".to_string());
        let event = build_base_cloud_event_builder_for_test()
            .id(uuid.to_string())
            .source(uri.to_string())
            .ty(UMessageType::UmessageTypePublish)
            .build()
            .unwrap();

        let validator = CloudEventValidators::validator(&CloudEventValidators::Notification);
        let status = validator.validate(&event);

        assert_eq!(UCode::InvalidArgument, status.code());
        assert_eq!(
            "Invalid CloudEvent sink. Notification CloudEvent sink must be an uri.",
            status.message()
        );
    }

    #[test]
    fn test_notification_type_cloudevent_is_not_valid_invalid_sink() {
        let uuid = UUIDv8Builder::new().build();
        let uri = LongUriSerializer::deserialize("/body.access/1/door.front_left#Door".to_string());
        let sink = LongUriSerializer::deserialize("//bo.cloud".to_string());
        let event = build_base_cloud_event_builder_for_test()
            .id(uuid.to_string())
            .source(uri.to_string())
            .extension("sink", sink.to_string())
            .ty(UMessageType::UmessageTypePublish)
            .build()
            .unwrap();

        let validator = CloudEventValidators::validator(&CloudEventValidators::Notification);
        let status = validator.validate(&event);

        assert_eq!(UCode::InvalidArgument, status.code());
        assert_eq!(
        "Invalid Notification type CloudEvent sink [//bo.cloud/]. UriPart is missing uSoftware Entity name.",
        status.message()
        );
    }

    #[test]
    fn test_request_type_cloudevent_is_valid_when_everything_is_valid() {
        let uuid = UUIDv8Builder::new().build();
        let source = LongUriSerializer::deserialize("//bo.cloud/petapp//rpc.response".to_string());
        let sink =
            LongUriSerializer::deserialize("//VCU.myvin/body.access/1/rpc.UpdateDoor".to_string());
        let event = build_base_cloud_event_builder_for_test()
            .id(uuid.to_string())
            .source(source.to_string())
            .extension("sink", sink.to_string())
            .ty(UMessageType::UmessageTypeRequest)
            .build()
            .unwrap();

        let validator = CloudEventValidators::validator(&CloudEventValidators::Request);
        let status = validator.validate(&event);

        assert_eq!(UStatus::ok(), status);
    }

    #[test]
    fn test_request_type_cloudevent_is_not_valid_invalid_source() {
        let uuid = UUIDv8Builder::new().build();
        let source = LongUriSerializer::deserialize("//bo.cloud/petapp//dog".to_string());
        let sink =
            LongUriSerializer::deserialize("//VCU.myvin/body.access/1/rpc.UpdateDoor".to_string());
        let event = build_base_cloud_event_builder_for_test()
            .id(uuid.to_string())
            .source(source.to_string())
            .extension("sink", sink.to_string())
            .ty(UMessageType::UmessageTypeRequest)
            .build()
            .unwrap();

        let validator = CloudEventValidators::validator(&CloudEventValidators::Request);
        let status = validator.validate(&event);

        assert_eq!(UCode::InvalidArgument, status.code());
        assert_eq!(
            "Invalid RPC Request CloudEvent source [//bo.cloud/petapp//dog]. Invalid RPC uri application response topic. UriPart is missing rpc.response.",
            status.message()
        );
    }

    #[test]
    fn test_request_type_cloudevent_is_not_valid_missing_sink() {
        let uuid = UUIDv8Builder::new().build();
        let source = LongUriSerializer::deserialize("//bo.cloud/petapp//rpc.response".to_string());
        let event = build_base_cloud_event_builder_for_test()
            .id(uuid.to_string())
            .source(source.to_string())
            .ty(UMessageType::UmessageTypeRequest)
            .build()
            .unwrap();

        let validator = CloudEventValidators::validator(&CloudEventValidators::Request);
        let status = validator.validate(&event);

        assert_eq!(UCode::InvalidArgument, status.code());
        assert_eq!(
            "Invalid RPC Request CloudEvent sink. Request CloudEvent sink must be uri for the method to be called.",
            status.message()
        );
    }

    #[test]
    fn test_request_type_cloudevent_is_not_valid_invalid_sink_not_rpc_command() {
        let uuid = UUIDv8Builder::new().build();
        let source = LongUriSerializer::deserialize("//bo.cloud/petapp//rpc.response".to_string());
        let sink =
            LongUriSerializer::deserialize("//VCU.myvin/body.access/1/UpdateDoor".to_string());
        let event = build_base_cloud_event_builder_for_test()
            .id(uuid.to_string())
            .source(source.to_string())
            .extension("sink", sink.to_string())
            .ty(UMessageType::UmessageTypeRequest)
            .build()
            .unwrap();

        let validator = CloudEventValidators::validator(&CloudEventValidators::Request);
        let status = validator.validate(&event);

        assert_eq!(UCode::InvalidArgument, status.code());
        assert_eq!(
            "Invalid RPC Request CloudEvent sink [//vcu.myvin/body.access/1/UpdateDoor]. Invalid RPC method uri. UriPart should be the method to be called, or method from response.",
            status.message()
        );
    }

    #[test]
    fn test_response_type_cloudevent_is_valid_when_everything_is_valid() {
        let uuid = UUIDv8Builder::new().build();
        let source =
            LongUriSerializer::deserialize("//VCU.myvin/body.access/1/rpc.UpdateDoor".to_string());
        let sink = LongUriSerializer::deserialize("//bo.cloud/petapp//rpc.response".to_string());
        let event = build_base_cloud_event_builder_for_test()
            .id(uuid.to_string())
            .source(source.to_string())
            .extension("sink", sink.to_string())
            .ty(UMessageType::UmessageTypeResponse)
            .build()
            .unwrap();

        let validator = CloudEventValidators::validator(&CloudEventValidators::Response);
        let status = validator.validate(&event);

        assert_eq!(UStatus::ok(), status);
    }

    #[test]
    fn test_response_type_cloudevent_is_not_valid_invalid_source() {
        let uuid = UUIDv8Builder::new().build();
        let source =
            LongUriSerializer::deserialize("//VCU.myvin/body.access/1/UpdateDoor".to_string());
        let sink = LongUriSerializer::deserialize("//bo.cloud/petapp//rpc.response".to_string());
        let event = build_base_cloud_event_builder_for_test()
            .id(uuid.to_string())
            .source(source.to_string())
            .extension("sink", sink.to_string())
            .ty(UMessageType::UmessageTypeResponse)
            .build()
            .unwrap();

        let validator = CloudEventValidators::validator(&CloudEventValidators::Response);
        let status = validator.validate(&event);

        assert_eq!(UCode::InvalidArgument, status.code());
        assert_eq!(
            "Invalid RPC Response CloudEvent source [//vcu.myvin/body.access/1/UpdateDoor]. Invalid RPC method uri. UriPart should be the method to be called, or method from response.",
            status.message()
        );
    }

    #[test]
    fn test_response_type_cloudevent_is_not_valid_missing_sink_and_invalid_source() {
        let uuid = UUIDv8Builder::new().build();
        let source =
            LongUriSerializer::deserialize("//VCU.myvin/body.access/1/UpdateDoor".to_string());
        let event = build_base_cloud_event_builder_for_test()
            .id(uuid.to_string())
            .source(source.to_string())
            .ty(UMessageType::UmessageTypeResponse)
            .build()
            .unwrap();

        let validator = CloudEventValidators::validator(&CloudEventValidators::Response);
        let status = validator.validate(&event);

        assert_eq!(UCode::InvalidArgument, status.code());
        assert_eq!(
        "Invalid RPC Response CloudEvent source [//vcu.myvin/body.access/1/UpdateDoor]. Invalid RPC method uri. UriPart should be the method to be called, or method from response. Invalid RPC Response CloudEvent sink. Response CloudEvent sink must be uri of the destination of the response.",
        status.message()
    );
    }

    #[test]
    fn test_response_type_cloudevent_is_not_valid_invalid_source_not_rpc_command() {
        let uuid = UUIDv8Builder::new().build();
        let source = LongUriSerializer::deserialize("//bo.cloud/petapp/1/dog".to_string());
        let sink =
            LongUriSerializer::deserialize("//VCU.myvin/body.access/1/UpdateDoor".to_string());
        let event = build_base_cloud_event_builder_for_test()
            .id(uuid.to_string())
            .source(source.to_string())
            .extension("sink", sink.to_string())
            .ty(UMessageType::UmessageTypeResponse)
            .build()
            .unwrap();

        let validator = CloudEventValidators::validator(&CloudEventValidators::Response);
        let status = validator.validate(&event);

        assert_eq!(UCode::InvalidArgument, status.code());
        assert_eq!(
        "Invalid RPC Response CloudEvent source [//bo.cloud/petapp/1/dog]. Invalid RPC method uri. UriPart should be the method to be called, or method from response. Invalid RPC Response CloudEvent sink [//vcu.myvin/body.access/1/UpdateDoor]. Invalid RPC uri application response topic. UriPart is missing rpc.response.",
        status.message()
    );
    }

    fn build_base_cloud_event_builder_for_test() -> EventBuilderV10 {
        let uri = UUri {
            authority: Some(UAuthority::default()),
            entity: Some(UEntity {
                name: "body.access".to_string(),
                ..Default::default()
            }),
            resource: Some(UResource {
                name: "door".to_string(),
                ..Default::default()
            }),
        };
        let source = LongUriSerializer::serialize(&uri);
        let payload = build_proto_payload_for_test();
        let attributes = UCloudEventAttributesBuilder::new()
            .with_hash("somehash".to_string())
            .with_priority(UPriority::UpriorityCs0)
            .with_ttl(3)
            .with_token("someOAuthToken".to_string())
            .build();

        UCloudEventBuilder::build_base_cloud_event(
            "testme",
            &source,
            &payload.encode_to_vec(),
            &payload.type_url,
            &attributes,
        )
    }

    fn build_base_cloud_event_for_test() -> Event {
        let mut builder = build_base_cloud_event_builder_for_test();
        builder = builder.ty(UMessageType::UmessageTypePublish);
        builder.build().unwrap()
    }

    fn build_proto_payload_for_test() -> Any {
        let event = EventBuilderV10::new()
            .id("hello")
            .source("/body.access")
            .ty(UMessageType::UmessageTypePublish)
            .data_with_schema(
                "application/octet-stream",
                "proto://type.googleapis.com/example.demo",
                Any::default().value,
            )
            .build()
            .unwrap();

        pack_event_into_any(&event)
    }

    fn pack_event_into_any(event: &Event) -> Any {
        let data_bytes: Vec<u8> = match event.data() {
            Some(Data::Binary(bytes)) => bytes.clone(),
            Some(Data::String(s)) => s.as_bytes().to_vec(),
            Some(Data::Json(j)) => j.to_string().into_bytes(),
            None => Vec::new(),
        };

        // The cloudevent crate uses the url crate for storing dataschema, which needs a schema prefix to work,
        // which gets added in UCloudEventBuilder::build_base_cloud_event() or in related test cases.
        // And this schema prefix needs to be removed again here:
        let schema = {
            let temp_schema = event.dataschema().unwrap().to_string();
            temp_schema
                .strip_prefix("proto://")
                .unwrap_or(&temp_schema)
                .to_string()
        };

        prost_types::Any {
            type_url: schema,
            value: data_bytes,
        }
    }
}
