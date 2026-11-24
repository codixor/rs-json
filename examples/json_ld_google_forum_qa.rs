use rs_json::ld::google::forum_qa::{
    Answer, DiscussionForumPosting, ForumComment, QAPage, Question,
};

fn main() {
    let qa = QAPage::new(
        Question::new(
            "How do I expose JSON-LD in rs-json?",
            "What is the cleanest way to emit Google JSON-LD from rs-json builders?",
        )
        .author_name("Example User")
        .accepted_answer(
            Answer::new("Use the typed builders under rs_json::ld::google.")
                .author_name("Maintainer"),
        ),
    );

    let forum = DiscussionForumPosting::new(
        "How are you structuring JSON-LD families?",
        "We split them into clear Google-oriented families.",
        "Example Maintainer",
        "2026-04-08T11:00:00Z",
    )
    .comment(
        ForumComment::new(
            "This makes the surface much easier to discover.",
            "Example Reader",
            "2026-04-08T11:30:00Z",
        )
        .upvote_count(4),
    );

    println!("{}", qa.into_document().to_json_string_pretty().unwrap());
    println!("{}", forum.into_document().to_json_string_pretty().unwrap());
}
