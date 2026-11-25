//! JSON-LD google video.
use rs_json::ld::google::media::{BroadcastEvent, Clip, VideoObject};

fn main() {
    let document = VideoObject::new(
        "Rust JSON-LD Builder Walkthrough",
        "A short walkthrough of the rs-json Google builder families.",
        "2026-04-08T10:00:00Z",
        "https://example.com/video-thumb.png",
    )
    .duration("PT8M42S")
    .embed_url("https://example.com/embed/video")
    .content_url("https://cdn.example.com/video.mp4")
    .clip(Clip::new(
        "Builder overview",
        0,
        120,
        "https://example.com/video?t=0",
    ))
    .broadcast_event(BroadcastEvent::new("2026-04-08T10:00:00Z").end_date("2026-04-08T10:09:00Z"))
    .region_allowed("AE")
    .family_friendly(true)
    .into_document();

    println!("{}", document.to_json_string_pretty().unwrap());
}
