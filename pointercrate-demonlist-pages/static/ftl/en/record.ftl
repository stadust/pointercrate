## Commonly referenced record data
record-submitted = Submitted
record-underconsideration = Under Consideration
record-approved = Approved
record-rejected = Rejected

record-videolink = Video Link
record-videoproof = Video Proof
record-rawfootage = Raw Footage
record-demon = Demon
record-holder = Record Holder
record-progress = Progress
record-submitter = Submitter ID

## Records tab (user area)
records = Records
record-manager = Record Manager
    .all-option = All Demons

record-listed = Record #{ $record-id }
    .progress = { $percent }% on { $demon }

record-viewer = Record #
    .welcome = Click on a record on the left to get started!
    .delete = Delete Record

    .copy-data-success = Copied record data to clipboard!
    .copy-data-error = Error copying to clipboard

    .confirm-delete = Are you sure? This will irrevocably delete this record and all notes made on it!

record-note = Add Note
    .placeholder = Add note here. Click 'Add' above when done!
    .public-checkbox = Public note

    .submit = Add

record-note-listed = Record Note #{ $note-id }
    .confirm-delete = This action will irrevocably delete this note. Proceed?

    .author = This note was left by { $author }.
    .author-submitter = This note was left as a comment by the submitter.
    .editors = This note was subsequently modified by: { $editors }.
    .transferred = This note was not originally left on this record.
    .public = This note is public.

record-status-filter-panel = Filter
    .info = Filter by record status

record-status-filter-all = All

record-idsearch-panel = Search record by ID
    .info = Records can be uniquely identified by ID. Entering a record's ID below will select it on the left (provided the record exists)
    .id-field = Record ID:

    .submit = Find by ID

    .id-validator-valuemissing = Record ID required

record-playersearch-panel = Filter by player
    .info = Players can be uniquely identified by name and ID. Entering either in the appropriate place below will filter the view on the left. Reset by clicking "Find ..." when the text field is empty.

    .id-field = Player ID:
    .id-submit = Find by ID

    .name-field = Player name:
    .name-submit = Find by name

# Record viewer dialogs
record-videolink-dialog = Change video link
    .info = Change the video link for this record. Note that as a list mod, you can leave the text field empty to remove the video from this record.
    .videolink-field = Video link:

    .submit = Edit

    .videolink-validator-typemismatch = Please enter a valid URL

record-demon-dialog = Change record demon
    .info = Change the demon associated with this record. Search up the demon this record should be associated with below. Then click it to modify the record

record-holder-dialog = Change record holder
    .info = Type the new holder of the record into the text field below. If the player already exists, it will appear as a suggestion below the text field. Then click the button below.
    .submit = Edit

record-progress-dialog = Change record progress
    .info = Change the progress value of this record. Has to be between the demon's record requirement and 100 (inclusive).
    .progress-field = Progress:

    .submit = Edit

    .progress-validator-rangeunderflow = Record progress cannot be negative
    .progress-validator-rangeoverflow = Record progress cannot be larger than 100%
    .progress-validator-badinput = Record progress must be a valid integer
    .progress-validator-stepmismatch = Record progress mustn't be a decimal
    .progress-validator-valuemissing = Please enter a progress value

# The giant information box below the record manager, split
# into different sections here
#
# Each section (except .a and .b) will begin with a bolded version of
# the appropriate record state, or a bolded version of .note for .note-a/b
# attributes
#
record-manager-help = Manage Records
    .a = Use the list on the left to select records for editing/viewing. Use the panel on the right to filter the record list by status, player, etc.. Clicking the { record-status-filter-all } field at the top allows to filter by demon.

    .b = There are four possible record states a record can be in: { record-rejected }, { record-approved }, { record-submitted } and { record-underconsideration }. For simplicity of explanation we will assume that Bob is a player and Cataclysm is a demon he has a record on.

    .rejected = If the record is { record-rejected }, it means that Bob has no other record in other states on Cataclysm and no submissions for Bob on Cataclysm are possible. Conversely, this means if Bob has a record on Catalysm thats not rejected, we immediately know that no rejected record for Bob on Cataclysm exists.
    Rejecting any record of Bobs on Cataclysm will delete all other records of Bob on Cataclysm to ensure the above uniqueness.

    .approved = If the record is { record-approved }, it means that no submissions with less progress than the { record-approved } record exist or are permitted.
    Changing a record to { record-approved } will delete all submissions for Bob on Cataclysm with less progress.

    .submitted = If the record is { record-submitted }, no further constraints on uniqueness are in place. This means that multiple submissions for Bob on Cataclysm are possible, as long as they provide different video links. However, due to the above, all duplicates are deleted as soon as one of the submissions is accepted or rejected.

    .underconsideration = If the record is { record-underconsideration } it is conceptually still a submission. The only difference is, that no more submissions for Bob on Cataclysm are allowed now.

    .note = Note

    .note-a = If a player is banned, they cannot have { record-approved }/{ record-submitted } records on the list. All records marked as { record-submitted } are deleted, all others are changed to { record-rejected }.

    .note-b = Banning a submitter will delete all their submissions that still have the status { record-submitted }. Records submitted by them that were already { record-approved }/{ record-rejected } will not be affected.