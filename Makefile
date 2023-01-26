SHELL=bash
MAIN=build 

GREEN  := $(shell tput -Txterm setaf 2)
YELLOW := $(shell tput -Txterm setaf 3)
WHITE  := $(shell tput -Txterm setaf 7)
CYAN   := $(shell tput -Txterm setaf 6)
RESET  := $(shell tput -Txterm sgr0)

BUILD=build

.PHONY: all
all: build

.PHONY: wheels
wheels:
	@mkdir -p $(BUILD)/wheels
	docker build -t berlin_py_build -f Dockerfile.wheels .
	docker run --platform "linux/amd64" --rm --entrypoint maturin -v $(shell pwd)/$(BUILD)/wheels:/app/build/target/wheels berlin_py_build build

.PHONY: build
build: Dockerfile ## Builds ./Dockerfile image name: berlin_rs
	docker build -t berlin_rs .

.PHONY: run
run: build ## First builds ./Dockerfile with image name: berlin_rs and then runs a container, with name: berlin, on port 3001
	docker run -p 3001:3001 --name berlin -ti --rm berlin_rs

 
help: ## Show this help.
	@echo ''
	@echo 'Usage:'
	@echo '  ${YELLOW}make${RESET} ${GREEN}<target>${RESET}'
	@echo ''
	@echo 'Targets:'
	@awk 'BEGIN {FS = ":.*?## "} { \
		if (/^[a-zA-Z_-]+:.*?##.*$$/) {printf "    ${YELLOW}%-20s${GREEN}%s${RESET}\n", $$1, $$2} \
		else if (/^## .*$$/) {printf "  ${CYAN}%s${RESET}\n", substr($$1,4)} \
		}' $(MAKEFILE_LIST)