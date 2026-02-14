# Review PR

Use the following guidelines to review PRs for github project In review items.
Review one PR at a time.

Review comments are preferred to be github PR inline comments, but in cases where it's better to be a review comment, use github PR review comments.

## Review Criteria

Review PR should be focused on the following (review these one step at a time):

* Basic checks:
- Is the PR description clear (what, why, how)?
- Is the PR solving the problem it's intended to solve?

* Issues checks:
- Are there unnecessary complexities that can be simplified?
- Are there potential logic bugs, race conditions, or security issues?
- Is the code well tested?
- Would it cover all edge cases?
- Could this change introduce performance bottlenecks?

* Style checks:
- Is there inline documentation/comments where the intent isn't obvious?
- Is the PR following the code style?
- Is the PR following the architecture?
- Is the PR following the brand guidelines?
- Is the PR following best practices?
- Is there any unnecessary code that can be removed?

## Get all project items

To get all project items from GitHub, use this command:
```
gh project item-list 2 --owner yaseralnajjar --format json
```

## Reviewing PRs

Edit ticket fields (Status):

1. Pickup the highest prioritized ticket in the `Review` column.

2. Read the github issue details using github mcp.

3. Find the relevant PR `<github-issue-id>-<github-issue-title>` (stop execution if the PR doesn't exist).

4. Check already existing comments to avoid writing redundant comments using github mcp.

5. Review the PR and check the following (do not use start review functionality):
* Solving the problem it's intended to solve.
* Would not make a problem given @.cursor/.rules/architecture-overview.mdc
* Following @.cursor/.rules/code-style.mdc
* Other considerations...

6. Reply to already existing PR comments (if it adds value or if you have a different opinion):
```
gh api \
  --method POST \
  -H "Accept: application/vnd.github+json" \
  /repos/{owner}/{repo}/pulls/{pull_number}/comments/{comment_id}/replies /
  -f body='Great stuff!'
```

7. Write review comments (if it adds value or if you have a different opinion) using gh api:
* start_line: first line to start the comment
* line: last line to highlight for this comment
```
gh api \
  --method POST \
  -H "Accept: application/vnd.github+json" \
  /repos/{owner}/{repo}/pulls/{pull_number}/comments \
  --raw-field body="$(cat <<'EOF'
Great stuff!
This is a multiline comment.
EOF
)" \
  -f commit_id='2f4942a9732838932f36d64c7a422225c68f96e6' \
  -f path='dir1/dir2/file.py' \
  -F start_line=1 \
  -f start_side='RIGHT' \
  -F line=3 \
  -f side='RIGHT'
```
