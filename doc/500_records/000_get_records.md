% records

<div class='panel fade js-scroll-anim' data-anim='fade'>

# Record listing{id=get-records}

## `GET`{.verb} `/records/`

<div class='info-green'>
<b>Pagination:</b><br>
This endpoint supports [pagination and filtering](/documentation/#pagination) via query parameters. Please see the documentation on pagination for information
on the additional request and response fields headers.
</div>

Allows to retrieve a list of records.

If you do not have `LIST_HELPER` or higher permissions, or don't send an access token along, you will only receive `APPROVED` records,
and have the `submitter` field always set to `null`.

### Filtering:

The result can be filtered by any of the following fields: `id`, `progress`, `status`, `player`, `demon`.

Pagination is done via the `id` field.

### Request:

| Header        | Expected Value                                             | Optional |
| ------------- | ---------------------------------------------------------- | -------- |
| Authorization | [Pointercrate access token](/documentation/#access-tokens) | true     |

### Response: `200 OK`

| Header       | Value              |
| ------------ | ------------------ |
| Content-Type | `application/json` |

| Field | Type                                           | Description       |
| ----- | ---------------------------------------------- | ----------------- |
| -     | List[[Record](/documentation/objects/#record)] | A list of records |

### Example request:

```json
GET /api/v1/records/
Accept: application/json
```

</div>
