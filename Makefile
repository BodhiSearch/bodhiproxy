.PHONY: all ci clean build test

ci:
	$(MAKE) -C server ci
	$(MAKE) -C pyserver ci

clean:
	$(MAKE) -C server clean
	$(MAKE) -C pyserver clean

build:
	$(MAKE) -C server build
	$(MAKE) -C pyserver build

test:
	$(MAKE) -C server test
	$(MAKE) -C pyserver test
