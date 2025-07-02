## Demon information, including information fetched by dash-rs
## Fields included in forms may have validators
demon-name = Demon Name
    .validator-valuemissing = Please specify a name

demon-password = Level Password

demon-id = Level ID
    .validator-rangeunderflow = Level ID must be positive

demon-length = Level Length

demon-objects = Object Count

demon-difficulty = In-Game Difficulty

demon-gdversion = Created In

demon-ngsong = Newgrounds Song

demon-score = Demonlist score ({$percent}%)

demon-video = Verification Video
    .validator-typemismatch = Please enter a valid URL

demon-thumbnail = Thumbnail
    .validator-typemismatch = Please enter a valid URL
    .validator-valuemissing = Please enter a URL

demon-position = Position
    .validator-rangeunderflow = Demon position must be at least 1
    .validator-badinput = Demon position must be a valid integer
    .validator-stepmismatch = Demon position mustn't be a decimal
    .validator-valuemissing = Please specify a position

demon-requirement = Requirement
    .validator-rangeunderflow = Record requirement cannot be negative
    .validator-rangeoverflow = Record requirement cannot be larger than 100%
    .validator-badinput = Record requirement must be a valid integer
    .validator-stepmismatch = Record requirement mustn't be a decimal
    .validator-valuemissing = Please specify a requirement value

demon-publisher = Publisher
    .validator-valuemissing = Please specify a publisher

demon-verifier = Verifier
    .validator-valuemissing = Please specify a verifier

demon-creators = Creators

demon-headline-by = by { $creator }
demon-headline-verified-by = verified by { $verifier }
demon-headline-published-by = published by { $publisher }

# { $verified-and-published } represents two possible variations of text
# either .same-verifier-publisher OR .unique-verifier-publisher
#
# { $more } in .more-creators is transformed into a tooltip listing all of
# a demon's creators, with the text being .more-creators-tooltip
demon-headline = by { $creator }
    .same-verifier-publisher = verified and published by { $publisher }
    .unique-verifier-publisher = { demon-headline-published-by }, { demon-headline-verified-by }

    .no-creators = by Unknown, { $verified-and-published }

    .one-creator = { demon-headline-by }, { $verified-and-published }
    .one-creator-is-publisher = { demon-headline-by }, verified by { $verifier }
    .one-creator-is-verifier = { demon-headline-by }, published by { $publisher }

    .two-creators = by { $creator1 } and { $creator2 }, { $verified-and-published }

    .more-creators = { demon-headline-by } and { $more }, { $verified-and-published }
    .more-creators-tooltip = more

## Position history table
movements = Position History
    .date = Date
    .change = Change

movements-newposition = New Position
    .legacy = Legacy

movements-reason = Reason
    .added = Added to list
    .addedabove = { $demon } was added above
    .moved = Moved
    .movedabove = { $demon } was moved up past this demon
    .movedbelow = { $demon } was moved down past this demon

## Records table
demon-records = Records

demon-records-qualify = {$percent}% { $percent ->
    [100] required to qualify
    *[other] or better required to qualify
}

demon-records-total = {$num-records} { $num-records ->
    [one] record registered
    *[other] records registered
}, out of which {$num-completions} { $num-completions ->
    [one] is 100%
    *[other] are 100%
}

## Demons tab in user area
demons = Demons
demon-manager = Demon Manager

demon-listed = {$demon} (ID: {$demon-id})
    .publisher = by {$publisher}

demon-viewer = Demon #
    .welcome = Click on a demon on the left to get started!

    .video-field = { demon-video }:
    .thumbnail-field = { demon-thumbnail }:
    .position-field = { demon-position }:
    .requirement-field = { demon-requirement }:
    .publisher-field = { demon-publisher }:
    .verifier-field = { demon-verifier }:
    .creators-field = { demon-creators }:

demon-add-panel = Add Demon
    .button = Add a demon!

# Demon addition form
demon-add-form = Add Demon
    .name-field = { demon-name }:
    .name-validator-valuemissing = Please provide a name for the demon

    .levelid-field = Geometry Dash Level ID:
    .position-field = { demon-position }:
    .requirement-field = { demon-requirement }:
    .verifier-field = { demon-verifier }:
    .publisher-field = { demon-publisher }:
    .video-field = { demon-video }:
    .creators-field = { demon-creators }:

    .submit = Add Demon

    .edit-success = Successfully added demon!

# Demon viewer dialogs
demon-video-dialog = Change verification video link
    .info = Change the verification video link for this record. Leave empty to remove the verification video.
    .video-field = Video link:
    .submit = Edit

demon-name-dialog = Change demon name
    .info = Change the name of this demon. Multiple demons with the same name ARE supported!
    .name-field = Name:
    .submit = Edit

# { $video-id } will be replaced by https://i.ytimg.com/vi/{.info-videoid}/mqdefault.jpg but italicized
# in english, this looks like https://i.ytimg.com/vi/VIDEO_ID/mqdefault.jpg
demon-thumbnail-dialog = Change thumbnail link
    .info = Change the thumbnail link for this record. To link it to the thumbnail of a youtube video, set it to { $video-id }.
    .info-videoid = VIDEO_ID

    .thumbnail-field = Thumbnail link:
    .submit = Edit

demon-position-dialog = Change demon position
    .info = Change the position of this demon. Has be be greater than 0 and be at most the current list size.
    .position-field = Position:
    .submit = Edit

demon-requirement-dialog = Change demon requirement
    .info = Change the record requirement for this demon. Has be lie between 0 and and 100 (inclusive).
    .requirement-field = Requirement:
    .submit = Edit

demon-publisher-dialog = Change demon publisher
    .info = Type the new publisher of the demon into the text field below. If the player already exists, it will appear as a suggestion below the text field. Then click the button below.
    .submit = Edit

demon-verifier-dialog = Change demon verifier
    .info = Type the new verifier of the demon into the text field below. If the player already exists, it will appear as a suggestion below the text field. Then click the button below.
    .submit = Edit

demon-creator-dialog = Add creator
    .info = Type the creator to add to this demon into the text field below. If the player already exists, it will appear as a suggestion below the text field. Then click the button below.
    .submit = Add Creator

    .edit-success = Successfully added creator!