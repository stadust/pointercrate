<div class='panel fade js-scroll-anim' data-anim='fade'>
<div style="top:0;right:0;left:0;bottom:0;background: rgba(0,0,0,0.1);z-index: 500; position:absolute"></div>

# Adding creators (Deprecated)

## `POST`{.verb} `/v1/demons/` `position`{.param} `/creators/`

<div class='info-yellow'>
<b>Access Restrictions:</b><br>
Access to this endpoint requires at least `ListModerator` permissions.
</div>

Adds a creator the creator list of the demon at the specified position.

### Request:

| Header        | Expected Value                                             | Optional |
| ------------- | ---------------------------------------------------------- | -------- |
| Authorization | [Pointercrate access token](/documentation/#access-tokens) | false    |
| Content-Type  | `application/json`                                         | false    |

| Field   | Type   | Description                                       | Optional |
| ------- | ------ | ------------------------------------------------- | -------- |
| creator | string | The creator to add. Needs to be the player's name | false    |

### Response: `201 CREATED`

_Nothing_

### Errors:

| Status code | Error code | Description                                         |
| ----------- | ---------- | --------------------------------------------------- |
| 404         | 40401      | No demon at the specified `position`                |
| 409         | 40905      | The given player is already registered as a creator |

### Example request:

```json
POST /api/v1/demons/2/creators/
Accept: application/json
Authorization: Bearer <omitted>
Content-Type: application/json

{
    "creator": "ViPriN"
}
```

</div>
