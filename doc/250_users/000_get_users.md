% users

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
Access to this endpoint requires at least `MODERATOR` permissions.
</div>

Allows to retrieve a potentially filtered list of all pointercrate users.

### Filtering:

The result can be filtered by any of the following fields: `id`, `name`, `permissions`, `display_name`.

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

```http
GET /api/v1/users/
Accept: application/json
Authorization: Bearer <omitted>
```

</div>
