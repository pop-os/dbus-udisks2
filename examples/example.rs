extern crate dbus_udisks2;

use dbus_udisks2::{UDisks2, Disks};
use std::env::args;

fn main() {
    match args().nth(1) {
        Some(ref device) => print_block(device),
        None => print()
    }
}

fn print() {
    let udisks2 = UDisks2::new().unwrap();
    let disks = Disks::new(&udisks2);
    for device in disks.devices {
        println!("{:#?}", device);
    }
}

fn print_block(block_name: &str) {
    let udisks2 = UDisks2::new().unwrap();
    for block in udisks2.get_blocks() {
        if block.device.to_str().unwrap() == block_name {
            println!("{:#?}", block);
        }
    }
}
