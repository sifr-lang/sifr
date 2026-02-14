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
{
  "field_id": "PVTSSF_lAHOAKAfcc4BPKkLzg9p5HQ",
  "field_name": "Priority",
  "options": [
    {
      "id": "6dda94fa",
      "name": "P0"
    },
    {
      "id": "2b598731",
      "name": "P1"
    },
    {
      "id": "1925f16b",
      "name": "P2"
    }
  ]
}
{
  "field_id": "PVTSSF_lAHOAKAfcc4BPKkLzg9p5Ik",
  "field_name": "Size",
  "options": [
    {
      "id": "972337ac",
      "name": "XS"
    },
    {
      "id": "24189a8f",
      "name": "S"
    },
    {
      "id": "bd5043f1",
      "name": "M"
    },
    {
      "id": "bc57cbe0",
      "name": "L"
    },
    {
      "id": "05f38345",
      "name": "XL"
    }
  ]
}
{
  "field_id": "PVTSSF_lAHOAKAfcc4BPKkLzg9p5JU",
  "field_name": "Type",
  "options": [
    {
      "id": "fb75e245",
      "name": "Epic"
    },
    {
      "id": "a2752bac",
      "name": "Task"
    }
  ]
}
```

If not found, use the following command to get all the single select fields and their options:

```
gh project field-list 2 --owner yaseralnajjar --format json | jq '.fields[] | select(.type == "ProjectV2SingleSelectField") | {field_id: .id, field_name: .name, options: .options}'
```
