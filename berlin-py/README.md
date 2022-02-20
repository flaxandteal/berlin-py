To test locally:

```shell
 pip3 install poetry
 poetry run maturin develop # development build to test if it compiles
 poetry run maturin build
 poetry shell
 pip install --no-index ../target/wheels/berlin-0.1.0-cp39-cp39-macosx_11_0_arm64.whl
```

Afterwards berlin should be functional inside a python shell/script. Example:

```python
import berlin

db = berlin.load('../data')
loc = db.query('manchester population', 'gb', 1)[0];
print("location:", loc.words)
```