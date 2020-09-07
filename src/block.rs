use crate::DbusObjects;
use dbus::arg::RefArg;
use std::path::PathBuf;
use utils::*;

#[derive(Clone, Debug, Default)]
pub struct Block {
    pub crypto_backing_device: String,
    pub device_number: u64,
    pub device: PathBuf,
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
    pub mdraid: PathBuf,
    pub mdraid_member: PathBuf,
    pub mount_points: Vec<PathBuf>,
    pub partition: Option<Partition>,
    pub path: String,
    pub preferred_device: PathBuf,
    pub read_only: bool,
    pub size: u64,
    pub swapspace: Option<bool>,
    pub symlinks: Vec<PathBuf>,
    pub table: Option<PartitionTable>,
    pub userspace_mount_options: Vec<String>,
    pub configuration: Option<BlockConfiguration>,
}

impl Block {
    /// This will be true if this block contains an encrypted volume.
    pub fn is_encrypted(&self) -> bool {
        self.encrypted.is_some()
    }

    /// If this block contains an encrypted volume, find the block associated with it.
    pub fn get_encrypted_block<'a>(&self, within: &'a [Block]) -> Option<&'a Block> {
        if self.encrypted.is_some() {
            within.iter().find(|b| b.crypto_backing_device == self.path)
        } else {
            None
        }
    }
}

impl ParseFrom for Block {
    fn parse_from(path: &str, objects: &DbusObjects) -> Option<Block> {
        if objects.get("org.freedesktop.UDisks2.Loop").is_some() {
            return None;
        }

        let mut block = Block::default();
        block.path = path.to_owned();

        match objects.get("org.freedesktop.UDisks2.Block") {
            Some(object) => {
                for (key, ref value) in object {
                    match key.as_str() {
                        "CryptoBackingDevice" => {
                            block.crypto_backing_device = get_string(value).unwrap()
                        }
                        "Device" => block.device = PathBuf::from(get_byte_array(value).unwrap()),
                        "DeviceNumber" => block.device_number = get_u64(value),
                        "Drive" => block.drive = get_string(value).unwrap(),
                        "HintAuto" => block.hint_auto = get_bool(value),
                        "HintIconName" => block.hint_icon_name = get_string(value),
                        "HintIgnore" => block.hint_ignore = get_bool(value),
                        "HintName" => block.hint_name = get_string(value),
                        "HintPartitionable" => block.hint_partitionable = get_bool(value),
                        "HintSymbolicIconName" => block.hint_symbolic_icon_name = get_string(value),
                        "HintSystem" => block.hint_system = get_bool(value),
                        "Id" => block.id = get_string(value).unwrap_or_default(),
                        "IdLabel" => block.id_label = get_string(value),
                        "IdType" => block.id_type = get_string(value),
                        "IdUsage" => block.id_usage = get_string(value),
                        "IdUUID" => block.id_uuid = get_string(value),
                        "IdVersion" => block.id_version = get_string(value),
                        "MDRaid" => {
                            block.mdraid = get_string(value).map(PathBuf::from).unwrap_or_default()
                        }
                        "MDRaidMember" => {
                            block.mdraid_member =
                                get_string(value).map(PathBuf::from).unwrap_or_default()
                        }
                        "PreferredDevice" => {
                            block.preferred_device = PathBuf::from(get_byte_array(value).unwrap())
                        }
                        "ReadOnly" => block.read_only = get_bool(value),
                        "Size" => block.size = get_u64(value),
                        "Symlinks" => {
                            block.symlinks = get_array_of_byte_arrays(value)
                                .map(|paths| {
                                    paths.into_iter().map(PathBuf::from).collect::<Vec<_>>()
                                })
                                .unwrap_or_default()
                        }
                        "UserspaceMountOptions" => {
                            block.userspace_mount_options =
                                get_string_array(value).unwrap_or_default()
                        }
                        "Configuration" => {
                            let mut configuration = BlockConfiguration::default();
                            for value in value.as_iter().unwrap() {
                                if let Some(mut iterator) = value.as_iter() {
                                    if let Some(mut iterator) =
                                        iterator.next().and_then(|i| i.as_iter())
                                    {
                                        if let (Some(key), Some(mut array)) = (
                                            iterator.next(),
                                            iterator.next().and_then(|i| i.as_iter()),
                                        ) {
                                            if let Some(key) = key.as_str() {
                                                if key == "fstab" {
                                                    while let (Some(key), Some(value)) =
                                                        (array.next(), array.next())
                                                    {
                                                        if let Some(key) = key.as_str() {
                                                            match key {
                                                                "fsname" => {
                                                                    configuration.fstab.fsname =
                                                                        vva(value)
                                                                            .unwrap_or_default()
                                                                }
                                                                "dir" => {
                                                                    configuration.fstab.dir =
                                                                        vva(value)
                                                                            .unwrap_or_default()
                                                                }
                                                                "type" => {
                                                                    configuration.fstab.type_ =
                                                                        vva(value)
                                                                            .unwrap_or_default()
                                                                }
                                                                "opts" => {
                                                                    configuration.fstab.opts =
                                                                        vva(value)
                                                                            .unwrap_or_default()
                                                                }
                                                                "freq" => {
                                                                    configuration.fstab.freq = value
                                                                        .as_u64()
                                                                        .unwrap_or_default()
                                                                        as i32
                                                                }
                                                                "passno" => {
                                                                    configuration.fstab.passno =
                                                                        value
                                                                            .as_u64()
                                                                            .unwrap_or_default()
                                                                            as i32
                                                                }
                                                                _ => {
                                                                    eprintln!("unhandled block config fstab key: {:?}, {:?}", key, value);
                                                                }
                                                            }
                                                        }
                                                    }
                                                } else if key == "crypttab" {
                                                    while let (Some(key), Some(value)) =
                                                        (array.next(), array.next())
                                                    {
                                                        if let Some(key) = key.as_str() {
                                                            match key {
                                                                "name" => {
                                                                    configuration.crypttab.name =
                                                                        vva(value)
                                                                            .unwrap_or_default()
                                                                }
                                                                "device" => {
                                                                    configuration.crypttab.device =
                                                                        vva(value)
                                                                            .unwrap_or_default()
                                                                }
                                                                "passphrase-path" => {
                                                                    configuration
                                                                        .crypttab
                                                                        .passphrase_path =
                                                                        vva(value)
                                                                            .unwrap_or_default()
                                                                }
                                                                "options" => {
                                                                    configuration.crypttab.options =
                                                                        vva(value)
                                                                            .unwrap_or_default()
                                                                }
                                                                _ => {
                                                                    eprintln!("unhandled block config crypttab key: {:?}, {:?}", key, value);
                                                                }
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    eprintln!("unknown block config key: {}", key);
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            block.configuration = Some(configuration);
                        }
                        _ => {
                            #[cfg(debug_assertions)]
                            eprintln!("unhandled org.freedesktop.UDisks2.Block.{}", key);
                            eprintln!("value: {:#?}", value);
                        }
                    }
                }
            }
            None => return None,
        }

        for (key, object) in objects {
            match key.as_str() {
                "org.freedesktop.UDisks2.Block" => (),
                "org.freedesktop.UDisks2.Swapspace" => {
                    block.swapspace = Some(object.get("Active").map_or(false, get_bool));
                }
                "org.freedesktop.UDisks2.PartitionTable" => {
                    let mut table = PartitionTable::default();
                    for (key, ref value) in object {
                        match key.as_str() {
                            "Type" => table.type_ = get_string(value).unwrap_or_default(),
                            "Partitions" => {
                                table.partitions = get_string_array(value).unwrap_or_default();
                                table.partitions.sort_unstable();
                            }
                            _ => {
                                #[cfg(debug_assertions)]
                                eprintln!(
                                    "unhandled org.freedesktop.UDisks2.PartitionTable.{}",
                                    key
                                );
                            }
                        }
                    }

                    block.table = Some(table);
                }
                "org.freedesktop.UDisks2.Partition" => {
                    let mut partition = Partition::default();
                    for (key, value) in object {
                        match key.as_str() {
                            "Type" => partition.type_ = get_string(value).unwrap_or_default(),
                            "Name" => partition.name = get_string(value).unwrap_or_default(),
                            "UUID" => partition.uuid = get_string(value).unwrap_or_default(),
                            "Table" => {
                                partition.table =
                                    get_string(value).expect("partition is not part of a table")
                            }
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
                    block.mount_points = object
                        .get("MountPoints")
                        .and_then(get_array_of_byte_arrays)
                        .map(|paths| paths.into_iter().map(PathBuf::from).collect::<Vec<_>>())
                        .unwrap_or_default()
                }
                "org.freedesktop.UDisks2.Encrypted" => {
                    let mut encrypted = Encrypted::default();
                    for (key, ref value) in object {
                        match key.as_str() {
                            "HintEncryptionType" => {
                                encrypted.hint_encryption_type =
                                    get_string(value).unwrap_or_default()
                            }
                            "MetadataSize" => encrypted.metadata_size = get_u64(value),
                            "CleartextDevice" => {
                                encrypted.cleartext_device = get_string(value).unwrap_or_default()
                            }
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
pub struct BlockConfiguration {
    pub fstab: BlockConfigurationFstab,
    pub crypttab: BlockConfigurationCrypttab,
}

#[derive(Clone, Debug, Default)]
pub struct BlockConfigurationFstab {
    pub fsname: String,
    pub dir: String,
    pub type_: String,
    pub opts: String,
    pub freq: i32,
    pub passno: i32,
}

#[derive(Clone, Debug, Default)]
pub struct BlockConfigurationCrypttab {
    pub name: String,
    pub device: String,
    pub passphrase_path: String,
    pub options: String,
}

#[derive(Clone, Debug, Default)]
pub struct Encrypted {
    pub hint_encryption_type: String,
    pub metadata_size: u64,
    pub cleartext_device: String,
}

#[derive(Clone, Debug, Default)]
pub struct PartitionTable {
    pub type_: String,
    // Partitions are listed by their dbus paths.
    pub partitions: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct Partition {
    // Defines the file system by a type UUID.
    pub type_: String,
    // An optional label that may be applied to a disk.
    pub name: String,
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
