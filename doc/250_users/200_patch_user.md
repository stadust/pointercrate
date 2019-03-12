<div class='panel fade js-scroll-anim' data-anim='fade'>

# Modifying a user{id=patch-user}

## `PATCH`{.verb} `/users/` `user_id`{.param} `/`

<div class='info-yellow'>
<b>Access Restrictions:</b><br>
Access to this endpoint requires at least `Moderator` or have the requested user fall inside your jurisdiction. Exactly which fields you can modify is explained below
</div>

Modifies a given user. There are three types of users that can make requests to this endpoint

+ People whose highest role is `Administrator`: Administrators can patch the `display_name` and `permissions` fields of everyone but themselves.
+ People whose highest role is `Moderator`: Moderators can patch the `display_name` fields of everyone but themselves
+ People whose highest role mark them as a team leader: These people can assign and remove roles _that are related to their team_ and only _if the user falls under their jurisdiction_

Also note that you cannot grant (or revoke) other users `Administrator` permissions.

### Request:

| Header        | Expected Value                                                                           | Optional |
| ------------- | ---------------------------------------------------------------------------------------- | -------- |
| Content-Type  | `application/json`                                                           | false    |
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
| ----------- | ---------- | ----------------------------------------------------------------------------------------|
|403|40303| Attempt to patch your own account through this endpoint|

### Example request:

```json
PATCH /api/v1/users/1/
Accept: application/json
Authorization: Bearer <omitted>
Content-Type: application/json
If-Match: 10434480491831244259

{
    "display_name": "testtest",
    "permissions": 3
}
```

</div>
