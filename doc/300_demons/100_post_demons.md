<div class='panel fade js-scroll-anim' data-anim='fade'>

# Adding demons

## `POST`{.verb} `/demons/`

<div class='info-yellow'>
<b>Access Restrictions:</b><br>
Access to this endpoint requires at least `LIST_MODERATOR` permissions.
</div>

Adds a demon to the demonlist. Adding a demon automatically shifts the other demons around to make
room for the newly added one.

The `video` value, if provided, must meet the requirements specified [here](/documentation/#video).

### Request:

| Header       | Expected Value     | Optional |
| ------------ | ------------------ | -------- |
| Content-Type | `application/json` | false    |

| Field       | Type         | Description                            | Optional |
| ----------- | ------------ | -------------------------------------- | -------- |
| name        | string       | The name of the demon                  | false    |
| position    | integer      | The position of the demon              | false    |
| requirement | integer      | The record requirement for the demon   | false    |
| verifier    | string       | The name of the verifier of the demon  | false    |
| publisher   | string       | The name of the publisher of the demon | false    |
| creators    | List[string] | The names of the creatorsof the demon  | false    |
| video       | string       | A link to the verification video       | true     |

### Response: `201 CREATED`

| Header       | Value                                           |
| ------------ | ----------------------------------------------- |
| Content-Type | `application/json`                              |
| Location     | The location of the newly created demon         |
| ETag         | unsigned 64 bit hash of the newly created demon |

| Field | Type                                   | Description                    |
| ----- | -------------------------------------- | ------------------------------ |
| data  | [Demon](/documentation/objects/#demon) | The newly created demon object |

### Errors:

| Status code | Error code | Description                                                                                          |
| ----------- | ---------- | ---------------------------------------------------------------------------------------------------- |
| 409         | 40904      | A demon with the specified name already exists on the list                                           |
| 422         | 42212      | The `requirement` value is either smaller than `0` or greater than `100`                             |
| 422         | 42213      | The `position` value is either smaller than `1` or greater than current amount of demons on the list |

### Example request:

```json
POST /api/v1/demons/
Accept: application/json
Authorization: Bearer <omitted>
Content-Type: application/json

{
    "name": "Cadrega City",
    "position": 11,
    "requirement": 54,
    "verifier": "Sunix",
    "publisher": "Pennutoh",
    "creators": ["Pennutoh"],
    "video": "https://www.youtube.com/watch?v=cHEGAqOgddA"
}
```

</div>
