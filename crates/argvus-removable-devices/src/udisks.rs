use std::collections::HashMap;

use zbus::message::Type;
use zbus::{Connection, MatchRule};
use zvariant::OwnedValue;
use zvariant::Value;

use crate::device::Device;

const SERVICE: &str = "org.freedesktop.UDisks2";
const MANAGER_PATH: &str = "/org/freedesktop/UDisks2";

const IFACE_BLOCK: &str = "org.freedesktop.UDisks2.Block";
const IFACE_FILESYSTEM: &str = "org.freedesktop.UDisks2.Filesystem";
const IFACE_ENCRYPTED: &str = "org.freedesktop.UDisks2.Encrypted";
const IFACE_PARTITION_TABLE: &str = "org.freedesktop.UDisks2.PartitionTable";
const IFACE_PARTITION: &str = "org.freedesktop.UDisks2.Partition";
const IFACE_LOOP: &str = "org.freedesktop.UDisks2.Loop";
const IFACE_DRIVE: &str = "org.freedesktop.UDisks2.Drive";

#[derive(Default)]
pub struct RawBlock {
  pub object_path: String,
  pub device: String,
  pub drive: String,
  pub id_label: String,
  pub id_type: String,
  pub hint_name: String,
  pub preferred_device: String,
  pub partition_type: String,
  pub size: u64,
  pub read_only: bool,
  pub hint_ignore: bool,
  pub mount_points: Vec<String>,
  pub has_filesystem: bool,
  pub has_encrypted: bool,
  pub is_partition_table: bool,
  pub is_loop: bool,
  pub is_cleartext: bool,
  pub cleartext: String,
  pub encryption_type: String,
}

#[derive(Default)]
pub struct RawDrive {
  pub object_path: String,
  pub vendor: String,
  pub model: String,
  pub connection_bus: String,
  pub removable: bool,
  pub ejectable: bool,
  pub media_removable: bool,
  pub optical: bool,
}

#[derive(Default)]
pub struct Enumerated {
  pub blocks: Vec<RawBlock>,
  pub drives: Vec<RawDrive>,
}

pub struct UdisksClient {
  connection: Option<Connection>,
  error: String,
}

impl UdisksClient {
  pub fn new() -> Self {
    UdisksClient {
      connection: None,
      error: String::new(),
    }
  }

  pub fn last_error(&self) -> &str {
    &self.error
  }

  pub fn connection(&self) -> Option<Connection> {
    self.connection.clone()
  }

  pub async fn connect(&mut self) -> bool {
    if self.connection.is_some() {
      return true;
    }
    match Connection::system().await {
      Ok(conn) => {
        self.connection = Some(conn);
        true
      }
      Err(e) => {
        self.error = e.to_string();
        false
      }
    }
  }

  pub async fn enumerate(&mut self) -> Enumerated {
    let mut out = Enumerated::default();
    let conn = match &self.connection {
      Some(c) => c.clone(),
      None => {
        self.error = "not connected".to_string();
        return out;
      }
    };
    let builder = match zbus::fdo::ObjectManagerProxy::builder(&conn).destination(SERVICE) {
      Ok(builder) => builder,
      Err(e) => {
        self.error = e.to_string();
        return out;
      }
    };
    let builder = match builder.path(MANAGER_PATH) {
      Ok(builder) => builder,
      Err(e) => {
        self.error = e.to_string();
        return out;
      }
    };
    let proxy = match builder.build().await {
      Ok(p) => p,
      Err(e) => {
        self.error = e.to_string();
        return out;
      }
    };
    let objects = match proxy.get_managed_objects().await {
      Ok(o) => o,
      Err(e) => {
        self.error = e.to_string();
        return out;
      }
    };

    for (obj_path, interfaces) in objects {
      let path = obj_path.to_string();
      let mut block: Option<RawBlock> = None;
      for (iface, props) in interfaces {
        let iface = iface.to_string();
        if iface == IFACE_DRIVE {
          let mut drive = RawDrive {
            object_path: path.clone(),
            ..RawDrive::default()
          };
          parse_drive(&props, &mut drive);
          out.drives.push(drive);
        } else if iface == IFACE_BLOCK {
          let raw_block = block.get_or_insert_with(|| RawBlock {
            object_path: path.clone(),
            ..RawBlock::default()
          });
          parse_block(&props, raw_block, BlockSource::Block);
        } else if iface == IFACE_FILESYSTEM {
          let raw_block = block.get_or_insert_with(|| RawBlock {
            object_path: path.clone(),
            ..RawBlock::default()
          });
          parse_block(&props, raw_block, BlockSource::Filesystem);
        } else if iface == IFACE_ENCRYPTED {
          let raw_block = block.get_or_insert_with(|| RawBlock {
            object_path: path.clone(),
            ..RawBlock::default()
          });
          parse_block(&props, raw_block, BlockSource::Encrypted);
        } else if iface == IFACE_PARTITION_TABLE {
          block
            .get_or_insert_with(|| RawBlock {
              object_path: path.clone(),
              ..RawBlock::default()
            })
            .is_partition_table = true;
        } else if iface == IFACE_PARTITION {
          if let Some(partition_type) = prop_string(&props, "Type") {
            block
              .get_or_insert_with(|| RawBlock {
                object_path: path.clone(),
                ..RawBlock::default()
              })
              .partition_type = partition_type;
          }
        } else if iface == IFACE_LOOP {
          block
            .get_or_insert_with(|| RawBlock {
              object_path: path.clone(),
              ..RawBlock::default()
            })
            .is_loop = true;
        }
      }
      if let Some(raw_block) = block {
        out.blocks.push(raw_block);
      }
    }

    // Mark blocks that are the cleartext of an encrypted device so the
    // model can skip them (the LUKS container is the item shown in the bar).
    let cleartext_set: std::collections::HashSet<String> = out
      .blocks
      .iter()
      .filter(|raw_block| !raw_block.cleartext.is_empty() && raw_block.cleartext != "/")
      .map(|raw_block| raw_block.cleartext.clone())
      .collect();
    for raw_block in out.blocks.iter_mut() {
      if cleartext_set.contains(&raw_block.object_path) {
        raw_block.is_cleartext = true;
      }
    }
    out
  }

  async fn call_method(
    &self,
    object_path: &str,
    interface: &str,
    method: &str,
    options: &HashMap<String, Value<'static>>,
  ) -> Result<(), String> {
    let conn = self
      .connection
      .as_ref()
      .ok_or_else(|| "not connected to system bus".to_string())?;
    let body = (options.clone(),);
    conn
      .call_method(Some(SERVICE), object_path, Some(interface), method, &body)
      .await
      .map(|_| ())
      .map_err(|e| e.to_string())
  }

  pub async fn mount(&self, device: &Device) -> Result<(), String> {
    let target = if !device.cleartext.is_empty() && device.cleartext != "/" {
      &device.cleartext
    } else {
      &device.object_path
    };
    self
      .call_method(target, IFACE_FILESYSTEM, "Mount", &no_user_interaction())
      .await
  }

  pub async fn unmount(&self, device: &Device) -> Result<(), String> {
    let target = if !device.cleartext.is_empty() && device.cleartext != "/" {
      &device.cleartext
    } else {
      &device.object_path
    };
    self
      .call_method(target, IFACE_FILESYSTEM, "Unmount", &no_user_interaction())
      .await
  }

  pub async fn eject(&self, device: &Device) -> Result<(), String> {
    if device.drive_path.is_empty() {
      return Err(
        crate::i18n::tr(
          "device has no drive to eject",
          "o dispositivo não tem unidade para ejetar",
        )
        .to_string(),
      );
    }
    self
      .call_method(
        &device.drive_path,
        IFACE_DRIVE,
        "Eject",
        &no_user_interaction(),
      )
      .await
  }

  pub async fn power_off(&self, device: &Device) -> Result<(), String> {
    if device.drive_path.is_empty() {
      return Err(
        crate::i18n::tr(
          "device has no drive to power off",
          "o dispositivo não tem unidade para desligar",
        )
        .to_string(),
      );
    }
    self
      .call_method(
        &device.drive_path,
        IFACE_DRIVE,
        "PowerOff",
        &no_user_interaction(),
      )
      .await
  }

  pub async fn lock(&self, device: &Device) -> Result<(), String> {
    self
      .call_method(
        &device.object_path,
        IFACE_ENCRYPTED,
        "Lock",
        &empty_options(),
      )
      .await
  }
}

fn no_user_interaction() -> HashMap<String, Value<'static>> {
  let mut m = HashMap::new();
  m.insert("auth.no_user_interaction".to_string(), Value::Bool(true));
  m
}

fn empty_options() -> HashMap<String, Value<'static>> {
  HashMap::new()
}

fn prop_string(props: &HashMap<String, OwnedValue>, name: &str) -> Option<String> {
  let value = props.get(name)?;
  if let Ok(text) = <&str>::try_from(value) {
    return Some(text.to_string());
  }
  if let Ok(text) = <&zvariant::Str>::try_from(value) {
    return Some(text.to_string());
  }
  if let Ok(object_path) = <&zvariant::ObjectPath>::try_from(value) {
    return Some(object_path.to_string());
  }
  None
}

fn prop_bool(props: &HashMap<String, OwnedValue>, name: &str) -> Option<bool> {
  let value = props.get(name)?;
  <bool>::try_from(value).ok()
}

fn prop_u64(props: &HashMap<String, OwnedValue>, name: &str) -> Option<u64> {
  let value = props.get(name)?;
  <u64>::try_from(value).ok()
}

// `ay` byte arrays (Device, ...), trailing NULs stripped.
fn prop_bytes(props: &HashMap<String, OwnedValue>, name: &str) -> Option<Vec<u8>> {
  let value = props.get(name)?.try_clone().ok()?;
  let mut bytes: Vec<u8> = Vec::try_from(value).ok()?;
  while bytes.last() == Some(&0) {
    bytes.pop();
  }
  Some(bytes)
}

// `aay` (MountPoints), trailing NULs stripped.
fn prop_mount_points(props: &HashMap<String, OwnedValue>, name: &str) -> Option<Vec<String>> {
  let value = props.get(name)?.try_clone().ok()?;
  let arrays: Vec<Vec<u8>> = Vec::try_from(value).ok()?;
  let mut out = Vec::new();
  for mut bytes in arrays {
    while bytes.last() == Some(&0) {
      bytes.pop();
    }
    if !bytes.is_empty() {
      out.push(String::from_utf8_lossy(&bytes).into_owned());
    }
  }
  Some(out)
}

#[derive(Clone, Copy, PartialEq)]
enum BlockSource {
  Block,
  Filesystem,
  Encrypted,
}

fn parse_block(props: &HashMap<String, OwnedValue>, raw_block: &mut RawBlock, source: BlockSource) {
  match source {
    BlockSource::Block => {
      if let Some(bytes) = prop_bytes(props, "Device") {
        raw_block.device = String::from_utf8_lossy(&bytes).into_owned();
      }
      if let Some(value) = prop_string(props, "Drive") {
        raw_block.drive = value;
      }
      if let Some(value) = prop_string(props, "IdLabel") {
        raw_block.id_label = value;
      }
      if let Some(value) = prop_string(props, "IdType") {
        raw_block.id_type = value;
      }
      if let Some(value) = prop_string(props, "HintName") {
        raw_block.hint_name = value;
      }
      if let Some(value) = prop_string(props, "PreferredDevice") {
        raw_block.preferred_device = value;
      }
      if let Some(value) = prop_string(props, "PartitionType") {
        raw_block.partition_type = value;
      }
      if let Some(value) = prop_u64(props, "Size") {
        raw_block.size = value;
      }
      if let Some(value) = prop_bool(props, "ReadOnly") {
        raw_block.read_only = value;
      }
      if let Some(value) = prop_bool(props, "HintIgnore") {
        raw_block.hint_ignore = value;
      }
      if let Some(value) = prop_string(props, "HintEncryptionType") {
        raw_block.encryption_type = value;
      }
    }
    BlockSource::Filesystem => {
      if let Some(value) = prop_mount_points(props, "MountPoints") {
        raw_block.mount_points = value;
      }
      raw_block.has_filesystem = props.contains_key("MountPoints");
    }
    BlockSource::Encrypted => {
      if let Some(value) = prop_string(props, "CleartextDevice") {
        raw_block.cleartext = value;
      }
      raw_block.has_encrypted = true;
    }
  }
}

fn parse_drive(props: &HashMap<String, OwnedValue>, drive: &mut RawDrive) {
  if let Some(value) = prop_string(props, "Vendor") {
    drive.vendor = value;
  }
  if let Some(value) = prop_string(props, "Model") {
    drive.model = value;
  }
  if let Some(value) = prop_string(props, "ConnectionBus") {
    drive.connection_bus = value;
  }
  if let Some(value) = prop_bool(props, "Removable") {
    drive.removable = value;
  }
  if let Some(value) = prop_bool(props, "Ejectable") {
    drive.ejectable = value;
  }
  if let Some(value) = prop_bool(props, "MediaRemovable") {
    drive.media_removable = value;
  }
  if let Some(value) = prop_bool(props, "Optical") {
    drive.optical = value;
  }
}

// True when the message is a UDisks2 change signal we care about.
pub fn is_relevant_signal(msg: &zbus::Message) -> bool {
  let header = msg.header();
  let iface = header
    .interface()
    .map(|s| s.to_string())
    .unwrap_or_default();
  let member = header.member().map(|s| s.to_string()).unwrap_or_default();
  matches!(
    (iface.as_str(), member.as_str()),
    ("org.freedesktop.DBus.Properties", "PropertiesChanged")
      | ("org.freedesktop.DBus.ObjectManager", "InterfacesAdded")
      | ("org.freedesktop.DBus.ObjectManager", "InterfacesRemoved")
  )
}

// Signal headers contain UDisks2's unique bus name (for example, :1.42), not
// its well-known name. Match the well-known name at the bus level instead of
// comparing it directly with the sender header.
pub fn signal_match_rule() -> Result<MatchRule<'static>, zbus::Error> {
  Ok(
    MatchRule::builder()
      .msg_type(Type::Signal)
      .sender(SERVICE)?
      .path_namespace(MANAGER_PATH)?
      .build(),
  )
}
