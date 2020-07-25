<div class='panel fade js-scroll-anim' data-anim='fade'>

# Modifying your account{id=patch-me}

## `PATCH`{.verb} `/auth/me/`

Modifies the currently logged in account (that is, the account whose credentials are sent).

Note that after updating your password, you will have to [log in](#login) again, as changing passwords invalidates access tokens.

Modifying your account requires you to provide your password instead of just an access token, to ensure that if you for some reason leak your access token,
other people at least cannot change your password, allowing you to invalidate the leaked token by doing so yourself.

### Request:

| Header        | Expected Value                                                                           | Optional |
| ------------- | ---------------------------------------------------------------------------------------- | -------- |
| Content-Type  | `application/json`                                                           | false    |
| Authorization | Basic access authentication header                                                       | false    |
| If-Match      | Conditional request header. Needs to be set to the current etag value of the user object | false    |

| Field           | Type   | Description                                                                   | Optional |
| --------------- | ------ | ----------------------------------------------------------------------------- | -------- |
| password        | string | Set to update your password                                                   | true     |
| display_name    | string | Set to update your diplay name. Set to `null` to reset it                     | true     |
| youtube_channel | string | Set to update the link to your youtube channel displayed along with your name | true     |

### Response: `200 OK` or `204 NO CONTENT`

In case the password was changed, a `204` is returned and the user has to reauthenticate. Otherwise, a `200` response is generated.

| Header       | Value                                    |
| ------------ | ---------------------------------------- |
| Content-Type | `application/json`                       |
| ETag         | unsigned 64 bit hash of your user object |

| Field | Type                                 | Description                                                 |
| ----- | ------------------------------------ | ----------------------------------------------------------- |
| data  | [User](/documentation/objects/#user) | A user object representing the account you just logged into |

### Response: `304 NOT MODIFIED`

Returned when the `PATCH` operation did not make any changes. Note that this is also returned when you only change your password,
as you hashed password is not part of your user object hash.

| Header | Value                                    |
| ------ | ---------------------------------------- |
| ETag   | unsigned 64 bit hash of your user object |

### Errors:

| Status code | Error code | Description                                                                               | Data                                |
| ----------- | ---------- | ----------------------------------------------------------------------------------------- | ----------------------------------- |
| 422         | 42202      | The choosen name does not meet the criteria described [here](#registering-for-an-account) | -                                   |
| 422         | 42204      | The choosen password is too short                                                         | -                                   |
| 422         | 42225      | The channel URL does not match the expected format                                        | `expected`: The expected URL format |
| 422         | 42226      | The provided channel URL isn't a YouTube URL                                              | -                                   |

### Example request:

```json
PATCH /api/v1/auth/me/
Accept: application/json
Authorization: Basic <omitted>
Content-Type: application/json
If-Match: 10434480491831244259

{
    "display_name": "stardust1971",
    "password": "password1234"
}
```

</div>
