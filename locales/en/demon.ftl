## Demon information, including information fetched by dash-rs
demon-password = Level Password
demon-id = Level ID
demon-length = Level Length
demon-objects = Object Count
demon-difficulty = In-Game Difficulty
demon-gdversion = Created In
demon-ngsong = Newgrounds Song

demon-score = Demonlist score ({$percent}%)

demon-video = Verification Video
demon-thumbnail = Thumbnail
demon-position = Position
demon-requirement = Requirement
demon-publisher = Publisher
demon-verifier = Verifier
demon-creators = Creators

## Position history table
movements = Position History
    .date = Date
    .change = Change
    .newposition = New Position

movement-reason = Reason
    .added = Added to list
    .addedabove = {$demon} was added above
    .moved = Moved
    .movedabove = {$demon} was moved up past this demon
    .movedbelow = {$demon} was moved down past this demon

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

demon-add-panel = Add Demon
    .button = Add a demon!

# Demon addition form
demon-add-form = Add demon
    .name-field = Demon name
    .levelid-field = Geometry Dash Level ID
    .position-field = Position
    .requirement-field = Requirement
    .verifier-field = Verifier
    .publisher-field = Publisher
    .video-field = Verification Video
    .creators-field = Creators

    .submit = Add Demon

# Demon viewer dialogs
demon-video-dialog = Change verification video link
    .info = Change the verification video link for this record. Leave empty to remove the verification video.
    .video-field = Video link
    .submit = Edit

demon-name-dialog = Change demon name
    .info = Change the name of this demon. Multiple demons with the same name ARE supported!
    .name-field = Name
    .submit = Edit

# {$videoId} will be replaced by https://i.ytimg.com/vi/{.info-videoid}/mqdefault.jpg, italicized
# in english, this looks like https://i.ytimg.com/vi/VIDEO_ID/mqdefault.jpg
demon-thumbnail-dialog = Change thumbnail link
    .info = Change the thumbnail link for this record. To link it to the thumbnail of a youtube video, set it to {$video-id}.
    .info-videoid = VIDEO_ID

    .thumbnail-field = Thumbnail link
    .submit = Edit

demon-position-dialog = Change demon position
    .info = Change the position of this demon. Has be be greater than 0 and be at most the current list size.
    .position-field = Position
    .submit = Edit

demon-requirement-dialog = Change demon requirement
    .info = Change the record requirement for this demon. Has be lie between 0 and and 100 (inclusive).
    .requirement-field = Requirement
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