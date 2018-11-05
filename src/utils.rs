use dbus::arg::{Variant, RefArg};
use std::collections::HashMap;

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
    atostr(arg.0.as_iter())
}

pub fn atostr<'a>(array: Option<Box<Iterator<Item = &'a RefArg> + 'a>>) -> Option<String> {
    array.and_then(|bytes| {
        let mut inner_vec = bytes.flat_map(|byte| byte.as_u64().map(|x| x as u8))
            .collect::<Vec<u8>>();

        if inner_vec.last() == Some(&0) {
            inner_vec.pop();
        }

        String::from_utf8(inner_vec).ok()
    })
}

pub fn vva(value: &RefArg) -> Option<String> {
    let viter = value.as_iter().and_then(|mut i| {
        i.next().and_then(|i| {
            i.as_iter().and_then(|mut i| {
                i.next().and_then(|i| i.as_iter())
            })
        })
    });

    atostr(viter)
}

pub fn get_array_of_byte_arrays(arg: &Variant<Box<RefArg>>) -> Option<Vec<String>> {
    arg.0.as_iter().and_then(|items| {
            let vector = items.flat_map(|item| atostr(item.as_iter())).collect::<Vec<_>>();
            if vector.is_empty() { None } else { Some(vector) }
        })
}

pub(crate) trait ParseFrom {
    fn parse_from(path: &str, objects: &HashMap<String, HashMap<String, Variant<Box<RefArg>>>>) -> Option<Self> where Self: Sized;
}
