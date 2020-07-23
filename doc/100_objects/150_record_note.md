<div class='panel fade js-scroll-anim' data-anim='fade'>

# Record notes{id=record-note}

Users with `ListHelper` and up permissions can comment on records by leaving record notes. Submitters of records can also add initial notes to records. Each record can have an arbitrary amount of notes, and each note keeps track of who created and subsequently edited it.

| Field        | Type   | Description                                                     |
| ------------ | ------ | --------------------------------------------------------------- |
|id|int|The internal ID of this note|
| author       | string? | The author's username (see [User](#user)). Is `null` if the note was left by the submitter                                               |
| content | string | The comment left |
| editors | List[string] | The usernames of everyone who edited this note, in order of edits|
|transferred| boolean| Value indicating whether this note was originally left on a different record, but later transferred to the current one due to internal record merging |

## Example objects

```json
{
  "author":"stadust",
  "content":"This is a new record note :o",
  "editors":["stadust"],
  "id":3,
  "transferred":false
}
```

</div>