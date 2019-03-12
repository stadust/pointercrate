% users

<div class='panel fade js-scroll-anim' data-anim='fade'>

# Who has access to whom?

Generally, few people are supposed to have access to user information. People with the `Moderator` and `Administrator` permissions always do. But there are a few cases where people with fewer permissions can gain access. Generally, the leader(s) of a specific team should have access to their team members. We'll now say the members of their team fall into their jurisdiction. So for example, a `ListAdministrator` has access to users with the `ListHelper` and `ListModerator` permissions. However, this does not solve the problem of appoining new team members. They do not have an permissions yet and thus do not fall into the leaders' jurisdiction. The most conversative option would be to require to have a `Administrator` appoint new team members. In the future it is planned to have people be able to request permissions from a team leader, which they can then grant or deny. That way, `ListAdministrators` will not have access to the entire user database, while still being able to completely manage their own team.

**Note**: Due to time constraints during implementation, the jurisdiction system is not implemented. The documentation describes how the endpoints _would work if it was implemented_. In reality, right now, all access to the user database needs to happen from an Administrator or Moderator account.  

</div>
