# Add Ticket

Use the following guidelines to add github project items.
Add one ticket at a time.

## Available field-id and single-selection-option-id

Check @.cursor/references/available-field-id-and-single-selection-option-id.md

## Adding items/tickets

1. Output ticket description into `/plans/issues/active/<issue-title>.md`:
   - If it's a task, use @.cursor/references/task-template.md
   - If it's an epic, use @.cursor/references/prd-solution-design-template.md

2. Create github issue using github mcp.

3. Rename issue file into `/plans/issues/active/<gh-issue-id>-<issue-title>.md`

4. Add item to the project (last added github issue):
```
gh project item-add 2 --owner sifr-lang --url https://github.com/sifr-lang/sifr/issues/<issue-id>
```

5. Edit ticket fields (Status and Type):
* Status = Backlog
* Type = Epic or Task
```
gh project item-edit --project-id PVT_kwDOD7GYsM4BVyfV --id <item-id> --field-id <field-id> --single-select-option-id <single-select-option-id>
```
