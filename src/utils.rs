use dbus::arg::{Variant, RefArg};
use std::collections::HashMap;
use std::ffi::CString;

pub fn get_string(arg: &Variant<Box<RefArg>>) -> Option<String> {
    arg.0.as_str().and_then(|x| {
        if x.is_empty() { None } else { Some(x.to_owned()) }
    })
}

pub fn get_u64(arg: &Variant<Box<RefArg>>) -> u64 {
    arg.0.as_u64().unwrap_or(0)
}

pub fn get_bool(arg: &Variant<Box<RefArg>>) -> bool {
    arg.0.as_u64().unwrap_or(0) != 0
}

pub fn get_string_array(arg: &Variant<Box<RefArg>>) -> Option<Vec<String>> {
    arg.0.as_iter().and_then(|items| {
            let vector = items.flat_map(|item| item.as_str().map(String::from))
                .collect::<Vec<_>>();
            if vector.is_empty() { None } else { Some(vector) }
        })
}

pub fn get_byte_array(arg: &Variant<Box<RefArg>>) -> Option<String> {
    arg.0.as_iter().and_then(|bytes| {
        let inner_vec = bytes.flat_map(|byte| byte.as_u64().map(|x| x as u8))
            .collect::<Vec<u8>>();
        String::from_utf8(inner_vec).ok().map(|mut x| {
            x.pop();
            x
        })
    })
}

pub fn get_array_of_byte_arrays(arg: &Variant<Box<RefArg>>) -> Option<Vec<String>> {
    arg.0.as_iter().and_then(|items| {
            let vector = items.flat_map(|item| {
                item.as_iter()
                    .and_then(|bytes| {
                        let inner_vec = bytes.flat_map(|byte| byte.as_u64().map(|x| x as u8))
                            .collect::<Vec<u8>>();
                        String::from_utf8(inner_vec).ok().map(|mut x| {
                            x.pop();
                            x
                        })
                    })
            }).collect::<Vec<_>>();
            if vector.is_empty() { None } else { Some(vector) }
        })
}

pub(crate) trait ParseFrom {
    fn parse_from(path: &str, objects: &HashMap<String, HashMap<String, Variant<Box<RefArg>>>>) -> Option<Self> where Self: Sized;
}