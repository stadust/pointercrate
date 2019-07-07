<div class='panel fade js-scroll-anim' data-anim='fade'>

# Submitting Records

## `POST`{.verb} `/records/`

<div class='info-yellow'>
<b>Rate Limits:</b><br>
This endpoint is ratelimited at 3 successfully submitted records per 20 minutes per submitter and 20 records per hour globally. Further there are only 5 new IP adresses allowed to make submissions per hour (this limit is only relevant for your first time submitting a record).
</div>

<div class='info-yellow'>
<b>Access Restrictions:</b><br>
Unless you set `status` to `SUBMITTED` (or omit the field), access to this endpoint requires at least `LIST_HELPER` permissions.
</div>

Either adds a record directly to the list, or submits a record to the list mods for approval. The record must meet the demons requirement and the holder in question needn't be banned.

The `video` value, if provided, must meet the requirements specified [here](/documentation/#video).

### Request:

| Header       | Expected Value     | Optional |
| ------------ | ------------------ | -------- |
| Content-Type | `application/json` | false    |

| Field    | Type                                                  | Description                                                      | Optional |
| -------- | ----------------------------------------------------- | ---------------------------------------------------------------- | -------- |
| progress | integer                                               | The records progress                                             | false    |
| player   | string                                                | The name of the player holding the record                        | false    |
| demon    | string                                                | The name of the demon the record is made on                      | false    |
| video    | URL                                                   | The video of the record                                          | true     |
| status   | [RecordStatus](/documentation/objects/#record-status) | The status the newly record should have, defaults to `SUBMITTED` | true     |
| check    | boolean                                               | _deprecated_                                                     | true     |

### Response: `201 CREATED`

| Header       | Value                                            |
| ------------ | ------------------------------------------------ |
| Content-Type | `application/json`                               |
| Location     | The location of the newly created record         |
| ETag         | unsigned 64 bit hash of the newly created record |

| Field | Type                                     | Description                     |
| ----- | ---------------------------------------- | ------------------------------- |
| data  | [Record](/documentation/objects/#record) | The newly created record object |

### Response: `204 NO RESPONSE`

When `check` is set to `true`, and the record passed all internal validation, meaning it can be submitted.

### Errors:

| Status code | Error code | Description                                                             |
| ----------- | ---------- | ----------------------------------------------------------------------- |
| 403         | 40304      | You have been banned from submitting records                            |
| 404         | 40401      | The provided demon does not exist                                       |
| 422         | 42218      | The record holder is banned                                             |
| 422         | 42219      | The demon is on the legacy list                                         |
| 422         | 42215      | The record does not meat the demons requirement                         |
| 422         | 42220      | The demon is on the extended list but the record's progress isn't `100` |
| 422         | 42217      | The record has already been approved/rejected/submitted/approved        |

### Example request:

```json
POST /api/v1/records/
Accept: application/json
Content-Type: application/json

{
    "progress": 79,
    "player": "stadust",
    "demon": "Bloodlust"
}
```

</div>
