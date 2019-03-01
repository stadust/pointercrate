<div class='panel fade js-scroll-anim' data-anim='fade'>

# Invalidating access tokens{id=invalidate}

## `POST`{.verb} `/auth/invalidate/`

Invalidates all access tokens to your account.

### Request:

| Header        | Expected Value                     | Optional |
| ------------- | ---------------------------------- | -------- |
| Authorization | Basic access authentication header | false    |

### Response: `204 NO CONTENT`

_Nothing_

### Example request

```json
POST /api/v1/auth/invalidate/
Accept: application/json
Authorization: Basic <omitted>
```

</div>
