% account

<div class='panel fade js-scroll-anim' data-anim='fade'>

# Registering for an account{id=register}

## `POST`{.verb} `/auth/register/`

<div class='info-yellow'>
<b>Rate Limits:</b><br>
This endpoint is ratelimited at 1 successful request per day and at 5 failed requests per 6 hours
</div>

Registers a new pointercrate account.

When registering, you only choose your username and your password. Your chosen username must be at least `3` spaces long and may not contain leading or trailing spaces. Your chosen password must be at least `10` characters long and has no further restrictions imposed.

The username isn't changable afterward, but you can set your `display_name` to nearly any value you want via [`PATCH /auth/me/`](#patch-me).

Registering for an account does not provide an access token, it needs to be aquired by using the [login](#login) endpoint.

### Request

| Header       | Expected Value     | Optional |
| ------------ | ------------------ | -------- |
| Content-Type | `application/json` | false    |

| Field    | Type   | Description   | Optional |
| -------- | ------ | ------------- | -------- |
| name     | string | Your username | false    |
| password | string | Your password | false    |

### Response: `201 CREATED`

| Header       | Value                                    |
| ------------ | ---------------------------------------- |
| Content-Type | `application/json`                       |
| Location     | `/auth/me/`                              |
| ETag         | unsigned 64 bit hash of your user object |

| Field | Type                                 | Description                                              |
| ----- | ------------------------------------ | -------------------------------------------------------- |
| data  | [User](/documentation/objects/#user) | A user object representing your newly registered account |

### Errors

| Status code | Error code | Description                                                |
| ----------- | ---------- | ---------------------------------------------------------- |
| 409         | 40902      | The chosen name is already in use                          |
| 422         | 42202      | The chosen name does not meet the above mentioned criteria |
| 422         | 42204      | The chosen password is too short                           |

### Example request

```json
POST /api/v1/auth/register/
Accept: application/json
Content-Type: application/json

{
    "name": "stadust",
    "password": "password123"
}
```

</div>
