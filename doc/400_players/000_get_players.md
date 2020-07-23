% players

<div class='panel fade js-scroll-anim' data-anim='fade'>

# Player listing{id=get-players}

## `GET`{.verb} `/players/`

<div class='info-green'>
<b>Pagination:</b><br>
This endpoint supports [pagination and filtering](/documentation/#pagination) via query parameters. Please see the documentation on pagination for information
on the additional request and response fields headers.
</div>

<div class='info-yellow'>
<b>Access Restrictions:</b><br>
Access to this endpoint requires at least `EXTENDED_ACCESS` permissions.
</div>

Allows to retrieve a potentially filtered list of all players having records on the list, or are associated with a demon in some other way.

### Filtering:

The result can be filtered by any of the following fields: `id`, `name`, `name_contains`, `banned` and `nationality` (both by country code and country name).

Pagination is done via the `id` field.

### Request:

| Header        | Expected Value                                             | Optional |
| ------------- | ---------------------------------------------------------- | -------- |
| Authorization | [Pointercrate access token](/documentation/#access-tokens) | false    |

### Response: `200 OK`

| Header       | Value              |
| ------------ | ------------------ |
| Content-Type | `application/json` |

| Field | Type                                           | Description       |
| ----- | ---------------------------------------------- | ----------------- |
| -     | List[[Player](/documentation/objects/#player)] | A list of players |

### Example request:

```json
GET /api/v1/players/
Accept: application/json
Authorization: Bearer <omitted>
```

</div>
