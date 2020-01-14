///
/// Relevant OS information that commands
/// may/may-not care about
///
#[derive(Debug, PartialEq, Default, Clone)]
pub struct OsInfo {
    pub os_type: OsType,
}

impl OsInfo {
    pub fn new() -> OsInfo {
        OsInfo {
            #[cfg(target_os = "macos")]
            os_type: OsType::Mac,

            #[cfg(target_os = "windows")]
            os_type: OsType::Windows,

            // this wont take effect if either of the above apply
            ..Default::default()
        }
    }
}

///
/// Deliberately not an exhaustive list.
/// If it's not window or mac, assume linux
///
#[derive(Debug, PartialEq, Clone)]
pub enum OsType {
    Mac,
    Windows,
    Linux,
}

impl Default for OsType {
    fn default() -> Self {
        OsType::Linux
    }
}
