<div class='panel fade js-scroll-anim' data-anim='fade'>

# Record objects{id=record}

Each record on the list is represented by a `Record` object. The following assumptions can be made about these:

* The `progress` value lies within `demon.requirement` and `100`
* Every `video` value is unique
* Every combination of `demon`, `player` and `status` values is unique
* Every `video` value is in one of the formats listed [here](/documentation/#video)

| Field     | Type                           | Description                                             |
| --------- | ------------------------------ | ------------------------------------------------------- |
| id        | integer                        | The record's id                                         |
| progress  | integer                        | The progress achieved by the record's holder            |
| video     | URL                            | The record's video. Can be `null`                       |
| status    | [RecordStatus](#record-status) | The record's status.                                    |
| player    | [Player](#player)              | The record holder                                       |
| demon     | [Demon](#demon)                | The demon the record was made on                        |
| submitter | [Submitter](#submitter)        | The internal ID of the person that submitted the record |

## Enum RecordStatus{id=record-status}

| Value       | Description                                               |
| ----------- | --------------------------------------------------------- |
| `APPROVED`  | The record has been approved and is displayed on the list |
| `REJECTED`  | The record has been rejected                              |
| `SUBMITTED` | The record has been submitted and is awaiting review      |

### Example object:

```
{
  "demon": {
    "name": "Cadrega City",
    "position": 34,
    "state": "MAIN"
  },
  "id": 2,
  "player": {
    "banned": false,
    "id": 5,
    "name": "AeonAir"
  },
  "progress": 100,
  "status": "APPROVED",
  "submitter": {
    "banned": false,
    "id": 2
  },
  "video": null
}
```

</div>
