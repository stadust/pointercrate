<div class='panel fade js-scroll-anim' data-anim='fade'>

# Pagination and Filtering{id=pagination}

Some endpoints in the pointercrate API support or require pagination due to the potentially huge amount of data they can return.
This mostly applies to the endpoints that return lists of objects, like [`GET /records/`](/documentation/records/#get-records).

Objects returned by endpoints supporting pagination are totally ordered by an ID field, which is specified in the endpoint's documentation.

If an endpoint supports pagination, it's documentation will contain a notice similar to this one:

<div class='info-green'>
<b>Pagination:</b><br>
This endpoint supports [pagination and filtering](#pagination) via query parameters. Please see the documentation on pagination for information
on the additional request and response headers.
</div>

## Pagination Query Parameters

Pagination is done via specific query parameters, which tell pointercrate which part of the result set to return.

Note that there is no way to get the total amount of pages, as both page bounds and size can be chosen abitrarily.

| Query Parameter | Description                                                                                         | Default                               |
| --------------- | --------------------------------------------------------------------------------------------------- | ------------------------------------- |
| limit           | The maximum amount of object to return. Must lie between `1` and `100`                              | `50`                                    |
| after           | The id of the last object on the previous page, thus specifying the start point of the current page | `null` |
| before          | The id of the first object on the next page, thus specifying the end point of the current page      | `null`  |

Omitting `before` or `after`, which implicitly sets them to `null`, makes the server act like they're set to negative/positive infinity respectively.

## Pagination Response Headers

Paginatable endpoints provide the `Links` header to simply access to the next, previous, first and last page, using the `limit` set on the request.
The header is set to a comma-seperated
list of links in the form `<[link]>; rel=[page]`, where page is one of `next`, `prev`, `first` or `last`.

Note that the `next` and `prev` links are only provided if there actually is a next or previous page of results respectively. The server always provides the `first` and `last` links.

## Filtering

Most endpoints that support pagination also support filtering their results beyond simply using the pagination parameters.

If this is supported, the documentation specifies the filterable fields for a given endpoint.
It is then possible to specify conditions in the query string, which the returned objects must meet.

There are two ways of filtering the result set:

- **Filtering by equality**: The objects returned can be filtered by a specific field's value by specifying the field and a value in the query string, i.e. `/api/v1/players/?banned=true`
- **Filtering by inequality**: The objects returned can be filtered by whether a field is smaller/greater than a specific value by specifying the field,
  suffixed with either `__lt` or `__gt`, and the value to check for inequality against in the query string, i.e. `/api/v1/records/?progress__gt=75`. Note that this doesn't work for all fields (since a lexicographical filtering on the record status hardly seems useful)
- **Filtering by infix**: Some string-values fields support filtering objects where said string field contains a specific infix. This is done by suffixing the field's name with `_contains`.

Multiple conditions can be combined, i.e. `/api/v1/records/?after=200&limit=10&status=APPROVED&progress__lt=100`. This request would return the first 10 approved records with a record ID greater than 200 and a progress less than 100.

Note that filtering explicitly on the ID field is *not* possible. You have to use the special `before` and `after` parameters for that. You also cannot use equality filtering on the ID field. Use the specific endpoint for retrieving single objects instead.

### Errors:

These error conditions can occur at any endpoint supporting pagination and are thus not listed specifically for each of them.

| Status code | Error code | Description                                                     |
| ----------- | ---------- | --------------------------------------------------------------- |
| 422         | 42207      | The `limit` parameter is smaller than `1` or greater than `100` |

</div>
