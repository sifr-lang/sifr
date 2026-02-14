# Work on Ticket

Use the following guidelines to create PRs for github project Ready items.
Work on one ticket at a time.

Do not commit changes, and stop execution if there are uncommitted changes.

## Get all project items

To get all project items from GitHub, use this command:
```
gh project item-list 2 --owner yaseralnajjar --format json
```

## Available field-id and single-selection-option-id

Check @.cursor/references/available-field-id-and-single-selection-option-id.md

## Creating PRs

1. Pickup the highest prioritized ticket in the `Ready` column.

2. Read the github issue details using github mcp.

3. Create a git branch `<github-issue-id>-<github-issue-title>` 
   * stop execution if the branch is existing and ask whether it needs to checkout the existing branch).
   * stop execution if the current branch has uncommitted changes (and ask the user if they to git stash).
   * stop execution if the current branch isn't `main` branch and ask the user if they want to checkout `main`
   * stop execution if the branch they are creating a new branch from isn't up-to-date with remote branch and ask the user if they want to `git pull` before creating a new branch.
   * stop execution if `git pull` causes any git conflicts and ask the user to go and solve the conflicts.

4. Make changes accordingly.

5. Run tests locally and make sure all tests are passing.

6. Create a PR in github with title `<github-issue-id>-<github-issue-title>` using github mcp. Use this template @.cursor/references/pr-template.md

7. Update ticket `Status` into `In review`:
```
gh project item-edit --project-id PVT_kwHOAKAfcc4BPKkL --id <item-id> --field-id <field-id> --single-select-option-id <single-select-option-id>
```
