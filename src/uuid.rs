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

use std::{hash::Hash, str::FromStr};

pub use crate::up_core_api::uuid::UUID;

mod uuidbuilder;
use uuid_simd::{AsciiCase, Out};
pub use uuidbuilder::UUIDBuilder;

const BITMASK_VERSION: u64 = 0b1111 << 12;
pub(crate) const VERSION_CUSTOM: u64 = 0b1000 << 12;
const BITMASK_VARIANT: u64 = 0b11 << 62;
pub(crate) const VARIANT_RFC4122: u64 = 0b10 << 62;

#[derive(Debug)]
pub struct UuidConversionError {
    message: String,
}

impl UuidConversionError {
    pub fn new<T: Into<String>>(message: T) -> UuidConversionError {
        UuidConversionError {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for UuidConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error converting Uuid: {}", self.message)
    }
}

impl std::error::Error for UuidConversionError {}

impl UUID {
    /// Creates a new UUID from a byte array.
    ///
    /// # Arguments
    ///
    /// `bytes` - the byte array
    ///
    /// # Returns
    ///
    /// a uProtocol [`UUID`] with the given timestamp, counter and random values.
    ///
    /// # Errors
    ///
    /// Returns an error if the given bytes contain an invalid version and/or variant identifier.
    pub(crate) fn from_bytes(bytes: &[u8; 16]) -> Result<Self, UuidConversionError> {
        let mut msb = [0_u8; 8];
        let mut lsb = [0_u8; 8];
        msb.copy_from_slice(&bytes[..8]);
        lsb.copy_from_slice(&bytes[8..]);
        Self::from_u64_pair(u64::from_be_bytes(msb), u64::from_be_bytes(lsb))
    }

    /// Creates a new UUID from a high/low value pair.
    ///
    /// # Arguments
    ///
    /// `msb` - the most significant 8 bytes
    /// `lsb` - the least significant 8 bytes
    ///
    /// # Returns
    ///
    /// a uProtocol [`UUID`] with the given timestamp, counter and random values.
    ///
    /// # Errors
    ///
    /// Returns an error if the given bytes contain an invalid version and/or variant identifier.
    pub(crate) fn from_u64_pair(msb: u64, lsb: u64) -> Result<Self, UuidConversionError> {
        if msb & BITMASK_VERSION != VERSION_CUSTOM {
            return Err(UuidConversionError::new("not a v8 UUID"));
        }
        if lsb & BITMASK_VARIANT != VARIANT_RFC4122 {
            return Err(UuidConversionError::new("not an RFC4122 UUID"));
        }
        Ok(UUID {
            msb,
            lsb,
            ..Default::default()
        })
    }

    /// Serializes this UUID to a hyphenated string as defined by
    /// [RFC 4122, Section 3](https://www.rfc-editor.org/rfc/rfc4122.html#section-3)
    /// using lower case characters.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use up_rust::UUID;
    ///
    /// // timestamp = 1, ver = 0b1000
    /// let msb = 0x0000000000018000_u64;
    /// // variant = 0b10, random = 0x0010101010101a1a
    /// let lsb = 0x8010101010101a1a_u64;
    /// let uuid = UUID { msb, lsb, ..Default::default() };
    /// assert_eq!(uuid.to_hyphenated_string(), "00000000-0001-8000-8010-101010101a1a");
    /// ```
    pub fn to_hyphenated_string(&self) -> String {
        let mut bytes = [0_u8; 16];
        bytes[..8].clone_from_slice(self.msb.to_be_bytes().as_slice());
        bytes[8..].clone_from_slice(self.lsb.to_be_bytes().as_slice());
        let mut out_bytes = [0_u8; 36];
        let out =
            uuid_simd::format_hyphenated(&bytes, Out::from_mut(&mut out_bytes), AsciiCase::Lower);
        String::from_utf8(out.to_vec()).unwrap()
    }

    fn is_custom_version(&self) -> bool {
        self.msb & BITMASK_VERSION == VERSION_CUSTOM
    }

    fn is_rfc_variant(&self) -> bool {
        self.lsb & BITMASK_VARIANT == VARIANT_RFC4122
    }

    /// Returns the point in time that this UUID has been created at.
    ///
    /// # Returns
    ///
    /// The number of milliseconds since UNIX EPOCH if this UUID is a uProtocol UUID,
    /// or [`Option::None`] otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use up_rust::UUID;
    ///
    /// // timestamp = 0x018D548EA8E0 (Monday, 29 January 2024, 9:30:52 AM GMT)
    /// // ver = 0b1000
    /// let msb = 0x018D548EA8E08000u64;
    /// // variant = 0b10
    /// let lsb = 0x8000000000000000u64;
    /// let creation_time = UUID { msb, lsb, ..Default::default() }.get_time();
    /// assert_eq!(creation_time.unwrap(), 0x018D548EA8E0_u64);
    ///
    /// // timestamp = 1, (invalid) ver = 0b1100
    /// let msb = 0x000000000001C000u64;
    /// // variant = 0b10
    /// let lsb = 0x8000000000000000u64;
    /// let creation_time = UUID { msb, lsb, ..Default::default() }.get_time();
    /// assert!(creation_time.is_none());
    /// ```
    pub fn get_time(&self) -> Option<u64> {
        if self.is_uprotocol_uuid() {
            // the timstamp is contained in the 48 most significant bits
            Some(self.msb >> 16)
        } else {
            None
        }
    }

    /// Checks if this is a valid uProtocol UUID.
    ///
    /// # Returns
    ///
    /// `true` if this UUID meets the formal requirements defined by the
    /// [uProtocol spec](https://github.com/eclipse-uprotocol/uprotocol-spec).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use up_rust::UUID;
    ///
    /// // timestamp = 1, ver = 0b1000
    /// let msb = 0x0000000000018000u64;
    /// // variant = 0b10
    /// let lsb = 0x8000000000000000u64;
    /// assert!(UUID { msb, lsb, ..Default::default() }.is_uprotocol_uuid());
    ///
    /// // timestamp = 1, (invalid) ver = 0b1100
    /// let msb = 0x000000000001C000u64;
    /// // variant = 0b10
    /// let lsb = 0x8000000000000000u64;
    /// assert!(!UUID { msb, lsb, ..Default::default() }.is_uprotocol_uuid());
    ///
    /// // timestamp = 1, ver = 0b1000
    /// let msb = 0x0000000000018000u64;
    /// // (invalid) variant = 0b01
    /// let lsb = 0x4000000000000000u64;
    /// assert!(!UUID { msb, lsb, ..Default::default() }.is_uprotocol_uuid());
    /// ```
    pub fn is_uprotocol_uuid(&self) -> bool {
        self.is_custom_version() && self.is_rfc_variant()
    }
}

impl Eq for UUID {}

impl Hash for UUID {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let bytes = (self.msb, self.lsb);
        bytes.hash(state)
    }
}

impl From<UUID> for String {
    fn from(value: UUID) -> Self {
        Self::from(&value)
    }
}

impl From<&UUID> for String {
    fn from(value: &UUID) -> Self {
        value.to_hyphenated_string()
    }
}

impl FromStr for UUID {
    type Err = UuidConversionError;

    /// Parses a string into a UUID.
    ///
    /// # Returns
    ///
    /// a uProtocol [`UUID`] based on the bytes encoded in the string.
    ///
    /// # Errors
    ///
    /// Returns an error
    /// * if the given string does not represent a UUID as defined by
    /// [RFC 4122, Section 3](https://www.rfc-editor.org/rfc/rfc4122.html#section-3), or
    /// * if the bytes encoded in the string contain an invalid version and/or variant identifier.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use up_rust::UUID;
    ///
    /// // parsing a valid uProtocol UUID succeeds
    /// let parsing_attempt = "00000000-0001-8000-8010-101010101a1A".parse::<UUID>();
    /// assert!(parsing_attempt.is_ok());
    /// let uuid = parsing_attempt.unwrap();
    /// assert!(uuid.is_uprotocol_uuid());
    /// assert_eq!(uuid.msb, 0x0000000000018000_u64);
    /// assert_eq!(uuid.lsb, 0x8010101010101a1a_u64);
    ///
    /// // parsing an invalid UUID fails
    /// assert!("a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8"
    ///     .parse::<UUID>()
    ///     .is_err());
    /// ```
    fn from_str(uuid_str: &str) -> Result<Self, Self::Err> {
        let mut uuid = [0u8; 16];
        uuid_simd::parse_hyphenated(uuid_str.as_bytes(), Out::from_mut(&mut uuid))
            .map_err(|err| UuidConversionError::new(err.to_string()))
            .and_then(|bytes| UUID::from_bytes(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_u64_pair() {
        // timestamp = 1, ver = 0b1000
        let msb = 0x0000000000018000u64;
        // variant = 0b10
        let lsb = 0x8000000000000000u64;
        let conversion_attempt = UUID::from_u64_pair(msb, lsb);
        assert!(conversion_attempt.is_ok());
        let uuid = conversion_attempt.unwrap();
        assert!(uuid.is_uprotocol_uuid());
        assert_eq!(uuid.get_time(), Some(0x1_u64));

        // timestamp = 1, (invalid) ver = 0b0000
        let msb = 0x0000000000010000u64;
        // (invalid) variant= 0b00
        let lsb = 0x00000000000000abu64;
        assert!(UUID::from_u64_pair(msb, lsb).is_err());
    }

    #[test]
    fn test_from_bytes() {
        // timestamp = 1, ver = 0b1000, variant = 0b10
        let bytes: [u8; 16] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x80, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
        let conversion_attempt = UUID::from_bytes(&bytes);
        assert!(conversion_attempt.is_ok());
        let uuid = conversion_attempt.unwrap();
        assert!(uuid.is_uprotocol_uuid());
        assert_eq!(uuid.get_time(), Some(0x1_u64));
    }
}
