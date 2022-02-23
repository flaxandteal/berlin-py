SHELL=bash
MAIN=build

BUILD=build

.PHONY: all
all: build

.PHONY: wheels
wheels:
	@mkdir -p $(BUILD)/wheels
	docker build -t berlin_py_build -f Dockerfile.wheels .
	docker run --platform "linux/amd64" --rm --entrypoint maturin -v $(shell pwd)/$(BUILD)/wheels:/app/build/target/wheels berlin_py_build build

.PHONY: build
build: Dockerfile
	docker build -t berlin_rs .

.PHONY: run
run: build
	docker run -ti --rm berlin_rs
