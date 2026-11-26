# Remove yourself from PRs that already have enough approvals

## Intent

Today the tool only removes you from open PRs with "NOJIRA" in the title.
Add a second rule that applies to every open PR: if at least 2 other people
have already approved it, remove you as a reviewer. Once the PR has the
approvals it needs, your review is redundant and the PR is just noise in
your review list.

## Goals

- Every run checks all open PRs in the targeted repos against both rules:
  - **NOJIRA**: title contains "nojira" (case-insensitive). Behaviour unchanged.
  - **Approved**: at least 2 people *other than you* have approved, and you
    are on the reviewer list.
- If both rules match a PR, it's reported as NOJIRA (the existing rule wins).
- `--dry-run`, `--verbose` and the repo exclusion list apply to both rules.

## Non-goals

- A configurable threshold. It's fixed at 2.
- Per-repo rules or thresholds.
- Renaming the binary or the config file.

## Anti-goals

- Never drop a PR below 2 approvals by removing you. Your own approval is
  never counted, so removing you always leaves at least 2 approvals.

## Approaches considered

1. **Always on, in the same run** (chosen). One pass over all open PRs,
   checking both rules. It works with no flag to remember, which is the
   point of the tool. Downside: the first run after upgrading starts acting
   on non-NOJIRA PRs right away, so do a `--dry-run` first.
2. **Off unless you pass a flag** (`--approved`). Nothing changes by
   default, but you have to remember the flag every run.
3. **A separate subcommand.** Breaks the existing command line and adds
   structure the tool doesn't need.

## Design

- **Data:** the open-PR list endpoint
  (`/rest/api/1.0/projects/{p}/repos/{r}/pull-requests?state=OPEN`) already
  includes `reviewers` and `participants` arrays. Each entry has `user`,
  `role` and `approved`, which is the same shape as the existing
  `Participant` type. Add both arrays to `PullRequest` with
  `#[serde(default)]`. The approved rule then needs no extra API call per PR.
- **Counting approvals:** count unique user slugs, other than yours, with
  `approved == true` across `reviewers` and `participants`.
- **Am I a reviewer:** your slug appears in `reviewers`.
- **Decision:** a pure function takes a PR and your username and returns
  one of these, so it can be unit-tested without HTTP:
  - remove because of NOJIRA
  - remove because of approvals (with the count)
  - leave alone (with the reason)
- **NOJIRA path:** unchanged. It still uses the participants call per PR,
  and removes you if you're any kind of participant.
- **Removal:** reuses the existing `remove_reviewer` (DELETE participant).
- **Output:** removal lines say which rule fired. For example:
  - `Removed john.doe from PR #123 [NOJIRA]: title`
  - `Removed john.doe from PR #456 [2 approvals]: title`

  The summary splits "Removed from" into a NOJIRA count and an approved
  count. "Total PRs checked" now means all open PRs.
- **Docs:** update the README and the clap `about` text so they describe
  both rules.

## Calls Claude made (open to correction)

- The threshold is fixed at 2.
- Approvals count from anyone except you, whether they're listed as a
  reviewer or only as a participant.
- The approved rule only removes you if you're on the reviewer list. If you
  only commented, you're left alone.
- NOJIRA takes precedence when both rules match.
- If you've marked the PR **Needs work**, the approved rule leaves you on it.
  Removing you would silently withdraw your objection.
- The approved rule matches your username case-insensitively, everywhere it
  checks. If the comparison were inconsistent, your own approval could be
  counted as "someone else's".

## Unverified

- That this Bitbucket Server instance returns `reviewers[].approved` in the
  PR list response. The API docs say it does. The first `--dry-run
  --verbose` run will confirm it.

## Left to the builder

- Exact wording of verbose output for PRs that are left alone.
- How to restructure the loop in `main.rs`.
