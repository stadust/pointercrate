<div class='panel fade js-scroll-anim' data-anim='fade'>

# Record retrieval

## `GET`{.verb} `/records/` `record_id`{.param} `/`

<div class='info-yellow'>
<b>Acces Restrictions:</b><br>
Access to this endpoint requires at least `EXTENDED_ACCESS` permissions.
</div>

Retrieves detailed information about the record with id `record_id`

### Request:

| Header        | Expected Value                                                                                                                                                                                              | Optional |
| ------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| Authorization | [Pointercrate access token](/documentation/#access-tokens)                                                                                                                                                  | false    |
| If-Match      | Conditional request header. If the etag value of the requested data matches any of the here provided values, the data is returned as requested. Otherwise a `412 PRECONDITION FAILED` response is generated | true     |
| If-None-Match | Conditional request header. If the etag value of the requested data does not match any of the here provided values, if it returned as requested. Otherwise, a `304 NOT MODIFED` response is generated       | true     |

### Response: `200 OK`

| Header       | Value                                    |
| ------------ | ---------------------------------------- |
| Content-Type | `application/json`                       |
| ETag         | base64 encoded hash of the record object |

| Field | Type                                     | Description                 |
| ----- | ---------------------------------------- | --------------------------- |
| data  | [Record](/documentation/objects/#record) | The requested record object |

### Response: `304 NOT MODIFIED`

Returned if the `If-None-Match` header is set, and the etag for the record object matches one of the set values.

| Header | Value                                    |
| ------ | ---------------------------------------- |
| ETag   | base64 encoded hash of the record object |

### Errors:

| Status code | Error code | Description                             |
| ----------- | ---------- | --------------------------------------- |
| 404         | 40401      | No record with id `record_id` was found |

### Example request:

```
GET {{api_base}}/records/2/
Accept: application/json
Authorization: Bearer <omitted>
```

</div>
