<div class='panel fade js-scroll-anim' data-anim='fade'>

# Modifying a player

## `PATCH`{.verb} `/players/` `player_id`{.param} `/`

<div class='info-yellow'>
<b>Access Restrictions:</b><br>
Access to this endpoint requires at least `ListModerator` permissions.
</div>

Modifies a given player.

Banning a player will _hide_, but not delete their records from the list. After he has been unbanned, they can be readded.

Renaming a player to the name of an already existing player will merge all their records. If the two players have a record on the same demon,
the record will the higher progress will take precedence.

### Request:

| Header        | Expected Value                                                                             | Optional |
| ------------- | ------------------------------------------------------------------------------------------ | -------- |
| Content-Type  | `application/json`                                                             | false    |
| Authorization | [Pointercrate access token](/documentation/#access-tokens)                                 | false    |
| If-Match      | Conditional request header. Needs to be set to the current etag value of the player object | false    |

| Field  | Type    | Description                              | Optional |
| ------ | ------- | ---------------------------------------- | -------- |
| name   | string  | Set to update the player's name          | true     |
| banned | boolean | Set to update the player's banned status | true     |
|nationality|string| Set to update the player's nationality. Can be either the nation's name, or its ISO countrycode| true|

### Response: `200 OK`

| Header       | Value                                     |
| ------------ | ----------------------------------------- |
| Content-Type | `application/json`                        |
| ETag         | unsigned 64 bit  hash of the updated player |

| Field | Type                                     | Description               |
| ----- | ---------------------------------------- | ------------------------- |
| data  | [Player](/documentation/objects/#player) | The updated player object |

### Response: `304 NOT MODIFIED`

Returned when the `PATCH` operation did not make any changes.

| Header | Value                             |
| ------ | --------------------------------- |
| ETag   | unsigned 64 bit  hash of the player |

### Errors:

| Status code | Error code | Description                                             |
| ----------- | ---------- | ------------------------------------------------------- |
| 400         | 40003      | Invalid data type for requested field                   |
| 403         | 40302      | The requested field cannot be updated via this endpoint |
| 404         | 40401      | No player with id `player_id` was found, or the specified nationality wasn't recognized                 |

### Example request:

```json
PATCH /api/v1/players/1/
Accept: application/json
Authorization: Bearer <omitted>
Content-Type: application/json
If-Match: FfbtbML27VL1ciOI1Ar0mX20Yhc=

{
    "banned": true
}
```

</div>
