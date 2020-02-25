<div class='panel fade js-scroll-anim' data-anim='fade'>

# Submitter retrieval

## `GET`{.verb} `/submitters/` `submitter_id`{.param} `/`

<div class='info-yellow'>
<b>Access Restrictions:</b><br>
Access to this endpoint requires at least `ListModerator` permissions.
</div>

### Request:

| Header        | Expected Value                                                                                                                                                                                              | Optional |
| ------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| Authorization | [Pointercrate access token](/documentation/#access-tokens)                                                                                                                                                  | false    |
| If-Match      | Conditional request header. If the etag value of the requested data matches any of the here provided values, the data is returned as requested. Otherwise a `412 PRECONDITION FAILED` response is generated | true     |
| If-None-Match | Conditional request header. If the etag value of the requested data does not match any of the here provided values, if it returned as requested. Otherwise, a `304 NOT MODIFED` response is generated       | true     |

### Response: `200 OK`

| Header       | Value                                       |
| ------------ | ------------------------------------------- |
| Content-Type | `application/json`                          |
| ETag         | unsigned 64 bit  hash of the submitter object |

| Field | Type                                           | Description                    |
| ----- | ---------------------------------------------- | ------------------------------ |
| data  | [Submitter](/documentation/objects/#submitter) | The requested submitter object |

### Response: `304 NOT MODIFIED`

Returned if the `If-None-Match` header is set, and the etag for the submitter object matches one of the set values.

| Header | Value                                       |
| ------ | ------------------------------------------- |
| ETag   | unsigned 64 bit  hash of the submitter object |

### Errors:

| Status code | Error code | Description                                   |
| ----------- | ---------- | --------------------------------------------- |
| 404         | 40401      | No submitter with id `submitter_id` was found |

### Example request:

```json
GET /api/v1/submitters/2/
Accept: application/json
Authorization: Bearer <omitted>
```

</div>
