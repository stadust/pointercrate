<div class='panel fade js-scroll-anim' data-anim='fade'>

# Error objects{id=error}

In case of a client or a (not completely fatal) server error, the API will provide an `Error` object with some information about what went wrong.

Although each HTTP response comes with a status code, you can still calculate the status code from the error code by performing integer division by `100`

| Field   | Type    | Description                                                     |
| ------- | ------- | --------------------------------------------------------------- |
| message | string  | A short message describing the error                            |
| code    | integer | The error code                                                  |
| data    | object  | A JSON object containing additional data relevant to the error. |

## Example object

```json
{
  "code": 42217,
  "data": {
    "existing": 13
  },
  "message": "This records has already been submitted"
}
```

</div>
