# Refinement

Use the following guidelines to manage github project Backlog items.

## Get all project items

To get all project items from GitHub, use this command:
```
gh project item-list 2 --owner yaseralnajjar
```

## Available field-id and single-selection-option-id

Check @.cursor/references/available-field-id-and-single-selection-option-id.md

## Refinement of items/tickets

Edit ticket fields (Priority and Status):

1. Assess the `Priority` of each ticket based on effort vs value matrix in the backlog then set it.

2. Change highly prioritized tickets `Status` into `Ready`:
```
gh project item-edit --project-id PVT_kwHOAKAfcc4BPKkL --id <item-id> --field-id <field-id> --single-select-option-id <single-select-option-id>
```
