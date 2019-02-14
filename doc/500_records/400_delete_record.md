<div class='panel fade js-scroll-anim' data-anim='fade'>

# Deleting a record

## `DELETE`{.verb} `/records/` `record_id`{.param} `/`

<div class='info-yellow'>
<b>Access Restrictions:</b><br>
Access to this endpoint requires at least `LIST_ADMIN` permissions.
</div>

Deletes the record with the given ID. This action is irrevesible. Note that if you
simply wans to reject a submission, you should use [`PATCH /records/record_id/`](#patch-record) and change its status to `REJECTED`
to ensure it isn't submitted again.

### Request:

| Header        | Expected Value                                                                             | Optional |
| ------------- | ------------------------------------------------------------------------------------------ | -------- |
| Authorization | [Pointercrate access token](/documentation/#access-tokens)                                 | false    |
| If-Match      | Conditional request header. Needs to be set to the current etag value of the record object | false    |

### Response: `204 NO CONTENT`

_Nothing_

### Errors:

| Status code | Error code | Description                                                                               |
| ----------- | ---------- | ----------------------------------------------------------------------------------------- |
| 404         | 40401      | No record with id `record_id` was found                                                   |
| 412         | 41200      | The value provided in the `If-Match` header doesn't match the current state of the object |
| 418         | 41800      | No `If-Match` header was provided                                                         |

### Example request:

```json
DELETE /api/v1/records/1/
Accept: application/json
Authorization: Bearer <omitted>
If-Match: FmdyN2c4jElWBIOVzGOuFKJhgrE=
```

</div>
