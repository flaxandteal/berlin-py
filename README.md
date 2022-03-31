# berlin-rs

A Python/Rust microservice to identify locations and tag them with UN-LOCODEs and
ISO-3166-2 subdivisions.


### Getting started

To test the Rust API locally:

```shell
  make run
```

This will make an API available on port 3001. It serves simple requests of the
form:

```shell
curl 'http://localhost:3001/berlin/search?q=house+prices+in+londo&state=gb' | jq
```

replacing `localhost` with the local endpoint (`jq` used for formatting).

This will return results of the form:

```json
{
  "time": "32.46ms",
  "query": {
    "raw": "house prices in londo",
    "normalized": "house prices in londo",
    "stop_words": [
      "in"
    ],
    "codes": [],
    "exact_matches": [
      "house"
    ],
    "not_exact_matches": [
      "house prices",
      "prices in",
      "prices",
      "in londo",
      "londo"
    ],
    "state_filter": "gb",
    "limit": 1,
    "levenshtein_distance": 2
  },
  "results": [
    {
      "loc": {
        "encoding": "UN-LOCODE",
        "id": "gb:lon",
        "key": "UN-LOCODE-gb:lon",
        "names": [
          "london"
        ],
        "codes": [
          "lon"
        ],
        "state": [
          "gb",
          "united kingdom of great britain and northern ireland"
        ],
        "subdiv": [
          "lnd",
          "london, city of"
        ]
      },
      "score": 1346,
      "offset": {
        "start": 16,
        "end": 21
      }
    }
  ]
}
```

A Python wheel can also be built, using

```shell
  make wheels
  pip install build/wheels/berlin-0.1.0-xyz.whl
```

where `xyz` is your architecture.

Afterwards berlin should be functional inside a python shell/script. Example:

```python
import berlin

db = berlin.load('../data')
loc = db.query('manchester population', 'gb', 1)[0];
print("location:", loc.words)
```

### Description

Berlin is a location search engine which  works on an in-memory collection of
all UN Locodes, subdivisions and states (countries). Here are the main
architectural highlights: On startup Berlin does a basic linguistic analysis of
the locations: split names into words, remove diacritics, transliterate
non-ASCII symbols to ASCII. For example,  this allows us to find  “Las Vegas”
when searching for “vegas”.  It employs string interning in order to both
optimise memory usage and allow direct lookups for exact matches. If we can
resolve (parts of) the search term to an existing interned string, it means
that we have a location with this name in the database.

When the user submits the search term, Berlin first does a preliminary analysis
of the search term: 1) split into words and pairs of words 2) try to identify
the former as existing locations (can be resolved to existing interned strings)
and tag them as “exact matches”. This creates many search terms from the
original phrase.  Pre-filtering step. Here we do three things 1) resolve exact
matches by direct lookup in the names and codes tables 2) do a prefix search
via a finite-state transducer 3) do a fuzzy search via a Levenshtein distance
enabled finite-state transducer.  The pre-filtered results are passed through a
string-similarity evaluation algorithm and sorted by score. The results below a
threshold are truncated.  A graph is built from the locations found during the
previous  step in order to link them together hierarchically if possible. This
further boosts some locations. For example, if the user searches for “new york
UK” it will boost the location in Lincolnshire and it will show up higher than
New York city in the USA.  It is also possible to request search only in a
specific country (which is enabled by default for the UK)

Berlin is able to find locations with a high degree of semantic accuracy. Speed
is roughly equal to 10-15 ms per every non-matching word (or typo) + 1 ms for
every exact match. A complex query of 8 words usually takes less than 100 ms
and all of the realistic queries in our test suite take less than 50 ms, while
the median is under 30 ms. Short queries containing  an exact match (case
insensitive) are faster than 10 ms.

The architecture would allow to easily implement as-you-type search suggestions
in under 10 milliseconds if deemed desirable.


### License

Prepared by Flax & Teal Limited for ONS Alpha project.
Copyright © 2022, Office for National Statistics (https://www.ons.gov.uk)

Released under MIT license, see [LICENSE](LICENSE.md) for details.

### License

Prepared by Flax & Teal Limited for ONS Alpha project.
Copyright © 2022, Office for National Statistics (https://www.ons.gov.uk)

Released under MIT license, see [LICENSE](LICENSE.md) for details.
