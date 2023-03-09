import berlin

print("Will load locations data")
db = berlin.load('../data')
loc = db.query('vegas population', 'gb', 1)[0];
print("location:", loc.words)
