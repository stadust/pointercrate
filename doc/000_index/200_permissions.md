<div class='panel fade js-scroll-anim' data-anim='fade'>

# Permissions

Different endpoints require different kinds of privileges to be used.
A [user](/documentation/objects/#user)'s permissions are saved as a bitmask, and by default every user has absolutely no permissions.

Permissions can be granted by other users with special permissions via the [`PATCH /users/user_id/`](/documentation/users/#patch-user) endpoint:

- A user with the `ADMINISTRATOR` permission can assign the `MODERATOR`, `LIST_ADMINISTRATOR` and `EXTENDED_ACCESS` permissions
- A user with the `LIST_ADMINISTRATOR` permission can assign the `LIST_HELPER` and `LIST_MODERATOR` permissions.
- A user with the `RESERVED2` permission can assign the `RESERVED1` permission

If an endpoints requires special permissions to be accessed, it's documentation will contain a notice similar to this one:

<div class='info-yellow'>
<b>Access Restrictions:</b><br>
Access to this endpoint requires at least `LIST_HELPER` permissions.
</div>

### Available permissions

| Permission           | Bit    | Description                                                                                                                                            |
| -------------------- | ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `EXTENDED_ACCESS`    | 0x1    | Users that have access to additional data retrieval endpoints                                                                                          |
| `LIST_HELPER`        | 0x2    | Users that help out in managing the demonlist by reviewing records                                                                                     |
| `LIST_MODERATOR`     | 0x4    | Users that moderate the demonlist and manage the demon placements                                                                                      |
| `LIST_ADMINISTRATOR` | 0x8    | Users that administrate the demonlist.                                                                                                                 |
| `RESERVED1`          | 0x10   | _Reserved for future use_                                                                                                                              |
| `RESERVED2`          | 0x20   | _Reserved for future use_                                                                                                                              |
| `MODERATOR`          | 0x2000 | Users that have access to the pointercrate user list                                                                                                   |
| `ADMINISTRATOR`      | 0x4000 | Users that can manage other users, including granting them permissions                                                                                 |
| `-`                  | 0x8000 | A permission users cannot have, but is required to assign certain other permissions, effectively preventing those permissions from ever being assigned |

### Errors

These error conditions can occur at any endpoint expecting requiring specific access permissions and are thus not listed specifically for each of them.

| Status code | Error code | Description                                                      | Data                                                                                  |
| ----------- | ---------- | ---------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| 403         | 40301      | You do not have the permissions required to perform this request | `required`: A list of permission-bitmasks that would allow you to perform the request |

</div>
