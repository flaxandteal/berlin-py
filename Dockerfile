# https://github.com/python-poetry/poetry/discussions/1879
# to improve ^^

## STAGE 1 - Core package(s)

FROM ghcr.io/pyo3/maturin:main as maturin

ADD . /app/build
WORKDIR /app/build
