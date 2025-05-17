player-banned = Banned
    .yes = Yes
    .no = No

player-nationality = Nationality
    .info = Note that this is to be understood as 'Country of legal residency' and nothing else. No exceptions.
    .none = None

player-subdivision = Political Subdivision
    .none = None

## Players tab
players = Players

player-manager = Player Manager

player-viewer = Player #
    .welcome = Click on a player on the left to get started!

    .info = Welcome to the player manager. Here you can ban or unban players. Banning a player will delete all records of theirs which are in the submitted or under consideration state. All approved records will instead be set to rejected.
    .records-redirect = Show records in record manager

player-idsearch-panel = Search player by ID
    .info = Players can be uniquely identified by ID. Entering a players's ID below will select it on the left (provided the player exists)
    .id-field = Player ID:

    .submit = Find by ID

    .id-validator-valuemissing = Player ID required

player-name-dialog = Change player name
    .info = Change the name of this player. This will update their name on every one of their records. If a player with the new name already exists, the player objects will be merged, with the new object receiving the ID of the player you are currently editing. In this case, the record lists of the players are merged and their creator/verifier/publisher information is updated. Internally, each record is moved to to the new player, an on conflicts the same rules apply as when editing a record's holder.
    .name-field = Name:

    .submit = Edit

    .name-validator-valuemissing = Please provide a name for the player

## List integration tab
list-integration = List Integration

claimed-player = Claimed Player
    .verified = Verified
    .unverified = Unverified

# .info-api-link is turned into a clickable link to the geolocation API
# pointercrate uses, and replaces { $info-api-link }
claim-geolocate = Geolocate statsviewer flag
    .info = Clicking the above button let's you set your claimed player's statsviewer flag via IP Geolocation. To offer this functionality, pointercrate uses { $info-api-link }. Clicking the above button also counts as your consent for pointercrate to send your IP to abstract.
    .info-api-link = abstract's IP geolocation API

    .submit = Go

    .edit-success = Set nationality to { $nationality }
    .edit-success-subdivision = Set nationality to { $nationality }/{ $subdivision }

claim-lock-submissions = Lock submissions
    .info = Whether submissions for your claimed player should be locked, meaning only you will be able to submit records for your claimed player (and only while logged in to this account holding the verified claim)

    .edit-success = Successfully applied change

claim-records = Your claimed player's records
    .info = A list of your claimed player's records, including all under consideration and rejected records and all submissions. Use this to track the status of your submissions. Clicking on a record will pull up any public notes a list mod left on the given record. The background color of each record tells you whether the record is { $record-approved-styled }, { $record-submitted-styled }, { $record-rejected-styled } or { $record-underconsideration-styled }.

    .record-notes = Notes for record { $record-id }:
    .record-notes-none = No public notes on this record!

claim-manager = Manage Claims
    .info-a = Manage claims using the interface below. The list can be filtered by player and user using the panels on the right. Invalid claims should be deleted using the trash icon.
    .info-b = To verify a claim, click the checkmark. Only verify claims you have verified to be correct (this will probably mean talking to the player that's being claimed, and asking if they initiated the claim themselves, or if the claim is malicious).
    .info-c = Once a claim on a player is verified, all other unverified claims on that player are auto-deleted. Users cannot put new, unverified claims on players that have a verified claim on them.
    .info-d = A claim with a green background is verified, a claim with a blue background is unverified/unchecked.

    .claim-no-records = The claimed player ({ $player-id }) does not have an approved record on the list

claim-listed-user = Claim by user:
claim-listed-player = Claim on player:

claim-initiate-panel = Initiate Claim
    .info = Select the player you wish to claim below

# { $discord } is replaced by .info-discord, which is turned into a
# clickable link to Pointercrate Central by default (this can be modified
# in pointercrate-example/src/main.rs)
claim-info-panel = Claiming 101
    .info-a = Player claiming is the process of associated a demonlist player with a pointercrate user account. A verified claim allows you to to modify some of the player's properties, such as nationality.
    .info-b = To initiate a claim, click the pen left of the 'Claimed Player' heading. Once initiated, you have an unverified claim on a player. These claims will then be manually verified by members of the pointercrate team. You can request verification in { $discord }.
    .info-c = You cannot initiate a claim on a player that already has a verified claim by a different user on it.

    .info-discord = this discord server

claim-video-panel = Record video
    .info = Clicking a claim in the 'Manage Claims' panel will pull up a random video of an approved record by the claimed player.