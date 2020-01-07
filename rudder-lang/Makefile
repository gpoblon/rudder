build:
	cargo build

test:
	cargo test

# TODO: integration and unit test distinction
# unitest:
	# cargo test unit-test

# intest:
	# cargo test integration-test

doc:
	cargo doc --help

script:
	sh tests/tester.sh $(technique)


stats:
	@ echo -n "TODOS  : " && grep -r TODO src | wc -l
	@ echo -n "Commits: " && git log --oneline | wc -l
	@ tokei
#	@ echo "Lines  : " && find . -name '*.rs' | xargs wc -l
