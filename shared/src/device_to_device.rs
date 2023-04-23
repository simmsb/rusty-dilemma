use core::hash::Hash;
use serde::{Deserialize, Serialize};

use crate::{device_to_host::DeviceToHost, host_to_device::HostToDeviceMsg};

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DeviceToDevice {
    ForwardedFromHost(HostToDeviceMsg),
    ForwardedToHost(DeviceToHost),
}
