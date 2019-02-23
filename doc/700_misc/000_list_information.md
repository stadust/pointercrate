<div class='panel fade js-scroll-anim' data-anim='fade'>

# List metadata

## `GET`{.verb} `/list_information/`

### Request

_No data or headers required_

### Response: `200 OK`

| Header       | Value              |
| ------------ | ------------------ |
| Content-Type | `application/json` |

| Field              | Type    | Description                                                                             |
| ------------------ | ------- | --------------------------------------------------------------------------------------- |
| extended_list_size | integer | The length of the demonlist, including the extended list, but excluding the legacy list |
| list_size          | integer | The length of the demonlist, excluding extended and legacy list                         |

### Example request

```json
GET /api/v1/list_information/
Accept: application/json
```

</div>
