use std::io::BufRead;
use std::path::PathBuf;
use std::str;
use std::u32;
use Layout;
use settings::Color;
use quick_xml::reader::Reader;
use run::parser::xml_util::{attribute, attribute_err, end_tag, optional_attribute_err, parse_attributes,
                      parse_base, parse_children, reencode_children, text, text_as_bytes_err,
                      text_err, text_parsed};
pub use run::parser::xml_util::{Error, Result};

fn parse_bool<S: AsRef<str>>(text: S) -> Result<bool> {
    match text.as_ref() {
        "True" => Ok(true),
        "False" => Ok(false),
        _ => Err(Error::Bool),
    }
}

fn parse_color<S: AsRef<str>>(text: S) -> Result<Color> {
    let text = text.as_ref();
    let a = u32::from_str_radix(&text[..2], 16)?;
    let r = u32::from_str_radix(&text[2..4], 16)?;
    let g = u32::from_str_radix(&text[4..6], 16)?;
    let b = u32::from_str_radix(&text[6..8], 16)?;
    Ok(Color::from((r as f32, g as f32, b as f32, a as f32)))
}

fn color<R>(reader: &mut Reader<R>, buf: &mut Vec<u8>, setting: &mut Color) -> Result<()>
where
    R: BufRead,
{
    text(reader, buf, |t| {
        if let Ok(color) = parse_color(t) {
            *setting = color;
        }
    })
}

pub fn parse<R: BufRead>(source: R, path: Option<PathBuf>) -> Result<Layout> {
    let reader = &mut Reader::from_reader(source);
    reader.expand_empty_elements(true);
    reader.trim_text(true);
    let mut buf = Vec::with_capacity(4096);
    let mut layout = Layout::new();

    parse_base(reader, &mut buf, b"Layout", |reader, tag| {
        parse_children(reader, tag.into_buf(), |reader, tag| {
            if tag.name() == b"Settings" {
                let settings = layout.general_settings_mut();
                parse_children(reader, tag.into_buf(), |reader, tag| {
                    if tag.name() == b"TextColor" {
                        color(reader, tag.into_buf(), &mut settings.text_color)
                    } else if tag.name() == b"ThinSeparatorsColor" {
                    	color(reader, tag.into_buf(), &mut settings.thin_separators_color)
                    } else if tag.name() == b"SeparatorsColor" {
                    	color(reader, tag.into_buf(), &mut settings.separators_color)
                    } else if tag.name() == b"PersonalBestColor" {
                    	color(reader, tag.into_buf(), &mut settings.personal_best_color)
                    } else if tag.name() == b"AheadGainingTimeColor" {
                    	color(reader, tag.into_buf(), &mut settings.ahead_gaining_time_color)
                    } else if tag.name() == b"AheadLosingTimeColor" {
                    	color(reader, tag.into_buf(), &mut settings.ahead_losing_time_color)
                    } else if tag.name() == b"BehindGainingTimeColor" {
                    	color(reader, tag.into_buf(), &mut settings.behind_gaining_time_color)
                    } else if tag.name() == b"BehindLosingTimeColor" {
                    	color(reader, tag.into_buf(), &mut settings.behind_losing_time_color)
                    } else if tag.name() == b"BestSegmentColor" {
                    	color(reader, tag.into_buf(), &mut settings.best_segment_color)
                    } else if tag.name() == b"NotRunningColor" {
                    	color(reader, tag.into_buf(), &mut settings.not_running_color)
                    } else if tag.name() == b"PausedColor" {
                    	color(reader, tag.into_buf(), &mut settings.paused_color)
                    } else {
                        end_tag(reader, tag.into_buf())
                    }
                })
            } else {
                end_tag(reader, tag.into_buf())
            }
        })
    })?;
    Ok(layout)
}


