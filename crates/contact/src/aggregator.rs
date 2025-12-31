use crate::Subject;

#[evento::aggregator]
pub enum Contact {
    FormSubmitted {
        name: String,
        email: String,
        subject: Subject,
        message: String,
        to: String,
    },
    MarkedReadAndReply,
    Resolved,
    Reopened,
}
