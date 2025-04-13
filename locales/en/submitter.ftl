submitter-banned = Banned
    .yes = Yes
    .no = No

## Record submitter
record-submission-panel = Submit Records
    .info = Note: Please do not submit nonsense, it only makes it harder for us all and will get you banned. Also note that the form rejects duplicate submissions.
    .redirect = Submit a record!

# .note will prefix all notes in the record submission panel
# (not to be confused with record notes)
#
# {$guidelinesRedirect} will be replaced by .guidelines-redirect,
# which is turned into a clickable link to the submission guidelines
record-submission = Record Submission
    .note = Note

    .demon = Demon
    .demon-info = The demon the record was made on. Only demons in the top {$list-size} are accepted. This excludes legacy demons!

    .holder = Holder
    .holder-info = The player holding the record. Start typing to see suggestions of existing players. If this is your first submission, write your name, as you wish it to appear on the website, into the text field (ignoring any suggestions).

    .progress = Progress
    .progress-info = The progress made as percentage. Only values greater than or equal to the demons record requirement and smaller than or equal to 100 are accepted!
    .progress-placeholder = e. g. '50', '98'

    .video = Video
    .video-info = A proof video of the legitimacy of the given record. If the record was achieved on stream, but wasn't uploaded anywhere else, please provide a twitch link to that stream.
    .video-note = Please pay attention to only submit well-formed URLs!
    .video-placeholder = e. g. https://youtu.be/cHEGAqOgddA

    .raw-footage = Raw footage
    .raw-footage-info-a = The unedited and untrimmed video for this completion, uploaded to a non-compressing (e.g. not YouTube) file-sharing service such as google drive. If the record was achieved on stream (meaning there is no recording), please provide a link to the stream VOD
    .raw-footage-info-b = Any personal information possibly contained within raw footage (e.g. names, sensitive conversations) will be kept strictly confidential and will not be shared outside of the demonlist team. Conversely, you acknowledge that you might inadvertently share such information by providing raw footage. You have the right to request deletion of your record note by contacting a list administrator.
    .raw-footage-note = This is required for every record submitted to the list!
    .raw-footage-placeholder = https://drive.google.com/file/d/.../view?usp=sharing

    .notes = Notes or comments
    .notes-info = Provide any additional notes you'd like to pass on to the list moderator receiving your submission.
    .notes-placeholder = Your dreams and hopes for this record... or something like that

    .guidelines = By submitting the record you acknowledge the { -redirect-guidelines(text: "submission guidelines") }.

    .submit = Submit record

## Submitters tab
submitters = Submitters

submitter-manager = Submitter Manager

submitter-viewer = Submitter #
    .welcome = Click on a submitter on the left to get started!

    .info-a = Welcome to the submitter manager. Here you can ban or unban submitters with an absolute revolutionary UI that totally isn't a straight up copy of the player UI, just with even more emptiness.
    .info-b = Banning a submitter will delete all records they have submitted and which are still in the 'submitted' state. All submissions of their which are approved, rejected or under consideration are untouched.

    .records-redirect = Show records in record manager

submitter-listed = Submitter #{$submitter-id}

submitter-idsearch-panel = Search submitter by ID
    .info = Submitters can be uniquely identified by ID. Entering a submitters's ID below will select it on the left (provided the submitter exists)

    .id-field = Submitter ID
    .submit = Find by ID