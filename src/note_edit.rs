//! The Notes body's native, plain-text RichEdit control. Windows owns input,
//! selection, IME and undo; the panel supplies the surrounding card.

use std::sync::OnceLock;

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::DirectWrite::IDWriteTextFormat;
use windows::Win32::System::LibraryLoader::{LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32};
use windows::Win32::UI::Controls::RichEdit::*;
use windows::Win32::UI::Controls::{SetWindowTheme, EM_EMPTYUNDOBUFFER, EM_SETMARGINS, EM_SETRECT};
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::gfx::Color;

pub const BACKGROUND: Color = Color::rgb(12, 12, 12);
pub const FOREGROUND: Color = Color::rgb(224, 224, 224);

pub fn create(host: HWND, font: &IDWriteTextFormat) -> Result<HWND> {
    // Keep the system DLL loaded for the process lifetime: its window procedures
    // must remain valid for every editor window, including during teardown.
    static LIBRARY: OnceLock<Result<()>> = OnceLock::new();
    LIBRARY
        .get_or_init(|| unsafe {
            LoadLibraryExW(w!("Msftedit.dll"), None, LOAD_LIBRARY_SEARCH_SYSTEM32).map(|_| ())
        })
        .as_ref()
        .map_err(Clone::clone)?;

    let edit = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            MSFTEDIT_CLASS,
            w!(""),
            WS_CHILD
                | WS_VISIBLE
                | WS_VSCROLL
                | WINDOW_STYLE(
                    (ES_MULTILINE | ES_AUTOVSCROLL | ES_WANTRETURN | ES_NOHIDESEL) as u32,
                ),
            0,
            0,
            10,
            10,
            Some(host),
            None,
            None,
            None,
        )?
    };
    if let Err(err) = configure(edit, font) {
        unsafe {
            let _ = DestroyWindow(edit);
        }
        return Err(err);
    }
    Ok(edit)
}

fn configure(edit: HWND, font: &IDWriteTextFormat) -> Result<()> {
    unsafe {
        // Set this before loading any text. Rich formatting, images and OLE
        // objects from the clipboard must not enter the note body.
        let mode = TM_PLAINTEXT.0 | TM_MULTILEVELUNDO.0 | TM_MULTICODEPAGE.0;
        if SendMessageW(edit, EM_SETTEXTMODE, Some(WPARAM(mode as usize)), None).0 != 0 {
            return Err(Error::new(E_FAIL, "Could not enable plain-text notes"));
        }
        let _ = SendMessageW(
            edit,
            EM_SETEVENTMASK,
            None,
            Some(LPARAM(ENM_CHANGE as isize)),
        );
        let _ = SendMessageW(
            edit,
            EM_EXLIMITTEXT,
            None,
            Some(LPARAM((2 * crate::store::NOTE_BODY_MAX) as isize)),
        );
        let _ = SendMessageW(
            edit,
            EM_SETBKGNDCOLOR,
            None,
            Some(LPARAM(colorref(BACKGROUND))),
        );
        // RichEdit hides the vertical scrollbar when the document fits. Leaving
        // ES_DISABLENOSCROLL unset avoids an always-visible disabled scrollbar.
        let _ = SetWindowTheme(edit, w!("DarkMode_Explorer"), w!(""));

        let mut face = vec![0u16; font.GetFontFamilyNameLength() as usize + 1];
        font.GetFontFamilyName(&mut face)?;
        let mut format = CHARFORMATW {
            cbSize: size_of::<CHARFORMATW>() as u32,
            dwMask: CFM_FACE | CFM_SIZE | CFM_COLOR,
            yHeight: (font.GetFontSize() * 15.0).round() as i32, // DIPs to twips
            crTextColor: COLORREF(colorref(FOREGROUND) as u32),
            ..Default::default()
        };
        let len = (face.len() - 1).min(format.szFaceName.len() - 1);
        format.szFaceName[..len].copy_from_slice(&face[..len]);
        let _ = SendMessageW(
            edit,
            EM_SETCHARFORMAT,
            Some(WPARAM(SCF_ALL as usize)),
            Some(LPARAM(&format as *const _ as isize)),
        );
    }
    Ok(())
}

pub fn layout(edit: HWND) {
    unsafe {
        let mut rect = RECT::default();
        if GetClientRect(edit, &mut rect).is_ok() {
            // The panel already supplies the card's padding. Use the actual
            // client width so overflowing text stays clear of the scrollbar.
            let _ = SendMessageW(edit, EM_SETMARGINS, Some(WPARAM(3)), Some(LPARAM(0)));
            let _ = SendMessageW(
                edit,
                EM_SETRECT,
                None,
                Some(LPARAM(&rect as *const _ as isize)),
            );
        }
    }
}

pub fn set_text(edit: HWND, text: &str) {
    let text = normalize_newlines(text).replace('\n', "\r\n");
    unsafe {
        let _ = SetWindowTextW(edit, &HSTRING::from(text));
        // Switching notes starts a new undo history.
        let _ = SendMessageW(edit, EM_EMPTYUNDOBUFFER, None, None);
    }
}

pub fn text(edit: HWND) -> String {
    unsafe {
        // RichEdit stores paragraph breaks as CR. Read UTF-16 explicitly, then
        // use LF in the model/file just as the DirectWrite preview expects.
        let length = GETTEXTLENGTHEX {
            flags: GTL_NUMCHARS | GTL_PRECISE,
            codepage: 1200,
        };
        let len = SendMessageW(
            edit,
            EM_GETTEXTLENGTHEX,
            Some(WPARAM(&length as *const _ as usize)),
            None,
        )
        .0
        .max(0) as usize;
        let mut buf = vec![0u16; len + 1];
        let get = GETTEXTEX {
            cb: (buf.len() * size_of::<u16>()) as u32,
            flags: GT_DEFAULT,
            codepage: 1200,
            ..Default::default()
        };
        let got = SendMessageW(
            edit,
            EM_GETTEXTEX,
            Some(WPARAM(&get as *const _ as usize)),
            Some(LPARAM(buf.as_mut_ptr() as isize)),
        )
        .0
        .max(0) as usize;
        normalize_newlines(&String::from_utf16_lossy(&buf[..got.min(len)]))
    }
}

fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

fn colorref(color: Color) -> isize {
    let r = (color.0 * 255.0).round() as u32;
    let g = (color.1 * 255.0).round() as u32;
    let b = (color.2 * 255.0).round() as u32;
    (r | (g << 8) | (b << 16)) as isize
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use windows::Win32::Graphics::DirectWrite::*;
    use windows::Win32::UI::Controls::{
        EM_CANUNDO, EM_GETFIRSTVISIBLELINE, EM_GETLINECOUNT, EM_LINESCROLL, EM_REPLACESEL,
        EM_SETSEL, EM_UNDO,
    };
    use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};

    struct Editor {
        host: HWND,
        edit: HWND,
        changes: Box<AtomicUsize>,
    }

    impl Editor {
        fn new() -> Self {
            unsafe {
                let host = CreateWindowExW(
                    WS_EX_TOOLWINDOW,
                    w!("STATIC"),
                    w!(""),
                    WS_POPUP,
                    0,
                    0,
                    300,
                    150,
                    None,
                    None,
                    None,
                    None,
                )
                .unwrap();
                let changes = Box::new(AtomicUsize::new(0));
                assert!(SetWindowSubclass(
                    host,
                    Some(count_changes),
                    1,
                    changes.as_ref() as *const _ as usize,
                )
                .as_bool());
                let dw: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).unwrap();
                let font = dw
                    .CreateTextFormat(
                        w!("Segoe UI"),
                        None,
                        DWRITE_FONT_WEIGHT_NORMAL,
                        DWRITE_FONT_STYLE_NORMAL,
                        DWRITE_FONT_STRETCH_NORMAL,
                        14.0,
                        w!("en-us"),
                    )
                    .unwrap();
                let edit = create(host, &font).unwrap();
                SetWindowPos(edit, None, 0, 0, 300, 150, SWP_NOACTIVATE | SWP_NOZORDER).unwrap();
                layout(edit);
                Self {
                    host,
                    edit,
                    changes,
                }
            }
        }

        fn append(&self, value: &str) {
            let value = HSTRING::from(value);
            unsafe {
                let _ = SendMessageW(
                    self.edit,
                    EM_SETSEL,
                    Some(WPARAM(usize::MAX)),
                    Some(LPARAM(-1)),
                );
                let _ = SendMessageW(
                    self.edit,
                    EM_REPLACESEL,
                    Some(WPARAM(1)),
                    Some(LPARAM(value.as_ptr() as isize)),
                );
            }
        }
    }

    impl Drop for Editor {
        fn drop(&mut self) {
            unsafe {
                let _ = DestroyWindow(self.host);
            }
        }
    }

    unsafe extern "system" fn count_changes(
        hwnd: HWND,
        msg: u32,
        wp: WPARAM,
        lp: LPARAM,
        _id: usize,
        data: usize,
    ) -> LRESULT {
        if msg == WM_COMMAND && (wp.0 >> 16) as u16 == EN_CHANGE as u16 {
            (*(data as *const AtomicUsize)).fetch_add(1, Ordering::Relaxed);
        }
        DefSubclassProc(hwnd, msg, wp, lp)
    }

    #[test]
    fn native_editor_round_trips_unicode_newlines_and_saved_notes() {
        let editor = Editor::new();
        let long = format!("{}😀", "中".repeat(crate::store::NOTE_BODY_MAX - 1));
        for input in [
            "",
            "First line\n第二行 👩‍💻\n\nLast line\n",
            "Windows\r\nUnix\nRichEdit\rEnd",
            "{\\rtf1 literal plain text}",
            &long,
        ] {
            set_text(editor.edit, input);
            let body = text(editor.edit);
            assert_eq!(body, normalize_newlines(input));
            let note = crate::store::Note {
                id: 1,
                updated_at: 1,
                title: "Round trip".into(),
                body,
            };
            let serialized = crate::store::notes_to_text(std::slice::from_ref(&note));
            let loaded = crate::store::notes_from_text(&serialized);
            assert_eq!(loaded, [note]);
            set_text(editor.edit, &loaded[0].body);
            assert_eq!(text(editor.edit), loaded[0].body);
        }
    }

    #[test]
    fn native_edits_notify_and_undo_does_not_cross_notes() {
        let editor = Editor::new();
        unsafe {
            let mode = SendMessageW(editor.edit, EM_GETTEXTMODE, None, None).0 as i32;
            assert_ne!(mode & TM_PLAINTEXT.0, 0);
            assert_ne!(mode & TM_MULTILEVELUNDO.0, 0);
            set_text(editor.edit, "First note");
            editor.changes.store(0, Ordering::Relaxed);
            editor.append(" 中文 😀");
            assert_eq!(text(editor.edit), "First note 中文 😀");
            assert!(editor.changes.load(Ordering::Relaxed) > 0);
            assert_ne!(SendMessageW(editor.edit, EM_CANUNDO, None, None).0, 0);
            let _ = SendMessageW(editor.edit, EM_UNDO, None, None);
            assert_eq!(text(editor.edit), "First note");
            let _ = SendMessageW(editor.edit, EM_REDO, None, None);
            assert_eq!(text(editor.edit), "First note 中文 😀");
            set_text(editor.edit, "Second note");
            assert_eq!(SendMessageW(editor.edit, EM_CANUNDO, None, None).0, 0);
            assert_eq!(text(editor.edit), "Second note");
        }
    }

    #[test]
    fn native_layout_wraps_and_preserves_scroll_position() {
        let editor = Editor::new();
        unsafe {
            set_text(
                editor.edit,
                &"A long note should wrap inside the card. ".repeat(20),
            );
            assert!(SendMessageW(editor.edit, EM_GETLINECOUNT, None, None).0 > 1);
            set_text(editor.edit, &"One line\n".repeat(60));
            let _ = SendMessageW(editor.edit, EM_LINESCROLL, None, Some(LPARAM(20)));
            let first = SendMessageW(editor.edit, EM_GETFIRSTVISIBLELINE, None, None).0;
            assert!(first > 0);
            layout(editor.edit);
            assert_eq!(
                SendMessageW(editor.edit, EM_GETFIRSTVISIBLELINE, None, None).0,
                first
            );
            set_text(editor.edit, "Short note");
            assert_eq!(
                SendMessageW(editor.edit, EM_GETFIRSTVISIBLELINE, None, None).0,
                0
            );
        }
    }

    #[test]
    #[ignore]
    fn render_native_editor() {
        use windows::Win32::Graphics::Gdi::*;

        let editor = Editor::new();
        unsafe {
            use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        }
        let gfx = crate::gfx::Gfx::new(300, 150, 1.0).unwrap();
        configure(editor.edit, &gfx.fonts.note).unwrap();
        let dir = std::env::var("PREVIEW_DIR").expect("PREVIEW_DIR");
        for (name, body) in [
            ("short", "Weekend ideas\nTake a walk by the lake.\nBring a notebook and sketch the view.\n随手记下沿途的想法。".to_owned()),
            ("overflow", (1..=30).map(|i| format!("Line {i} · 随手记 😀\n")).collect::<String>()),
        ] {
            set_text(editor.edit, &body);
            layout(editor.edit);
            unsafe {
                let dc = CreateCompatibleDC(None);
                let bmi = BITMAPINFO {
                    bmiHeader: BITMAPINFOHEADER {
                        biSize: size_of::<BITMAPINFOHEADER>() as u32,
                        biWidth: 300,
                        biHeight: -150,
                        biPlanes: 1,
                        biBitCount: 32,
                        biCompression: BI_RGB.0,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let mut bits = std::ptr::null_mut();
                let bitmap = CreateDIBSection(Some(dc), &bmi, DIB_RGB_COLORS, &mut bits, None, 0).unwrap();
                let previous = SelectObject(dc, bitmap.into());
                let _ = SendMessageW(
                    editor.edit,
                    WM_PRINT,
                    Some(WPARAM(dc.0 as usize)),
                    Some(LPARAM((PRF_CLIENT | PRF_NONCLIENT | PRF_ERASEBKGND) as isize)),
                );
                let pixels = std::slice::from_raw_parts(bits as *const u8, 300 * 150 * 4);
                let mut out = Vec::new();
                out.extend_from_slice(&300u32.to_le_bytes());
                out.extend_from_slice(&150u32.to_le_bytes());
                out.extend_from_slice(pixels);
                std::fs::write(format!("{dir}/native_{name}.raw"), out).unwrap();
                SelectObject(dc, previous);
                let _ = DeleteObject(bitmap.into());
                let _ = DeleteDC(dc);
            }
        }
    }
}
