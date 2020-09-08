use super::*;

/// All of UDisks2's disk information collected into a convenient wrapper.
#[derive(Debug, Default, Clone)]
pub struct Disks {
    pub devices: Vec<DiskDevice>,
}

/// A collection of UDisks2 drives and their associated blocks.
///
/// # Implementation Details
/// - Block partitions are sorted by their physical offsets.
#[derive(Debug, Default, Clone)]
pub struct DiskDevice {
    pub drive: Drive,
    pub parent: Block,
    pub partitions: Vec<Block>,
}

impl Disks {
    fn new_cache(udisks2: &DiskCache) -> Self {
        let mut devices = Vec::new();

        let mut blocks = Vec::new();
        for block in udisks2.get_blocks() {
            blocks.push(block);
        }

        for drive in udisks2.get_drives() {
            let mut partitions = Vec::new();
            let mut parent = None;

            for block in blocks.iter().filter(|b| b.drive == drive.path) {
                if block.table.is_some() {
                    parent = Some(block.to_owned());
                } else {
                    partitions.push(block.to_owned());
                }
            }

            if let Some(parent) = parent {
                partitions.sort_unstable_by_key(|p| p.partition.as_ref().unwrap().offset);
                devices.push(DiskDevice {
                    drive,
                    parent,
                    partitions,
                });
            }
        }

        Disks { devices }
    }
    pub fn new(udisks2: &UDisks2) -> Self {
        Disks::new_cache(&udisks2.cache)
    }
    #[cfg(feature = "futures")]
    pub fn new_async<C>(udisks2: &crate::AsyncUDisks2<C>) -> Self {
        Disks::new_cache(&udisks2.cache)
    }
}
