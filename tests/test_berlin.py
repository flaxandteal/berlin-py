
def test_search_with_state(db):
    query = "Abercorn"
    state = "GB"
    limit = 2
    lev_distance = 2

    result = db.query(query, state, limit, lev_distance)
    assert len(result) == 1
    loc = result[0]
    assert loc.key == "UN-LOCODE-gb:abc"
    assert loc.encoding == "UN-LOCODE"
    assert loc.id == "gb:abc"
    assert list(loc.words) == ["abercarn"]
