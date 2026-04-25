.PHONY: dev test lint docker deploy clean

dev:
	cargo run --bin engine

test:
	cargo test --release -- --nocapture

lint:
	cargo fmt -- --check
	cargo clippy -- -D warnings

docker:
	docker buildx build -t apex-venue/engine:$(shell git rev-parse --short HEAD) -f docker/Dockerfile.engine .

deploy:
	kubectl apply -f k8s/

clean:
	cargo clean && rm -rf target/
