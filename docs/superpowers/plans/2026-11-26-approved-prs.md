# Approved-PRs Rule Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** On every open PR (not just NOJIRA ones), remove the user as a reviewer once at least 2 *other* people have approved.

**Architecture:** A new pure module `src/rules.rs` decides, for each PR, whether to remove the user and why. It uses the `reviewers`/`participants` arrays that the open-PR list endpoint already returns. `main.rs` loops over all open PRs, calls the decision function, and runs the NOJIRA path (unchanged) or the new approved path.

**Tech Stack:** Rust 2024, serde, tokio, reqwest, clap. Tests are inline `#[cfg(test)]` modules run with `cargo test`.

**Spec:** `docs/superpowers/specs/2026-11-26-approved-prs-design.md`

## Global Constraints

- Threshold is fixed: `MIN_OTHER_APPROVALS = 2`. No CLI flag.
- Only approvals from users other than the CLI `username` count. Never count the user's own approval.
- The approved rule only removes the user if they are in `reviewers`.
- The approved rule leaves the user alone if their own reviewer `status` is `"NEEDS_WORK"`.
- Username comparisons in `rules.rs` are ASCII case-insensitive (`eq_ignore_ascii_case`), used consistently.
- If a title contains "nojira" (case-insensitive), the PR goes to the NOJIRA path, even when the approved rule would also match.
- NOJIRA path behaviour is unchanged: it calls the participants endpoint and removes the user if they are any participant.
- No new dependencies. Don't touch the pre-existing `reqwest-middleware` unused-dependency warning.
- Work on a branch, not `master`: `git switch -c approved-prs` before Task 1.

## Review Focus

1. **PR JSON without `reviewers`/`participants`** (or entries without `status`): should parse fine and lead to "leave alone", not a failed repo. Test: `pr_without_reviewer_arrays_parses_and_is_left_alone` (Task 1).
2. **Username typed in a different case from the slug** (`John.Doe` vs `john.doe`): the user's own approval must not count as another person's. Test: `username_case_does_not_make_own_approval_count` (Task 1).
3. **Same person in both `reviewers` and `participants`**: counted once. Test: `same_user_in_both_lists_counts_once` (Task 1).
4. **User marked Needs work while 2+ others approved**: left alone. Test: `needs_work_from_me_is_left_alone` (Task 1).
5. **Approval from a non-reviewer participant**: counts toward the 2. Test: `participant_approval_counts` (Task 1).

---

### Task 1: Decision rules and PR reviewer data

**Files:**
- Create: `src/rules.rs` (decision logic plus `#[cfg(test)] mod tests`)
- Modify: `src/types.rs` (`PullRequest`, `Participant`)
- Modify: `src/main.rs:1-4` (add `mod rules;`)

**Interfaces:**
- Consumes: `types::{PullRequest, Participant, User}`
- Produces:
  - `pub const MIN_OTHER_APPROVALS: usize = 2;`
  - `pub enum Decision { Nojira, Approved { others: usize }, NotReviewer, NeedsWork, TooFewApprovals { others: usize } }` deriving `Debug, PartialEq, Eq`
  - `pub fn decide(pr: &PullRequest, username: &str) -> Decision`
  - `PullRequest` gains `#[serde(default)] pub reviewers: Vec<Participant>` and `#[serde(default)] pub participants: Vec<Participant>`
  - `Participant` gains `#[serde(default)] pub status: Option<String>`. Drop the `#[allow(dead_code)]` on `approved` (it's used now) and keep it on `role`.

- [ ] **Step 1: Write the failing tests in `src/rules.rs`**

Add a test helper `fn person(slug: &str, approved: bool, status: Option<&str>) -> Participant`, and `fn pr(title: &str, reviewers: Vec<Participant>, participants: Vec<Participant>) -> PullRequest`. Username in all tests is `"me"` unless stated.

| Test | Setup | Assertion |
|---|---|---|
| `nojira_title_is_nojira` | title `"NoJira: bump deps"`, no reviewers | `decide(..) == Decision::Nojira` |
| `nojira_wins_over_approved` | title `"nojira fix"`, reviewers `me`, `a`✓, `b`✓ | `== Decision::Nojira` |
| `two_other_approvals_removes` | reviewers `me`, `a`✓, `b`✓ | `== Decision::Approved { others: 2 }` |
| `three_other_approvals_removes` | reviewers `me`, `a`✓, `b`✓, `c`✓ | `== Decision::Approved { others: 3 }` |
| `own_approval_not_counted` | reviewers `me`✓, `a`✓ | `== Decision::TooFewApprovals { others: 1 }` |
| `unapproved_reviewers_not_counted` | reviewers `me`, `a`✓, `b`✗ | `== Decision::TooFewApprovals { others: 1 }` |
| `not_reviewer_is_left_alone` | reviewers `a`✓, `b`✓; participants `me` | `== Decision::NotReviewer` |
| `participant_approval_counts` | reviewers `me`, `a`✓; participants `b`✓ | `== Decision::Approved { others: 2 }` |
| `same_user_in_both_lists_counts_once` | reviewers `me`, `a`✓; participants `a`✓ | `== Decision::TooFewApprovals { others: 1 }` |
| `needs_work_from_me_is_left_alone` | reviewers `me` status `"NEEDS_WORK"`, `a`✓, `b`✓ | `== Decision::NeedsWork` |
| `username_case_does_not_make_own_approval_count` | username `"Me"`; reviewers `me`✓, `a`✓ | `== Decision::TooFewApprovals { others: 1 }` |
| `pr_without_reviewer_arrays_parses_and_is_left_alone` | `serde_json::from_str::<PullRequest>(r#"{"id":1,"title":"x"}"#).unwrap()` | `== Decision::NotReviewer` |
| `reviewer_entry_without_status_parses` | deserialize `{"id":1,"title":"x","reviewers":[{"user":{"slug":"me","name":"me","displayName":"Me"},"role":"REVIEWER","approved":false}]}` | parses; `reviewers[0].status == None` |

- [ ] **Step 2: Run the tests and check they fail**

Run: `cargo test rules`
Expected: compile errors (`decide`, `Decision`, `reviewers` not defined).

- [ ] **Step 3: Implement `types.rs` changes and `decide` in `src/rules.rs`**

Order in `decide`: NOJIRA title check → is `username` in `reviewers` (else `NotReviewer`) → is the user's reviewer `status == Some("NEEDS_WORK")` (then `NeedsWork`) → count unique slugs (lowercased into a `HashSet`) across `reviewers` and `participants` with `approved == true`, excluding `username` → `Approved { others }` if `>= MIN_OTHER_APPROVALS` else `TooFewApprovals { others }`. Use one private `fn is_me(slug: &str, username: &str) -> bool` (via `eq_ignore_ascii_case`) for every comparison. Register `mod rules;` in `main.rs`.

- [ ] **Step 4: Run the tests and check they pass**

Run: `cargo test`
Expected: 13 passed, 0 failed. `cargo clippy --all-targets` shows no warnings except the existing `reqwest-middleware` manifest warning. An unused-code warning for `decide` is OK until Task 2.

- [ ] **Step 5: Commit**

```bash
git add src/rules.rs src/types.rs src/main.rs
git commit -m "Add approval-count decision rules for open PRs"
```

---

### Task 2: Apply both rules in the main loop; update docs

**Files:**
- Modify: `src/main.rs:63-148` (PR loop and summary)
- Modify: `src/cli.rs:5` (`about` text)
- Modify: `README.md`

**Interfaces:**
- Consumes: `rules::{decide, Decision, MIN_OTHER_APPROVALS}` from Task 1; existing `BitbucketClient::{get_pr_participants, remove_reviewer}`.
- Produces: nothing used by later tasks.

- [ ] **Step 1: Restructure the loop in `main.rs`**

- Remove the NOJIRA pre-filter and its two verbose lines ("Found N PRs with 'NOJIRA'…", "No PRs found…"). Loop over all `prs` and set `total_prs_checked += prs.len()`.
- Per PR, `match decide(pr, &args.username)`:
  - `Nojira`: existing participants call and "not a participant" skip, then remove with tag `NOJIRA`.
  - `Approved { others }`: remove with tag `format!("{others} approvals")`.
  - `NotReviewer` / `NeedsWork` / `TooFewApprovals { others }`: verbose-only line (`"  Skipping: {username} is not a reviewer"`, `"  Skipping: {username} marked Needs work"`, `"  Skipping: only {others} other approval(s), need {MIN_OTHER_APPROVALS}"`). Counts as left alone.
- Pull the dry-run/remove/print logic into `async fn remove_from_pr(client: &BitbucketClient, args: &Cli, repo_slug: &str, pr: &PullRequest, tag: &str) -> bool`. It returns `true` if removed, or would be removed in dry-run.
- Exact output lines:
  - `"[DRY RUN] Would remove {user} from PR #{id} [{tag}]: {title}"`
  - `"Removed {user} from PR #{id} [{tag}]: {title}"`
- Summary replaces "Removed from" and "Skipped" with:
  ```
    Total PRs checked: N
    Removed (NOJIRA): N
    Removed (approved): N
    Left alone: N
  ```
  Keep "Repositories processed" as it is today.

- [ ] **Step 2: Update `cli.rs` about text**

`about = "Remove yourself from Bitbucket PRs with 'NOJIRA' in the title, or that already have 2+ approvals from others"`

- [ ] **Step 3: Update `README.md`**

- Intro and Features: describe both rules.
- How It Works: step 3 lists both rules, including the "others only / must be reviewer / Needs work left alone" points.
- Output examples: use the new `[NOJIRA]` and `[2 approvals]` tags and the new summary lines.
- Add a note under Usage: "Run with `--dry-run` first after upgrading — the approved rule acts on all open PRs, not just NOJIRA ones."

- [ ] **Step 4: Verify**

Run: `cargo build && cargo test && cargo clippy --all-targets`
Expected: builds; 13 passed; no warnings except the pre-existing `reqwest-middleware` manifest warning.
Run: `cargo run -- --help`
Expected: shows the new about text.

- [ ] **Step 5: Commit**

```bash
git add src/main.rs src/cli.rs README.md
git commit -m "Remove yourself from PRs with 2+ approvals from others"
```

---

**Live check (user, not the implementer):** `unsub-nojira <user> <project> --dry-run --verbose` against Bitbucket. This confirms the instance returns `reviewers[].approved`/`status` in the PR list.
