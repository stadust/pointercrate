## Demon information, including information fetched by dash-rs
demon-password = Level Password
demon-id = Level ID
demon-length = Level Length
demon-objects = Object Count
demon-difficulty = In-Game Difficulty
demon-gdversion = Created In
demon-ngsong = Newgrounds Song

demon-score = Demonlist score ({$percent}%)

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
records = Records

records-qualify = {$percent}% { $percent ->
    [100] required to qualify
    *[other] or better required to qualify
}

records-total = {$numRecords} { $numRecords ->
    [one] record registered
    *[other] records registered
}, out of which {$numCompletions} { $numCompletions ->
    [one] is 100%
    *[other] are 100%
}
