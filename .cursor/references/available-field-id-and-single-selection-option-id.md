# Available field-id and single-selection-option-id

These are the available field-id and single-selection-option-id:
```
{
  "field_id": "PVTSSF_lAHOAKAfcc4BPKkLzg9p4e8",
  "field_name": "Status",
  "options": [
    {
      "id": "f75ad846",
      "name": "Backlog"
    },
    {
      "id": "244b4188",
      "name": "Ready"
    },
    {
      "id": "47fc9ee4",
      "name": "In Progress"
    },
    {
      "id": "2d2c3b25",
      "name": "Review"
    },
    {
      "id": "98236657",
      "name": "Done"
    }
  ]
}
```

If not found, use the following command to get all the single select fields and their options:

```
gh project field-list 2 --owner yaseralnajjar --format json | jq '.fields[] | select(.type == "ProjectV2SingleSelectField") | {field_id: .id, field_name: .name, options: .options}'
```