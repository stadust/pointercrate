error-user-malformedchannelurl = Malformed channel URL
error-user-deleteself = You cannot delete your own account via this endpoint. Use DELETE /api/v1/auth/me/
error-user-patchself = You cannot modify your own account via this endpoint. Use PATCH /api/v1/auth/me/
error-user-permissionnotassignable = You cannot assign the following permissions: { $non-assignable }
error-user-usernotfound = No user with id { $user-id } found
error-user-usernotfoundname = No user with name { $user-name } found
error-user-nametaken = The chosen username is already taken
error-user-invalidusername = Invalid display or username! The name must be at least 3 characters long and not start/end with a space
error-user-invalidpassword = Invalid password! The password must be at least 10 characters long
error-user-notyoutube = The given URL is no YouTube URL
error-user-nonlegacyaccount = The given operation (change password) is invalid on non-legacy account, as password login is not supported for these

error-user-ratelimit-registration = Too many registrations!
error-user-ratelimit-soft-registration = Too many failed registration attempts!
error-user-ratelimit-login = Too many login attempts!