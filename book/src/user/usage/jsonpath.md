# JSONPath reference

Regardless of whether you want to use `rq`, the `rsonpath-lib` library,
or contribute to the project, you should be familiar with JSONPath, the core
query language we use to process JSONs.

The JSONPath language is defined by
[an IETF specification](https://datatracker.ietf.org/doc/draft-ietf-jsonpath-base/),
currently in draft. The `rsonpath` project implements a subset of the language
according to the spec **with two major differences outlined in
[`rsonpath`-specific behavior]((jsonpath/differences.md))**.

The below reference uses terminology from the spec, but tries to use less dry
language. If you already know the spec, you can probably skip this chapter.

## JSONs as trees

A JSON document is a tree structure, defined in the intuitive way.
A **node** is either an **atomic value**, i.e. a number, string,
`true`, `false`, or `null`, or a **complex value**, i.e. an object
or a list.

An object is a collection of **members** identified by **member names**
or **keys**. Each member name has a single **child node** associated.
A list is an ordered collection of child nodes identified by a zero-based
index.

## Anatomy of a query

A JSONPath query, in essence, defines a pattern that a path in a JSON
must match for the node at that path to be selected. The simplest query
is a sequence of keys.

```jsonpath
$.a.b.c.d
```

It will access the value of the `"a"` key in the root, then the value
under the `"b"` key in that object, then the value under `"c"`,
and finally the value under `"d"`. For example, in the JSON:

```json
{
    "a": { "b": { "c": { "d": 42 } } }
}
```

it will access the value `42` by digging into the structure key by key.

```console
$ rq '$.a.b.c.d' --json '{ "a": { "b": { "c": { "d": 42 } } } }'
42

```

In general, a JSONPath query is a sequence of **segments**. Each segment
contains one or more **selectors**. Canonically, selectors are delimited
within square brackets, but some selectors have a shorthand _dot-notation_.
For example, the query above is equivalent to:

```jsonpath
$['a']['b']['c']['d']
```

```console
$ rq "$['a']['b']['c']['d']" --json '{ "a": { "b": { "c": { "d": 42 } } } }'
42

```

A valid query starts with the `$` character, which represents the root
of the JSON. In particular, the query `$` simply selects the entire document.

## Segments

There are two types of segments:

- **child segment** selects immediate children, or, in other words, digs into
the structure of the document one level deeper. A child segment is either
a bracketed sequence of selectors `[<sel1>, ..., <selN>]`, or a shorthand
dot notation like `.a` or `.*`.

- **descendant segment** selects any subdocument, or, in other words, digs into
the structure of the document at any level deeper. A descendant segment
is either a bracketed sequence of selectors _preceded by two dots_
`..[<sel1>, ..., <selN>]`, or a shorthand double-dot notation like
`..a` or `..*`.

## Selectors

Note that we only cover selectors that are currently supported by `rsonpath`.
Issues to support more selectors can be found under the
[area: selector label](https://github.com/V0ldek/rsonpath/issues?q=is%3Aopen+is%3Aissue+label%3A%22area%3A+selector%22).

### Name selector

The name selector selects the child node under a given member name.
It's most commonly found under its shorthand form, `.key` or `..key`,
which works with simple alphanumeric member names.

In the canonical form, the name has to be enclosed between
single or double quotes, and enables escape sequences.
For example:

- `.a`, `['a']`, `["a"]` all select a child under the key `a`.
- `['"']` selects a child under the key `"`.
- `["'"]` selects a child under the key `'`.
- `['complex name']` selects a child under the key containing a space:

```console
$ rq "$['complex name']" --json '{ "complex name": 42 }'
42

```

### Wildcard selector

The wildcard selector selects any child node, be it under a member name
in an object, or a value in a list. It also has a common shorthand form,
`.*` or `..*`, whereas the canonical form is `[*]`. For example, running
on:

```json
{
    "a": 42,
    "b": [ 1, 2 ]
}
```

the query `$[*]` selects `42`, and `[ 1, 2 ]`.

```console
$ rq '$[*]' --json '{ "a": 42, "b": [ 1, 2 ] }'
42
[ 1, 2 ]

```

Using the descendant selector we can recursively extract elements from the list:

```console
$ rq '$..[*]' --json '{ "a": 42, "b": [ 1, 2 ] }'
42
[ 1, 2 ]
1
2

```

In general, the query `..*` selects _all_ subdocuments of the JSON.
It's not a smart query, as it can create outputs much longer than the source
document itself, consuming a lot of resources.

### Index selector

The index selector selects a value from a list at a given zero-based index.
It only has a bracketed form, `[index]`. For example, running on:

```json
[ 1, 2, 3 ]
```

- the query `$[0]` selects `1`;
- the query `$[1]` selects `2`;
- the query `$[2]` selects `3`; and
- the query `$[3]` selects nothing, since the list has only 3 elements.

```console
$ rq '$[0]' --json "[ 1, 2, 3 ]"
1

```

```console
$ rq '$[1]' --json "[ 1, 2, 3 ]"
2

```

```console
$ rq '$[2]' --json "[ 1, 2, 3 ]"
3

```

```console
$ rq '$[3]' --json "[ 1, 2, 3 ]"

```

## Combining segments

Segments can be chained arbitrarily to create complex queries.
For example, if we have a file `ex.json`

```json
{{#include jsonpath.in/ex.json}}
```

we can extract all phone numbers with:

```console
$ rq '$..phoneNumbers[*].number' ./ex.json
"0123-4567-8888"
"0123-4567-8910"
"0123-4567-9999"
"0123-4567-8910"

```

Note that each part of the query is needed here:

- the first segment is descendant, so that we pick up both the root's
number array and the one under "spouse";
- without specifying the "phoneNumbers" key (for example running `$..number`)
we wouldn't be able to filter out the two irrelevant "number" keys;
- the wildcard selector `[*]` makes sure we select all the numbers,
regardless of how long the list may be.

## Selector availability

Not all of JSONPath's functionality is supported by `rsonpath` as of right now.

### Supported segments

| Segment                        | Syntax                           | Supported | Since  | Tracking Issue |
|--------------------------------|----------------------------------|-----------|--------|---------------:|
| Child segment (single)         | `[<selector>]`                   | ✔️        | v0.1.0 |                |
| Child segment (multiple)       | `[<selector1>,...,<selectorN>]`  | ❌        |        |                |
| Descendant segment (single)    | `..[<selector>]`                 | ✔️        | v0.1.0 |                |
| Descendant segment (multiple)  | `..[<selector1>,...,<selectorN>]`| ❌        |        |                |

### Supported selectors

| Selector                                 | Syntax                           | Supported | Since  | Tracking Issue |
|------------------------------------------|----------------------------------|-----------|--------|---------------:|
| Root                                     | `$`                              | ✔️        | v0.1.0 |                |
| Name                                     | `.<member>`, `[<member>]`        | ✔️        | v0.1.0 |                |
| Wildcard                                 | `.*`, `..*`, `[*]`               | ✔️        | v0.4.0 |                |
| Index (array index)                      | `[<index>]`                      | ✔️        | v0.5.0 |                |
| Index (array index from end)             | `[-<index>]`                     | ❌        |        |                |
| Array slice (forward, positive bounds)   | `[<start>:<end>:<step>]`         | ❌        |        | [#152](https://github.com/V0ldek/rsonpath/issues/152) |
| Array slice (forward, arbitrary bounds)  | `[<start>:<end>:<step>]`         | ❌        |        |                |
| Array slice (backward, arbitrary bounds) | `[<start>:<end>:-<step>]`        | ❌        |        |                |
| Filters &ndash; existential tests        | `[?<path>]`                      | ❌        |        | [#154](https://github.com/V0ldek/rsonpath/issues/154) |
| Filters &ndash; const atom comparisons   | `[?<path> <binop> <atom>]`       | ❌        |        | [#156](https://github.com/V0ldek/rsonpath/issues/156) |
| Filters &ndash; logical expressions      | `&&`, `\|\|`, `!`                | ❌        |        |                |
| Filters &ndash; nesting                  | `[?<expr>[?<expr>]...]`          | ❌        |        |                |
| Filters &ndash; arbitrary comparisons    | `[?<path> <binop> <path>]`       | ❌        |        |                |
| Filters &ndash; function extensions      | `[?func(<path>)]`                | ❌        |        |                |
