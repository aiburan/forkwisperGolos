#[derive(Clone, Copy, Debug)]
pub struct PasteTarget {
    hwnd: isize,
}

pub fn capture_foreground_window() -> Option<PasteTarget> {
    capture_platform_foreground_window()
}

pub fn paste_into_target(target: PasteTarget) -> Result<(), String> {
    paste_into_platform_target(target)
}

#[cfg(not(target_os = "windows"))]
fn capture_platform_foreground_window() -> Option<PasteTarget> {
    None
}

#[cfg(not(target_os = "windows"))]
fn paste_into_platform_target(_target: PasteTarget) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "windows")]
fn capture_platform_foreground_window() -> Option<PasteTarget> {
    let hwnd = unsafe { windows_auto_paste::GetForegroundWindow() };
    if hwnd == 0 {
        None
    } else {
        Some(PasteTarget { hwnd })
    }
}

#[cfg(target_os = "windows")]
fn paste_into_platform_target(target: PasteTarget) -> Result<(), String> {
    windows_auto_paste::paste_into_target(target.hwnd)
}

#[cfg(target_os = "windows")]
mod windows_auto_paste {
    use std::mem::size_of;
    use std::thread;
    use std::time::Duration;

    type Bool = i32;
    type Dword = u32;
    type Hwnd = isize;
    type Int = i32;
    type Uint = u32;
    type UlongPtr = usize;
    type Word = u16;

    const INPUT_KEYBOARD: Dword = 1;
    const KEYEVENTF_KEYUP: Dword = 0x0002;
    const VK_CONTROL: Word = 0x11;
    const VK_V: Word = 0x56;

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct KeybdInput {
        w_vk: Word,
        w_scan: Word,
        dw_flags: Dword,
        time: Dword,
        dw_extra_info: UlongPtr,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct MouseInput {
        dx: i32,
        dy: i32,
        mouse_data: Dword,
        dw_flags: Dword,
        time: Dword,
        dw_extra_info: UlongPtr,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct HardwareInput {
        u_msg: Dword,
        w_param_l: Word,
        w_param_h: Word,
    }

    #[repr(C)]
    union InputData {
        mi: MouseInput,
        ki: KeybdInput,
        hi: HardwareInput,
    }

    #[repr(C)]
    struct Input {
        r#type: Dword,
        data: InputData,
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        pub fn GetForegroundWindow() -> Hwnd;
        fn IsWindow(hwnd: Hwnd) -> Bool;
        fn SetForegroundWindow(hwnd: Hwnd) -> Bool;
        fn SendInput(c_inputs: Uint, p_inputs: *mut Input, cb_size: Int) -> Uint;
    }

    pub fn paste_into_target(hwnd: Hwnd) -> Result<(), String> {
        if hwnd == 0 {
            return Err("auto paste target missing".into());
        }

        if unsafe { IsWindow(hwnd) } == 0 {
            return Err("auto paste target window is no longer valid".into());
        }

        if unsafe { SetForegroundWindow(hwnd) } == 0 {
            return Err("SetForegroundWindow failed".into());
        }

        thread::sleep(Duration::from_millis(120));

        if unsafe { GetForegroundWindow() } != hwnd {
            return Err("foreground window did not restore to saved target".into());
        }

        send_ctrl_v()
    }

    fn send_ctrl_v() -> Result<(), String> {
        let mut inputs = [
            keyboard_input(VK_CONTROL, 0),
            keyboard_input(VK_V, 0),
            keyboard_input(VK_V, KEYEVENTF_KEYUP),
            keyboard_input(VK_CONTROL, KEYEVENTF_KEYUP),
        ];

        let sent = unsafe {
            SendInput(
                inputs.len() as Uint,
                inputs.as_mut_ptr(),
                size_of::<Input>() as Int,
            )
        };

        if sent == inputs.len() as Uint {
            Ok(())
        } else {
            Err(format!(
                "SendInput sent {sent} of {} keyboard events",
                inputs.len()
            ))
        }
    }

    fn keyboard_input(vk: Word, flags: Dword) -> Input {
        Input {
            r#type: INPUT_KEYBOARD,
            data: InputData {
                ki: KeybdInput {
                    w_vk: vk,
                    w_scan: 0,
                    dw_flags: flags,
                    time: 0,
                    dw_extra_info: 0,
                },
            },
        }
    }
}
