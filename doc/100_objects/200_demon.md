<div class='panel fade js-scroll-anim' data-anim='fade'>

# Demon objects{id=demon}

Each demon on the list is represented by a `Demon` object. The following assumptions can be made about these:

- The `requirement` value lies between `0` and `100`.
- There are no holes in the positioning and the `position` value is greater than `0`
- Every `video` value, if provided, is in one of the formats listed [here](/documentation/#video), or `null`

Note that although on the website the record requirement for demons on the extended list is always displayed as `100%`,
`Demon` objects still save their requirement from when they were on the main list.

## Minimal Form

When embedded into other objects (for example, as part of a [Record](/documentation/objects/#record)), only the following minimal representation of each demon is provided:

| Field    | Type    | Description                                                                               |
| -------- | ------- | ----------------------------------------------------------------------------------------- |
| name     | string  | The name of the demon                                                                     |
| position | integer | The position of the demon                                                                 |
| id       | integer | The demons internal ID (has nothing to do with its level ID on the geometry dash servers) |

## Listed Form

When retrieving demons via [`GET /demons/`](/documentation/demons/#get-demons), only the following partial representation of each demon is provided:

| Field     | Type              | Description                                                                               |
| --------- | ----------------- | ----------------------------------------------------------------------------------------- |
| name      | string            | The name of the demon                                                                     |
| position  | integer           | The position of the demon                                                                 |
| id        | integer           | The demons internal ID (has nothing to do with its level ID on the geometry dash servers) |
| publisher | [Player](#player) | The player that published this demon                                                      |
| verifier  | [Player](#player) | The player that verified this demon                                                       |
| video     | URL?              | The verification video                                                                    |

## Full Form

The listed record objects do not contain the current demon embedded into the `demon` field.

| Field       | Type                    | Description                                                                               |
| ----------- | ----------------------- | ----------------------------------------------------------------------------------------- |
| name        | string                  | The name of the demon                                                                     |
| position    | integer                 | The position of the demon                                                                 |
| id          | integer                 | The demons internal ID (has nothing to do with its level ID on the geometry dash servers) |
| requirement | integer                 | The minimum percentage a record on this demon has to be, to be accepted                   |
| video       | URL?                    | The verification video.                                                                   |
| verifier    | [Player](#player)       | The demon's verifier                                                                      |
| publisher   | [Player](#player)       | The demon's publisher                                                                     |
| creators    | List[[Player](#player)] | The demon's creators                                                                      |

## Example objects

### Minimal form

```json
{
  "name": "Cadrega City",
  "position": 34,
  "id": 1
}
```

### Listed form

```json
{
  "name": "Cadrega City",
  "position": 34,
  "id": 1,
  "publisher": {
    "name": "Pennutoh",
    "id": 123,
    "banned": false
  },
  "verifier": {
    "banned": false,
    "id": 3,
    "name": "Sunix"
  },
  "video": "https://www.youtube.com/watch?v=cHEGAqOgddA"
}
```

### Long form

```json
{
  "creators": [
    {
      "banned": false,
      "id": 2,
      "name": "Pennutoh"
    }
  ],
  "name": "Cadrega City",
  "position": 34,
  "id": 1,
  "publisher": {
    "banned": false,
    "id": 2,
    "name": "Pennutoh"
  },
  "requirement": 54,
  "verifier": {
    "banned": false,
    "id": 3,
    "name": "Sunix"
  },
  "video": "https://www.youtube.com/watch?v=cHEGAqOgddA"
}
```

</div>
