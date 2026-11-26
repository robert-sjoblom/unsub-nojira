use std::collections::HashSet;

use crate::types::PullRequest;

/// Approvals from people other than you needed before you're removed.
pub const MIN_OTHER_APPROVALS: usize = 2;

#[derive(Debug, PartialEq, Eq)]
pub enum Decision {
    /// Title contains "nojira": handled by the NOJIRA rule.
    Nojira,
    /// Enough other people have approved and you're a reviewer: remove you.
    Approved { others: usize },
    /// You're not on the reviewer list: leave alone.
    NotReviewer,
    /// You've marked the PR Needs work: leave alone.
    NeedsWork,
    /// Not enough approvals from others yet: leave alone.
    TooFewApprovals { others: usize },
}

pub fn decide(pr: &PullRequest, username: &str) -> Decision {
    if pr.title.to_lowercase().contains("nojira") {
        return Decision::Nojira;
    }

    let Some(me) = pr.reviewers.iter().find(|r| is_me(&r.user.slug, username)) else {
        return Decision::NotReviewer;
    };

    if me.status.as_deref() == Some("NEEDS_WORK") {
        return Decision::NeedsWork;
    }

    let others = pr
        .reviewers
        .iter()
        .chain(&pr.participants)
        .filter(|p| p.approved && !is_me(&p.user.slug, username))
        .map(|p| p.user.slug.to_lowercase())
        .collect::<HashSet<_>>()
        .len();

    if others >= MIN_OTHER_APPROVALS {
        Decision::Approved { others }
    } else {
        Decision::TooFewApprovals { others }
    }
}

fn is_me(slug: &str, username: &str) -> bool {
    slug.eq_ignore_ascii_case(username)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Participant, PullRequest, User};

    fn person(slug: &str, approved: bool, status: Option<&str>) -> Participant {
        Participant {
            user: User {
                slug: slug.to_string(),
                name: slug.to_string(),
                display_name: slug.to_string(),
            },
            role: "REVIEWER".to_string(),
            approved,
            status: status.map(str::to_string),
        }
    }

    fn approved(slug: &str) -> Participant {
        person(slug, true, Some("APPROVED"))
    }

    fn unapproved(slug: &str) -> Participant {
        person(slug, false, Some("UNAPPROVED"))
    }

    fn pr(title: &str, reviewers: Vec<Participant>, participants: Vec<Participant>) -> PullRequest {
        PullRequest {
            id: 1,
            title: title.to_string(),
            reviewers,
            participants,
        }
    }

    #[test]
    fn nojira_title_is_nojira() {
        let pr = pr("NoJira: bump deps", vec![], vec![]);
        assert_eq!(decide(&pr, "me"), Decision::Nojira);
    }

    #[test]
    fn nojira_wins_over_approved() {
        let pr = pr(
            "nojira fix",
            vec![unapproved("me"), approved("a"), approved("b")],
            vec![],
        );
        assert_eq!(decide(&pr, "me"), Decision::Nojira);
    }

    #[test]
    fn two_other_approvals_removes() {
        let pr = pr(
            "Add feature",
            vec![unapproved("me"), approved("a"), approved("b")],
            vec![],
        );
        assert_eq!(decide(&pr, "me"), Decision::Approved { others: 2 });
    }

    #[test]
    fn three_other_approvals_removes() {
        let pr = pr(
            "Add feature",
            vec![
                unapproved("me"),
                approved("a"),
                approved("b"),
                approved("c"),
            ],
            vec![],
        );
        assert_eq!(decide(&pr, "me"), Decision::Approved { others: 3 });
    }

    #[test]
    fn own_approval_not_counted() {
        let pr = pr("Add feature", vec![approved("me"), approved("a")], vec![]);
        assert_eq!(decide(&pr, "me"), Decision::TooFewApprovals { others: 1 });
    }

    #[test]
    fn unapproved_reviewers_not_counted() {
        let pr = pr(
            "Add feature",
            vec![unapproved("me"), approved("a"), unapproved("b")],
            vec![],
        );
        assert_eq!(decide(&pr, "me"), Decision::TooFewApprovals { others: 1 });
    }

    #[test]
    fn not_reviewer_is_left_alone() {
        let pr = pr(
            "Add feature",
            vec![approved("a"), approved("b")],
            vec![unapproved("me")],
        );
        assert_eq!(decide(&pr, "me"), Decision::NotReviewer);
    }

    #[test]
    fn participant_approval_counts() {
        let pr = pr(
            "Add feature",
            vec![unapproved("me"), approved("a")],
            vec![approved("b")],
        );
        assert_eq!(decide(&pr, "me"), Decision::Approved { others: 2 });
    }

    #[test]
    fn same_user_in_both_lists_counts_once() {
        let pr = pr(
            "Add feature",
            vec![unapproved("me"), approved("a")],
            vec![approved("a")],
        );
        assert_eq!(decide(&pr, "me"), Decision::TooFewApprovals { others: 1 });
    }

    #[test]
    fn needs_work_from_me_is_left_alone() {
        let pr = pr(
            "Add feature",
            vec![
                person("me", false, Some("NEEDS_WORK")),
                approved("a"),
                approved("b"),
            ],
            vec![],
        );
        assert_eq!(decide(&pr, "me"), Decision::NeedsWork);
    }

    #[test]
    fn username_case_does_not_make_own_approval_count() {
        let pr = pr("Add feature", vec![approved("me"), approved("a")], vec![]);
        assert_eq!(decide(&pr, "Me"), Decision::TooFewApprovals { others: 1 });
    }

    #[test]
    fn pr_without_reviewer_arrays_parses_and_is_left_alone() {
        let pr: PullRequest = serde_json::from_str(r#"{"id":1,"title":"x"}"#).unwrap();
        assert_eq!(decide(&pr, "me"), Decision::NotReviewer);
    }

    #[test]
    fn reviewer_entry_without_status_parses() {
        let pr: PullRequest = serde_json::from_str(
            r#"{"id":1,"title":"x","reviewers":[{"user":{"slug":"me","name":"me","displayName":"Me"},"role":"REVIEWER","approved":false}]}"#,
        )
        .unwrap();
        assert_eq!(pr.reviewers[0].status, None);
    }

    #[test]
    fn sparse_reviewer_entries_parse_and_count() {
        // Entries missing name/displayName/role/approved must not fail the
        // whole PR list (and with it the NOJIRA rule for that repo).
        let pr: PullRequest = serde_json::from_str(
            r#"{"id":1,"title":"x","reviewers":[{"user":{"slug":"me"}},{"user":{"slug":"a"},"approved":true}],"participants":[{"user":{"slug":"b"},"approved":true}]}"#,
        )
        .unwrap();
        assert_eq!(decide(&pr, "me"), Decision::Approved { others: 2 });
    }
}
