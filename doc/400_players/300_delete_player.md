<div class='panel fade js-scroll-anim' data-anim='fade'>

# Deleting a player

## `DELETE`{.verb} `/players/` `player_id`{.param} `/`

<div class='info-yellow'>
<b>Access Restrictions:</b><br>
Access to this endpoint requires at least `LIST_MODERATOR` permissions.
</div>

Deletes a player. This only works if the player is no longer referenced by any records or demons.

### Request:

| Header        | Expected Value                                                                             | Optional |
| ------------- | ------------------------------------------------------------------------------------------ | -------- |
| Authorization | [Pointercrate access token](/documentation/#access-tokens)                                           | false    |
| If-Match      | Conditional request header. Needs to be set to the current etag value of the player object | false    |

### Response: `204 NO CONTENT`

_Nothing_

### Errors:

| Status code | Error code | Description                                                                               |
| ----------- | ---------- | ----------------------------------------------------------------------------------------- |
| 404         | 40401      | No player with id `player_id` was found                                                   |
| 409         | 40901      | The player is still referenced somewhere and cannot be deleted                            |
| 412         | 41200      | The value provided in the `If-Match` header doesn't match the current state of the object |
| 418         | 41800      | No `If-Match` header was provided                                                         |

### Example request:

```http
DELETE /api/v1/players/1/
Accept: application/json
Authorization: Bearer <omitted>
If-Match: FfbtbML27VL1ciOI1Ar0mX20Yhc=
```

</div>
