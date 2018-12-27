<div class='panel fade js-scroll-anim' data-anim='fade'>

# External Videos {id=video}

## Valid video formats

Pointercrate only accepts videos from a specific set of hosting services. It further normalizes all videos from a given host
into one specific URL format and ensures that every video link actually leads to a valid video. All query parameters, including timestamps on youtube videos,
are stripped from URLs.

The accepted URL formats are:

| Video host | URL formats                              |
| ---------- | ---------------------------------------- |
| YouTube    | `http[s]://www.youtube.com/watch?v={id}` |
| YouTube    | `http[s]://m.youtube.com/watch?v={id}`   |
| YouTube    | `http[s]://youtu.be/{id}`                |
| Twitch     | `http[s]://www.twitch.tv/videos/{id}`    |
| Twitch     | `http[s]://www.twitch.tv/{name}/v/{id}`  |
| Everyplay  | `http[s]://everyplay.com/videos/{id}`    |
| Vimeo      | `http[s]://vimeo.com/{id}`               |
| Bilibili   | `http[s]://www.bilibili.com/video/{id}`  |

They are normalized into the following:

| Video host | URL format                             |
| ---------- | -------------------------------------- |
| YouTube    | `https://www.youtube.com/watch?v={id}` |
| Twitch     | `https://www.twitch.tv/videos/{id}`    |
| Everyplay  | `https://everyplay.com/videos/{id}`    |
| Vimeo      | `https://vimeo.com/{id}`               |
| Bilibili   | `https://www.bilibili.com/video/{id}`  |

### Errors

These error conditions can occur at any endpoint expecting a video URL and are thus not listed specifically for each of them.

| Status code | Error code | Description                                                                             |
| ----------- | ---------- | --------------------------------------------------------------------------------------- |
| 400         | 40004      | The video URL does not meet the above requirements                                      |
| 400         | 40005      | The video URL has a malformed query string. This could, for example, be a trailing `&t` |

</div>
