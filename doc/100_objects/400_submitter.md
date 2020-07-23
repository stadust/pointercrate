<div class='panel fade js-scroll-anim' data-anim='fade'>

# Submitter objects{id=submitter}

Everyone who submits a record gets assigned an incremental submitter id, internally used to keep track of who has been banned from submitting records. The following invariant holds true for any submitter object:

- If the submitter is banned, he cannot submit new records and all his previously submitted records are either marked as `approved` or `rejected`

| Field  | Type    | Description                                      |
| ------ | ------- | ------------------------------------------------ |
| id     | int     | The submitter's ID                               |
| banned | boolean | Value indicating whether the submitter is banned |

## Example object

```json
{
  "banned": true,
  "id": 7
}
```

</div>
