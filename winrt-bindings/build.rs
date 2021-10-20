fn main() {
    windows::build!(
        Windows::Networking::Connectivity::{NetworkInformation, WlanConnectionProfileDetails},
        Windows::Win32::Security::Credentials::{CredReadW, CredWriteW, CredDeleteW},
        Windows::UI::ViewManagement::UISettings,
    );
}
