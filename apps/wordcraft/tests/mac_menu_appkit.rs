//! The macOS menu bar as AppKit sees it, on the main thread (`harness = false`): install it through
//! the real muda backend (`src/native_menu.rs`) and read NSApp back. The Window menu must be NSApp's
//! `windowsMenu`: that is where macOS adds Move & Resize, Fill and Center, the targets of the system
//! window tiling shortcuts (Fn+Ctrl+arrows, Fn+F), and lists the open windows.

#[cfg(target_os = "macos")]
#[path = "../src/native_menu.rs"]
#[allow(unused_imports, dead_code)]
mod native_menu;

#[cfg(target_os = "macos")]
fn main() {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSApplication;

    // nextest lists the tests first (`--list --format terse`, then with `--ignored`).
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--list") {
        if !args.iter().any(|a| a == "--ignored") {
            println!("menu_bar_in_appkit: test");
        }
        return;
    }
    let mtm = MainThreadMarker::new().expect("harness = false runs on the main thread");
    let nsapp = NSApplication::sharedApplication(mtm);
    nsapp.finishLaunching();

    let ctx = egui::Context::default();
    let mut app =
        wordcraft_ui_egui::WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), wordcraft_ui_egui::Services::default());
    let mut menu = None;
    native_menu::NativeMenu::frame(&mut menu, &mut app, &ctx);

    let bar = nsapp.mainMenu().expect("a main menu");
    let tops: Vec<String> = (0..bar.numberOfItems()).filter_map(|i| bar.itemAtIndex(i)?.submenu()).map(|m| m.title().to_string()).collect();
    assert!(tops.starts_with(&["WordCraft".into(), "File".into(), "Edit".into(), "View".into(), "Go".into(), "Insert".into()]));
    assert_eq!(nsapp.windowsMenu().map(|m| m.title().to_string()).as_deref(), Some("Window"), "AppKit knows the Window menu");

    // Word for Mac's shortcuts, as AppKit shows them.
    use objc2_app_kit::{NSEvent, NSEventModifierFlags, NSEventType, NSMenu};
    use objc2_foundation::{NSPoint, NSString};
    let item = |menu: &str, title: &str| {
        let m: objc2::rc::Retained<NSMenu> = bar.itemWithTitle(&NSString::from_str(menu)).and_then(|i| i.submenu()).expect(menu);
        m.itemWithTitle(&NSString::from_str(title)).unwrap_or_else(|| panic!("{menu} › {title}"))
    };
    let undo = item("Edit", "Undo");
    assert_eq!(undo.keyEquivalent().to_string(), "z");
    assert_eq!(undo.keyEquivalentModifierMask(), NSEventModifierFlags::Command);
    assert_eq!(item("WordCraft", "Hide WordCraft").keyEquivalent().to_string(), "h");
    assert_eq!(item("Edit", "Replace…").keyEquivalentModifierMask(), NSEventModifierFlags::Control);
    assert_eq!(item("Format", "Paragraph").keyEquivalentModifierMask(), NSEventModifierFlags::Command | NSEventModifierFlags::Option);

    assert!(!item("Table", "Delete Rows").isEnabled(), "no table");
    let key = |chars: &str, code: u16, flags: NSEventModifierFlags| {
        NSEvent::keyEventWithType_location_modifierFlags_timestamp_windowNumber_context_characters_charactersIgnoringModifiers_isARepeat_keyCode(
            NSEventType::KeyDown,
            NSPoint::new(0.0, 0.0),
            flags,
            0.0,
            0,
            None,
            &NSString::from_str(chars),
            &NSString::from_str(chars),
            false,
            code,
        )
        .expect("a key event")
    };
    // A key no item has goes on to the window.
    assert!(!bar.performKeyEquivalent(&key("j", 38, NSEventModifierFlags::Command | NSEventModifierFlags::Control)));
    // ⌘D is Format › Font: the menu takes it, and the frame opens the dialog.
    assert!(app.dialog.is_none());
    assert!(bar.performKeyEquivalent(&key("d", 2, NSEventModifierFlags::Command)));
    native_menu::NativeMenu::frame(&mut menu, &mut app, &ctx);
    assert!(app.dialog.is_some(), "⌘D opened the Font dialog");

    println!("mac_menu_appkit: ok");
}

#[cfg(not(target_os = "macos"))]
fn main() {}
