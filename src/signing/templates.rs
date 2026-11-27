use super::DesktopSigningRequest;
use crate::{
    annotations::{Annotation, Box2, Kind, Style},
    assets::Asset,
    signature_art::Lettering,
};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignatureTemplate {
    #[default]
    CleanSplit,
    NameFields,
    Compact,
    Ink,
    /// The layout used by saved signatures before templates were introduced.
    Classic,
}

impl SignatureTemplate {
    pub const CHOICES: [Self; 4] = [Self::CleanSplit, Self::NameFields, Self::Compact, Self::Ink];

    pub fn label(self) -> &'static str {
        match self {
            Self::CleanSplit => "Clean Split",
            Self::NameFields => "Name Fields",
            Self::Compact => "Compact",
            Self::Ink => "Ink",
            Self::Classic => "Classic",
        }
    }

    pub fn uses_artwork(self) -> bool {
        matches!(self, Self::CleanSplit | Self::Ink | Self::Classic)
    }

    pub fn artwork_beside_details(self) -> bool {
        matches!(self, Self::CleanSplit | Self::Classic)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignatureLanguage {
    #[default]
    English,
    Turkish,
    German,
}

impl SignatureLanguage {
    pub const CHOICES: [Self; 3] = [Self::English, Self::Turkish, Self::German];

    pub fn label(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Turkish => "Türkçe",
            Self::German => "Deutsch",
        }
    }

    fn words(self) -> [&'static str; 4] {
        match self {
            Self::English => [
                "Digitally signed",
                "Given name",
                "Family name",
                "Certificate holder",
            ],
            Self::Turkish => [
                "Dijital olarak imzalandı",
                "Ad",
                "Soyadı",
                "Sertifika sahibi",
            ],
            Self::German => [
                "Digital signiert",
                "Vorname",
                "Nachname",
                "Zertifikatsinhaber",
            ],
        }
    }

    pub(super) fn classic_heading(self, name: &str) -> String {
        match self {
            Self::English => format!("Digitally signed by {name}"),
            Self::Turkish => format!("Dijital olarak imzalayan: {name}"),
            Self::German => format!("Digital signiert von {name}"),
        }
    }

    pub(super) fn certificate_holder(self) -> &'static str {
        self.words()[3]
    }

    pub(super) fn date(self, now: OffsetDateTime) -> String {
        let (year, month, day) = now.to_calendar_date();
        let month = u8::from(month) as usize;
        let date = match self {
            Self::English => {
                let names = [
                    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov",
                    "Dec",
                ];
                format!("{day} {} {year}", names[month - 1])
            }
            Self::Turkish => {
                let names = [
                    "Oca", "Şub", "Mar", "Nis", "May", "Haz", "Tem", "Ağu", "Eyl", "Eki", "Kas",
                    "Ara",
                ];
                format!("{day} {} {year}", names[month - 1])
            }
            Self::German => format!("{day:02}.{month:02}.{year}"),
        };
        format!("{date} · {:02}:{:02} UTC", now.hour(), now.minute())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(default)]
pub struct SignatureDesign {
    pub template: SignatureTemplate,
    pub language: SignatureLanguage,
    /// Display-only overrides for the Name Fields template.
    pub given_name: String,
    pub family_name: String,
}

impl SignatureDesign {
    pub fn classic() -> Self {
        Self {
            template: SignatureTemplate::Classic,
            ..Self::default()
        }
    }

    pub fn name_fields<'a>(&'a self, name: &'a str) -> (&'a str, &'a str) {
        let name = name.trim();
        let (given, family) = name.rsplit_once(char::is_whitespace).unwrap_or((name, ""));
        let display = |overrides: &'a str, fallback: &'a str| {
            if overrides.trim().is_empty() {
                fallback.trim()
            } else {
                overrides.trim()
            }
        };
        (
            display(&self.given_name, given),
            display(&self.family_name, family),
        )
    }
}

fn divider(rect: Box2) -> Annotation {
    let (width, height) = (rect[2] - rect[0], rect[3] - rect[1]);
    Annotation::new(
        0,
        rect,
        Kind::Svg {
            source: format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\"><rect width=\"{width}\" height=\"{height}\" fill=\"#bdc4d1\"/></svg>"
            ),
            tint: false,
        },
        Style::default(),
    )
}

/// Place the composed rows vertically as a unit, keeping the name on one line.
fn details(
    request: &DesktopSigningRequest,
    rect: Box2,
    center_y: f32,
    date: &str,
) -> Result<(Vec<Annotation>, f32, f32)> {
    let [left, top, right, bottom] = rect;
    let width = right - left;
    let height = bottom - top;
    ensure!(
        width >= 1. && height >= 1.,
        "Move the text farther inside the signature box"
    );
    let style = &request.text_style;
    let font = Lettering::load(&style.font()?)?;
    let align = style.alignment;
    let [signed, given_label, family_label, placeholder] = request.design.language.words();
    let name = request.name.trim();
    let name = if name.is_empty() { placeholder } else { name };
    let muted = style.color.map(|c| c * 0.78 + 0.17);
    let mut size = style.size;
    loop {
        let mut marks = Vec::new();
        let mut rules = Vec::new();
        let mut y = 0.;
        let mut row = |text: &str, bounds: [f32; 2], y: f32, emphasis: f32, wrap: bool| {
            let [x, r] = bounds;
            let color = if emphasis >= 1. { style.color } else { muted };
            let (source, h) =
                font.render(text, (size * emphasis).max(1.), color, r - x, align, wrap)?;
            marks.push(Annotation::new(
                0,
                [x, y, r, y + h],
                Kind::Svg {
                    source,
                    tint: false,
                },
                Style::default(),
            ));
            Ok::<_, anyhow::Error>(h)
        };
        let full = [left, right];
        let composed = (|| -> Result<()> {
            if request.show_name && request.design.template == SignatureTemplate::NameFields {
                let (given, family) = request.design.name_fields(&request.name);
                if family.is_empty() || given.is_empty() {
                    let display = if !given.is_empty() {
                        given
                    } else if !family.is_empty() {
                        family
                    } else {
                        name
                    };
                    y += row(display, full, y, 1.12, false)?;
                    if !given.is_empty() || !family.is_empty() {
                        y += row(
                            if family.is_empty() {
                                given_label
                            } else {
                                family_label
                            },
                            full,
                            y,
                            0.74,
                            false,
                        )?;
                    }
                } else {
                    let middle = (left + right) * 0.5;
                    let gap = (width * 0.06).min(8.);
                    let given_column = [left, middle - gap];
                    let family_column = [middle + gap, right];
                    let first = row(given, given_column, y, 1.12, false)?;
                    let last = row(family, family_column, y, 1.12, false)?;
                    y += first.max(last);
                    for [x, r] in [given_column, family_column] {
                        rules.push(divider([x, y, r, y + 0.45]));
                    }
                    y += 1.8;
                    let first = row(given_label, given_column, y, 0.74, false)?;
                    let last = row(family_label, family_column, y, 0.74, false)?;
                    y += first.max(last);
                }
            } else if request.show_name {
                if request.design.template == SignatureTemplate::CleanSplit {
                    y += row(signed, full, y, 0.78, false)?;
                }
                y += row(name, full, y, 1.12, false)?;
            }
            if request.show_date {
                y += row(date, full, y, 0.78, false)?;
            }
            for text in [request.reason.trim(), request.location.trim()] {
                if !text.is_empty() {
                    y += row(text, full, y, 0.86, true)?;
                }
            }
            Ok(())
        })();
        match composed {
            Ok(()) if y <= height => {
                let start = (center_y - y * 0.5).clamp(top, (bottom - y).max(top));
                marks.extend(rules);
                for mark in &mut marks {
                    mark.rect[1] += start;
                    mark.rect[3] += start;
                }
                return Ok((marks, start, start + y));
            }
            result if size <= 1. => {
                result?;
                anyhow::bail!("Move the text farther inside the signature box");
            }
            _ => size = (size * 0.85).max(1.),
        }
    }
}

pub(super) fn compose(
    request: &DesktopSigningRequest,
    dimensions: [f32; 2],
    scale: f32,
    artwork: Option<&Asset>,
    place_artwork: impl Fn(&Asset, Box2, f32) -> Result<Annotation>,
    now: OffsetDateTime,
) -> Result<Vec<Annotation>> {
    let [w, h] = dimensions;
    let template = request.design.template;
    let padding = request.transform.outer_padding.map_or(7., |p| p * scale);
    let gap = request.transform.image_text_gap.map_or(12., |g| g * scale);
    let outer = [padding, padding, w - padding, h - padding];
    let (mut text, mut artwork_box) = (outer, outer);
    let overlay = template.artwork_beside_details() && request.transform.image_overlay;
    let mut vertical_rule = None;
    if artwork.is_some() && !overlay {
        let gap = gap.min((w - 2. * padding) * 0.2);
        if template == SignatureTemplate::Ink {
            artwork_box[3] = padding + (h - 2. * padding) * 0.48;
            text[1] = artwork_box[3] + gap.min((h - 2. * padding) * 0.10);
        } else {
            artwork_box[2] = padding + (w - 2. * padding - gap) * 0.34;
            text[0] = artwork_box[2] + gap;
            if gap >= 3. {
                vertical_rule = Some(artwork_box[2] + gap * 0.5);
            }
        }
    }
    let [dx, dy] = request.text_style.offset;
    let center_y = (text[1] + text[3]) * 0.5
        + h * dy / 100.
        + (text[3] - text[1]) * request.text_style.insets[1] / 100.;
    text[0] = (text[0] + w * dx / 100.).clamp(0., text[2] - 1.);
    text[1] = (text[1] + h * dy / 100.).clamp(0., text[3] - 1.);
    let text = super::inset(text, request.text_style.insets);
    let (mut marks, top, bottom) =
        details(request, text, center_y, &request.design.language.date(now))?;
    if let Some(x) = vertical_rule
        && x + 0.5 < text[0]
    {
        marks.push(divider([x, top, x + 0.5, bottom]));
    }
    if template == SignatureTemplate::Ink && artwork.is_some() && top > artwork_box[3] + 1. {
        let width = ((text[2] - text[0]) * 0.65).min(105.);
        let center = (text[0] + text[2]) * 0.5;
        let y = (top + artwork_box[3]) * 0.5;
        marks.push(divider([
            center - width * 0.5,
            y,
            center + width * 0.5,
            y + 0.45,
        ]));
    }
    if let Some(notation) = artwork {
        let mark = place_artwork(notation, artwork_box, 8.)?;
        if overlay {
            marks.push(mark);
        } else {
            marks.insert(0, mark);
        }
    }
    Ok(marks)
}
