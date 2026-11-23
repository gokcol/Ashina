use eframe::egui;
use std::path::Path;

#[derive(Default)]
pub(super) struct State {
    #[cfg(target_os = "macos")]
    picker: Option<objc2::rc::Retained<objc2_app_kit::NSSharingServicePicker>>,
    #[cfg(target_os = "windows")]
    registration: Option<windows_share::Registration>,
}

#[cfg(target_os = "macos")]
impl State {
    pub(super) fn show(
        &mut self,
        path: &Path,
        anchor: egui::Rect,
        zoom: f32,
        frame: &eframe::Frame,
    ) -> anyhow::Result<()> {
        use anyhow::Context;
        use objc2::{AnyThread, MainThreadMarker, runtime::AnyObject};
        use objc2_app_kit::{NSSharingServicePicker, NSView};
        use objc2_foundation::{NSArray, NSPoint, NSRect, NSRectEdge, NSSize, NSURL};
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};

        MainThreadMarker::new().context("Sharing requires the main window thread")?;
        let handle = frame.window_handle()?;
        let RawWindowHandle::AppKit(window) = handle.as_raw() else {
            anyhow::bail!("The document window is unavailable");
        };
        // The borrowed AppKit handle keeps the window's NSView alive for this call.
        let view = unsafe { window.ns_view.cast::<NSView>().as_ref() };
        let url = NSURL::from_path(path, false, None).context("Invalid PDF path")?;
        let item: &AnyObject = &url;
        let items = NSArray::from_slice(&[item]);
        // File URLs conform to NSPasteboardWriting.
        let picker = unsafe {
            NSSharingServicePicker::initWithItems(NSSharingServicePicker::alloc(), &items)
        };
        let zoom = f64::from(zoom);
        let size = NSSize::new(
            f64::from(anchor.width()) * zoom,
            f64::from(anchor.height()) * zoom,
        );
        let bounds = view.bounds();
        let (y, edge) = if view.isFlipped() {
            (
                bounds.origin.y + f64::from(anchor.top()) * zoom,
                NSRectEdge::MaxY,
            )
        } else {
            (
                bounds.origin.y + bounds.size.height - f64::from(anchor.bottom()) * zoom,
                NSRectEdge::MinY,
            )
        };
        let origin = NSPoint::new(bounds.origin.x + f64::from(anchor.left()) * zoom, y);
        if let Some(previous) = self.picker.take() {
            previous.close();
        }
        picker.showRelativeToRect_ofView_preferredEdge(NSRect::new(origin, size), view, edge);
        self.picker = Some(picker);
        Ok(())
    }
}

#[cfg(target_os = "windows")]
impl State {
    pub(super) fn show(
        &mut self,
        path: &Path,
        _anchor: egui::Rect,
        _zoom: f32,
        frame: &eframe::Frame,
    ) -> anyhow::Result<()> {
        self.registration = None;
        self.registration = Some(windows_share::show(path, frame)?);
        Ok(())
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
impl State {
    pub(super) fn show(
        &mut self,
        _path: &Path,
        _anchor: egui::Rect,
        _zoom: f32,
        _frame: &eframe::Frame,
    ) -> anyhow::Result<()> {
        anyhow::bail!("System sharing is unavailable on this platform")
    }
}

#[cfg(target_os = "windows")]
mod windows_share {
    use super::*;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows::{
        ApplicationModel::DataTransfer::{
            DataRequest, DataRequestedEventArgs, DataTransferManager,
        },
        Foundation::TypedEventHandler,
        Storage::{IStorageItem, StorageFile},
        Win32::{
            Foundation::HWND,
            System::WinRT::{
                RO_INIT_MULTITHREADED, RO_INIT_SINGLETHREADED, RO_INIT_TYPE, RoInitialize,
                RoUninitialize,
            },
            UI::Shell::IDataTransferManagerInterop,
        },
        core::{HSTRING, Interface, factory},
    };
    use windows_collections::IIterable;

    struct Apartment;

    impl Apartment {
        fn new(mode: RO_INIT_TYPE) -> windows::core::Result<Self> {
            // Callers use the existing UI STA or a newly spawned worker thread.
            unsafe { RoInitialize(mode)? };
            Ok(Self)
        }
    }

    impl Drop for Apartment {
        fn drop(&mut self) {
            // Balances initialization on this thread.
            unsafe { RoUninitialize() };
        }
    }

    pub(super) struct Registration {
        manager: DataTransferManager,
        token: i64,
        _apartment: Apartment,
    }

    impl Drop for Registration {
        fn drop(&mut self) {
            let _ = self.manager.RemoveDataRequested(self.token);
        }
    }

    fn normal_path(path: &Path) -> std::path::PathBuf {
        use std::path::{Component, Prefix};
        let mut components = path.components();
        let Some(Component::Prefix(prefix)) = components.next() else {
            return path.to_owned();
        };
        // Rust canonicalizes Windows files with a verbatim prefix. WinRT
        // expects a drive or UNC path in standard Windows syntax.
        let mut result: std::path::PathBuf = match prefix.kind() {
            Prefix::VerbatimDisk(letter) => format!("{}:\\", char::from(letter)).into(),
            Prefix::VerbatimUNC(server, share) => {
                let mut root = std::ffi::OsString::from(r"\\");
                // Keep the separator between the server and share and preserve UTF-16.
                root.push(server);
                root.push("\\");
                root.push(share);
                root.into()
            }
            _ => return path.to_owned(),
        };
        for component in components.filter(|c| *c != Component::RootDir) {
            result.push(component);
        }
        result
    }

    fn attach_pdf(
        request: &DataRequest,
        path: &HSTRING,
        name: &HSTRING,
    ) -> windows::core::Result<()> {
        let file = StorageFile::GetFileFromPathAsync(path)?.get()?;
        let items = IIterable::<IStorageItem>::from(vec![Some(file.cast()?)]);
        let data = request.Data()?;
        data.Properties()?.SetTitle(name)?;
        data.SetStorageItems(&items, true)
    }

    pub(super) fn show(path: &Path, frame: &eframe::Frame) -> anyhow::Result<Registration> {
        let handle = frame
            .window_handle()
            .map_err(|_| anyhow::anyhow!("The document window is unavailable"))?;
        let RawWindowHandle::Win32(window) = handle.as_raw() else {
            anyhow::bail!("The document window is unavailable");
        };
        let hwnd = HWND(window.hwnd.get() as *mut _);
        let apartment = Apartment::new(RO_INIT_SINGLETHREADED)?;
        let interop: IDataTransferManagerInterop = factory::<DataTransferManager, _>()?;
        // HWND is borrowed from eframe's active window on the UI thread.
        let manager: DataTransferManager = unsafe { interop.GetForWindow(hwnd)? };
        let name = HSTRING::from(
            path.file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("PDF")),
        );
        let path = HSTRING::from(normal_path(path).as_os_str());
        let handler = TypedEventHandler::<DataTransferManager, DataRequestedEventArgs>::new(
            move |_, args| {
                let request = args.ok()?.Request()?;
                let deferral = request.GetDeferral()?;
                let (file, title) = (path.clone(), name.clone());
                let (pending, complete) = (request.clone(), deferral.clone());
                let worker = std::thread::Builder::new()
                    .name("ashina-share".into())
                    .spawn(move || {
                        let apartment = Apartment::new(RO_INIT_MULTITHREADED);
                        let result = apartment
                            .as_ref()
                            .map_err(Clone::clone)
                            .and_then(|_| attach_pdf(&pending, &file, &title));
                        if result.is_err() {
                            let _ = pending.FailWithDisplayText(&HSTRING::from(
                                "Could not attach this PDF. Try Copy PDF.",
                            ));
                        }
                        let _ = complete.Complete();
                    });
                if worker.is_err() {
                    let _ = request.FailWithDisplayText(&HSTRING::from(
                        "Could not start sharing. Try Copy PDF.",
                    ));
                    deferral.Complete()?;
                }
                Ok(())
            },
        );
        let token = manager.DataRequested(&handler)?;
        let registration = Registration {
            manager,
            token,
            _apartment: apartment,
        };
        // Registration stays alive while the native sheet requests the document.
        unsafe { interop.ShowShareUIForWindow(hwnd)? };
        Ok(registration)
    }
}
