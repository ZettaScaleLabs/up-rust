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

use uuid::Uuid;

use crate::uprotocol::Uuid as uproto_Uuid;
use crate::uuid::serializer::uuidserializer::{UuidSerializationError, UuidSerializer};

pub struct MicroUuidSerializer;

impl UuidSerializer<[u8; 16]> for MicroUuidSerializer {
    fn serialize(uuid: &uproto_Uuid) -> [u8; 16] {
        *Uuid::from(uuid.clone()).as_bytes()
    }

    fn deserialize(uuid: [u8; 16]) -> Result<uproto_Uuid, UuidSerializationError> {
        Ok(Uuid::from_bytes(uuid).into())
    }
}
