% demons

<div class='panel fade js-scroll-anim' data-anim='fade'>

# Demon listing

## `GET`{.verb} `/demons/`

<div class='info-green'>
<b>Pagination:</b><br>
This endpoint supports [pagination and filtering](/documentation/#pagination) via query parameters. Please see the documentation on pagination for information
on the additional request and response fields headers.
</div>

Allows to retrieve a potentionally filtered version of the demonlist.

### Filtering:

The result can be filtered by any of the following fields: `name`, `name_contains`, `position`, `requirement`, `verifier.id` (via `verifier_id`), `publisher.id` (via `publisher_id`), `verifier.name` (via `verifier_name`), `publisher.name` (via `publisher.name`). To filter by creator, please use
[`GET /players/player_id/`](/documentation/players/#get-player) and inspect the relevant fields of the [Player](/documentation/objects/#player) object.

Pagination is done via the `position` field.

### Request:

_No request data required_

### Response: `200 OK`

| Header       | Value              |
| ------------ | ------------------ |
| Content-Type | `application/json` |

| Field | Type                                         | Description      |
| ----- | -------------------------------------------- | ---------------- |
| -     | List[[Demon](/documentation/objects/#demon)] | A list of demons |

### Example request:

```json
GET /api/v1/demons/
Accept: application/json
```

</div>
