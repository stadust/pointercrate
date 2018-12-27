<div class='panel fade js-scroll-anim' data-anim='fade'>

# Authentication

## Basic

Some endpoints in the API require you to authenticate using [HTTP Basic Authentication](https://en.wikipedia.org/wiki/Basic_access_authentication). Since all communication with the API is enforced to be done via HTTPS, this is OK.

## Access tokens{id=token-auth}

Pointercrate requires you to have a valid access token to issue requests to most endpoints.
An access token for your account can be retrieved via a successful call to the [login](/documentation/account/#login) endpoint.

Pointercrate access tokens are [JSON Web Tokens](https://jwt.io) and can be parsed by any standard compliant implementation.

Each access token is valid until you change your password, or is invalidated via a call to [invalidate](/documentation/account/#invalidate).

When an endpoint requires authentication via an access token, the `Authorization` header has to be set to the word `Bearer` followed by a space,
followed by your access token.

## Errors

These error conditions apply to any endpoint that require authentication and are thus not repeated for every one of them.

| Status code | Error code | Description                                                                                        |
| ----------- | ---------- | -------------------------------------------------------------------------------------------------- |
| 401         | 40100      | Authorization failed (e.g. because of a bad username, wrong password, wrong authorization method ) |

</div>
