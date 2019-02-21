% objects

<div class='panel fade js-scroll-anim' data-anim='fade'>

# User objects{id=user}

Each pointercrate user is represented by a `User` object.

If the display name is not `null`, it will replace a user's username wherever their name is displayed.

There is only one form of user objects:

| Field           | Type    | Description                                                  |
| --------------- | ------- | ------------------------------------------------------------ |
| id              | int     | The user's ID                                                |
| name            | string  | The user's name                                              |
| permissions     | bitmask | The user's access [permissions](/documentation/#permissions) |
| display_name    | string  | The user's display name. This can be `null`.                 |
| youtube_channel | string  | The user's linked youtube channel. This can be `null`        |

## Example object

```json
{
  "display_name": "stadust",
  "id": 2,
  "name": "stardust1971",
  "permissions": 0,
  "youtube_channel": null
}
```

</div>
