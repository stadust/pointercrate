<div class='panel fade js-scroll-anim' data-anim='fade'>

# Modifying a user{id=patch-user}

## `PATCH`{.verb} `/users/` `user_id`{.param} `/`

<div class='info-yellow'>
<b>Access Restrictions:</b><br>
Access to this endpoint requires are explained below!
</div>

Modifies a given user.

To modify the `display_name`, you need to be at least `Moderator`. To modify the `permissions`, you must have a strictly higher permission that those you want to assign

Note that if you only have `MODERATOR` but not `ADMINISTRATOR` permissions, you can only modify a users `display_name`, not their permissions.

Also note that you cannot grant (or revoke) other users `ADMINISTRATOR` permissions.

### Request:

| Header        | Expected Value                                                                           | Optional |
| ------------- | ---------------------------------------------------------------------------------------- | -------- |
| Content-Type  | `application/merge-patch+json`                                                           | false    |
| Authorization | [Pointercrate access token](/documentation/#access-tokens)                               | false    |
| If-Match      | Conditional request header. Needs to be set to the current etag value of the user object | false    |

| Field        | Type    | Description                          | Optional |
| ------------ | ------- | ------------------------------------ | -------- |
| display_name | string  | Set to update the users display name | true     |
| permissions  | bitmask | Set to update the users permissions  | true     |

### Response: `200 OK`

| Header       | Value                                    |
| ------------ | ---------------------------------------- |
| Content-Type | `application/json`                       |
| ETag         | unsigned 64 bit hash of the updated user |

| Field | Type                                 | Description             |
| ----- | ------------------------------------ | ----------------------- |
| data  | [User](/documentation/objects/#user) | The updated user object |

### Response: `304 NOT MODIFIED`

Returned when the `PATCH` operation did not make any changes.

| Header | Value                            |
| ------ | -------------------------------- |
| ETag   | unsigned 64 bit hash of the user |

### Errors:

| Status code | Error code | Description                                                                              |
| ----------- | ---------- | ---------------------------------------------------------------------------------------- |
| 400         | 40003      | Invalid data type for requested field                                                    |
| 403         | 40302      | The requested field cannot be updated via this endpoint, or with your set of permissions |
| 404         | 40401      | No user with id `user_id` was found                                                      |

### Example request:

```json
PATCH /api/v1/users/1/
Accept: application/json
Authorization: Bearer <omitted>
Content-Type: application/merge-patch+json
If-Match: JOa_QXhezgmqMWjzqD5rYXnHi3s=

{
    "display_name": "testtest",
    "permissions": 3
}
```

</div>
