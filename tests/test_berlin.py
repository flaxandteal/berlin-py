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
    assert loc.get_names() == ["abercarn"]
    assert loc.get_codes() == ["abc"]
    assert loc.get_state() == "gb"
    assert loc.get_subdiv() == "cay"
    assert db.get_state_name(loc.get_state()) == "united kingdom of great britain and northern ireland"
    assert db.get_subdiv_name(loc.get_state(), loc.get_subdiv()) == "caerphilly [caerffili gb-caf]"
