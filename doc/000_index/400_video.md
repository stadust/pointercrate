<div class='panel fade js-scroll-anim' data-anim='fade'>

# External Videos {id=video}

## Valid video formats

Pointercrate only accepts videos from a specific set of hosting services. It further normalizes all videos from a given host
into one specific URL format and ensures that every video link actually leads to a valid video. All query parameters, including timestamps on youtube videos,
are stripped from URLs.

If a host you want to see supported is missing or a URL format for one of the provided hosts is missing, please open an issue on the GitHub repository.

Please note that pointercrate asynchronously performs `HEAD` requests to any video URL submitted and discards any that don't return a successful response.

The accepted URL formats are:

| Video host | URL formats                               |
| ---------- | ----------------------------------------- |
| YouTube    | `http[s]://www.youtube.com/watch?v={id}`  |
| YouTube    | `http[s]://m.youtube.com/watch?v={id}`    |
| YouTube    | `http[s]://youtube.com/watch?w={id}`      |
| YouTube    | `http[s]://youtu.be/{id}`                 |
| Twitch     | `http[s]://www.twitch.tv/videos/{id}`     |
| Twitch     | `http[s]://twitch.tv/videos/{id}`         |
| Twitch     | `http[s]://www.twitch.tv/{name}/v/{id}`   |
| Twitch     | `http[s]://twitch.tv/{name}/v/{id}`       |
| Everyplay  | `http[s]://www.everyplay.com/videos/{id}` |
| Everyplay  | `http[s]://everyplay.com/videos/{id}`     |
| Vimeo      | `http[s]://www.vimeo.com/{id}`            |
| Vimeo      | `http[s]://vimeo.com/{id}`                |
| Bilibili   | `http[s]://www.bilibili.com/video/{id}`   |
| Bilibili   | `http[s]://bilibili.com/video/{id}`       |

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

| Status code | Error code | Description                                                                                                                                        | Data                                              |
| ----------- | ---------- | -------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------- |
| 422         | 42222      | Invalid protocol encountered while processing an URL. Only `http` and `https` are supported                                                        | `-`                                               |
| 422         | 42223      | Authentication information was discovered while processing an URL                                                                                  | `-`                                               |
| 422         | 42224      | An unknown/unsupported video host has been discovered while processing an URL (no, pornhub is no acceptable host, what is wrong with you people??) | `-`                                               |
| 422         | 42225      | The video URL does not match the expected format for the given host                                                                                | `expected`: The expected URL format for this host |

</div>
