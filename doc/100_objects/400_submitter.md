<div class='panel fade js-scroll-anim' data-anim='fade'>

# Submitter objects{id=submitter}

Everyone who submits a record gets assigned an incremental submitter id, internally used to keep track of who has been banned from submitting records. The following invariant holds true for any submitter object:

- If the submitter is banned, he cannot submit new records and all his previously submitted records are either marked as `approved` or `rejected`

## Minimal/Listed Form

The short form is returned by the [`GET /submitters/`](/documentation/submitter/#submitter-listing) pagination endpoint.

| Field  | Type    | Description                                      |
| ------ | ------- | ------------------------------------------------ |
| id     | int     | The submitter's ID                               |
| banned | boolean | Value indicating whether the submitter is banned |

## Full Form

The long form is returned by the [`GET /submitters/`](/documentation/submitter/#submitter-retrieval) endpoint.

| Field   | Type                    | Description                                      |
| ------- | ----------------------- | ------------------------------------------------ |
| id      | int                     | The submitter's ID                               |
| banned  | boolean                 | Value indicating whether the submitter is banned |
| records | List[[Record](#record)] | A list of records this submitter has submitted   |

## Example objects

### Short Form

```json
{
  "banned": false,
  "id": 2
}
```

### Long Form

```json
{
  "banned": true,
  "id": 7,
  "records": [
    {
      "id": 1,
      "progress": 100,
      "demon": {
        "name": "Cadrega City",
        "position": 1
      },
      "status": "rejected",
      "player": {
        "name": "Aquatias",
        "id": 3424,
        "banned": false
      }
    }
  ]
}
```

</div>
