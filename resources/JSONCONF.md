# Configuration

With this we can configure the transrichment passes.

On the upper most level, a configuration consists of an array of "transrichment passes"

```jsonc
[
    {
        "comment": "Pass1, where we do, xyz",   // 1) (optional)
        "transformers": [],                     // 2) (mandatory)
        "orderItems": [                         // 3) (optional)
            { "from": 3, "to": 0 },             // 3.1) (mandatory)    
            ...
        ]                        
    },
    {...}
]

```

1. An optional comment / description of this transrichment pass (configuration).
2. An array of transformers that makes up this transrichment pass.
3. Every enrichment pass can have an optional `orderItems` array. This used to re-order/re-assign column indices. **If you have re-ordered/re-assigned column indices in enrichmentPass N, enrichmentPass N+1 will work on the newly re-ordered/re-assigned column indices!**

## About column indices

Since we heavily operate on indices, here are some things to know and keep in mind:

* Indices are **only** virtual! Meaning, re-ordering/re-assigning indices is rather cheap and you should make us of it, if it makes your configuration more readable. In essence, indices are more like a second, numerical, header.
* This also means, that you can "park" / move columns out of the way. E.g. with configuration, where a lot of splitting has to be done, the split values can be "parked" somewhere (let's say, starting at index 100) and then later on, re-arranged with the `orderItems` subconfig. This way it's easier not to conflict with existing columns and their indices.

## `transformers` - Available transformers

There are several types of transformers available:

1. `splitItem`
2. `deleteItems`
3. `addItem`

### `splitItem` transformer

A transformer to split a column into two (new) columns.

```jsonc
{
    "type": "splitItem",            // 1) (mandatory)
    "cfg": {
        "idx": 3,                   // 2) (mandatory)
        "spec": {...},              // 3) (mandatory)
        "deleteAfterSplit": true,   // 4) (mandatory)
        "targetLeft": {             // 5) (mandatory)
            "idx": 3,               // 5.1) (mandatory)
            "header": "currency",   // 5.2) (optional)
            "targetType": "String"  // 5.3) (mandatory)
        },
        "targetRight": {            // 6) (mandatory)
            "idx": 4,               // 6.1) (mandatory)
            "header": "value",      // 6.2) (optional)
            "targetType": "Decimal" // 6.3) (mandatory)
        }
    }
}

```

1. The type (name) of transfomer to use. `splitItem` in this case.
2. The index (0-based) of the source column, that is to be split.
3. The "spec" (specification, or specialization). A subconfig for this specific type of splitter implementation. (Currently there are: `separatorChar` and `pattern`. See below.)
4. Whether to delete the source column or not. Deleting makes the index re-usable instantly.
5. The config for the "left" side of the split.
    1. The target index for the "left" value. MUST NOT be already in use!
    2. An optional header for this (new) column.
    3. The target type of the value.
6. The config for the "right" side of the split.
    1. The target index for the "right" value. MUST NOT be already in use!
    2. An optional header for this (new) column.
    3. The target type of the value.

#### `separatorChar` splitter spec

A splitter that splits at a designated character (e.g. `' '`, or `';'`), into a `left` and `right` portion.

```jsonc
{
    ...
    "spec": {
        "name": "separatorChar",    // 1) (mandatory)
        "char": " "                 // 2) (mandatory)
        "splitNone": true           // 3) (optional)
    },
    ...
}

```

1. The name (type) of the specific splitter implementation.
2. The character to split at.
3. Information, if a `None` (our representation for an absent value) should be split as well. This is useful, if this column is "nullable" and can have empty/blank/absent values. In this case, we will split a `None` in two `None`s. (Note: This defaults to `true`, if not specified!)

#### `pattern` splitter spec

A splitter that splits, using a given (rust-style) regex pattern, into left and right. Therefore, the pattern MUST make use of two pattern groups.

```jsonc
{
    ...
    "spec": {
        "name": "pattern",                                  // 1) (mandatory)
        "pattern": "(\\d+\\.\\d+) \\(([[:alpha:]].+)\\)"    // 2) (mandatory)
        "splitNone": true                                   // 3) (optional)
    },
    ...
}

```

1. The name (type) of the specific splitter implementation.
2. The pattern to use for splitting. Must be valid rust regex AND be escaped.
3. Information, if a `None` (our representation for an absent value) should be split as well. This is useful, if this column is "nullable" and can have empty/blank/absent values. In this case, we will split a `None` in two `None`s. (Note: This defaults to `true`, if not specified!)

### `deleteItems` transformer

A transformer that deletes column(s) ("items").

```jsonc
{
    "type": "deleteItems",  // 1) (mandatory)
    "cfg": [1,2]            // 2) (mandatory)
}
```

1. The type (name) of transfomer to use. `deleteItems` in this case.
2. A simple array of indices to delete.

### `addItem` transformer

A transformer that adds a column.

```jsonc
{
    "type": "addItem",                  // 1) (mandatory)
    "cfg": {
        "spec": {...},                  // 2) (mandatory)
        "target": {
            "idx": 5,                   // 3) (mandatory)
            "header": "report_date",    // 4) (optional)
            "targetType": "NaiveDate"   // 5) (mandatory)
        }
    }
}

```

1. The type (name) of transfomer to use. `addItem` in this case.
2. The "spec" (specification, or specialization). A subconfig for this specific type of add implementation. (Currently there are: `static`, `meta`, `runtime` and `runtimeStateful`. See below.)
3. The index of where to add this column to
4. An optional header for this newly added column
5. The target type of the value.

#### `static` addItem spec

The `static` addItem is the most straight forward one. All it does is add a column, where every value/cell of the column has the same static (fixed) value. This is useful, for enriching simple things, that are known ahead of time. E.g. a report's name or a region, or things like that.

```jsonc
{
    ...
    "spec": {
        "name": "static",   // 1) (mandatory)
        "value": "Europe"   // 2) (mandatory)
    },
    ...
}

```

1. The name (type) of the addItem spec. `static` in this case.
2. The (stringified) value to enrich. This will be typed though, in the `target` object that follows the `spec` object. (see above)

#### `meta` addItem spec

The `meta` addItem _**only**_ works in conjunction with a given metadata HashMap at _**runtime**_! The `meta` addItem will use the `key` as the key to the metadata HashMap and the value will be, whatever the value is in the metadata HashMap.

```jsonc
{
    ...
    "spec": {
        "name": "meta",         // 1) (mandatory)
        "key": "report_date"    // 2) (mandatory)
    },
    ...
}

```

1. The name (type) of the addItem spec. `meta` in this case.
2. The key to use, to look up the value in the metadata HashMap. The found value will be used then for each cell of the column as the value.

#### `runtime` addItem spec

The `runtime` addItem can add certain predefined things/values at runtime.

Currently the only implemented runtime value (`rtValue`) is `CurrentDateTimeUtcAsFixedOffset`, which will add the current dateTime as UTC. (And technically, it will do so as a `DateTime<FixedOffset>` with offset=0, i.e. UTC.)

```jsonc
{
    ...
    "spec": {
        "name": "runtime",                              // 1) (mandatory)
        "rtValue": "CurrentDateTimeUtcAsFixedOffset",   // 2) (mandatory)
        "asSingleton": true                             // 3) (optional)
    },
    "target": {
        ...
        "targetType": "DateTime"                        // 4) (mandatory)
    }
}

```

1. The name (type) of the addItem spec. `runtime` in this case.
2. The type of runtime value to enrich, `CurrentDateTimeUtcAsFixedOffset` in this case.
3. Should the value be created once and then be used, or newly created for every "cell"? In the case of `CurrentDateTimeUtcAsFixedOffset` this means that:
    1. when `true`, each cell/row of this column will have the same dateTime
    2. when `false`, each cell/rows of this column can potentially have a different dateTime, depending on how fast things are happening.
4. It is good paractive to the set correct `targetType` in the `target` object, `DateTime` in this case, **BUT**, at least for `CurrentDateTimeUtcAsFixedOffset` this is ignored, as it is already known!

#### `runtimeStateful` addItem spec

The `runtimeStateful` addItem can add certain predefined things/values at runtime, **BUT** this addItem needs state, internally!

Currently the only implemented stateful runtime value (`rtValue`) is `RowEnumeration`, which will add an enumaration column. Enumeration starts at **1** and continues to however many rows/lines there are.

```jsonc
{
    ...
    "spec": {
        "name": "runtimeStateful",  // 1) (mandatory)
        "rtValue": "RowEnumeration" // 2) (mandatory)
    },
    "target": {
        ...
        "targetType": "UInt128"     // 3) (mandatory)
    }
}

```

1. The name (type) of the addItem spec. `runtimeStateful` in this case.
2. The type of stateful runtime value to enrich, `RowEnumeration` in this case.
3. It is good paractive to the set correct `targetType` in the `target` object, `UInt128` in this case, **BUT**, at least for `RowEnumeration` this is ignored, as it is already known!

## Data Types

The following data types are supported.

* `Char`, `String`
* `Int8`, `Int16`, `Int32`, `Int64`, `Int128`
* `UInt8`, `UInt16`, `UInt32`, `UInt64`, `UInt128`
* `Float32`, `Float64`
* `Bool`
* `Decimal` (*)
* `NaiveDate`, `NaiveDateTime`, `DateTime` (**)

(see also: <https://doc.rust-lang.org/book/ch03-02-data-types.html>)

(*) = through the `rust_decimal` crate. See: <https://docs.rs/rust_decimal/latest/rust_decimal/>

(**) = through the `chrono` crate. See: <https://docs.rs/chrono/latest/chrono/>
