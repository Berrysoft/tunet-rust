fn main() {
    windows::build!(
        Windows::Networking::Connectivity::*,
        Windows::Win32::Security::Credentials::*,
    );
}
