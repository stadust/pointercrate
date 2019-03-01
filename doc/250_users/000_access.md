% users

<div class='panel fade js-scroll-anim' data-anim='fade'>

# Who has access to whom?

Generally, few people are supposed to have access to user information. People with the `Moderator` and `Administrator` permissions always do. But there are a few cases where people with fewer permissions can gain access. Generally, the leader(s) of a specific team should have access to their team members. We'll now say the members of their team fall into their jurisdiction. So for example, a `ListAdministrator` has access to users with the `ListHelper` and `ListModerator` permissions. However, this does not solve the problem of appoining new team members. They do not have an permissions yet and thus do not fall into the leaders' jurisdiction. The most conversative option would be to require to have a `Moderator` or higher to appoint new team members, though this does not scale well. We therefore allow team leaders, such as users with the `ListAdminstrator` permission, access to the user database.

</div>
