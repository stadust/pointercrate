<div class='panel fade js-scroll-anim' data-anim='fade'>

# Player objects{id=player}

Each player on the list is represented by a `Player` object. The following assumptions can be made about these:

* If a player is banned, he does not have any approved or submitted records on the list

Note that it is not possible to retrieve a player's demonlist score via the API.

## Short Form

When retrieving players via [`GET /players/`](/documentation/players/#get-players), or as the field of another object,
only a shorter representation of each player is provided.

| Field  | Type    | Description                                   |
| ------ | ------- | --------------------------------------------- |
| id     | integer | The player's id                               |
| name   | string  | The player's name                             |
| banned | boolean | Value indicating whether the player is banned |

## Long Form

| Field     | Type                  | Description                                   |
| --------- | --------------------- | --------------------------------------------- |
| id        | integer               | The player's id                               |
| name      | string                | The player's name                             |
| banned    | boolean               | Value indicating whether the player is banned |
| created   | List[[Demon](#demon)] | A list of demons the player created           |
| beaten    | List[[Demon](#demon)] | A list of demons the player has beaten        |
| published | List[[Demon](#demon)] | A list of demons the player has published     |
| verified  | List[[Demon](#demon)] | A list of demons the player has verified      |

### Example object:

```
{
  "banned": false,
  "beaten": [],
  "created": [
    {
      "name": "Cadrega City",
      "position": 34,
      "state": "MAIN"
    }
  ],
  "id": 2,
  "name": "Pennutoh",
  "published": [
    {
      "name": "Cadrega City",
      "position": 34,
      "state": "MAIN"
    }
  ],
  "verified": []
}
```

</div>
