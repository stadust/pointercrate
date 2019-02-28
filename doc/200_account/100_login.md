<div class='panel fade js-scroll-anim' data-anim='fade'>

# Login to a pointercrate account{id=login}

## `POST`{.verb} `/auth/`

<div class='info-yellow'>
<b>Rate Limits:</b><br>
This endpoint is ratelimited at 3 requests per 30 minutes
</div>

Logs into an existing pointercrate user account, providing an acccess token upon success.

### Request:

| Header        | Expected Value                     | Optional |
| ------------- | ---------------------------------- | -------- |
| Authorization | Basic access authentication header | false    |

### Response: `200 OK`

| Header       | Value                                    |
| ------------ | ---------------------------------------- |
| Content-Type | `application/json`                       |
| ETag         | unsigned 64 bit hash of your user object |

| Field | Type                                                       | Description                                                               |
| ----- | ---------------------------------------------------------- | ------------------------------------------------------------------------- |
| data  | [User](/documentation/objects/#user)                       | A user object representing the account you just logged into               |
| token | [Pointercrate access token](/documentation/#access-tokens) | Your access token to use when performing requests to the pointercrate api |

### Example request:

```json
POST /api/v1/auth/
Accept: application/json
Authorization: Basic <omitted>
```

</div>
