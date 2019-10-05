<div class='panel fade js-scroll-anim' data-anim='fade'>

# Player objects{id=player}

Each player on the list is represented by a `Player` object. The following invariant holds true for any player object:

- If the player is banned, they do not have any approved or submitted records on the list

Note that it is not possible to retrieve a player's demonlist score via the API. You can calculate it yourself based on the `records` list

## Minimal Form

When part of the representation of another object, a player has the following fields:

| Field  | Type    | Description                                   |
| ------ | ------- | --------------------------------------------- |
| id     | integer | The player's id                               |
| name   | string  | The player's name                             |
| banned | boolean | Value indicating whether the player is banned |

## Listed Form

When retrieving players via [`GET /players/`](/documentation/players/#get-players)

| Field       | Type                        | Description                                   |
| ----------- | --------------------------- | --------------------------------------------- |
| id          | integer                     | The player's id                               |
| name        | string                      | The player's name                             |
| banned      | boolean                     | Value indicating whether the player is banned |
| nationality | [Nationality](#nationality) | The player's nationality, if set              |

## Full Form

The listed record objects do not contain the current player embedded into the `player` field.

| Field       | Type                        | Description                                   |
| ----------- | --------------------------- | --------------------------------------------- |
| id          | integer                     | The player's id                               |
| name        | string                      | The player's name                             |
| banned      | boolean                     | Value indicating whether the player is banned |
| nationality | [Nationality](#nationality) | The player's nationality, if set              |
| created     | List[[Demon](#demon)]       | A list of demons the player created           |
| records     | List[[Record](#record)]     | A list of records the player has on the list  |
| published   | List[[Demon](#demon)]       | A list of demons the player has published     |
| verified    | List[[Demon](#demon)]       | A list of demons the player has verified      |

## Example objects

### Minimal form

```json
{
  "id": 4,
  "name": "Pennutoh",
  "banned": false
}
```

### Listed Form

```json
{
  "id": 4,
  "name": "Pennutoh",
  "banned": false,
  "nationality": {
    "nation": "Andorra",
    "country_code": "AD"
  }
}
```

### Full form

```json
{
  "banned": false,
  "beaten": [],
  "records": [
    {
      "id": 12,
      "name": "Cadrega City",
      "progress": 100,
      "status": "approved"
    }
  ],
  "id": 2,
  "name": "Pennutoh",
  "nationality": {
    "nation": "Andorra",
    "country_code": "AD"
  },
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
