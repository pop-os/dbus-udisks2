use dbus::arg::{Variant, RefArg};
use std::collections::HashMap;
use utils::*;

#[derive(Clone, Debug, Default)]
pub struct Drive {
    pub can_power_off: bool,
    pub connection_bus: Option<String>,
    pub ejectable: bool,
    pub id: Option<String>,
    pub media_available: bool,
    pub media_change_detected: bool,
    pub media_compatibility: Option<Vec<String>>,
    pub media_removable: bool,
    pub media: Option<String>,
    pub model: Option<String>,
    pub optical: bool,
    pub optical_blank: bool,
    pub optical_num_tracks: u64,
    pub optical_num_audio_tracks: u64,
    pub optical_num_data_tracks: u64,
    pub optical_num_sessions: u64,
    pub path: String,
    pub removable: bool,
    pub revision: Option<String>,
    pub rotation_rate: i64,
    pub seat: Option<String>,
    pub serial: Option<String>,
    pub sibling_id: Option<String>,
    pub size: u64,
    pub sort_key: Option<String>,
    pub time_detected: u64,
    pub time_media_detected: u64,
    pub vendor: Option<String>,
    pub wwn: Option<String>,
}

impl ParseFrom for Drive {
    fn parse_from(path: &str, objects: &HashMap<String, HashMap<String, Variant<Box<RefArg>>>>) -> Option<Drive> {
        if let Some(object) = objects.get("org.freedesktop.UDisks2.Drive") {
            let mut drive = Drive::default();
            drive.path = path.to_owned();
            drive.parse(object);

            Some(drive)
        } else {
            None
        }
    }
}

impl Drive {
    fn parse(&mut self, objects: &HashMap<String, Variant<Box<RefArg>>>) {
        for (key, ref value) in objects {
            match key.as_str() {
                "CanPowerOff" => self.can_power_off = get_bool(value),
                "ConnectionBus" => self.connection_bus = get_string(value),
                "Ejectable" => self.ejectable = get_bool(value),
                "Id" => self.id = get_string(value),
                "Media" => self.media = get_string(value),
                "MediaAvailable" => self.media_available = get_bool(value),
                "MediaChangeDetected" => self.media_change_detected = get_bool(value),
                "MediaCompatibility" => self.media_compatibility = get_string_array(value),
                "MediaRemovable" => self.media_removable = get_bool(value),
                "Model" => self.model = get_string(value),
                "Optical" => self.optical = get_bool(value),
                "OpticalBlank" => self.optical_blank = get_bool(value),
                "OpticalNumTracks" => self.optical_num_tracks = get_u64(value),
                "OpticalNumAudioTracks" => self.optical_num_audio_tracks = get_u64(value),
                "OpticalNumDataTracks" => self.optical_num_data_tracks = get_u64(value),
                "OpticalNumSessions" => self.optical_num_sessions = get_u64(value),
                "Removable" => self.removable = get_bool(value),
                "Revision" => self.revision = get_string(value),
                "RotationRate" => self.rotation_rate = get_u64(value) as i64,
                "Seat" => self.seat = get_string(value),
                "Serial" => self.serial = get_string(value),
                "SiblingId" => self.sibling_id = get_string(value),
                "Size" => self.size = get_u64(value),
                "SortKey" => self.sort_key = get_string(value),
                "TimeDetected" => self.time_detected = get_u64(value),
                "TimeMediaDetected" => self.time_media_detected = get_u64(value),
                "Vendor" => self.vendor = get_string(value),
                "WWN" => self.wwn = get_string(value),
                _ => {
                    #[cfg(debug_assertions)]
                    eprintln!("unhandled org.freedesktop.UDisks2.Drive::{}", key);
                }
            }
        }
    }
}