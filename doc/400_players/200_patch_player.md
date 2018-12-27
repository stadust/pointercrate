<div class='panel fade js-scroll-anim' data-anim='fade'>

# Modifying a player

## `PATCH`{.verb} `/players/` `player_id`{.param} `/`

<div class='info-yellow'>
<b>Access Restrictions:</b><br>
Access to this endpoint requires at least `LIST_MODERATOR` permissions.
</div>

Modifies a given player.

Banning a player will _hide_, but not delete their records from the list. After he has been unbanned, they can be readded.

Renaming a player to the name of an already existing player will merge all their records. If the two players have a record on the same demon,
the record will the higher progress will take precedence.

### Request:

| Header        | Expected Value                                                                             | Optional |
| ------------- | ------------------------------------------------------------------------------------------ | -------- |
| Content-Type  | `application/merge-patch+json`                                                             | false    |
| Authorization | [Pointercrate access token](/documentation/#access-tokens)                                 | false    |
| If-Match      | Conditional request header. Needs to be set to the current etag value of the player object | false    |

| Field  | Type    | Description                              | Optional |
| ------ | ------- | ---------------------------------------- | -------- |
| name   | string  | Set to update the player's name          | true     |
| banned | boolean | Set to update the player's banned status | true     |

### Response: `200 OK`

| Header       | Value                                     |
| ------------ | ----------------------------------------- |
| Content-Type | `application/json`                        |
| ETag         | base64 encoded hash of the updated player |

| Field | Type                                     | Description               |
| ----- | ---------------------------------------- | ------------------------- |
| data  | [Player](/documentation/objects/#player) | The updated player object |

### Response: `304 NOT MODIFIED`

Returned when the `PATCH` operation did not make any changes.

| Header | Value                             |
| ------ | --------------------------------- |
| ETag   | base64 encoded hash of the player |

### Errors:

| Status code | Error code | Description                                             |
| ----------- | ---------- | ------------------------------------------------------- |
| 400         | 40003      | Invalid data type for requested field                   |
| 403         | 40302      | The requested field cannot be updated via this endpoint |
| 404         | 40401      | No player with id `player_id` was found                 |

### Example request:

```http
PATCH /api/v1/players/1/
Accept: application/json
Authorization: Bearer <omitted>
Content-Type: application/merge-patch+json
If-Match: FfbtbML27VL1ciOI1Ar0mX20Yhc=

{
    "banned": true
}
```

</div>
