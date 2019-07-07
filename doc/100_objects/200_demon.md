<div class='panel fade js-scroll-anim' data-anim='fade'>

# Demon objects{id=demon}

Each demon on the list is represented by a `Demon` object. The following assumptions can be made about these:

- The `requirement` value lies between `0` and `100`.
- There are no holes in the positioning and the `position` value is greater than `0`
- Every `video` value, if provided, is in one of the formats listed [here](/documentation/#video), or `null`

Note that although on the website the record requirement for demons on the extended list is always displayed as `100%`,
`Demon` objects still save their requirement from when they were on the main list.

## Embedded Form

When embedded into other objects (for example, as part of a [Record](/documentation/objects/#record)), only the following minimal representation of each demon is provided:

| Field    | Type    | Description               |
| -------- | ------- | ------------------------- |
| name     | string  | The name of the demon     |
| position | integer | The position of the demon |

## Short Form

When retrieving demons via [`GET /demons/`](/documentation/demons/#get-demons), only the following partial representation of each demon is provided:

| Field     | Type    | Description                                      |
| --------- | ------- | ------------------------------------------------ |
| name      | string  | The name of the demon                            |
| position  | integer | The position of the demon                        |
| publisher | string  | The name of the player that published this demon |
| video     | URL?    | The verification video                           |

## Long Form

| Field       | Type                    | Description                                                             |
| ----------- | ----------------------- | ----------------------------------------------------------------------- |
| name        | string                  | The name of the demon                                                   |
| position    | integer                 | The position of the demon                                               |
| requirement | integer                 | The minimum percentage a record on this demon has to be, to be accepted |
| video       | URL?                    | The verification video.                                                 |
| verifier    | [Player](#player)       | The demon's verifier                                                    |
| publisher   | [Player](#player)       | The demon's publisher                                                   |
| creators    | List[[Player](#player)] | The demon's creators                                                    |
| records     | List[[Record](#record)] | All approved records for this demon                                     |

## Example objects

### Embedded form

```json
{
  "name": "Cadrega City",
  "position": 34
}
```

### Short form

```json
{
  "name": "Cadrega City",
  "position": 34,
  "publisher": "Pennutoh",
  "state": "MAIN",
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
  "publisher": {
    "banned": false,
    "id": 2,
    "name": "Pennutoh"
  },
  "records": [],
  "requirement": 54,
  "state": "MAIN",
  "verifier": {
    "banned": false,
    "id": 3,
    "name": "Sunix"
  },
  "video": "https://www.youtube.com/watch?v=cHEGAqOgddA"
}
```

</div>
