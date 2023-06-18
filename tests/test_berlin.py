from berlin import Location

def test_search_with_state(db):
    query = "Abercorn"
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

def test_retrieve(db):
    loc = db.retrieve("UN-LOCODE-gb:abc")
    assert isinstance(loc, Location)
    assert loc.key == "UN-LOCODE-gb:abc"
    assert loc.encoding == "UN-LOCODE"
    assert loc.id == "gb:abc"
    assert list(loc.words) == ["abercarn"]
