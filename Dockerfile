# https://github.com/python-poetry/poetry/discussions/1879
# to improve ^^

## STAGE 1 - Core package(s)

FROM konstin2/maturin as maturin

ADD . /app/build
WORKDIR /app/build/berlin-py
