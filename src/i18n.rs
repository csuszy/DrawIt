#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Language {
    Hu,
    En,
}

impl Default for Language {
    fn default() -> Self {
        Self::detect_system()
    }
}

#[allow(dead_code)]
impl Language {
    /// Detects Windows system UI language:
    /// Hungarian (0x000E) -> Language::Hu, anything else -> Language::En
    pub fn detect_system() -> Self {
        #[link(name = "kernel32")]
        extern "system" {
            fn GetUserDefaultUILanguage() -> u16;
        }
        let lang_id = unsafe { GetUserDefaultUILanguage() };
        let primary_lang = lang_id & 0x03FF;
        if primary_lang == 0x000E {
            Language::Hu
        } else {
            Language::En
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "en" | "english" => Language::En,
            "hu" | "hungarian" => Language::Hu,
            "system" | "auto" => Language::detect_system(),
            _ => Language::detect_system(),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Hu => "hu",
            Language::En => "en",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Hu => "Magyar",
            Language::En => "English",
        }
    }

    // --- Toolbar Main & Popover Strings ---
    pub fn canvas_btn(&self) -> &'static str {
        match self {
            Language::Hu => "Kijelző ▾",
            Language::En => "Canvas ▾",
        }
    }

    pub fn snip_btn(&self) -> &'static str {
        match self {
            Language::Hu => "Kivágás",
            Language::En => "Snip",
        }
    }

    pub fn export_display_btn(&self) -> &'static str {
        match self {
            Language::Hu => "Kijelző mentése ▾",
            Language::En => "Save Display ▾",
        }
    }

    pub fn menu_help(&self) -> &'static str {
        match self {
            Language::Hu => "Súgó és billentyűk [F1]",
            Language::En => "Help & Shortcuts [F1]",
        }
    }

    pub fn menu_settings(&self) -> &'static str {
        match self {
            Language::Hu => "Beállítások...",
            Language::En => "Settings...",
        }
    }

    pub fn menu_check_updates(&self) -> &'static str {
        #[cfg(feature = "store")]
        {
            match self {
                Language::Hu => "Frissítések keresése (Microsoft Store)...",
                Language::En => "Check for Updates (Microsoft Store)...",
            }
        }
        #[cfg(not(feature = "store"))]
        {
            match self {
                Language::Hu => "Frissítések keresése...",
                Language::En => "Check for Updates...",
            }
        }
    }

    pub fn menu_about_prefix(&self) -> &'static str {
        match self {
            Language::Hu => "Névjegy (DrawIt v{})",
            Language::En => "About (DrawIt v{})",
        }
    }

    pub fn menu_exit(&self) -> &'static str {
        match self {
            Language::Hu => "Kilépés a rajzolóból",
            Language::En => "Exit DrawIt",
        }
    }

    pub fn palette_title(&self) -> &'static str {
        match self {
            Language::Hu => "Színek és paletta",
            Language::En => "Colors & Palette",
        }
    }

    pub fn custom_color_title(&self) -> &'static str {
        match self {
            Language::Hu => "Egyedi színskála és pipetta",
            Language::En => "Custom Color & Picker",
        }
    }

    pub fn opacity_title(&self) -> &'static str {
        match self {
            Language::Hu => "Átlátszatlanság",
            Language::En => "Opacity",
        }
    }

    pub fn thickness_title(&self) -> &'static str {
        match self {
            Language::Hu => "Vonalvastagság",
            Language::En => "Stroke Thickness",
        }
    }

    pub fn pipette_btn(&self) -> &'static str {
        match self {
            Language::Hu => "Pipetta",
            Language::En => "Picker",
        }
    }

    pub fn canvas_mode_title(&self) -> &'static str {
        match self {
            Language::Hu => "Kijelző és rajzolás módja",
            Language::En => "Canvas & Drawing Mode",
        }
    }

    pub fn export_title(&self) -> &'static str {
        match self {
            Language::Hu => "Kijelző képének mentése",
            Language::En => "Save Display Capture",
        }
    }

    pub fn export_all_monitors(&self) -> &'static str {
        match self {
            Language::Hu => "Teljes asztal (összes kijelző)",
            Language::En => "Full Desktop (All Displays)",
        }
    }

    pub fn export_copy(&self) -> &'static str {
        match self {
            Language::Hu => "Másolás",
            Language::En => "Copy",
        }
    }

    // --- Snip Floating Bar ---
    pub fn snip_copy(&self) -> &'static str {
        match self {
            Language::Hu => "Másolás",
            Language::En => "Copy",
        }
    }

    pub fn snip_copy_text(&self) -> &'static str {
        match self {
            Language::Hu => "Szöveg Másolás",
            Language::En => "Copy Text",
        }
    }

    pub fn snip_save_png(&self) -> &'static str {
        match self {
            Language::Hu => "Mentés PNG",
            Language::En => "Save PNG",
        }
    }

    pub fn snip_save_pdf(&self) -> &'static str {
        match self {
            Language::Hu => "Mentés PDF",
            Language::En => "Save PDF",
        }
    }

    pub fn snip_cancel(&self) -> &'static str {
        match self {
            Language::Hu => "Mégse",
            Language::En => "Cancel",
        }
    }

    // --- Toast Notifications ---
    pub fn toast_clear(&self) -> &'static str {
        match self {
            Language::Hu => "Rajzok törölve (Tiszta vászon)",
            Language::En => "Drawings cleared (Clean canvas)",
        }
    }

    pub fn toast_undo(&self) -> &'static str {
        match self {
            Language::Hu => "Visszavonás",
            Language::En => "Undo",
        }
    }

    pub fn toast_redo(&self) -> &'static str {
        match self {
            Language::Hu => "Újra",
            Language::En => "Redo",
        }
    }

    pub fn toast_display_copied(&self) -> &'static str {
        match self {
            Language::Hu => "Kijelző képe másolva a vágólapra!",
            Language::En => "Display copied to clipboard!",
        }
    }

    pub fn toast_display_saved_png(&self) -> &'static str {
        match self {
            Language::Hu => "Kijelző képe elmentve PNG fájlba!",
            Language::En => "Display saved as PNG file!",
        }
    }

    pub fn toast_display_saved_pdf(&self) -> &'static str {
        match self {
            Language::Hu => "Kijelző elmentve PDF fájlba!",
            Language::En => "Display saved as PDF file!",
        }
    }

    pub fn toast_snip_copied(&self) -> &'static str {
        match self {
            Language::Hu => "Kivágott terület másolva a vágólapra!",
            Language::En => "Snipped region copied to clipboard!",
        }
    }

    pub fn toast_ocr_copied(&self) -> &'static str {
        match self {
            Language::Hu => "Szöveg sikeresen másolva a vágólapra!",
            Language::En => "Text successfully copied to clipboard!",
        }
    }

    pub fn toast_ocr_too_small(&self) -> &'static str {
        match self {
            Language::Hu => "Túl kicsi a kép a felismeréshez!",
            Language::En => "Image is too small for recognition!",
        }
    }

    pub fn toast_ocr_no_text(&self) -> &'static str {
        match self {
            Language::Hu => "Sikertelen felismerés (nem található szöveg)!",
            Language::En => "Recognition failed (no text detected)!",
        }
    }

    pub fn toast_ocr_failed(&self) -> &'static str {
        match self {
            Language::Hu => "Sikertelen felismerés (OCR hiba)!",
            Language::En => "Recognition failed (OCR error)!",
        }
    }

    pub fn toast_snip_saved_png(&self) -> &'static str {
        match self {
            Language::Hu => "Kivágás elmentve PNG fájlba!",
            Language::En => "Snip saved as PNG file!",
        }
    }

    pub fn toast_snip_saved_pdf(&self) -> &'static str {
        match self {
            Language::Hu => "Kivágás elmentve PDF fájlba!",
            Language::En => "Snip saved as PDF file!",
        }
    }

    pub fn toast_counter_reset(&self) -> &'static str {
        match self {
            Language::Hu => "Számláló nullázva (#1)",
            Language::En => "Counter reset to #1",
        }
    }

    pub fn toast_settings_saved(&self) -> &'static str {
        match self {
            Language::Hu => "Beállítások elmentve",
            Language::En => "Settings saved",
        }
    }

    pub fn toast_snip_cancelled(&self) -> &'static str {
        match self {
            Language::Hu => "Képmetszés megszakítva",
            Language::En => "Snip cancelled",
        }
    }

    pub fn toast_pipette_cancelled(&self) -> &'static str {
        match self {
            Language::Hu => "Pipetta megszakítva",
            Language::En => "Picker cancelled",
        }
    }

    pub fn toast_stroke_font_size(&self, th: f32, fs: f32) -> String {
        match self {
            Language::Hu => format!("Betűméret: {:.0} pt  |  Vonalvastagság: {:.0} px", fs, th),
            Language::En => format!("Font size: {:.0} pt  |  Stroke width: {:.0} px", fs, th),
        }
    }

    pub fn toast_stroke_size(&self, th: f32) -> String {
        match self {
            Language::Hu => format!("Vonalvastagság: {:.0} px", th),
            Language::En => format!("Stroke width: {:.0} px", th),
        }
    }

    pub fn toast_pipette_sampled(&self, r: u8, g: u8, b: u8, tool_name: &str) -> String {
        match self {
            Language::Hu => format!("Szín mintavételezve: #{:02X}{:02X}{:02X} ({})", r, g, b, tool_name),
            Language::En => format!("Sampled color: #{:02X}{:02X}{:02X} ({})", r, g, b, tool_name),
        }
    }

    pub fn toast_curve_locked(&self) -> &'static str {
        match self {
            Language::Hu => "Ív rögzítve",
            Language::En => "Curve placed",
        }
    }

    pub fn toast_filled_rect_selected(&self) -> &'static str {
        match self {
            Language::Hu => "Takarás (kitöltött téglalap) kiválasztva",
            Language::En => "Mask (filled rectangle) selected",
        }
    }

    pub fn toast_rect_selected(&self) -> &'static str {
        match self {
            Language::Hu => "Téglalap (körvonal) kiválasztva",
            Language::En => "Rectangle (outline) selected",
        }
    }

    pub fn toast_color_selected(&self) -> &'static str {
        match self {
            Language::Hu => "Szín kiválasztva",
            Language::En => "Color selected",
        }
    }

    pub fn toast_pipette_prompt(&self) -> &'static str {
        match self {
            Language::Hu => "Pipetta: Kattints a képernyőre a szín mintavételezéséhez! (Esc: mégse)",
            Language::En => "Picker: Click anywhere on screen to sample color! (Esc to cancel)",
        }
    }

    pub fn toast_opacity(&self, pct: i32) -> String {
        match self {
            Language::Hu => format!("Átlátszatlanság: {}%", pct),
            Language::En => format!("Opacity: {}%", pct),
        }
    }

    pub fn toast_curve_bending(&self) -> &'static str {
        match self {
            Language::Hu => "Mozgasd az egeret az ív hajlításához, majd kattints a rögzítéshez!",
            Language::En => "Move mouse to bend curve, then click to place!",
        }
    }

    pub fn toast_update_available(&self, tag: &str) -> String {
        match self {
            Language::Hu => format!("Új DrawIt verzió érhető el: {} (Logo menü)", tag),
            Language::En => format!("New DrawIt version available: {} (Menu)", tag),
        }
    }

    pub fn toast_kb_rebound(&self, ch: char, tool_name: &str) -> String {
        match self {
            Language::Hu => format!("'{}' billentyű beállítva: {}", ch, tool_name),
            Language::En => format!("'{}' key assigned to: {}", ch, tool_name),
        }
    }

    pub fn toast_kb_reset(&self) -> &'static str {
        match self {
            Language::Hu => "Alapértelmezett gombkiosztás visszaállítva!",
            Language::En => "Default keybindings restored!",
        }
    }

    // --- Settings Modal ---
    pub fn settings_title(&self) -> &'static str {
        match self {
            Language::Hu => "DrawIt – Beállítások",
            Language::En => "DrawIt – Settings",
        }
    }

    pub fn settings_tab_general(&self) -> &'static str {
        match self {
            Language::Hu => "Általános",
            Language::En => "General",
        }
    }

    pub fn settings_tab_keybindings(&self) -> &'static str {
        match self {
            Language::Hu => "Gombkiosztás",
            Language::En => "Keybindings",
        }
    }

    pub fn settings_kb_title(&self) -> &'static str {
        match self {
            Language::Hu => "Rajzeszközök gyorsbillentyűi (kattints a gombra a módosításhoz):",
            Language::En => "Drawing tool shortcuts (click key pill to customize):",
        }
    }

    pub fn settings_kb_desc(&self) -> &'static str {
        match self {
            Language::Hu => "Kattints egy billentyűre az új gomb lenyomásához. Ütközés esetén a gombok automatikusan felcserélődnek.",
            Language::En => "Click a key to assign a new shortcut. Conflicting keys will automatically swap.",
        }
    }

    pub fn settings_kb_rebinding_hint(&self, tool_name: &str) -> String {
        match self {
            Language::Hu => format!("Nyomj le egy új billentyűt (A–Z vagy 0–9) a(z) „{}” eszközhöz! [Esc: Mégse]", tool_name),
            Language::En => format!("Press a new key (A–Z or 0–9) for \"{}\"! [Esc: Cancel]", tool_name),
        }
    }

    pub fn settings_kb_reset_btn(&self) -> &'static str {
        match self {
            Language::Hu => "Alapértelmezett gombok visszaállítása",
            Language::En => "Reset to Default Keys",
        }
    }

    pub fn settings_kb_spring_note(&self) -> &'static str {
        match self {
            Language::Hu => "• Rugós segédbillentyűk (nyomva tartva): Shift = Vonal  |  Ctrl = Nyíl  |  Ctrl+Shift = Számozott nyíl",
            Language::En => "• Spring modifier keys (hold down): Shift = Line  |  Ctrl = Arrow  |  Ctrl+Shift = Numbered Arrow",
        }
    }

    pub fn settings_section_lang(&self) -> &'static str {
        match self {
            Language::Hu => "Nyelv / Language:",
            Language::En => "Language / Nyelv:",
        }
    }

    pub fn settings_section_shortcuts(&self) -> &'static str {
        match self {
            Language::Hu => "Gyorsbillentyűk & Rugós eszközök (Hold / Tap):",
            Language::En => "Keyboard Shortcuts & Spring Tools (Hold / Tap):",
        }
    }

    pub fn settings_tip_1(&self) -> &'static str {
        match self {
            Language::Hu => "• Tipp: Tartsd nyomva a billentyűt az ideiglenes rajzhoz (elengedve visszaugrik),",
            Language::En => "• Tip: Hold a key to draw temporarily (releases back to previous tool),",
        }
    }

    pub fn settings_tip_2(&self) -> &'static str {
        match self {
            Language::Hu => "  vagy nyomd meg egyszer a végleges eszközváltáshoz!",
            Language::En => "  or tap once to permanently switch tools!",
        }
    }

    pub fn settings_section_hotkey(&self) -> &'static str {
        match self {
            Language::Hu => "Globális aktiváló gyorsbillentyű (Bárhonnan a rendszerből):",
            Language::En => "Global Activation Hotkey (Anywhere from Windows):",
        }
    }

    pub fn settings_section_system(&self) -> &'static str {
        match self {
            Language::Hu => "Rendszer és Indítás:",
            Language::En => "System & Startup:",
        }
    }

    pub fn settings_autostart(&self) -> &'static str {
        match self {
            Language::Hu => "Indítás a Windows-zal (Autostart a tálcán a háttérben)",
            Language::En => "Start with Windows (Launch silently in system tray)",
        }
    }

    pub fn settings_check_updates(&self) -> &'static str {
        #[cfg(feature = "store")]
        {
            match self {
                Language::Hu => "Frissítések keresése a Microsoft Store-ban (Megnyitás)",
                Language::En => "Check for updates in Microsoft Store (Open Store)",
            }
        }
        #[cfg(not(feature = "store"))]
        {
            match self {
                Language::Hu => "Frissítések automatikus keresése indításkor (GitHub Releases)",
                Language::En => "Automatically check for updates on startup (GitHub Releases)",
            }
        }
    }

    pub fn settings_section_ocr(&self) -> &'static str {
        match self {
            Language::Hu => "Optikai szövegfelismerés (OCR állapot)",
            Language::En => "Optical Character Recognition (OCR Status)",
        }
    }

    pub fn settings_ocr_active(&self, lang_tag: &str) -> String {
        match self {
            Language::Hu => format!("Elérhető és működik (Windows natív motor: {})", lang_tag),
            Language::En => format!("Available and active (Windows native engine: {})", lang_tag),
        }
    }

    pub fn settings_ocr_active_desc(&self) -> &'static str {
        match self {
            Language::Hu => "A Képmetsző (Snip) kijelölésnél megjelenik a 'Szöveg Másolás' gomb a vágólapra mentéshez.",
            Language::En => "The 'Copy Text' button is enabled in the Snipping tool for direct clipboard copy.",
        }
    }

    pub fn settings_ocr_inactive(&self) -> &'static str {
        match self {
            Language::Hu => "Nem érhető el (Nincs telepítve Windows OCR nyelvi csomag)",
            Language::En => "Not available (No Windows OCR language package installed)",
        }
    }

    pub fn settings_ocr_inactive_desc(&self) -> &'static str {
        match self {
            Language::Hu => "Telepíts OCR nyelvi csomagot a Windows Gépházban (Idő és nyelv -> Nyelv).",
            Language::En => "Install an OCR language package in Windows Settings (Time & Language -> Language).",
        }
    }

    pub fn settings_portable_note(&self) -> &'static str {
        match self {
            Language::Hu => "A beállítások automatikusan hordozható formátumban (settings.json) tárolódnak.",
            Language::En => "Settings are stored automatically in portable format (settings.json).",
        }
    }

    pub fn settings_btn_save(&self) -> &'static str {
        match self {
            Language::Hu => "Mentés és Bezárás",
            Language::En => "Save & Close",
        }
    }

    // --- Help Modal ---
    pub fn help_title(&self) -> &'static str {
        match self {
            Language::Hu => "DrawIt – Útmutató és Gyorsbillentyűk",
            Language::En => "DrawIt – Guide & Shortcuts",
        }
    }

    pub fn help_group_activation(&self) -> &'static str {
        match self {
            Language::Hu => "Aktiválás:",
            Language::En => "Activation:",
        }
    }

    pub fn help_group_tools(&self) -> &'static str {
        match self {
            Language::Hu => "Rajzeszközök:",
            Language::En => "Drawing Tools:",
        }
    }

    pub fn help_tool_lines(&self) -> &'static [&'static str] {
        match self {
            Language::Hu => &[
                "• Rajz / Vonal / Nyíl: Szabadkézi toll, egyenes vonalak és folyamatnyilak",
                "• Téglalap / Kör: Területek keretezése és fókusz kiemelése (körvonal)",
                "• Takarás: Kitöltött téglalap érzékeny adatok, jelszavak azonnali maszkolására",
                "• Számozás: Húzzon egy nyilat, a számozott körjelvény automatikusan a nyíl tövére kerül",
                "• Ív: Húzzon egy vonalat, engedje fel az egeret, hajlítsa meg a kurzorral, majd kattintson",
                "• Szöveg: Kattintson bárhová és gépeljen közvetlenül a vászonra (Enter lezár)",
                "• Elmosás: Érzékeny adatok, képernyőterületek azonnali elhomályosítása",
                "• Kivágás: Képernyőrészlet kijelölése kép- vagy Szöveg Másoláshoz (OCR), mentéshez",
            ],
            Language::En => &[
                "• Pen / Line / Arrow: Freehand drawing, straight lines and directional arrows",
                "• Rectangle / Circle: Outlining areas and highlighting key focal points",
                "• Mask: Solid filled rectangle to quickly conceal sensitive data or passwords",
                "• Numbering: Drag an arrow, numbered badge is placed at the arrow tail",
                "• Curved Arrow/Line: Drag line, release, bend with mouse cursor, click to finish",
                "• Text: Click anywhere and type directly onto the screen (press Enter to commit)",
                "• Blur: Instantly blur out sensitive screen details and personal info",
                "• Snip: Select rectangular region to copy, Copy Text (OCR), or export",
            ],
        }
    }

    pub fn help_group_shortcuts(&self) -> &'static str {
        match self {
            Language::Hu => "Színek és Gyorsbillentyűk:",
            Language::En => "Colors & Shortcuts:",
        }
    }

    pub fn help_shortcut_lines(&self) -> &'static [&'static str] {
        match self {
            Language::Hu => &[
                "• Színválasztó: 18 gyorsszín, Beépített színkeverő és Pipetta képernyő-mintavevő",
                "• Ctrl + Görgő (vagy görgő): Vonalvastagság és szövegméret gyors állítása (1–32 px)",
                "• Esc: Kilépés / Mégse   |   Ctrl + Z / Ctrl + Y: Visszavonás / Újra",
                "• Delete / Radír: Rajzok azonnali letörlése (Tiszta lap, számláló nullázása 1-re)",
                "• F1: Ezen súgóablak megjelenítése vagy bezárása",
            ],
            Language::En => &[
                "• Color Picker: 18 quick presets, custom palette mixer, and screen color picker",
                "• Ctrl + Scroll (or scroll): Rapidly adjust stroke thickness & text size (1–32 px)",
                "• Esc: Exit / Cancel   |   Ctrl + Z / Ctrl + Y: Undo / Redo",
                "• Delete / Eraser: Clear all annotations (Clean screen, resets counter to #1)",
                "• F1: Show or dismiss this help guide",
            ],
        }
    }

    pub fn help_btn_ok(&self) -> &'static str {
        match self {
            Language::Hu => "Rendben",
            Language::En => "Got it",
        }
    }

    // --- About Modal ---
    pub fn about_title(&self) -> &'static str {
        match self {
            Language::Hu => "DrawIt – Névjegy",
            Language::En => "DrawIt – About",
        }
    }

    pub fn about_subtitle(&self) -> &'static str {
        match self {
            Language::Hu => "Gyors, hordozható képernyőrajzoló és annotáló segédprogram",
            Language::En => "Fast, portable screen annotation and presentation tool",
        }
    }

    pub fn about_bullets(&self) -> &'static [&'static str] {
        match self {
            Language::Hu => &[
                "• 100% natív Rust kód hardveres Direct2D gyorsítással",
                "• Azonnali indulás, elhanyagolható memóriahasználat (~15-25 MB)",
                "• Nem igényel telepítést, .NET keretrendszert vagy admin jogot",
                "• Többmonitoros intelligens elhelyezés és kijelző-mentés",
            ],
            Language::En => &[
                "• 100% native Rust code with hardware-accelerated Direct2D",
                "• Instant startup, tiny memory footprint (~15-25 MB)",
                "• Fully standalone: no installation, .NET runtime, or admin rights needed",
                "• Multi-monitor intelligent toolbar placement & display export",
            ],
        }
    }

    pub fn about_btn_close(&self) -> &'static str {
        match self {
            Language::Hu => "Bezárás",
            Language::En => "Close",
        }
    }

    // --- System Tray ---
    pub fn tray_tip_prefix(&self) -> &'static str {
        match self {
            Language::Hu => "DrawIt – Képernyő rajzoló és jegyzetelő",
            Language::En => "DrawIt – Screen Annotation & Drawing",
        }
    }

    pub fn tray_start(&self) -> &'static str {
        match self {
            Language::Hu => "Rajzolás indítása",
            Language::En => "Start Drawing",
        }
    }

    pub fn tray_settings(&self) -> &'static str {
        match self {
            Language::Hu => "Beállítások...",
            Language::En => "Settings...",
        }
    }

    pub fn tray_updates(&self) -> &'static str {
        #[cfg(feature = "store")]
        {
            match self {
                Language::Hu => "Frissítések keresése (Microsoft Store)...",
                Language::En => "Check for Updates (Microsoft Store)...",
            }
        }
        #[cfg(not(feature = "store"))]
        {
            match self {
                Language::Hu => "Frissítések keresése...",
                Language::En => "Check for Updates...",
            }
        }
    }

    pub fn tray_help(&self) -> &'static str {
        match self {
            Language::Hu => "Útmutató és billentyűk (F1)",
            Language::En => "Guide & Shortcuts (F1)",
        }
    }

    pub fn tray_about(&self) -> &'static str {
        match self {
            Language::Hu => "Névjegy",
            Language::En => "About",
        }
    }

    pub fn tray_autostart(&self) -> &'static str {
        match self {
            Language::Hu => "Indítás a Windows-zal",
            Language::En => "Start with Windows",
        }
    }

    pub fn tray_exit(&self) -> &'static str {
        match self {
            Language::Hu => "Kilépés",
            Language::En => "Exit",
        }
    }

    // --- Updater ---
    pub fn updater_title(&self) -> &'static str {
        match self {
            Language::Hu => "DrawIt Frissítés",
            Language::En => "DrawIt Update",
        }
    }

    pub fn updater_avail_title(&self) -> &'static str {
        match self {
            Language::Hu => "DrawIt Frissítés elérhető",
            Language::En => "DrawIt Update Available",
        }
    }

    pub fn updater_err_title(&self) -> &'static str {
        match self {
            Language::Hu => "Frissítési hiba",
            Language::En => "Update Error",
        }
    }

    pub fn updater_err_check(&self, err: &str) -> String {
        match self {
            Language::Hu => format!("A frissítések keresése során hiba lépett fel:\n{}", err),
            Language::En => format!("An error occurred while checking for updates:\n{}", err),
        }
    }

    pub fn updater_avail_msg(&self, name: &str, cur: &str, tag: &str) -> String {
        match self {
            Language::Hu => format!(
                "Új DrawIt verzió érhető el: {}!\n\nJelenlegi verzió: v{}\nLegújabb kiadás: {}\n\nSzeretnéd most letölteni és azonnal frissíteni az alkalmazást?",
                name, cur, tag
            ),
            Language::En => format!(
                "A new DrawIt version is available: {}!\n\nCurrent version: v{}\nLatest release: {}\n\nWould you like to download and update the app now?",
                name, cur, tag
            ),
        }
    }

    pub fn updater_err_apply(&self, err: &str) -> String {
        match self {
            Language::Hu => format!("A frissítés letöltése vagy telepítése sikertelen:\n{}", err),
            Language::En => format!("Failed to download or install update:\n{}", err),
        }
    }

    pub fn updater_latest_msg(&self, cur: &str) -> String {
        #[cfg(feature = "store")]
        {
            match self {
                Language::Hu => format!(
                    "A legújabb DrawIt verziót használod (v{}).\nNincs újabb frissítés a Microsoft Store-ban.",
                    cur
                ),
                Language::En => format!(
                    "You are already using the latest DrawIt version (v{}).\nNo newer update available on Microsoft Store.",
                    cur
                ),
            }
        }
        #[cfg(not(feature = "store"))]
        {
            match self {
                Language::Hu => format!(
                    "A legújabb DrawIt verziót használod (v{}).\nNincs újabb frissítés a GitHubon.",
                    cur
                ),
                Language::En => format!(
                    "You are already using the latest DrawIt version (v{}).\nNo newer update available on GitHub.",
                    cur
                ),
            }
        }
    }

    // --- File Dialogs ---
    pub fn file_filter_png(&self) -> windows::core::PCWSTR {
        const FILTER_PNG_HU: windows::core::PCWSTR = windows::core::w!("PNG kép (*.png)\0*.png\0Minden fájl (*.*)\0*.*\0\0");
        const FILTER_PNG_EN: windows::core::PCWSTR = windows::core::w!("PNG Image (*.png)\0*.png\0All Files (*.*)\0*.*\0\0");
        match self {
            Language::Hu => FILTER_PNG_HU,
            Language::En => FILTER_PNG_EN,
        }
    }

    pub fn file_filter_pdf(&self) -> windows::core::PCWSTR {
        const FILTER_PDF_HU: windows::core::PCWSTR = windows::core::w!("PDF dokumentum (*.pdf)\0*.pdf\0Minden fájl (*.*)\0*.*\0\0");
        const FILTER_PDF_EN: windows::core::PCWSTR = windows::core::w!("PDF Document (*.pdf)\0*.pdf\0All Files (*.*)\0*.*\0\0");
        match self {
            Language::Hu => FILTER_PDF_HU,
            Language::En => FILTER_PDF_EN,
        }
    }

    pub fn default_snip_name(&self, timestamp: &str) -> String {
        match self {
            Language::Hu => format!("Kivagas_{}.png", timestamp),
            Language::En => format!("Snip_{}.png", timestamp),
        }
    }

    pub fn default_snip_pdf_name(&self, timestamp: &str) -> String {
        match self {
            Language::Hu => format!("Kivagas_{}.pdf", timestamp),
            Language::En => format!("Snip_{}.pdf", timestamp),
        }
    }

    pub fn default_desktop_name(&self) -> &'static str {
        match self {
            Language::Hu => "DrawIt_Teljes_Asztal",
            Language::En => "DrawIt_Full_Desktop",
        }
    }

    pub fn default_display_name_prefix(&self, idx: usize) -> String {
        match self {
            Language::Hu => format!("DrawIt_{}_Kijelzo", idx + 1),
            Language::En => format!("DrawIt_Display_{}", idx + 1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_parsing_and_names() {
        let sys_lang = Language::detect_system();
        println!("Detected Windows system language: {:?}", sys_lang);
        assert_eq!(Language::default(), sys_lang);
        assert_eq!(Language::from_str("hu"), Language::Hu);
        assert_eq!(Language::from_str("HU"), Language::Hu);
        assert_eq!(Language::from_str("hungarian"), Language::Hu);
        assert_eq!(Language::from_str("en"), Language::En);
        assert_eq!(Language::from_str("EN"), Language::En);
        assert_eq!(Language::from_str("english"), Language::En);
        assert_eq!(Language::from_str("unknown"), sys_lang);

        assert_eq!(Language::Hu.as_str(), "hu");
        assert_eq!(Language::En.as_str(), "en");
        assert_eq!(Language::Hu.display_name(), "Magyar");
        assert_eq!(Language::En.display_name(), "English");
    }

    #[test]
    fn test_language_strings_non_empty() {
        for lang in [Language::Hu, Language::En] {
            assert!(!lang.canvas_btn().is_empty());
            assert!(!lang.snip_btn().is_empty());
            assert!(!lang.export_display_btn().is_empty());
            assert!(!lang.settings_title().is_empty());
            assert!(!lang.settings_tab_general().is_empty());
            assert!(!lang.settings_tab_keybindings().is_empty());
            assert!(!lang.settings_kb_reset_btn().is_empty());
            assert!(!lang.toast_kb_reset().is_empty());
            assert!(!lang.help_title().is_empty());
            assert!(!lang.about_title().is_empty());
            assert!(!lang.toast_clear().is_empty());
            assert!(!lang.toast_undo().is_empty());
            assert!(!lang.toast_redo().is_empty());
            assert!(!lang.help_tool_lines().is_empty());
            assert!(!lang.help_shortcut_lines().is_empty());
            assert!(!lang.about_bullets().is_empty());
        }
    }
}
