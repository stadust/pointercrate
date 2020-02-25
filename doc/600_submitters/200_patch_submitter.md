<div class='panel fade js-scroll-anim' data-anim='fade'>

# Banning a submitter

## `PATCH`{.verb} `/submitters/` `submitter_id`{.param} `/`

<div class='info-yellow'>
<b>Access Restrictions:</b><br>
Access to this endpoint requires at least `ListModerator` permissions.
</div>

### Request:

| Header        | Expected Value                                                                                | Optional |
| ------------- | --------------------------------------------------------------------------------------------- | -------- |
| Content-Type  | `application/json`                                                                | false    |
| Authorization | [Pointercrate access token](/documentation/#access-tokens)                                    | false    |
| If-Match      | Conditional request header. Needs to be set to the current etag value of the submitter object | false    |

| Field  | Type    | Description                          | Optional |
| ------ | ------- | ------------------------------------ | -------- |
| banned | boolean | Update the submitter's banned status | true     |

### Response: `200 OK`

| Header       | Value                                        |
| ------------ | -------------------------------------------- |
| Content-Type | `application/json`                           |
| ETag         |unsigned 64 bit  hash of the updated submitter |

| Field | Type                                           | Description                  |
| ----- | ---------------------------------------------- | ---------------------------- |
| data  | [Submitter](/documentation/objects/#submitter) | The updated submitter object |

### Response: `304 NOT MODIFIED`

Returned when the `PATCH` operation did not make any changes.

| Header | Value                                |
| ------ | ------------------------------------ |
| ETag   | unsigned 64 bit  hash of the submitter |

### Errors:

| Status code | Error code | Description                                             |
| ----------- | ---------- | ------------------------------------------------------- |
| 400         | 40003      | Invalid data type for requested field                   |
| 403         | 40302      | The requested field cannot be updated via this endpoint |
| 404         | 40401      | No submitter with id `submitter_id` was found           |

### Example request:

```json
PATCH /api/v1/submitters/2/
Accept: application/json
Authorization: Bearer <omitted>
Content-Type: application/json
If-Match: Pi0YjDmf-_EGc9fDY7xZJHQCC20=

{
    "banned": true
}
```

</div>
