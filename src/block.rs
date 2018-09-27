use dbus::arg::{Variant, RefArg};
use std::collections::HashMap;
use std::ops::Deref;
use utils::*;

pub struct EncryptedBlock<'a>(&'a Block);

impl<'a> Deref for EncryptedBlock<'a> {
    type Target = Block;
    
    fn deref(&self) -> &Block {
        self.0
    }
}

impl<'a> EncryptedBlock<'a> {
    pub fn find_inner(&self, within: &'a [Block]) -> Option<&'a Block> {
        within.iter().find(|b| &b.crypto_backing_device == &self.0.path)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Block {
    pub crypto_backing_device: String,
    pub device_number: u64,
    pub device: String,
    pub drive: String,
    pub encrypted: Option<Encrypted>,
    pub hint_auto: bool,
    pub hint_icon_name: Option<String>,
    pub hint_ignore: bool,
    pub hint_name: Option<String>,
    pub hint_partitionable: bool,
    pub hint_symbolic_icon_name: Option<String>,
    pub hint_system: bool,
    pub id_label: Option<String>,
    pub id_type: Option<String>,
    pub id_usage: Option<String>,
    pub id_uuid: Option<String>,
    pub id_version: Option<String>,
    pub id: String,
    pub loopback: bool,
    pub mdraid: Option<String>,
    pub mdraid_member: Option<String>,
    pub mount_points: Option<Vec<String>>,
    pub partition: Option<Partition>,
    pub path: String,
    pub preferred_device: String,
    pub read_only: bool,
    pub size: u64,
    pub swapspace: Option<bool>,
    pub symlinks: Option<Vec<String>>,
    pub table: Option<PartitionTable>,
    pub userspace_mount_options: Option<Vec<String>>,
}

impl Block {
    pub fn as_encrypted_device(&self) -> Option<EncryptedBlock> {
        if self.encrypted.is_some() {
            Some(EncryptedBlock(self))
        } else {
            None
        }
    }
}

impl ParseFrom for Block {
    fn parse_from(path: &str, objects: &HashMap<String, HashMap<String, Variant<Box<RefArg>>>>) -> Option<Block> {
        if objects.get("org.freedesktop.UDisks2.Loop").is_some() {
            return None;
        }
        
        let mut block = Block::default();
        block.path = path.to_owned();

        match objects.get("org.freedesktop.UDisks2.Block") {
            Some(object) => {
                for (key, ref value) in object {
                    match key.as_str() {
                        "CryptoBackingDevice" => block.crypto_backing_device = get_string(value).unwrap(),
                        "Device" => block.device = get_byte_array(value).unwrap(),
                        "DeviceNumber" => block.device_number = get_u64(value),
                        "Drive" => block.drive = get_string(value).unwrap(),
                        "HintAuto" => block.hint_auto = get_bool(value),
                        "HintIconName" => block.hint_icon_name = get_string(value),
                        "HintIgnore" => block.hint_ignore = get_bool(value),
                        "HintName" => block.hint_name = get_string(value),
                        "HintPartitionable" => block.hint_partitionable = get_bool(value),
                        "HintSymbolicIconName" => block.hint_symbolic_icon_name = get_string(value),
                        "HintSystem" => block.hint_system = get_bool(value),
                        "Id" => block.id = get_string(value).expect("block without ID"),
                        "IdLabel" => block.id_label = get_string(value),
                        "IdType" => block.id_type = get_string(value),
                        "IdUsage" => block.id_usage = get_string(value),
                        "IdUUID" => block.id_type = get_string(value),
                        "IdVersion" => block.id_version = get_string(value),
                        "MDRaid" => block.mdraid = get_string(value),
                        "MDRaidMember" => block.mdraid_member = get_string(value),
                        "PreferredDevice" => block.preferred_device = get_byte_array(value).unwrap(),
                        "ReadOnly" => block.read_only = get_bool(value),
                        "Size" => block.size = get_u64(value),
                        "Symlinks" => block.symlinks = get_array_of_byte_arrays(value),
                        "UserspaceMountOptions" => block.userspace_mount_options = get_string_array(value),
                        _ => {
                            #[cfg(debug_assertions)]
                            eprintln!("unhandled org.freedesktop.UDisks2.Block.{}", key);
                        }
                    }
                }
                
            }
            None => return None
        }

        for (key, object) in objects {
            match key.as_str() {
                "org.freedesktop.UDisks2.Block" => (),
                "org.freedesktop.UDisks2.Swapspace" => {
                    block.swapspace = Some(object.get("Active").map_or(false, get_bool));
                },
                "org.freedesktop.UDisks2.PartitionTable" => {
                    let mut table = PartitionTable::default();
                    for (key, ref value) in object {
                        match key.as_str() {
                            "Type" => table.type_ = get_string(value),
                            "Partitions" => table.partitions = get_string_array(value),
                            _ => {
                                #[cfg(debug_assertions)]
                                eprintln!("unhandled org.freedesktop.UDisks2.PartitionTable.{}", key);
                            }
                        }
                    }

                    block.table = Some(table);
                },
                "org.freedesktop.UDisks2.Partition" => {
                    let mut partition = Partition::default();
                    for (key, value) in object {
                        match key.as_str() {
                            "Type" => partition.type_ = get_string(value),
                            "Name" => partition.name = get_string(value),
                            "UUID" => partition.uuid = get_string(value).expect("partition lacks a UUID"),
                            "Table" => partition.table = get_string(value).expect("partition is not part of a table"),
                            "Flags" => partition.flags = get_u64(value),
                            "Offset" => partition.offset = get_u64(value),
                            "Size" => partition.size = get_u64(value),
                            "Number" => partition.number = get_u64(value) as u32,
                            "IsContained" => partition.is_contained = get_bool(value),
                            "IsContainer" => partition.is_container = get_bool(value),
                            _ => {
                                #[cfg(debug_assertions)]
                                eprintln!("unhandled org.freedesktop.UDisks2.Partition.{}", key);
                            }
                        }
                    }

                    block.partition = Some(partition);
                }
                "org.freedesktop.UDisks2.Filesystem" => {
                    block.mount_points = object.get("MountPoints").map_or(None, get_array_of_byte_arrays);
                }
                "org.freedesktop.UDisks2.Encrypted" => {
                    let mut encrypted = Encrypted::default();
                    for (key, ref value) in object {
                        match key.as_str() {
                            "HintEncryptionType" => encrypted.hint_encryption_type = get_string(value).unwrap(),
                            "MetadataSize" => encrypted.metadata_size = get_u64(value),
                            "CleartextDevice" => encrypted.cleartext_device = get_string(value).unwrap(),
                            _ => {
                                #[cfg(debug_assertions)]
                                eprintln!("unhandled org.freedesktop.UDisks2.Encrypted.{}", key);
                            }
                        }
                    }

                    block.encrypted = Some(encrypted);
                }
                _ => {
                    #[cfg(debug_assertions)]
                    eprintln!("unhandled org.freedesktop.UDisks2.{}", key);
                }
            }
        }

        Some(block)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Encrypted {
    pub hint_encryption_type: String,
    pub metadata_size: u64,
    pub cleartext_device: String
}

#[derive(Clone, Debug, Default)]
pub struct PartitionTable {
    pub type_: Option<String>,
    // Partitions are listed by their dbus paths.
    pub partitions: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default)]
pub struct Partition {
    // Defines the file system by a type UUID.
    pub type_: Option<String>,
    // An optional label that may be applied to a disk.
    pub name: Option<String>,
    // Points to the dbus path that this partition exists within.
    pub table: String,
    pub flags: u64,
    pub number: u32,
    pub offset: u64,
    pub size: u64,
    pub uuid: String,
    pub is_container: bool,
    pub is_contained: bool,
}