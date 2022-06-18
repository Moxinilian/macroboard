#![allow(clippy::bad_bit_mask)]

use keycode::{KeyMap, KeyMappingCode};
use libc::{c_char, O_RDONLY, O_RDWR, O_WRONLY};
use nix::{ioctl_read_buf, ioctl_write_int};
use std::{
    collections::{HashMap, HashSet},
    ffi::CString,
    fs::{File, OpenOptions},
    os::unix::prelude::{FromRawFd, IntoRawFd, OpenOptionsExt, RawFd},
    path::Path,
};
use tinyset::SetU32;

use input::{
    event::{keyboard::KeyboardEventTrait, tablet_pad::KeyState, EventTrait, KeyboardEvent},
    Event, Libinput, LibinputInterface,
};

use crate::trigger::Triggers;

const UAPI_IOC_MAGIC: u8 = b'E';
const UAPI_IOC_EVIOCGRAB: u8 = 0x90;
const UAPI_IOC_EVIOCGNAME: u8 = 0x06;
ioctl_write_int!(eviocgrab, UAPI_IOC_MAGIC, UAPI_IOC_EVIOCGRAB);
ioctl_read_buf!(eviocgname, UAPI_IOC_MAGIC, UAPI_IOC_EVIOCGNAME, c_char);

/// Converts a KeyMappingCode to the platform-specific representation.
/// Current platform: Linux.
pub fn keymap(code: KeyMappingCode) -> u32 {
    KeyMap::from(code).evdev as u32
}

struct Interface {
    to_own: HashSet<String>,
}

impl LibinputInterface for Interface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<RawFd, i32> {
        let fd = OpenOptions::new()
            .custom_flags(flags)
            .read((flags & O_RDONLY != 0) | (flags & O_RDWR != 0))
            .write((flags & O_WRONLY != 0) | (flags & O_RDWR != 0))
            .open(path)
            .map(|file| file.into_raw_fd())
            .map_err(|err| err.raw_os_error().unwrap())?;

        let mut name_buf = [0; 256];
        // SAFETY: buffer is not referenced after end of ioctl call
        unsafe {
            eviocgname(fd, &mut name_buf).map_err(|_| -1)?;
        }

        let first_null = name_buf.iter().position(|&c| c == 0).ok_or(-2)?;
        let cstr = CString::from_vec_with_nul(
            name_buf[..first_null + 1]
                .iter()
                .map(|&x| x as u8)
                .collect(),
        )
        .map_err(|_| -3)?;

        if self.to_own.contains(cstr.to_str().map_err(|_| -3)?) {
            // SAFETY: this ioctl does not impact memory
            unsafe {
                eviocgrab(fd, 1).map_err(|_| -4)?;
            }
        }

        Ok(fd)
    }

    fn close_restricted(&mut self, fd: RawFd) {
        unsafe {
            File::from_raw_fd(fd);
        }
    }
}

pub fn run_input_handler(mut triggers: Triggers) {
    let mut input = Libinput::new_with_udev(Interface {
        to_own: triggers.devices_to_own(),
    });
    input.udev_assign_seat("seat0").unwrap();

    let mut pressed = HashMap::new();

    loop {
        input.dispatch().unwrap();
        for event in &mut input {
            if let Event::Keyboard(KeyboardEvent::Key(e)) = event {
                let device = e.device();
                let name = device.name();
                let pressed = pressed.entry(name.to_owned()).or_insert_with(SetU32::new);
                match e.key_state() {
                    KeyState::Pressed => {
                        pressed.insert(e.key());
                        if triggers.try_run(name, pressed) {
                            pressed.remove(e.key());
                        }
                    }
                    KeyState::Released => {
                        pressed.remove(e.key());
                        triggers.release(name);
                    }
                }
            }
        }
    }
}
