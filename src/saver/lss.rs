use std::io::{Write, Result};
use std::fmt::{self, Display, Formatter};
use chrono::{DateTime, UTC};
use sxd_document::Package;
use sxd_document::dom::{Document, Element};
use sxd_document::writer::format_document;
use {Run, Time};

struct CData<D: Display>(D);

impl<D: Display> Display for CData<D> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "") // TODO Fix CData
        // write!(fmt, "<![CDATA[{}]]>", self.0)
    }
}

fn fmt_bool(value: bool) -> &'static str {
    if value { "True" } else { "False" }
}

fn fmt_date(date: DateTime<UTC>) -> String {
    date.format("%m/%d/%Y %T").to_string()
}

fn time<'a>(document: &Document<'a>, element_name: &str, time: Time) -> Element<'a> {
    let element = document.create_element(element_name);

    if let Some(time) = time.real_time {
        add_element(document, &element, "RealTime", &time.to_string());
    }

    if let Some(time) = time.game_time {
        add_element(document, &element, "GameTime", &time.to_string());
    }

    element
}

fn to_element<'a>(document: &Document<'a>, element_name: &str, value: &str) -> Element<'a> {
    let element = document.create_element(element_name);
    let value = document.create_text(value);
    element.append_child(value);
    element
}

fn add_element(document: &Document, parent: &Element, element_name: &str, value: &str) {
    parent.append_child(to_element(document, element_name, value));
}

pub fn save<W: Write>(run: &Run, mut writer: W) -> Result<()> {
    let package = Package::new();
    let doc = &package.as_document();

    let parent = doc.create_element("Run");
    parent.set_attribute_value("version", "1.6.0");
    doc.root().append_child(parent);

    add_element(doc,
                &parent,
                "GameIcon",
                &CData(run.game_icon()).to_string());
    add_element(doc, &parent, "GameName", run.game_name());
    add_element(doc, &parent, "CategoryName", run.category_name());

    let metadata = doc.create_element("Metadata");

    let run_element = doc.create_element("Run");
    run_element.set_attribute_value("id", run.metadata().run_id());
    metadata.append_child(run_element);

    let platform = to_element(doc, "Platform", run.metadata().platform_name());
    platform.set_attribute_value("usesEmulator", fmt_bool(run.metadata().uses_emulator()));
    metadata.append_child(platform);

    add_element(doc, &metadata, "Region", run.metadata().region_name());

    let variables = doc.create_element("Variables");
    for (name, value) in run.metadata().variables() {
        let variable = to_element(doc, "Variable", name);
        variable.set_attribute_value("name", value);
        variables.append_child(variable);
    }
    metadata.append_child(variables);
    parent.append_child(metadata);

    add_element(doc, &parent, "Offset", &run.offset().to_string());
    add_element(doc,
                &parent,
                "AttemptCount",
                &run.attempt_count().to_string());

    let attempt_history = doc.create_element("AttemptHistory");
    for attempt in run.attempt_history() {
        let element = time(doc, "Attempt", attempt.time());
        element.set_attribute_value("id", &attempt.index().to_string());

        if let Some(started) = attempt.started() {
            element.set_attribute_value("started", &fmt_date(started.time));
            element.set_attribute_value("isStartedSynced",
                                        fmt_bool(started.synced_with_atomic_clock));
        }

        if let Some(ended) = attempt.ended() {
            element.set_attribute_value("ended", &fmt_date(ended.time));
            element.set_attribute_value("isEndedSynced", fmt_bool(ended.synced_with_atomic_clock));
        }

        attempt_history.append_child(element);
    }
    parent.append_child(attempt_history);

    let segments_element = doc.create_element("Segments");
    for segment in run.segments() {
        let segment_element = doc.create_element("Segment");

        add_element(doc, &segment_element, "Name", segment.name());
        add_element(doc,
                    &segment_element,
                    "Icon",
                    &CData(segment.icon()).to_string());

        let split_times = doc.create_element("SplitTimes");
        for comparison in run.custom_comparisons() {
            let split_time = time(doc,
                                  "SplitTime",
                                  segment.comparison(comparison).unwrap_or_default());
            split_time.set_attribute_value("name", comparison);
            split_times.append_child(split_time);
        }
        segment_element.append_child(split_times);

        segment_element.append_child(time(doc, "BestSegmentTime", segment.best_segment_time()));

        let history = doc.create_element("SegmentHistory");
        for (index, &history_time) in segment.segment_history() {
            let element = time(doc, "Time", history_time);
            element.set_attribute_value("id", &index.to_string());
            history.append_child(element);
        }
        segment_element.append_child(history);

        segments_element.append_child(segment_element);
    }
    parent.append_child(segments_element);

    let auto_splitter_settings = doc.create_element("AutoSplitterSettings");
    // TODO Add Auto Splitter Settings
    parent.append_child(auto_splitter_settings);

    format_document(doc, &mut writer)
}

#[cfg(test)]
mod test {
    use {Run, Segment, Timer};
    use super::*;

    #[test]
    fn test() {
        let segments = vec![Segment::new("Hi"), Segment::new("okok")];
        let mut run = Run::new(segments);
        run.set_game_name("Wind Waker");
        let mut timer = Timer::new(run);
        timer.start();
        timer.split();
        timer.split();
        timer.reset(true);

        let mut buf = Vec::new();
        save(timer.run(), &mut buf).unwrap();
        let xml = ::std::str::from_utf8(&buf).unwrap();
        println!("{}", xml);
        panic!();
    }
}
