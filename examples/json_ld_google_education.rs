//! JSON-LD google education.
use rs_json::ld::google::education::{Course, CourseList, Quiz};

fn main() {
    let quiz = Quiz::new()
        .name("Rust JSON-LD Fundamentals")
        .about("A short flashcard quiz about structured data.")
        .flashcard("What does @context usually point to?", "https://schema.org");

    let course_list = CourseList::new().course(
        "https://example.com/courses/rs-json",
        Course::new(
            "Structured data with rs-json",
            "Learn how to emit Google JSON-LD with typed builders.",
        )
        .provider_name("Example Academy")
        .image("https://example.com/course.png"),
    );

    println!("{}", quiz.into_document().to_json_string_pretty().unwrap());
    println!(
        "{}",
        course_list.into_document().to_json_string_pretty().unwrap()
    );
}
