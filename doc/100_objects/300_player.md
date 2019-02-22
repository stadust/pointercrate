<div class='panel fade js-scroll-anim' data-anim='fade'>

# Player objects{id=player}

Each player on the list is represented by a `Player` object. The following invariant holds true for any player object:

- If the player is banned, they do not have any approved or submitted records on the list

Note that it is not possible to retrieve a player's demonlist score via the API. You can calculate it yourself based on the `records` list

## Short/Embedded Form

When retrieving players via [`GET /players/`](/documentation/players/#get-players), or as the field of another object,
only a shorter representation of each player is provided.

| Field  | Type    | Description                                   |
| ------ | ------- | --------------------------------------------- |
| id     | integer | The player's id                               |
| name   | string  | The player's name                             |
| banned | boolean | Value indicating whether the player is banned |

## Long Form

| Field     | Type                    | Description                                   |
| --------- | ----------------------- | --------------------------------------------- |
| id        | integer                 | The player's id                               |
| name      | string                  | The player's name                             |
| banned    | boolean                 | Value indicating whether the player is banned |
| created   | List[[Demon](#demon)]   | A list of demons the player created           |
| beaten    | _see below_             | _see below_                                   |
| records   | List[[Record](#record)] | A list of records the player has on the list  |
| published | List[[Demon](#demon)]   | A list of demons the player has published     |
| verified  | List[[Demon](#demon)]   | A list of demons the player has verified      |

**Note**: The `beaten` fields is provided temporarily for compatibility purposes with some discord bots built against the API. You should use `records` instead

## Example objects

### Short/Embedded form

```json
{
  "id": 4,
  "name": "Pennutoh",
  "banned": false
}
```

### Long form

```json
{
  "banned": false,
  "beaten": [],
  "records": [
    {
      "id": 12,
      "name": "Cadrega City",
      "progress": 100,
      "status": "approved",
      "player": "Pennutoh"
    }
  ],
  "id": 2,
  "name": "Pennutoh",
  "published": [
    {
      "name": "Cadrega City",
      "position": 34
    }
  ],
  "verified": []
}
```

</div>
