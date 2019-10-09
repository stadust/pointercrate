<div class='panel fade js-scroll-anim' data-anim='fade'>

# Modifying a record{id=patch-record}

## `PATCH`{.verb} `/records/` `record_id`{.param} `/`

<div class='info-yellow'>
<b>Access Restrictions:</b><br>
Access to this endpoint requires at least `LIST_HELPER` permissions.
</div>

Modifies a given record.

### Request

| Header        | Expected Value                                                                             | Optional |
| ------------- | ------------------------------------------------------------------------------------------ | -------- |
| Content-Type  | `application/json`                                                                         | false    |
| Authorization | [Pointercrate access token](/documentation/#access-tokens)                                 | false    |
| If-Match      | Conditional request header. Needs to be set to the current etag value of the record object | false    |

| Field    | Type                           | Description                                                                       | Optional |
| -------- | ------------------------------ | --------------------------------------------------------------------------------- | -------- |
| progress | integer                        | Set to update the progress                                                        | true     |
| video    | URL                            | Set to update the video. Can be `null`                                            | true     |
| status   | [RecordStatus](#record-status) | Set to update the record's status                                                 | true     |
| player   | string                         | Set to update the record holder. Needs to be the name of the player               | true     |
| demon    | string                         | Set to update the demon the record was made on. Needs to be the name of the demon | true     |
| notes    | string                         | Set to update the record's notes                                                  | true     |

### Response: `200 OK`

| Header       | Value                                      |
| ------------ | ------------------------------------------ |
| Content-Type | `application/json`                         |
| ETag         | unsigned 64 bit hash of the updated record |

| Field | Type                                     | Description               |
| ----- | ---------------------------------------- | ------------------------- |
| data  | [Record](/documentation/objects/#record) | The updated record object |

### Response: `304 NOT MODIFIED`

Returned when the `PATCH` operation did not make any changes.

| Header | Value                              |
| ------ | ---------------------------------- |
| ETag   | unsigned 64 bit hash of the record |

### Errors

| Status code | Error code | Description                                                                                                     |
| ----------- | ---------- | --------------------------------------------------------------------------------------------------------------- |
| 400         | 40003      | Invalid data type for requested field                                                                           |
| 403         | 40302      | The requested field cannot be updated via this endpoint                                                         |
| 404         | 40401      | No record with id `record_id` was found                                                                         |
| 404         | 40401      | The updated value for demon does not exist                                                                      |
| 412         | 41200      | The value provided in the `If-Match` header doesn't match the current state of the object                       |
| 418         | 41800      | No `If-Match` header was provided                                                                               |
| 422         | 42215      | The updated progress value does not meat the demons requirement                                                 |
| 422         | 42216      | The update status value is not a valid member of the [RecordStatus](/documentation/objects/#record-status) enum |
| 422         | 42221      | The record holder is banned and you tried to set the record status to `APPROVED`                                |

### Example request

```json
PATCH /api/v1/records/2/
Accept: application/json
Authorization: Bearer <omitted>
Content-Type: application/json
If-Match: VV4v4HlCVToXCSqxdpaV3IQGRLw=

{
    "status": "approved",
    "notes": "Record made on approved low-detail copyable"
}
```

</div>
