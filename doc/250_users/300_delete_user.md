<div class='panel fade js-scroll-anim' data-anim='fade'>

# Deleting a user

## `DELETE`{.verb} `/users/` `user_id`{.param} `/`

<div class='info-yellow'>
<b>Access Restrictions:</b><br>
Access to this endpoint requires at least `ADMINISTRATOR` permissions.
</div>

Deletes a user account. This action is irreversible!

### Request:

| Header        | Expected Value                                                                           | Optional |
| ------------- | ---------------------------------------------------------------------------------------- | -------- |
| Authorization | [Pointercrate access token](/documentation/#access-tokens)                               | false    |
| If-Match      | Conditional request header. Needs to be set to the current etag value of the user object | false    |

### Response: `204 NO CONTENT`

_Nothing_

### Errors:

| Status code | Error code | Description                                                                                     |
| ----------- | ---------- | ----------------------------------------------------------------------------------------------- |
| 403         | 40300      | Attempt to delete your own account. Use [`DELETE /auth/me/`](/documentation/account/#delete-me) |
| 404         | 40401      | No user with id `user_id` was found                                                             |
| 412         | 41200      | The value provided in the `If-Match` header doesn't match the current state of the object       |
| 418         | 41800      | No `If-Match` header was provided                                                               |

### Example request:

```
DELETE /api/v1/users/1/
Accept: application/json
Authorization: Bearer <omitted>
```

</div>
