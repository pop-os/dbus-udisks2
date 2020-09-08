use dbus::nonblock::NonblockReply;
use dbus_udisks2::{AsyncUDisks2, Disks};
use std::env::args;
use std::ops::Deref;

#[tokio::main]
async fn main() {
    // Connect to the D-Bus session bus (this is blocking, unfortunately).
    let (resource, conn) = dbus_tokio::connection::new_system_sync().unwrap();

    // The resource is a task that should be spawned onto a tokio compatible
    // reactor ASAP. If the resource ever finishes, you lost connection to D-Bus.
    tokio::spawn(async {
        let err = resource.await;
        panic!("Lost connection to D-Bus: {}", err);
    });

    match args().nth(1) {
        Some(ref device) => print_block(conn, device).await,
        None => print(conn).await,
    }
}

async fn print<T: NonblockReply, C: Deref<Target = T>>(conn: C) {
    let udisks2 = AsyncUDisks2::new(conn).await.unwrap();
    let disks = Disks::new_async(&udisks2);
    for device in disks.devices {
        println!("{:#?}", device);
        if let Ok(smart_data) = udisks2.smart_attributes(&device.drive, false).await {
            println!("{:#?}", smart_data);
        }
    }
}

async fn print_block<T: NonblockReply, C: Deref<Target = T>>(conn: C, block_name: &str) {
    let udisks2 = AsyncUDisks2::new(conn).await.unwrap();
    for block in udisks2.get_blocks() {
        if block.device.to_str().unwrap() == block_name {
            println!("{:#?}", block);
        }
    }
}
