<div class='panel fade js-scroll-anim' data-anim='fade'>

# Deleting your account{id=delete-me}

## `DELETE`{.verb} `/auth/me/`

Deletes your pointercrate account. Note that this action is irreversible!

Deleting your account requires you to provide your password instead of just an access token, to ensure that if you for some reason leak your access token,
other people at least cannot delete your account.

### Request

| Header        | Expected Value                                                                           | Optional |
| ------------- | ---------------------------------------------------------------------------------------- | -------- |
| Authorization | Basic access authentication header                                                       | false    |
| If-Match      | Conditional request header. Needs to be set to the current etag value of the user object | false    |

### Response: `204 NO CONTENT`

_Nothing_

### Example request

```json
DELETE /appi/v1/auth/me/
Accept: application/json
Authorization: Basic <omitted>
If-Match: 10434480491831244259
```

</div>
