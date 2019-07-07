<div class='panel fade js-scroll-anim' data-anim='fade'>

# Record objects{id=record}

Each record on the list is represented by a `Record` object. The following invariants hold true for all player objects

- The `progress` value lies within `demon.requirement` and `100`
- Every `video` value is unique
- Every combination of `demon`, `player` and `status` values is unique
- Every `video` value is in one of the formats listed [here](/documentation/#video), or `null`

The object only contains the submitter information if the object has been requested with sufficient permissions.
Requests without `ExtendedAccess` permissions can only retrieve approved records

## Embedded Form

The embedded form of record objects is returned if a record object is part of another object

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

## Long Form

The long form of record objects is returned by [`GET /records/`](/documentation/records/#get-records) and [`GET /records/{record_id}`](/documentation/records/#record-retrieval). There is no short form for the pagination endpoint

| Field     | Type                           | Description                                             |
| --------- | ------------------------------ | ------------------------------------------------------- |
| id        | integer                        | The record's id                                         |
| progress  | integer                        | The progress achieved by the record's holder            |
| video     | URL?                           | The record's video.                                     |
| status    | [RecordStatus](#record-status) | The record's status.                                    |
| player    | [Player](#player)              | The record holder                                       |
| demon     | [Demon](#demon)                | The demon the record was made on                        |
| submitter | integer?                       | The internal ID of the person that submitted the record |

## Enum RecordStatus{id=record-status}

| Value       | Description                                               |
| ----------- | --------------------------------------------------------- |
| `approved`  | The record has been approved and is displayed on the list |
| `rejected`  | The record has been rejected                              |
| `submitted` | The record has been submitted and is awaiting review      |

## Example objects

### Embedded form

```json
{
  "id": 1,
  "progress": 100,
  "demon": "Cadrega City",
  "status": "approved",
  "player": "Aquatias"
}
```

### Long form

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
  "video": null
}
```

</div>
