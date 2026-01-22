use std::ffi::c_void;
use windows_62_2::Win32::NetworkManagement::IpHelper::IP_ADAPTER_ADDRESSES_LH;

pub const AF_UNSPEC: u32 = 0;

#[repr(transparent)]
pub struct GetAdaptersAdressesFlags(pub u32);

#[link(name = "iphlpapi")]
unsafe extern "system" {
    pub fn GetAdaptersAddresses(
        family: u32,
        flags: GetAdaptersAdressesFlags,
        reserved: *const c_void,
        adapteraddresses: *mut IP_ADAPTER_ADDRESSES_LH,
        sizepointer: *mut u32,
    ) -> u32;
}
