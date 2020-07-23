<div class='panel fade js-scroll-anim' data-anim='fade'>

# User listing

## `GET`{.verb} `/users/`

<div class='info-green'>
<b>Pagination:</b><br>
This endpoint supports [pagination and filtering](/documentation/#pagination) via query parameters. Please see the documentation on pagination for information
on the additional request and response fields headers.
</div>

<div class='info-yellow'>
<b>Access Restrictions:</b><br>
Access to this endpoint requires at least `Moderator` or be the leader of a team.
</div>

Allows the retrieval of a list of all pointercrate users (if you are pointercrate staff), or a list of all users that fall under your juristiction as a team leader.

### Filtering

The result can be filtered by any of the following fields: `id`, `name`, `has_permissions`, `display_name` or `name_contains` (which only matches against the actual username, not the display name).

Pagination is done via the `id` field.

### Request:

| Header        | Expected Value                                             | Optional |
| ------------- | ---------------------------------------------------------- | -------- |
| Authorization | [Pointercrate access token](/documentation/#access-tokens) | false    |

### Response: `200 OK`

| Header       | Value              |
| ------------ | ------------------ |
| Content-Type | `application/json` |

| Field | Type                                       | Description     |
| ----- | ------------------------------------------ | --------------- |
| -     | List[[User](/documentation/objects/#user)] | A list of users |

### Example request:

```json
GET /api/v1/users/?name_contains=dust&has_permissions=1
Accept: application/json
Authorization: Bearer <omitted>
```

</div>
