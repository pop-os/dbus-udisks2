extern crate dbus_udisks2;

use dbus_udisks2::UDisks2;
use std::env::args;

fn main() {
    let udisks2 = UDisks2::new().unwrap();
    match args().nth(1) {
        Some(ref device) => print_block(&udisks2, device),
        None => print(&udisks2)
    }
}

fn print(udisks2: &UDisks2) {
    for block in udisks2.get_blocks() {
        println!("{:#?}", block);
    }

    for drive in udisks2.get_drives() {
        println!("{:#?}", drive);
    }
}

fn print_block(udisks2: &UDisks2, block_name: &str) {
    for block in udisks2.get_blocks() {
        if block.path.ends_with(block_name) {
            println!("{:#?}", block);
        }
    }
}