#[cfg(windows)]
mod win_api {
    use std::ffi::c_void;
    use std::ptr::{null, null_mut};

    pub type HWND = *mut c_void;
    pub type HRGN = *mut c_void;
    pub type HDC = *mut c_void;
    pub type HFONT = *mut c_void;
    pub type HBRUSH = *mut c_void;
    pub type HPEN = *mut c_void;
    pub type HGDIOBJ = *mut c_void;
    pub type HINSTANCE = *mut c_void;
    pub type HMENU = *mut c_void;
    pub type HCURSOR = *mut c_void;
    pub type BOOL = i32;

    pub const BAR_H: i32 = 26;
    pub const CORNER_DIAM: i32 = 28;
    const BTN_W: i32 = 28;

    const WS_POPUP: u32 = 0x8000_0000;
    const WS_VISIBLE: u32 = 0x1000_0000;
    const WS_EX_TOOLWINDOW: u32 = 0x0000_0080;
    const WS_EX_TOPMOST: u32 = 0x0000_0008;

    const SW_HIDE: i32 = 0;
    const SW_SHOWNOACTIVATE: i32 = 4;
    const SW_MINIMIZE: i32 = 6;

    const SWP_NOSIZE: u32 = 0x0001;
    const SWP_NOMOVE: u32 = 0x0002;
    const SWP_NOZORDER: u32 = 0x0004;
    const SWP_NOACTIVATE: u32 = 0x0010;
    const HWND_TOP: HWND = 0 as HWND;
    const HWND_TOPMOST: HWND = (-1isize) as HWND;

    const WM_DESTROY: u32 = 0x0002;
    const WM_MOVE: u32 = 0x0003;
    const WM_PAINT: u32 = 0x000F;
    const WM_CLOSE: u32 = 0x0010;
    const WM_MOUSEACTIVATE: u32 = 0x0021;
    const WM_NCHITTEST: u32 = 0x0084;
    const WM_NCLBUTTONDOWN: u32 = 0x00A1;
    const WM_TIMER: u32 = 0x0113;
    const WM_MOUSEMOVE: u32 = 0x0200;
    const WM_LBUTTONUP: u32 = 0x0202;
    const WM_MOUSELEAVE: u32 = 0x02A3;

    const HTCLIENT: isize = 1;
    const HTCAPTION: isize = 2;

    const TME_LEAVE: u32 = 0x0000_0002;
    const TRANSPARENT: i32 = 1;
    const DT_LEFT: u32 = 0x0000;
    const DT_VCENTER: u32 = 0x0004;
    const DT_SINGLELINE: u32 = 0x0020;
    const DT_END_ELLIPSIS: u32 = 0x0000_8000;
    const GWLP_USERDATA: i32 = -21;

    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    pub struct RECT {
        pub left: i32,
        pub top: i32,
        pub right: i32,
        pub bottom: i32,
    }

    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    pub struct POINT {
        pub x: i32,
        pub y: i32,
    }

    #[repr(C)]
    pub struct PAINTSTRUCT {
        pub hdc: HDC,
        pub f_erase: BOOL,
        pub rc_paint: RECT,
        pub f_restore: BOOL,
        pub f_inc_update: BOOL,
        pub rgb_reserved: [u8; 32],
    }

    #[repr(C)]
    pub struct MSG {
        pub hwnd: HWND,
        pub message: u32,
        pub w_param: usize,
        pub l_param: isize,
        pub time: u32,
        pub pt: POINT,
    }

    #[repr(C)]
    pub struct WNDCLASSEXW {
        pub cb_size: u32,
        pub style: u32,
        pub lpfn_wnd_proc: unsafe extern "system" fn(HWND, u32, usize, isize) -> isize,
        pub cb_cls_extra: i32,
        pub cb_wnd_extra: i32,
        pub h_instance: HINSTANCE,
        pub h_icon: *mut c_void,
        pub h_cursor: HCURSOR,
        pub hbr_background: HBRUSH,
        pub lpsz_menu_name: *const u16,
        pub lpsz_class_name: *const u16,
        pub h_icon_sm: *mut c_void,
    }

    #[repr(C)]
    pub struct TRACKMOUSEEVENT {
        pub cb_size: u32,
        pub dw_flags: u32,
        pub hwnd_track: HWND,
        pub dw_hover_time: u32,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        pub fn GetModuleHandleW(lpModuleName: *const u16) -> HINSTANCE;
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        pub fn EnumWindows(
            lpEnumFunc: unsafe extern "system" fn(HWND, isize) -> BOOL,
            lParam: isize,
        ) -> BOOL;
        pub fn GetWindowThreadProcessId(hWnd: HWND, lpdwProcessId: *mut u32) -> u32;
        pub fn IsWindowVisible(hWnd: HWND) -> BOOL;
        pub fn IsWindow(hWnd: HWND) -> BOOL;
        pub fn IsIconic(hWnd: HWND) -> BOOL;
        pub fn GetWindowRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
        pub fn GetClientRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
        pub fn SetWindowRgn(hWnd: HWND, hRgn: HRGN, bRedraw: BOOL) -> i32;
        pub fn SetWindowPos(
            hWnd: HWND,
            hWndInsertAfter: HWND,
            X: i32,
            Y: i32,
            cx: i32,
            cy: i32,
            uFlags: u32,
        ) -> BOOL;
        pub fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> BOOL;
        pub fn PostMessageW(hWnd: HWND, Msg: u32, wParam: usize, lParam: isize) -> BOOL;
        pub fn GetForegroundWindow() -> HWND;
        pub fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> u16;
        pub fn CreateWindowExW(
            dwExStyle: u32,
            lpClassName: *const u16,
            lpWindowName: *const u16,
            dwStyle: u32,
            X: i32,
            Y: i32,
            nWidth: i32,
            nHeight: i32,
            hWndParent: HWND,
            hMenu: HMENU,
            hInstance: HINSTANCE,
            lpParam: *mut c_void,
        ) -> HWND;
        pub fn DestroyWindow(hWnd: HWND) -> BOOL;
        pub fn DefWindowProcW(hWnd: HWND, Msg: u32, wParam: usize, lParam: isize) -> isize;
        pub fn GetMessageW(
            lpMsg: *mut MSG,
            hWnd: HWND,
            wMsgFilterMin: u32,
            wMsgFilterMax: u32,
        ) -> BOOL;
        pub fn TranslateMessage(lpMsg: *const MSG) -> BOOL;
        pub fn DispatchMessageW(lpMsg: *const MSG) -> isize;
        pub fn PostQuitMessage(nExitCode: i32);
        pub fn SetTimer(
            hWnd: HWND,
            nIDEvent: usize,
            uElapse: u32,
            lpTimerFunc: *const c_void,
        ) -> usize;
        pub fn KillTimer(hWnd: HWND, uIDEvent: usize) -> BOOL;
        pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: i32, dwNewLong: isize) -> isize;
        pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: i32) -> isize;
        pub fn BeginPaint(hWnd: HWND, lpPaint: *mut PAINTSTRUCT) -> HDC;
        pub fn EndPaint(hWnd: HWND, lpPaint: *const PAINTSTRUCT) -> BOOL;
        pub fn InvalidateRect(hWnd: HWND, lpRect: *const RECT, bErase: BOOL) -> BOOL;
        pub fn FillRect(hDC: HDC, lprc: *const RECT, hbr: HBRUSH) -> i32;
        pub fn DrawTextW(
            hdc: HDC,
            lpchText: *const u16,
            cchText: i32,
            lprc: *mut RECT,
            format: u32,
        ) -> i32;
        pub fn ScreenToClient(hWnd: HWND, lpPoint: *mut POINT) -> BOOL;
        pub fn TrackMouseEvent(lpEventTrack: *mut TRACKMOUSEEVENT) -> BOOL;
        pub fn LoadCursorW(hInstance: HINSTANCE, lpCursorName: *const u16) -> HCURSOR;
        pub fn SystemParametersInfoW(
            uiAction: u32,
            uiParam: u32,
            pvParam: *mut c_void,
            fWinIni: u32,
        ) -> BOOL;
    }

    #[link(name = "gdi32")]
    unsafe extern "system" {
        pub fn CreateRoundRectRgn(x1: i32, y1: i32, x2: i32, y2: i32, w: i32, h: i32) -> HRGN;
        pub fn CreateSolidBrush(color: u32) -> HBRUSH;
        pub fn CreatePen(iStyle: i32, cWidth: i32, color: u32) -> HPEN;
        pub fn CreateFontW(
            cHeight: i32,
            cWidth: i32,
            cEscapement: i32,
            cOrientation: i32,
            cWeight: i32,
            bItalic: u32,
            bUnderline: u32,
            bStrikeOut: u32,
            iCharSet: u32,
            iOutPrecision: u32,
            iClipPrecision: u32,
            iQuality: u32,
            iPitchAndFamily: u32,
            pszFaceName: *const u16,
        ) -> HFONT;
        pub fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ;
        pub fn DeleteObject(ho: HGDIOBJ) -> BOOL;
        pub fn SetBkMode(hdc: HDC, mode: i32) -> i32;
        pub fn SetTextColor(hdc: HDC, color: u32) -> u32;
        pub fn MoveToEx(hdc: HDC, x: i32, y: i32, lppt: *mut POINT) -> BOOL;
        pub fn LineTo(hdc: HDC, x: i32, y: i32) -> BOOL;
    }

    #[link(name = "dwmapi")]
    unsafe extern "system" {
        pub fn DwmSetWindowAttribute(
            hwnd: HWND,
            dwAttribute: u32,
            pvAttribute: *const c_void,
            cbAttribute: u32,
        ) -> i32;
    }

    const fn rgb(r: u8, g: u8, b: u8) -> u32 {
        (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
    }

    struct SearchCtx {
        target_pid: u32,
        found: HWND,
    }

    unsafe extern "system" fn enum_cb(hwnd: HWND, lparam: isize) -> BOOL {
        let ctx = unsafe { &mut *(lparam as *mut SearchCtx) };
        let mut pid = 0u32;
        unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
        if pid == ctx.target_pid && unsafe { IsWindowVisible(hwnd) } != 0 {
            ctx.found = hwnd;
            return 0;
        }
        1
    }

    pub fn find_window_by_pid(pid: u32) -> Option<HWND> {
        let mut ctx = SearchCtx {
            target_pid: pid,
            found: null_mut(),
        };
        unsafe { EnumWindows(enum_cb, (&mut ctx as *mut SearchCtx) as isize) };
        if ctx.found.is_null() {
            None
        } else {
            Some(ctx.found)
        }
    }

    struct BarState {
        scrcpy_hwnd: HWND,
        title_utf16: Vec<u16>,
        font: HFONT,
        always_on_top: bool,
        hover_btn: u8,
        tracking_leave: bool,
        syncing_pos: bool,
        bar_hidden: bool,
        last_scrcpy_rect: RECT,
    }

    fn apply_rounded_regions(bar_hwnd: HWND, scrcpy_hwnd: HWND, w: i32, h: i32) {
        if w <= 0 || h <= 0 {
            return;
        }
        unsafe {
            let bar_rgn = CreateRoundRectRgn(0, 0, w + 1, BAR_H + 20, CORNER_DIAM, CORNER_DIAM);
            if !bar_rgn.is_null() {
                SetWindowRgn(bar_hwnd, bar_rgn, 1);
            }
            let body_rgn = CreateRoundRectRgn(0, -20, w + 1, h + 1, CORNER_DIAM, CORNER_DIAM);
            if !body_rgn.is_null() {
                SetWindowRgn(scrcpy_hwnd, body_rgn, 1);
            }
        }
    }

    fn hit_button(cx: i32, w: i32) -> u8 {
        if cx >= w - BTN_W && cx <= w {
            2
        } else if cx >= w - BTN_W * 2 && cx < w - BTN_W {
            1
        } else {
            0
        }
    }

    unsafe extern "system" fn bar_wnd_proc(
        hwnd: HWND,
        msg: u32,
        w_param: usize,
        l_param: isize,
    ) -> isize {
        let ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut BarState;
        if ptr.is_null() {
            return unsafe { DefWindowProcW(hwnd, msg, w_param, l_param) };
        }
        let st = unsafe { &mut *ptr };

        match msg {
            WM_MOUSEACTIVATE | WM_NCLBUTTONDOWN => {
                if unsafe { IsWindow(st.scrcpy_hwnd) } != 0 {
                    unsafe {
                        SetWindowPos(
                            st.scrcpy_hwnd,
                            hwnd,
                            0,
                            0,
                            0,
                            0,
                            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                        );
                    }
                }
                unsafe { DefWindowProcW(hwnd, msg, w_param, l_param) }
            }
            WM_NCHITTEST => {
                let x = (l_param & 0xFFFF) as i16 as i32;
                let y = ((l_param >> 16) & 0xFFFF) as i16 as i32;
                let mut pt = POINT { x, y };
                let mut rc = RECT::default();
                unsafe {
                    ScreenToClient(hwnd, &mut pt);
                    GetClientRect(hwnd, &mut rc);
                }
                if hit_button(pt.x, rc.right) != 0 {
                    HTCLIENT
                } else {
                    HTCAPTION
                }
            }
            WM_MOUSEMOVE => {
                let cx = (l_param & 0xFFFF) as i16 as i32;
                let mut rc = RECT::default();
                unsafe { GetClientRect(hwnd, &mut rc) };
                let btn = hit_button(cx, rc.right);
                if btn != st.hover_btn {
                    st.hover_btn = btn;
                    unsafe { InvalidateRect(hwnd, null(), 0) };
                }
                if !st.tracking_leave {
                    let mut tme = TRACKMOUSEEVENT {
                        cb_size: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                        dw_flags: TME_LEAVE,
                        hwnd_track: hwnd,
                        dw_hover_time: 0,
                    };
                    if unsafe { TrackMouseEvent(&mut tme) } != 0 {
                        st.tracking_leave = true;
                    }
                }
                0
            }
            WM_MOUSELEAVE => {
                st.tracking_leave = false;
                if st.hover_btn != 0 {
                    st.hover_btn = 0;
                    unsafe { InvalidateRect(hwnd, null(), 0) };
                }
                0
            }
            WM_LBUTTONUP => {
                let cx = (l_param & 0xFFFF) as i16 as i32;
                let mut rc = RECT::default();
                unsafe { GetClientRect(hwnd, &mut rc) };
                match hit_button(cx, rc.right) {
                    1 => {
                        st.bar_hidden = true;
                        unsafe {
                            ShowWindow(hwnd, SW_HIDE);
                            ShowWindow(st.scrcpy_hwnd, SW_MINIMIZE);
                        }
                    }
                    2 => unsafe {
                        PostMessageW(st.scrcpy_hwnd, WM_CLOSE, 0, 0);
                    },
                    _ => {}
                }
                0
            }
            WM_MOVE => {
                if !st.syncing_pos && unsafe { IsWindow(st.scrcpy_hwnd) } != 0 {
                    let mut bar_rc = RECT::default();
                    if unsafe { GetWindowRect(hwnd, &mut bar_rc) } != 0 {
                        st.syncing_pos = true;
                        unsafe {
                            SetWindowPos(
                                st.scrcpy_hwnd,
                                null_mut(),
                                bar_rc.left,
                                bar_rc.top + BAR_H,
                                0,
                                0,
                                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                            );
                            GetWindowRect(st.scrcpy_hwnd, &mut st.last_scrcpy_rect);
                        }
                        st.syncing_pos = false;
                    }
                }
                0
            }
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = unsafe { std::mem::zeroed() };
                let hdc = unsafe { BeginPaint(hwnd, &mut ps) };
                let mut rc = RECT::default();
                unsafe { GetClientRect(hwnd, &mut rc) };
                let w = rc.right;

                unsafe {
                    let bg = CreateSolidBrush(rgb(24, 26, 32));
                    FillRect(hdc, &rc, bg);
                    DeleteObject(bg);

                    let border_rc = RECT {
                        left: 0,
                        top: BAR_H - 1,
                        right: w,
                        bottom: BAR_H,
                    };
                    let border_br = CreateSolidBrush(rgb(40, 44, 54));
                    FillRect(hdc, &border_rc, border_br);
                    DeleteObject(border_br);

                    if st.hover_btn == 1 {
                        let min_rc = RECT {
                            left: w - BTN_W * 2,
                            top: 0,
                            right: w - BTN_W,
                            bottom: BAR_H - 1,
                        };
                        let hbr = CreateSolidBrush(rgb(46, 51, 64));
                        FillRect(hdc, &min_rc, hbr);
                        DeleteObject(hbr);
                    } else if st.hover_btn == 2 {
                        let cls_rc = RECT {
                            left: w - BTN_W,
                            top: 0,
                            right: w,
                            bottom: BAR_H - 1,
                        };
                        let hbr = CreateSolidBrush(rgb(196, 43, 28));
                        FillRect(hdc, &cls_rc, hbr);
                        DeleteObject(hbr);
                    }

                    let old_font = if !st.font.is_null() {
                        SelectObject(hdc, st.font)
                    } else {
                        null_mut()
                    };
                    SetBkMode(hdc, TRANSPARENT);
                    SetTextColor(hdc, rgb(184, 192, 204));
                    let mut text_rc = RECT {
                        left: 12,
                        top: 0,
                        right: (w - BTN_W * 2 - 6).max(12),
                        bottom: BAR_H - 1,
                    };
                    DrawTextW(
                        hdc,
                        st.title_utf16.as_ptr(),
                        -1,
                        &mut text_rc,
                        DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
                    );
                    if !old_font.is_null() {
                        SelectObject(hdc, old_font);
                    }

                    let min_pen = CreatePen(0, 1, rgb(184, 192, 204));
                    let old_pen = SelectObject(hdc, min_pen);
                    let mx = w - BTN_W * 2 + BTN_W / 2;
                    let my = BAR_H / 2;
                    MoveToEx(hdc, mx - 4, my, null_mut());
                    LineTo(hdc, mx + 5, my);

                    let cls_color = if st.hover_btn == 2 {
                        rgb(255, 255, 255)
                    } else {
                        rgb(184, 192, 204)
                    };
                    let cls_pen = CreatePen(0, 1, cls_color);
                    SelectObject(hdc, cls_pen);
                    let cx = w - BTN_W + BTN_W / 2;
                    let cy = BAR_H / 2;
                    MoveToEx(hdc, cx - 4, cy - 4, null_mut());
                    LineTo(hdc, cx + 5, cy + 5);
                    MoveToEx(hdc, cx + 4, cy - 4, null_mut());
                    LineTo(hdc, cx - 5, cy + 5);

                    SelectObject(hdc, old_pen);
                    DeleteObject(min_pen);
                    DeleteObject(cls_pen);

                    EndPaint(hwnd, &ps);
                }
                0
            }
            WM_TIMER => {
                if unsafe { IsWindow(st.scrcpy_hwnd) } == 0 {
                    unsafe {
                        KillTimer(hwnd, 1);
                        DestroyWindow(hwnd);
                    }
                    return 0;
                }
                let hidden = unsafe {
                    IsIconic(st.scrcpy_hwnd) != 0 || IsWindowVisible(st.scrcpy_hwnd) == 0
                };
                if hidden {
                    if !st.bar_hidden {
                        st.bar_hidden = true;
                        unsafe { ShowWindow(hwnd, SW_HIDE) };
                    }
                    return 0;
                } else if st.bar_hidden {
                    st.bar_hidden = false;
                    unsafe { ShowWindow(hwnd, SW_SHOWNOACTIVATE) };
                }

                let mut srect = RECT::default();
                if unsafe { GetWindowRect(st.scrcpy_hwnd, &mut srect) } != 0 {
                    let sw = srect.right - srect.left;
                    let sh = srect.bottom - srect.top;
                    let lw = st.last_scrcpy_rect.right - st.last_scrcpy_rect.left;
                    let lh = st.last_scrcpy_rect.bottom - st.last_scrcpy_rect.top;
                    let moved = srect.left != st.last_scrcpy_rect.left
                        || srect.top != st.last_scrcpy_rect.top;
                    let resized = sw != lw || sh != lh;

                    if (moved || resized) && !st.syncing_pos {
                        st.syncing_pos = true;
                        let z = if st.always_on_top { HWND_TOPMOST } else { HWND_TOP };
                        unsafe {
                            SetWindowPos(
                                hwnd,
                                z,
                                srect.left,
                                srect.top - BAR_H,
                                sw,
                                BAR_H,
                                SWP_NOACTIVATE,
                            );
                        }
                        if resized {
                            apply_rounded_regions(hwnd, st.scrcpy_hwnd, sw, sh);
                            unsafe { InvalidateRect(hwnd, null(), 0) };
                        }
                        st.last_scrcpy_rect = srect;
                        st.syncing_pos = false;
                    } else if unsafe { GetForegroundWindow() } == st.scrcpy_hwnd {
                        let z = if st.always_on_top { HWND_TOPMOST } else { HWND_TOP };
                        unsafe {
                            SetWindowPos(
                                hwnd,
                                z,
                                0,
                                0,
                                0,
                                0,
                                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                            );
                        }
                    }
                }
                0
            }
            WM_DESTROY => {
                unsafe { PostQuitMessage(0) };
                0
            }
            _ => unsafe { DefWindowProcW(hwnd, msg, w_param, l_param) },
        }
    }

    pub fn run_rounded_with_thin_bar(scrcpy_hwnd: HWND, title: String, always_on_top: bool) {
        let mut srect = RECT::default();
        if unsafe { GetWindowRect(scrcpy_hwnd, &mut srect) } == 0 {
            return;
        }
        if srect.top < BAR_H + 8 {
            let new_top = BAR_H + 8;
            unsafe {
                SetWindowPos(
                    scrcpy_hwnd,
                    null_mut(),
                    srect.left,
                    new_top,
                    0,
                    0,
                    SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                );
                GetWindowRect(scrcpy_hwnd, &mut srect);
            }
        }

        let sw = srect.right - srect.left;
        let sh = srect.bottom - srect.top;

        let class_name: Vec<u16> = "AndroidMirrorThinBar\0".encode_utf16().collect();
        let font_name: Vec<u16> = "Segoe UI\0".encode_utf16().collect();
        let mut title_utf16: Vec<u16> = title.encode_utf16().collect();
        title_utf16.push(0);

        let h_inst = unsafe { GetModuleHandleW(null()) };
        let wc = WNDCLASSEXW {
            cb_size: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0x0003,
            lpfn_wnd_proc: bar_wnd_proc,
            cb_cls_extra: 0,
            cb_wnd_extra: 0,
            h_instance: h_inst,
            h_icon: null_mut(),
            h_cursor: unsafe { LoadCursorW(null_mut(), 32512 as *const u16) },
            hbr_background: null_mut(),
            lpsz_menu_name: null(),
            lpsz_class_name: class_name.as_ptr(),
            h_icon_sm: null_mut(),
        };
        unsafe { RegisterClassExW(&wc) };

        let ex_style = WS_EX_TOOLWINDOW | if always_on_top { WS_EX_TOPMOST } else { 0 };
        let bar_hwnd = unsafe {
            CreateWindowExW(
                ex_style,
                class_name.as_ptr(),
                title_utf16.as_ptr(),
                WS_POPUP | WS_VISIBLE,
                srect.left,
                srect.top - BAR_H,
                sw,
                BAR_H,
                scrcpy_hwnd,
                null_mut(),
                h_inst,
                null_mut(),
            )
        };
        if bar_hwnd.is_null() {
            return;
        }

        let font = unsafe {
            CreateFontW(
                -12,
                0,
                0,
                0,
                500,
                0,
                0,
                0,
                1,
                0,
                0,
                5,
                0,
                font_name.as_ptr(),
            )
        };

        let mut state = Box::new(BarState {
            scrcpy_hwnd,
            title_utf16,
            font,
            always_on_top,
            hover_btn: 0,
            tracking_leave: false,
            syncing_pos: false,
            bar_hidden: false,
            last_scrcpy_rect: srect,
        });

        unsafe {
            SetWindowLongPtrW(
                bar_hwnd,
                GWLP_USERDATA,
                (&mut *state as *mut BarState) as isize,
            );
        }

        apply_rounded_regions(bar_hwnd, scrcpy_hwnd, sw, sh);
        unsafe {
            SetTimer(bar_hwnd, 1, 80, null());
            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            SetWindowLongPtrW(bar_hwnd, GWLP_USERDATA, 0);
            if !state.font.is_null() {
                DeleteObject(state.font);
            }
        }
    }
}

/// Center the app window within the usable desktop work area (excluding the taskbar).
#[cfg(windows)]
pub fn center_own_window_in_work_area() {
    use std::thread;
    use std::time::Duration;
    use win_api::*;

    let pid = std::process::id();
    thread::spawn(move || {
        let mut hwnd: Option<HWND> = None;
        for _ in 0..30 {
            if let Some(h) = find_window_by_pid(pid) {
                hwnd = Some(h);
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }
        let Some(hwnd) = hwnd else { return };

        let mut work = RECT::default();
        let mut rc = RECT::default();
        unsafe {
            if SystemParametersInfoW(0x0030, 0, (&mut work as *mut RECT).cast(), 0) == 0 {
                return;
            }
            if GetWindowRect(hwnd, &mut rc) == 0 {
                return;
            }
        }
        let work_w = work.right - work.left;
        let work_h = work.bottom - work.top;
        let win_w = (rc.right - rc.left).min(work_w - 16);
        let win_h = (rc.bottom - rc.top).min(work_h - 16);
        let x = work.left + (work_w - win_w) / 2;
        let y = work.top + (work_h - win_h) / 2;

        unsafe {
            SetWindowPos(
                hwnd,
                std::ptr::null_mut(),
                x,
                y,
                win_w,
                win_h,
                0x0004 | 0x0010,
            );
        }
    });
}

#[cfg(not(windows))]
pub fn center_own_window_in_work_area() {}

/// Apply Windows 10 dark title bar and, when `rounded` is enabled, attach a rounded 26px drag bar.
#[cfg(windows)]
pub fn apply_window_style_loop(pid: u32, rounded: bool, title: String, always_on_top: bool) {
    use std::thread;
    use std::time::Duration;
    use win_api::*;

    let mut hwnd: Option<HWND> = None;
    for _ in 0..40 {
        if let Some(h) = find_window_by_pid(pid) {
            hwnd = Some(h);
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }

    let Some(hwnd) = hwnd else { return };
    let dark: i32 = 1;
    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            20,
            (&dark as *const i32).cast(),
            std::mem::size_of::<i32>() as u32,
        );
    }

    if !rounded {
        return;
    }

    run_rounded_with_thin_bar(hwnd, title, always_on_top);
}
