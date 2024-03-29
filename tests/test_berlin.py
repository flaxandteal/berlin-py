import pytest
from berlin import Location

def test_search_long(db):
    for query, match in (
        ("Where are all the dentists in Abercarn", "Abercarn"),
        ("Whereareallthedentistsandhowdoifindtheminabercarn", None),
        ("Whereareallthedentistsandhowdoi findthem iabercarn", "iabercarn"),
        ("Whereareallthedentistsandhowdoi findthem in bognore regis", "bognore regis"),
    ):
        state = "GB"
        limit = 2
        lev_distance = 3

        result = db.query(query, limit, lev_distance, state=state)
        if match:
            assert len(result) == 1, f"Did not find {match} in {query}"
            loc = result[0]
            assert isinstance(loc.get_score(), int)
            offset = query.find(match) or 0
            assert loc.get_offset() == (offset, offset + len(match))

def test_search_with_state(db):
    for query in ("Dentists in Abercarn", "Dental Abercarn"):
        state = "GB"
        limit = 2
        lev_distance = 2

        result = db.query(query, limit, lev_distance, state=state)
        assert len(result) == 1
        loc = result[0]
        assert loc.key == "UN-LOCODE-gb:abc"
        assert loc.encoding == "UN-LOCODE"
        assert loc.id == "gb:abc"
        assert list(loc.words) == ["abercarn"]

        assert isinstance(loc.get_score(), int)
        assert loc.get_score() == 1008
        offset = query.find("Abercarn") or 0
        assert loc.get_offset() == (offset, offset + len("Abercarn"))

def test_retrieve(db):
    loc = db.retrieve("UN-LOCODE-gb:abc")
    assert isinstance(loc, Location)
    assert loc.key == "UN-LOCODE-gb:abc"
    assert loc.encoding == "UN-LOCODE"
    assert loc.id == "gb:abc"
    assert list(loc.words) == ["abercarn"]
    assert loc.get_names() == ["abercarn"]
    assert loc.get_codes() == ["abc"]
    assert loc.get_state_code() == "gb"
    assert loc.get_subdiv_code() == "cay"
    assert db.get_state_key(loc.get_state_code()) == "ISO-3166-1-gb"
    assert db.get_subdiv_key(loc.get_state_code(), loc.get_subdiv_code()) == "ISO-3166-2-gb:cay"
    assert loc.subdiv.id == "gb:cay"

def test_retrieve_with_score(db):
    loc = db.retrieve("UN-LOCODE-gb:abc")
    with pytest.raises(AttributeError):
        loc.get_score()
    with pytest.raises(AttributeError):
        loc.get_offset()

def test_retrieve_country(db):
    loc = db.retrieve("ISO-3166-1-gb")
    assert isinstance(loc, Location)
    assert loc.key == "ISO-3166-1-gb"
    assert loc.encoding == "ISO-3166-1"
    assert loc.id == "gb"
    assert sorted(loc.words) == ["britain", "great", "ireland", "kingdom", "northern", "united"]
    assert loc.get_names() == ["united kingdom of great britain and northern ireland"]
    assert loc.get_codes() == ["gb", "gbr", "uk"]
    assert loc.get_state_code() == "gb"
    assert loc.get_subdiv_code() is None
    assert db.get_state_key(loc.get_state_code()) == "ISO-3166-1-gb"
    assert loc.state.get_state_code() == "gb"
    assert loc.subdiv is None

def test_retrieve_country_children(db):
    loc = db.retrieve("ISO-3166-1-gb")
    assert len(loc.children) == 3
    assert {child.key for child in loc.children} == {"ISO-3166-2-gb:cay", "ISO-3166-2-gb:abd", "ISO-3166-2-gb:wsx"}

    aberdeen = next(child for child in loc.children if child.key == "ISO-3166-2-gb:abd")
    assert len(aberdeen.children) == 1

    stonehaven = aberdeen.children[0]
    assert stonehaven.get_names() == ["stonehaven"]

    assert stonehaven.children == []

def test_search_for_generic_with_state(db):
    for query in ("Dentists in Two2", "Dental Two2"):
        state = "GB"
        limit = 2
        lev_distance = 2

        result = db.query(query, limit, lev_distance, state=state)
        assert len(result) == 1
        loc = result[0]
        assert loc.key == "MY-STANDARD-gb:my-2"
        assert loc.encoding == "MY-STANDARD"
        assert loc.id == "gb:my-2"
        assert list(loc.words) == ["My Two2"]
