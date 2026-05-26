use crate::config::MouseHotkey;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;

#[derive(Clone, Copy, Debug)]
pub enum MouseHotkeyEvent {
    Pressed,
}

pub fn start_listener(
    hotkey: MouseHotkey,
    consume: bool,
    sender: Sender<MouseHotkeyEvent>,
) -> Result<Option<JoinHandle<()>>, String> {
    if hotkey == MouseHotkey::Disabled {
        eprintln!("mouse hotkey disabled");
        return Ok(None);
    }

    start_platform_listener(hotkey, consume, sender)
}

#[cfg(not(target_os = "windows"))]
fn start_platform_listener(
    hotkey: MouseHotkey,
    _consume: bool,
    _sender: Sender<MouseHotkeyEvent>,
) -> Result<Option<JoinHandle<()>>, String> {
    eprintln!(
        "mouse hotkey '{}' ignored: global mouse hook is only implemented on Windows",
        hotkey.as_str()
    );
    Ok(None)
}

#[cfg(target_os = "windows")]
fn start_platform_listener(
    hotkey: MouseHotkey,
    consume: bool,
    sender: Sender<MouseHotkeyEvent>,
) -> Result<Option<JoinHandle<()>>, String> {
    windows_mouse_hook::start(hotkey, consume, sender).map(Some)
}

#[cfg(target_os = "windows")]
mod windows_mouse_hook {
    use super::MouseHotkeyEvent;
    use crate::config::MouseHotkey;
    use std::ffi::c_void;
    use std::ptr::null_mut;
    use std::sync::mpsc::{Sender, channel};
    use std::sync::{Mutex, OnceLock};
    use std::thread::{self, JoinHandle};
    use std::time::Duration;

    type Bool = i32;
    type Dword = u32;
    type Hhook = *mut c_void;
    type Hinstance = *mut c_void;
    type Hwnd = *mut c_void;
    type Lparam = isize;
    type Lresult = isize;
    type Uint = u32;
    type Wparam = usize;

    const HC_ACTION: i32 = 0;
    const WH_MOUSE_LL: i32 = 14;
    const WM_MBUTTONDOWN: Wparam = 0x0207;
    const WM_MBUTTONUP: Wparam = 0x0208;
    const WM_XBUTTONDOWN: Wparam = 0x020B;
    const WM_XBUTTONUP: Wparam = 0x020C;
    const XBUTTON1: Dword = 0x0001;
    const XBUTTON2: Dword = 0x0002;

    #[derive(Clone, Copy)]
    struct HookConfig {
        hotkey: MouseHotkey,
        consume: bool,
    }

    #[repr(C)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[repr(C)]
    struct Msg {
        hwnd: Hwnd,
        message: Uint,
        w_param: Wparam,
        l_param: Lparam,
        time: Dword,
        pt: Point,
    }

    #[repr(C)]
    struct MsllHookStruct {
        pt: Point,
        mouse_data: Dword,
        flags: Dword,
        time: Dword,
        dw_extra_info: usize,
    }

    static HOOK_CONFIG: OnceLock<HookConfig> = OnceLock::new();
    static HOOK_SENDER: OnceLock<Mutex<Sender<MouseHotkeyEvent>>> = OnceLock::new();

    #[link(name = "user32")]
    unsafe extern "system" {
        fn SetWindowsHookExW(
            id_hook: i32,
            lpfn: Option<unsafe extern "system" fn(i32, Wparam, Lparam) -> Lresult>,
            hmod: Hinstance,
            dw_thread_id: Dword,
        ) -> Hhook;
        fn CallNextHookEx(hhk: Hhook, n_code: i32, w_param: Wparam, l_param: Lparam) -> Lresult;
        fn UnhookWindowsHookEx(hhk: Hhook) -> Bool;
        fn GetMessageW(
            lp_msg: *mut Msg,
            h_wnd: Hwnd,
            w_msg_filter_min: Uint,
            w_msg_filter_max: Uint,
        ) -> Bool;
        fn TranslateMessage(lp_msg: *const Msg) -> Bool;
        fn DispatchMessageW(lp_msg: *const Msg) -> Lresult;
        fn GetLastError() -> Dword;
    }

    pub fn start(
        hotkey: MouseHotkey,
        consume: bool,
        sender: Sender<MouseHotkeyEvent>,
    ) -> Result<JoinHandle<()>, String> {
        HOOK_CONFIG
            .set(HookConfig { hotkey, consume })
            .map_err(|_| "mouse hotkey listener already configured".to_string())?;
        HOOK_SENDER
            .set(Mutex::new(sender))
            .map_err(|_| "mouse hotkey listener already has a sender".to_string())?;

        let (ready_tx, ready_rx) = channel();
        let handle = thread::Builder::new()
            .name("mouse-hotkey-hook".into())
            .spawn(move || {
                // Low-level mouse hooks require a message loop on the installing thread.
                let hook = unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(hook_proc), null_mut(), 0) };
                if hook.is_null() {
                    let code = unsafe { GetLastError() };
                    let _ = ready_tx.send(Err(format!(
                        "SetWindowsHookExW(WH_MOUSE_LL) failed with Windows error {code}"
                    )));
                    return;
                }

                let _ = ready_tx.send(Ok(()));
                eprintln!(
                    "mouse hotkey listener started: hotkey={}, consume={consume}",
                    hotkey.as_str()
                );

                let mut msg = Msg {
                    hwnd: null_mut(),
                    message: 0,
                    w_param: 0,
                    l_param: 0,
                    time: 0,
                    pt: Point { x: 0, y: 0 },
                };

                loop {
                    let result = unsafe { GetMessageW(&mut msg, null_mut(), 0, 0) };
                    if result <= 0 {
                        break;
                    }
                    unsafe {
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                }

                unsafe {
                    UnhookWindowsHookEx(hook);
                }
            })
            .map_err(|e| format!("failed to spawn mouse hotkey thread: {e}"))?;

        match ready_rx.recv_timeout(Duration::from_secs(2)) {
            Ok(Ok(())) => Ok(handle),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(format!("mouse hotkey listener did not initialize: {e}")),
        }
    }

    unsafe extern "system" fn hook_proc(
        n_code: i32,
        w_param: Wparam,
        l_param: Lparam,
    ) -> Lresult {
        if n_code == HC_ACTION {
            let is_down = is_configured_hotkey_down(w_param, l_param);
            let is_up = is_configured_hotkey_up(w_param, l_param);

            if is_down {
                if let Some(sender) = HOOK_SENDER.get()
                    && let Ok(sender) = sender.lock()
                {
                    let _ = sender.send(MouseHotkeyEvent::Pressed);
                }
            }

            if (is_down || is_up) && HOOK_CONFIG.get().map(|c| c.consume).unwrap_or(false) {
                return 1;
            }
        }

        unsafe { CallNextHookEx(null_mut(), n_code, w_param, l_param) }
    }

    fn is_configured_hotkey_down(w_param: Wparam, l_param: Lparam) -> bool {
        let Some(config) = HOOK_CONFIG.get() else {
            return false;
        };

        match (config.hotkey, w_param) {
            (MouseHotkey::Middle, WM_MBUTTONDOWN) => true,
            (MouseHotkey::XButton1, WM_XBUTTONDOWN) => unsafe { xbutton(l_param) == XBUTTON1 },
            (MouseHotkey::XButton2, WM_XBUTTONDOWN) => unsafe { xbutton(l_param) == XBUTTON2 },
            _ => false,
        }
    }

    fn is_configured_hotkey_up(w_param: Wparam, l_param: Lparam) -> bool {
        let Some(config) = HOOK_CONFIG.get() else {
            return false;
        };

        match (config.hotkey, w_param) {
            (MouseHotkey::Middle, WM_MBUTTONUP) => true,
            (MouseHotkey::XButton1, WM_XBUTTONUP) => unsafe { xbutton(l_param) == XBUTTON1 },
            (MouseHotkey::XButton2, WM_XBUTTONUP) => unsafe { xbutton(l_param) == XBUTTON2 },
            _ => false,
        }
    }

    unsafe fn xbutton(l_param: Lparam) -> Dword {
        let hook = unsafe { &*(l_param as *const MsllHookStruct) };
        (hook.mouse_data >> 16) & 0xffff
    }
}
