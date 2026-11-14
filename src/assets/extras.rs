//! Optional artwork, appended so all original catalog IDs remain stable.
use super::*;
pub(super) fn append(out: &mut Vec<BuiltinAsset>) {
    let mut add = |category, name: &str, body: &str| {
        let source = format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><g fill="none" stroke="#000" stroke-width="6" stroke-linecap="round" stroke-linejoin="round">{body}</g></svg>"##
        );
        let mut asset = Asset::new(
            name,
            Kind::Svg { source, tint: true },
            Style {
                color: [0.12, 0.46, 0.83],
                ..Default::default()
            },
            1.,
        );
        asset.id = format!("b27a{:028x}", out.len());
        out.push(BuiltinAsset {
            category,
            asset,
            default_visible: false,
        });
    };
    for (name, body) in [
        (
            "Right-angle up",
            r#"<path d="M12 82H76V18 M59 35L76 18L93 35"/>"#,
        ),
        (
            "Right-angle down",
            r#"<path d="M12 18H76V82 M59 65L76 82L93 65"/>"#,
        ),
        (
            "Rising steps",
            r#"<path d="M10 82H34V58H58V34H86 M70 18L86 34L70 50"/>"#,
        ),
        (
            "Falling steps",
            r#"<path d="M10 18H34V42H58V66H86 M70 50L86 66L70 82"/>"#,
        ),
        (
            "Loop clockwise",
            r#"<path d="M12 80H58C96 80 96 20 58 20C22 20 22 59 51 59H80 M65 44L80 59L65 74"/>"#,
        ),
        (
            "Loop counterclockwise",
            r#"<path d="M88 80H42C4 80 4 20 42 20C78 20 78 59 49 59H20 M35 44L20 59L35 74"/>"#,
        ),
        (
            "Circular clockwise",
            r#"<path d="M80 32A35 35 0 1 0 84 64 M62 33L82 33L83 12"/>"#,
        ),
        (
            "Circular counterclockwise",
            r#"<path d="M20 32A35 35 0 1 1 16 64 M38 33L18 33L17 12"/>"#,
        ),
        (
            "Merge arrows",
            r#"<path d="M12 18L46 50L84 50 M12 82L46 50 M68 34L84 50L68 66"/>"#,
        ),
        (
            "Three-way arrow",
            r#"<path d="M50 88V16 M50 52L16 24 M50 52L84 24 M35 31L50 16L65 31 M16 44V24H36 M64 24H84V44"/>"#,
        ),
        (
            "Horizontal exchange",
            r#"<path d="M12 32H86 M70 16L86 32L70 48 M88 68H14 M30 52L14 68L30 84"/>"#,
        ),
        (
            "Vertical exchange",
            r#"<path d="M32 88V14 M16 30L32 14L48 30 M68 12V86 M52 70L68 86L84 70"/>"#,
        ),
        (
            "Double chevron right",
            r#"<path d="M18 16L50 50L18 84 M50 16L82 50L50 84"/>"#,
        ),
        (
            "Double chevron left",
            r#"<path d="M82 16L50 50L82 84 M50 16L18 50L50 84"/>"#,
        ),
        (
            "Return to origin",
            r#"<path d="M86 18V54Q86 78 62 78H16 M34 60L16 78L34 96"/>"#,
        ),
    ] {
        add("Arrows", name, body);
    }
    for (name, body) in [
        (
            "Lock",
            r#"<rect x="21" y="42" width="58" height="46" rx="8"/><path d="M32 42V28A18 18 0 0 1 68 28V42 M50 61V71"/>"#,
        ),
        (
            "Shield check",
            r#"<path d="M50 10L84 24V51Q82 76 50 91Q18 76 16 51V24Z M32 49L45 62L69 35"/>"#,
        ),
        (
            "Calendar",
            r#"<rect x="15" y="22" width="70" height="66" rx="7"/><path d="M31 12V32 M69 12V32 M15 42H85 M32 58H39 M59 58H66 M32 72H39"/>"#,
        ),
        (
            "Clock",
            r#"<circle cx="50" cy="50" r="37"/><path d="M50 25V50L69 62"/>"#,
        ),
        (
            "Attachment",
            r#"<path d="M34 61L63 32C78 17 96 37 80 53L46 87C20 108 0 77 22 55L57 20C72 5 92 16 89 33 M33 64L61 36"/>"#,
        ),
        (
            "Link",
            r#"<path d="M41 30L53 18C73 -1 101 27 82 47L70 59 M59 70L47 82C27 101 -1 73 18 53L30 41 M32 68L68 32"/>"#,
        ),
        ("Bookmark", r#"<path d="M26 12H74V90L50 72L26 90Z"/>"#),
        (
            "Question",
            r#"<circle cx="50" cy="50" r="39"/><path d="M34 34C34 14 75 16 67 38C64 47 50 47 50 61 M50 75H50.1"/>"#,
        ),
        (
            "Document check",
            r#"<path d="M22 10H61L80 29V90H22Z M61 10V29H80 M35 60L46 71L68 47"/>"#,
        ),
        (
            "Upload",
            r#"<path d="M50 67V12 M31 31L50 12L69 31 M17 65V86H83V65"/>"#,
        ),
        (
            "Download",
            r#"<path d="M50 12V67 M31 48L50 67L69 48 M17 65V86H83V65"/>"#,
        ),
        (
            "Location pin",
            r#"<path d="M50 93C38 76 19 55 19 36A31 31 0 0 1 81 36C81 55 62 76 50 93Z"/><circle cx="50" cy="36" r="11"/>"#,
        ),
        (
            "Magnifier",
            r#"<circle cx="41" cy="41" r="28"/><path d="M62 62L88 88"/>"#,
        ),
        (
            "Bell",
            r#"<path d="M18 72H82L72 58V37A22 22 0 0 0 28 37V58Z M42 85Q50 95 58 85 M50 10V15"/>"#,
        ),
        (
            "Balance scales",
            r#"<path d="M50 12V86 M30 88H70 M17 29H83 M22 30L8 59H36Z M78 30L64 59H92Z"/>"#,
        ),
    ] {
        add("Symbols", name, body);
    }
    for (label, color) in [
        ("ACTION REQUIRED", "#a82018"),
        ("FOR APPROVAL", "#192d70"),
        ("VERIFIED", "#3d6c25"),
        ("ON HOLD", "#98670d"),
        ("SUPERSEDED", "#594091"),
        ("ARCHIVED", "#526171"),
        ("ORIGINAL", "#192d70"),
        ("CERTIFIED COPY", "#3d6c25"),
        ("INTERNAL USE ONLY", "#594091"),
        ("DO NOT DISTRIBUTE", "#a82018"),
        ("FOLLOW UP", "#98670d"),
        ("URGENT", "#a82018"),
        ("CANCELLED", "#526171"),
        ("APPROVED WITH COMMENTS", "#3d6c25"),
        ("RESUBMIT", "#a82018"),
    ] {
        let mut asset = Asset::new(
            label,
            label_stamp(label, color, false),
            Style::default(),
            stamp_width(label) / 64.,
        );
        asset.id = format!("b27a{:028x}", out.len());
        asset.stamp = true;
        out.push(BuiltinAsset {
            category: "Stamps",
            asset,
            default_visible: false,
        });
    }
}
