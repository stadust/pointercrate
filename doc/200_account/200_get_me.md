<div class='panel fade js-scroll-anim' data-anim='fade'>

# Retrieve account information{id=get-me}

## `GET`{.verb} `/auth/me/`

Gets information about the currently logged in account (that is, the account whose access token is sent).

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
| ETag         | unsigned 64 bit hash of your user object |

| Field | Type                                 | Description                                                 |
| ----- | ------------------------------------ | ----------------------------------------------------------- |
| data  | [User](/documentation/objects/#user) | A user object representing the account you just logged into |

### Response: `304 NOT MODIFIED`

Returned if the `If-None-Match` header is set, and the etag for the user object matches one of the set values.

| Header | Value                                    |
| ------ | ---------------------------------------- |
| ETag   | unsigned 64 bit hash of your user object |

### Example request:

```json
GET /api/v1/auth/me/
Accept: application/json
Authorization: Bearer <omitted>
```

</div>
