<div class='panel fade js-scroll-anim' data-anim='fade'>

# Modifying a demon

## `PATCH`{.verb} `/demons/` `position`{.param} `/`

<div class='info-yellow'>
<b>Access Restrictions:</b><br>
Access to this endpoint requires at least `LIST_MODERATOR` permissions.
</div>

Modifies a given demon.

Note that updating the position of a demon will automatically shift around the other demons to ensure position consitency.

The `video` value, if provided, must meet the requirements specified [here](/documentation/#video).

### Request:

| Header        | Expected Value                                                                            | Optional |
| ------------- | ----------------------------------------------------------------------------------------- | -------- |
| Content-Type  | `application/json`                                                            | false    |
| Authorization | [Pointercrate access token](/documentation/#access-tokens)                                | false    |
| If-Match      | Conditional request header. Needs to be set to the current etag value of the demon object | false    |

| Field       | Type    | Description                                                     | Optional |
| ----------- | ------- | --------------------------------------------------------------- | -------- |
| name        | string  | Set to update the name of the demon                             | true     |
| position    | integer | Set to update the position of the demon                         | true     |
| video       | string  | Set to update the verification video                            | true     |
| requirement | integer | Set to update the record requirement                            | true     |
| verifier    | string  | Set to update the verifier. Needs to be the name of the player  | true     |
| publisher   | string  | Set to update the publisher. Needs to be the name of the player | true     |

### Response: `200 OK`

| Header       | Value                                     |
| ------------ | ----------------------------------------- |
| Content-Type | `application/json`                        |
| ETag         | unsigned 64 bit hash of the updated demon |

| Field | Type                                   | Description              |
| ----- | -------------------------------------- | ------------------------ |
| data  | [Demon](/documentation/objects/#demon) | The updated demon object |

### Response: `304 NOT MODIFIED`

Returned when the `PATCH` operation did not make any changes.

| Header | Value                             |
| ------ | --------------------------------- |
| ETag   | unsigned 64 bit hash of the demon |

### Errors:

| Status code | Error code | Description                                                                                          |
| ----------- | ---------- | ---------------------------------------------------------------------------------------------------- |
| 404         | 40401      | No demon at the specified `position`                                                                 |
| 409         | 40904      | A demon with the updated name already exists on the list                                             |
| 422         | 42212      | The `requirement` value is smaller than `0` or greater than `100`                                    |
| 422         | 42213      | The `position` value is either smaller than `1` or greater than current amount of demons on the list |

### Example request:

```json
PATCH /api/v1/demons/1/
Accept: application/json
Authorization: Bearer <omitted>
Content-Type: application/json
If-Match: cPOrB3TM19Ffsm8PAkD2jNqB61A=

{
    "position": 17,
    "requirement": 45
}
```

</div>
