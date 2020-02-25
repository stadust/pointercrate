<div class='panel fade js-scroll-anim' data-anim='fade'>

# Deleting a user

## `DELETE`{.verb} `/users/` `user_id`{.param} `/`

<div class='info-yellow'>
<b>Access Restrictions:</b><br>
Access to this endpoint requires at least `Administrator` permissions.
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

| Status code | Error code | Description                                                                              |
| ----------- | ---------- | ----------------------------------------------------------------------------------------|
|403|40302| Attempt to delete your own account through this endpoint|

### Example request:

```json
DELETE /api/v1/users/1/
Accept: application/json
Authorization: Bearer <omitted>
```

</div>
