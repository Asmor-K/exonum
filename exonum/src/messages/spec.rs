#[macro_export]
macro_rules! message {
    ($name:ident {
        const TYPE = $extension:expr;
        const ID = $id:expr;
        const SIZE = $body:expr;

        $($field_name:ident : $field_type:ty [$from:expr => $to:expr])*
    }) => (
        #[derive(Clone, PartialEq)]
        pub struct $name {
            raw: $crate::messages::RawMessage
        }

        impl $crate::messages::Message for $name {
            fn raw(&self) -> &$crate::messages::RawMessage {
                &self.raw
            }
        }

        impl<'a> $crate::messages::Field<'a> for $name {
            fn read(buffer: &'a [u8], from: usize, to: usize) -> Self {
                let raw_message: $crate::messages::RawMessage = $crate::messages::Field::read(buffer, from, to);
                $crate::messages::FromRaw::from_raw(raw_message).unwrap()
            }

            fn write(&self, buffer: &'a mut Vec<u8>, from: usize, to: usize) {
                $crate::messages::Field::write(&self.raw, buffer, from, to);
            }

            fn check(buffer: &'a [u8], from: usize, to: usize) -> Result<(), $crate::messages::Error> {

                let raw_message: $crate::messages::RawMessage = $crate::messages::Field::read(buffer, from, to);
                $(raw_message.check::<$field_type>($from, $to)?;)*
                Ok(())
            }

            fn field_size() -> usize {
                1
            }
        }

        impl $crate::messages::FromRaw for $name {
            fn from_raw(raw: $crate::messages::RawMessage)
                -> Result<$name, $crate::messages::Error> {
                $(raw.check::<$field_type>($from, $to)?;)*
                Ok($name { raw: raw })
            }
        }

        impl $name {
            #![cfg_attr(feature="cargo-clippy", allow(too_many_arguments))]
            pub fn new($($field_name: $field_type,)*
                       secret_key: &$crate::crypto::SecretKey) -> $name {
                use $crate::messages::{RawMessage, MessageWriter};
                let mut writer = MessageWriter::new($extension, $id, $body);
                $(writer.write($field_name, $from, $to);)*
                $name { raw: RawMessage::new(writer.sign(secret_key)) }
            }
            pub fn new_with_signature($($field_name: $field_type,)*
                       signature: &$crate::crypto::Signature) -> $name {
                use $crate::messages::{RawMessage, MessageWriter};
                let mut writer = MessageWriter::new($extension, $id, $body);
                $(writer.write($field_name, $from, $to);)*
                $name { raw: RawMessage::new(writer.append_signature(signature)) }

            }
            $(pub fn $field_name(&self) -> $field_type {
                self.raw.read::<$field_type>($from, $to)
            })*
        }

        impl ::std::fmt::Debug for $name {
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter)
                -> Result<(), ::std::fmt::Error> {
                fmt.debug_struct(stringify!($name))
                 $(.field(stringify!($field_name), &self.$field_name()))*
                   .finish()
            }
        }

        impl $crate::serialize::json::ExonumJsonSerialize for $name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: $crate::serialize::json::reexport::Serializer {
                    use $crate::serialize::json::reexport::SerializeStruct;
                    use $crate::serialize::json;
                    
                    pub struct Body<'a>{_self: &'a $name};
                    impl<'a> $crate::serialize::json::reexport::Serialize for Body<'a> {
                        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                            where S: $crate::serialize::json::reexport::Serializer
                        {
                            let mut structure = serializer.serialize_struct(stringify!($name), counter!($($field_name)*) )?;
                            $(structure.serialize_field(stringify!($field_name), &json::wrap(&self._self.$field_name()))?;)*

                            structure.end()
                        }
                    }
                        
                    let mut structure = serializer.serialize_struct(stringify!($name), 4 )?;
                    structure.serialize_field("body", &Body{_self: &self})?;
                    structure.serialize_field("signature", &json::wrap(self.raw.signature()))?;
                    structure.serialize_field("message_id", &json::wrap(&self.raw.message_type()))?;
                    structure.serialize_field("service_id", &json::wrap(&self.raw.service_id()))?;
                    
                    structure.end()               
                }
        }

        impl $crate::serialize::json::ExonumJsonDeserializeField for $name {
            fn deserialize<B> (value: &$crate::serialize::json::reexport::Value, buffer: & mut B, from: usize, to: usize ) -> Result<(), Box<::std::error::Error>>
            where B: $crate::serialize::json::WriteBufferWrapper
            {
                // deserialize full field
                let structure = Self::deserialize_owned(value)?;
                // then write it
                buffer.write(from, to, structure); 
                Ok(())
            }
        }

        impl $crate::serialize::json::ExonumJsonDeserialize for $name {
            fn deserialize_owned(value: &$crate::serialize::json::reexport::Value) -> Result<Self, Box<::std::error::Error>> {
                use $crate::serialize::json::ExonumJsonDeserializeField;
                use $crate::serialize::json::reexport::from_value;
                use $crate::messages::{RawMessage, MessageWriter};
                
                // if we could deserialize values, try append signature
                let obj = value.as_object().ok_or("Can't cast json as object.")?;
 
                let body = obj.get("body").ok_or("Can't get body from json.")?;
 
                let signature = from_value(obj.get("signature").ok_or("Can't get signature from json")?.clone())?;
                let message_type = from_value(obj.get("message_id").ok_or("Can't get message_type from json")?.clone())?;
                let service_id = from_value(obj.get("service_id").ok_or("Can't get service_id from json")?.clone())?;
                if service_id != $extension {
                    return Err("service_id didn't equal real service_id.".into())
                }

                if message_type != $id {
                    return Err("message_id didn't equal real message_id.".into())
                }

                let mut writer = MessageWriter::new(service_id, message_type, $body);
                let obj = body.as_object().ok_or("Can't cast body as object.")?;
                $(
                    let val = obj.get(stringify!($field_name)).ok_or("Can't get object from json.")?;
                    <$field_type as ExonumJsonDeserializeField>::deserialize(val, &mut writer, $from, $to )?;
                )*

                Ok($name { raw: RawMessage::new(writer.append_signature(&signature)) })
            }
        }

        //\TODO: Rewrite Deserialize and Serializa implementation
        impl<'de> $crate::serialize::json::reexport::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: $crate::serialize::json::reexport::Deserializer<'de>
            {
                use $crate::serialize::json::reexport::Error;
                let value = <$crate::serialize::json::reexport::Value>::deserialize(deserializer)?;
                $crate::serialize::json::reexport::from_value(value)
                        .map_err(|_| D::Error::custom("Can not deserialize value."))
            }
        }

        impl $crate::serialize::json::reexport::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer
                {
                    $crate::serialize::json::wrap(self).serialize(serializer)
                }
        }

    )
}
