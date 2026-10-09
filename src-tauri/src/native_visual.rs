//! Host-owned local visual inspection. Pixels are derived by the bounded
//! processor, never decoded by this UI and never supplied by an IPC caller.
//! Viewing does not construct disclosure/financial/file authority.
use std::time::Instant;

#[derive(Clone)]
pub(crate) struct VisualPage {
    pub label: String,
    pub width: u32,
    pub height: u32,
    bgra: Vec<u8>,
}

impl VisualPage {
    pub(crate) fn pixel_sha256(&self) -> String {
        use sha2::{Digest, Sha256};
        Sha256::digest(&self.bgra)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }
    pub(crate) fn from_rgba(
        label: String,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> Result<Self, &'static str> {
        if label.is_empty()
            || label.len() > 160
            || label.chars().any(char::is_control)
            || !(1..=4096).contains(&width)
            || !(1..=4096).contains(&height)
            || u64::from(width) * u64::from(height) > 4_194_304
            || rgba.len() != width as usize * height as usize * 4
        {
            return Err("visual_preview_invalid");
        }
        // Display transparency over white. The immutable transmitted/stored
        // PNG still preserves its alpha channel; this is disclosed in the UI.
        let bgra = rgba
            .chunks_exact(4)
            .flat_map(|pixel| {
                let alpha = u32::from(pixel[3]);
                let composite = |channel: u8| {
                    ((u32::from(channel) * alpha + 255 * (255 - alpha) + 127) / 255) as u8
                };
                [
                    composite(pixel[2]),
                    composite(pixel[1]),
                    composite(pixel[0]),
                    255,
                ]
            })
            .collect();
        Ok(Self {
            label,
            width,
            height,
            bgra,
        })
    }
}

pub(crate) fn inspect_owned(
    pages: &[VisualPage],
    owner: usize,
    deadline: Instant,
    require_all: bool,
) -> Result<bool, &'static str> {
    inspect_owned_cancellable(
        pages,
        owner,
        deadline,
        require_all,
        &crate::native_review::ReviewCancellation::default(),
    )
}

pub(crate) fn inspect_owned_cancellable(
    pages: &[VisualPage],
    owner: usize,
    deadline: Instant,
    require_all: bool,
    cancellation: &crate::native_review::ReviewCancellation,
) -> Result<bool, &'static str> {
    if cancellation.is_cancelled() {
        return Ok(false);
    }
    if pages.is_empty()
        || pages.len() > 24
        || pages.iter().any(|page| {
            page.label.is_empty()
                || page.label.len() > 160
                || page.label.chars().any(char::is_control)
                || !(1..=4096).contains(&page.width)
                || !(1..=4096).contains(&page.height)
                || u64::from(page.width) * u64::from(page.height) > 4_194_304
                || page.bgra.len() as u64 != u64::from(page.width) * u64::from(page.height) * 4
        })
        || pages.iter().map(|page| page.bgra.len()).sum::<usize>() > 64 * 1024 * 1024
        || Instant::now() >= deadline
    {
        return Err("visual_preview_limit");
    }
    #[cfg(windows)]
    {
        crate::native_review::validate_owner_handle(owner)
            .map_err(|_| "native_owner_unavailable")?;
        windows::show(pages, owner, deadline, require_all, cancellation)
    }
    #[cfg(not(windows))]
    {
        let _ = (pages, owner, deadline, require_all, cancellation);
        Err("unsupported_platform")
    }
}

#[cfg(windows)]
mod windows {
    use super::*;
    use std::{
        cell::{Cell, RefCell},
        ffi::c_void,
        ptr,
    };
    use windows_sys::Win32::{
        Foundation::{HWND, RECT},
        Graphics::Gdi::{
            BeginPaint, CreateSolidBrush, DeleteObject, EndPaint, FillRect, IntersectClipRect,
            InvalidateRect, RestoreDC, SaveDC, SetStretchBltMode, StretchDIBits, BITMAPINFO,
            BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HALFTONE, SRCCOPY,
        },
        UI::WindowsAndMessaging::*,
    };
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetModuleHandleW(name: *const u16) -> *mut c_void;
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        fn EnableWindow(window: HWND, enabled: i32) -> i32;
        fn SetFocus(window: HWND) -> HWND;
    }
    const SLOT: i32 = (2 * std::mem::size_of::<isize>()) as i32;
    const PREVIOUS: usize = 101;
    const NEXT: usize = 102;
    const FIT: usize = 103;
    const ACTUAL: usize = 104;
    const IN: usize = 105;
    const OUT: usize = 106;
    const LEFT: usize = 107;
    const RIGHT: usize = 108;
    const UP: usize = 109;
    const DOWN: usize = 110;
    const DONE: usize = 111;
    const LABEL: usize = 112;

    struct State<'a> {
        pages: &'a [VisualPage],
        page: Cell<usize>,
        zoom: Cell<u32>,
        x: Cell<i32>,
        y: Cell<i32>,
        visited: RefCell<Vec<bool>>,
        deadline: Instant,
        cancellation: &'a crate::native_review::ReviewCancellation,
        require_all: bool,
        done: Cell<bool>,
        failed: Cell<bool>,
    }
    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }
    fn template() -> Vec<u32> {
        let style = WS_POPUP | WS_CAPTION | WS_SYSMENU | DS_MODALFRAME as u32;
        let mut w = vec![
            style as u16,
            (style >> 16) as u16,
            0,
            0,
            0,
            0,
            0,
            600,
            390,
            0,
            0,
        ];
        w.extend(wide("Aster: processed pages — local visual inspection"));
        if !w.len().is_multiple_of(2) {
            w.push(0);
        }
        w.chunks_exact(2)
            .map(|p| u32::from(p[0]) | (u32::from(p[1]) << 16))
            .collect()
    }
    unsafe fn control(
        dialog: HWND,
        class: &str,
        text: &str,
        id: usize,
        rect: (i32, i32, i32, i32),
        style: u32,
    ) -> HWND {
        let c = wide(class);
        let t = wide(text);
        let (x, y, w, h) = rect;
        unsafe {
            CreateWindowExW(
                0,
                c.as_ptr(),
                t.as_ptr(),
                WS_CHILD | WS_VISIBLE | style,
                x,
                y,
                w,
                h,
                dialog,
                id as *mut c_void,
                GetModuleHandleW(ptr::null()),
                ptr::null(),
            )
        }
    }
    unsafe fn update(dialog: HWND, state: &State<'_>) {
        let page = &state.pages[state.page.get()];
        let zoom = if state.zoom.get() == 0 {
            "Fit".to_owned()
        } else {
            format!("{}%", state.zoom.get())
        };
        let label = wide(&format!(
            "{} · page {}/{} · {} · transparency displayed over white",
            page.label,
            state.page.get() + 1,
            state.pages.len(),
            zoom
        ));
        unsafe {
            SetWindowTextW(GetDlgItem(dialog, LABEL as i32), label.as_ptr());
            EnableWindow(
                GetDlgItem(dialog, PREVIOUS as i32),
                i32::from(state.page.get() > 0),
            );
            EnableWindow(
                GetDlgItem(dialog, NEXT as i32),
                i32::from(state.page.get() + 1 < state.pages.len()),
            );
            EnableWindow(
                GetDlgItem(dialog, DONE as i32),
                i32::from(!state.require_all || state.visited.borrow().iter().all(|v| *v)),
            );
        }
    }
    unsafe fn paint(dialog: HWND, state: &State<'_>) {
        let mut ps = unsafe { std::mem::zeroed() };
        let dc = unsafe { BeginPaint(dialog, &mut ps) };
        if dc.is_null() {
            state.failed.set(true);
            return;
        }
        let mut bounds: RECT = unsafe { std::mem::zeroed() };
        if unsafe { GetClientRect(dialog, &mut bounds) } == 0 {
            state.failed.set(true);
            unsafe { EndPaint(dialog, &ps) };
            return;
        }
        let view = RECT {
            left: 12,
            top: 72,
            right: bounds.right - 12,
            bottom: bounds.bottom - 84,
        };
        let width = (view.right - view.left).max(1);
        let height = (view.bottom - view.top).max(1);
        let page = &state.pages[state.page.get()];
        let zoom = state.zoom.get();
        let ratio = if zoom == 0 {
            (f64::from(width) / f64::from(page.width))
                .min(f64::from(height) / f64::from(page.height))
        } else {
            f64::from(zoom) / 100.0
        };
        let w = (f64::from(page.width) * ratio).round().max(1.0) as i32;
        let h = (f64::from(page.height) * ratio).round().max(1.0) as i32;
        state.x.set(state.x.get().clamp(0, (w - width).max(0)));
        state.y.set(state.y.get().clamp(0, (h - height).max(0)));
        let x = view.left + (width - w).max(0) / 2 - state.x.get();
        let y = view.top + (height - h).max(0) / 2 - state.y.get();
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: page.width as i32,
                biHeight: -(page.height as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                ..unsafe { std::mem::zeroed() }
            },
            ..unsafe { std::mem::zeroed() }
        };
        let brush = unsafe { CreateSolidBrush(0x00ffffff) };
        let saved = unsafe { SaveDC(dc) };
        if brush.is_null() || saved == 0 {
            state.failed.set(true);
        } else {
            unsafe {
                IntersectClipRect(dc, view.left, view.top, view.right, view.bottom);
                FillRect(dc, &view, brush);
                SetStretchBltMode(dc, HALFTONE);
            }
            let result = unsafe {
                StretchDIBits(
                    dc,
                    x,
                    y,
                    w,
                    h,
                    0,
                    0,
                    page.width as i32,
                    page.height as i32,
                    page.bgra.as_ptr().cast(),
                    &info,
                    DIB_RGB_COLORS,
                    SRCCOPY,
                )
            };
            if result <= 0 {
                state.failed.set(true);
            } else {
                state.visited.borrow_mut()[state.page.get()] = true;
            }
            unsafe { RestoreDC(dc, saved) };
        }
        if !brush.is_null() {
            unsafe { DeleteObject(brush) };
        }
        unsafe {
            EndPaint(dialog, &ps);
            update(dialog, state);
        }
    }
    unsafe extern "system" fn procedure(
        dialog: HWND,
        message: u32,
        wparam: usize,
        lparam: isize,
    ) -> isize {
        std::panic::catch_unwind(||unsafe{
            if message==WM_INITDIALOG{SetWindowLongPtrW(dialog,SLOT,lparam);}
            let ptr=GetWindowLongPtrW(dialog,SLOT) as *const State<'_>;
            if ptr.is_null(){return 0;}
            let state=&*ptr;
            if matches!(message,WM_INITDIALOG|WM_COMMAND|WM_TIMER)&&state.cancellation.is_cancelled(){state.done.set(false);EndDialog(dialog,0);return 1;}
            match message{
                WM_INITDIALOG=>{
                    let mut r:RECT=std::mem::zeroed();
                    if GetClientRect(dialog,&mut r)==0{state.failed.set(true);EndDialog(dialog,0);return 1;}
                    let controls=[
                        control(dialog,"STATIC","All processed pages. Viewing is not approval to send, spend or access files.",98,(12,8,r.right-24,28),0),
                        control(dialog,"STATIC","",LABEL,(12,38,r.right-24,28),0),
                        control(dialog,"BUTTON","Previous",PREVIOUS,(12,r.bottom-72,80,26),WS_TABSTOP),
                        control(dialog,"BUTTON","Next",NEXT,(96,r.bottom-72,60,26),WS_TABSTOP),
                        control(dialog,"BUTTON","Fit",FIT,(160,r.bottom-72,50,26),WS_TABSTOP),
                        control(dialog,"BUTTON","100%",ACTUAL,(214,r.bottom-72,60,26),WS_TABSTOP),
                        control(dialog,"BUTTON","Zoom in",IN,(278,r.bottom-72,78,26),WS_TABSTOP),
                        control(dialog,"BUTTON","Zoom out",OUT,(360,r.bottom-72,80,26),WS_TABSTOP),
                        control(dialog,"BUTTON","Left",LEFT,(12,r.bottom-38,60,26),WS_TABSTOP),
                        control(dialog,"BUTTON","Right",RIGHT,(76,r.bottom-38,60,26),WS_TABSTOP),
                        control(dialog,"BUTTON","Up",UP,(140,r.bottom-38,60,26),WS_TABSTOP),
                        control(dialog,"BUTTON","Down",DOWN,(204,r.bottom-38,60,26),WS_TABSTOP),
                        control(dialog,"BUTTON","Cancel",IDCANCEL as usize,(r.right-176,r.bottom-38,76,26),WS_TABSTOP|BS_DEFPUSHBUTTON as u32),
                        control(dialog,"BUTTON","Done",DONE,(r.right-96,r.bottom-38,84,26),WS_TABSTOP),
                    ];
                    if controls.iter().any(|h|h.is_null())||SetTimer(dialog,1,200,None)==0{
                        state.failed.set(true);EndDialog(dialog,0);return 1;
                    }
                    update(dialog,state);SetFocus(GetDlgItem(dialog,IDCANCEL));0
                },
                WM_PAINT=>{paint(dialog,state);if state.failed.get(){EndDialog(dialog,0);}1},
                WM_COMMAND=>{
                    let id=wparam&0xffff;
                    if (wparam>>16)&0xffff!=BN_CLICKED as usize||lparam as HWND!=GetDlgItem(dialog,id as i32){return 0;}
                    if Instant::now()>=state.deadline{EndDialog(dialog,0);return 1;}
                    match id{
                        id if id==IDCANCEL as usize=>{EndDialog(dialog,0);return 1;},
                        DONE=>{if !state.failed.get()&&(!state.require_all||state.visited.borrow().iter().all(|v|*v)){state.done.set(true);EndDialog(dialog,1);}return 1;},
                        PREVIOUS if state.page.get()>0=>{state.page.set(state.page.get()-1);state.x.set(0);state.y.set(0);},
                        NEXT if state.page.get()+1<state.pages.len()=>{state.page.set(state.page.get()+1);state.x.set(0);state.y.set(0);},
                        FIT=>{state.zoom.set(0);state.x.set(0);state.y.set(0);},
                        ACTUAL=>{state.zoom.set(100);},
                        IN=>{state.zoom.set(if state.zoom.get()==0{100}else{(state.zoom.get()+25).min(300)});},
                        OUT=>{state.zoom.set(if state.zoom.get()==0{75}else{state.zoom.get().saturating_sub(25).max(25)});},
                        LEFT=>state.x.set(state.x.get().saturating_sub(120)),
                        RIGHT=>state.x.set(state.x.get().saturating_add(120)),
                        UP=>state.y.set(state.y.get().saturating_sub(120)),
                        DOWN=>state.y.set(state.y.get().saturating_add(120)),
                        _=>return 0,
                    }
                    InvalidateRect(dialog,ptr::null(),1);1
                },
                WM_TIMER if Instant::now()>=state.deadline=>{EndDialog(dialog,0);1},
                WM_CLOSE=>{EndDialog(dialog,0);1},
                _=>0,
            }
        }).unwrap_or_else(|_|{unsafe{EndDialog(dialog,0);}1})
    }
    pub(super) fn show(
        pages: &[VisualPage],
        owner: usize,
        deadline: Instant,
        require_all: bool,
        cancellation: &crate::native_review::ReviewCancellation,
    ) -> Result<bool, &'static str> {
        let state = State {
            pages,
            page: Cell::new(0),
            zoom: Cell::new(0),
            x: Cell::new(0),
            y: Cell::new(0),
            visited: RefCell::new(vec![false; pages.len()]),
            deadline,
            cancellation,
            require_all,
            done: Cell::new(false),
            failed: Cell::new(false),
        };
        let t = template();
        let result = unsafe {
            DialogBoxIndirectParamW(
                GetModuleHandleW(ptr::null()),
                t.as_ptr().cast(),
                owner as HWND,
                Some(procedure),
                (&state as *const State<'_>) as isize,
            )
        };
        if result == -1 || state.failed.get() {
            return Err("visual_preview_unavailable");
        }
        Ok(result == 1
            && state.done.get()
            && Instant::now() < deadline
            && !state.cancellation.is_cancelled())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancelled_visual_inspection_denies_before_any_os_display() {
        let (sender, receiver) = tokio::sync::watch::channel(false);
        let signal = crate::native_review::ReviewCancellation::for_request(receiver);
        let page = VisualPage::from_rgba("Synthetic".into(), 1, 1, &[0, 0, 0, 255]).unwrap();
        sender.send(true).unwrap();
        assert_eq!(
            inspect_owned_cancellable(
                &[page],
                0,
                Instant::now() + std::time::Duration::from_secs(1),
                true,
                &signal,
            ),
            Ok(false)
        );
    }

    #[test]
    fn exact_pixel_bounds_and_white_alpha_preview_without_original_bytes() {
        let page = VisualPage::from_rgba("Synthetic page".into(), 1, 1, &[255, 0, 0, 128]).unwrap();
        assert_eq!(page.bgra, vec![127, 127, 255, 255]);
        assert!(VisualPage::from_rgba("Synthetic".into(), 1, 1, &[0; 3]).is_err());
        assert!(VisualPage::from_rgba("Synthetic".into(), 4097, 1, &[]).is_err());
        assert!(VisualPage::from_rgba("bad\nlabel".into(), 1, 1, &[0; 4]).is_err());
    }
    #[test]
    fn missing_owner_denies_before_any_native_display() {
        let page = VisualPage::from_rgba("Synthetic".into(), 1, 1, &[0, 0, 0, 255]).unwrap();
        assert!(inspect_owned(
            &[page],
            0,
            Instant::now() + std::time::Duration::from_secs(1),
            true
        )
        .is_err());
    }
}
