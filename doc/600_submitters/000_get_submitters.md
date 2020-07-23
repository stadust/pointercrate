% submitters

<div class='panel fade js-scroll-anim' data-anim='fade'>

# Submitter listing

## `GET`{.verb} `/submitters/`

<div class='info-green'>
<b>Pagination:</b><br>
This endpoint supports [pagination and filtering](/documentation/#pagination) via query parameters. Please see the documentation on pagination for information
on the additional request and response fields headers.
</div>

<div class='info-yellow'>
<b>Access Restrictions:</b><br>
Access to this endpoint requires at least `ListAdministrator` permissions.
</div>

### Filtering:

The result can be filtered by any of the following fields: `banned`.

Pagination is done via the `submitter_id` field.

### Request:

| Header       | Value              |
| ------------ | ------------------ |
| Content-Type | `application/json` |

| Header        | Expected Value                                             | Optional |
| ------------- | ---------------------------------------------------------- | -------- |
| Authorization | [Pointercrate access token](/documentation/#access-tokens) | false    |

### Response: `200 OK`

| Field | Type                                            | Description          |
| ----- | ----------------------------------------------- | -------------------- |
| -     | List[[Submitter](/documentation/objects/#user)] | A list of submitters |

### Example request:

```json
GET /api/v1/submitters/
Accept: application/json
Authorization: Bearer <omitted>
```

</div>
