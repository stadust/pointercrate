<div class='panel fade js-scroll-anim' data-anim='fade'>

# Record objects{id=record}

Each record on the list is represented by a `Record` object. The following invariants hold true for all player objects

- The `progress` value lies within `demon.requirement` and `100`
- Every `video` value is unique
- Every combination of `demon`, `player` and `status` values is unique
- Every `video` value is in one of the formats listed [here](/documentation/#video), or `null`

The object only contains the submitter information if the requestee has `ListModerator` permissions. The object only contains the notes if the requestee has `ListHelper` permissions.
Requests without `ExtendedAccess` permissions can only retrieve approved records.

## Minimal Form

The minimal (formerly called embedded form) form of record objects is returned if a record object is part of another object.

| Field    | Type                           | Description                                |
| -------- | ------------------------------ | ------------------------------------------ |
| id       | integer                        | The record's id                            |
| progress | integer                        | The progress achieved by the record holder |
| status   | [RecordStatus](#record-status) | The record's status.                       |
| video    | URL?                           | The record's video.                        |

Depending on the context the object is returned in, one (or both) of the following fields will be present:

| Field  | Type              | Description                      |
| ------ | ----------------- | -------------------------------- |
| player | [Player](#player) | The record holder                |
| demon  | [Demon](#demon)   | The demon the record was made on |

## Listed Form

The listed form (formerly called short form) of record objects is returned by [`GET /records/`](/documentation/records/#get-records).

| Field     | Type                           | Description                                |
| --------- | ------------------------------ | ------------------------------------------ |
| id        | integer                        | The record's id                            |
| progress  | integer                        | The progress achieved by the record holder |
| status    | [RecordStatus](#record-status) | The record's status.                       |
| video     | URL?                           | The record's video.                        |
| player    | [Player](#player)              | The record holder                          |
| demon     | [Demon](#demon)                | The demon the record was made on           |

## Full Form

The full (formerly called long form) form of record objects is returned by [`GET /records/{record_id}`](/documentation/records/#record-retrieval). The `notes` field is always `null` if you do not have at least `ListHelper` permissions.

| Field     | Type                           | Description                                                  |
| --------- | ------------------------------ | ------------------------------------------------------------ |
| id        | integer                        | The record's id                                              |
| progress  | integer                        | The progress achieved by the record's holder                 |
| video     | URL?                           | The record's video.                                          |
| status    | [RecordStatus](#record-status) | The record's status.                                         |
| notes     | List[[RecordNote](#record-note)]?                        | Notes on the record                                          |
| player    | [Player](#player)              | The record holder                                            |
| demon     | [Demon](#demon)                | The demon the record was made on                             |
| submitter | [Submitter](#submitter)?       | The person that submitted the record, as an submitter object |

## Enum RecordStatus{id=record-status}

| Value       | Description                                               |
| ----------- | --------------------------------------------------------- |
| `approved`  | The record has been approved and is displayed on the list |
| `rejected`  | The record has been rejected                              |
| `submitted` | The record has been submitted and is awaiting review      |
| `under consideration`| The record is awaiting more thorough review      |

## Example objects

### Minimal form

Here with an embedded demon object:

```json
{
  "id": 1,
  "progress": 100,
  "demon": {
    "name": "Cadrega City",
    "position": 1
  },
  "status": "approved",
  "player": "Aquatias"
}
```

### Full form

Here without an embedded submitter object

```json
{
  "demon": {
    "name": "Cadrega City",
    "position": 34
  },
  "id": 2,
  "player": {
    "banned": false,
    "id": 5,
    "name": "AeonAir"
  },
  "progress": 100,
  "status": "approved",
  "submitter": null,
  "video": null,
  "notes":[]
}
```

</div>
