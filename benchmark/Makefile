BENCHMARK_PREFIX := 0xeyrie/benchmark:0001

.PHONY: docker-image-gevm
docker-image-gevm:
	docker build  --pull . -t $(BENCHMARK_PREFIX)-gevm -f ./Dockerfile.gevm

.PHONY: docker-image-revmffi
docker-image-revmffi:
	docker build  --pull . -t $(BENCHMARK_PREFIX)-revmffi -f ./Dockerfile.revmffi


.PHONY: docker-images
docker-images: docker-image-revmffi docker-image-gevm

.PHONY: docker-publish
docker-publish: docker-images
	docker push $(BENCHMARK_PREFIX)-revmffi
	docker push $(BENCHMARK_PREFIX)-gevm

.PHONY: profiling
profiling:
	@echo "Running Profiling..."
	@export BENCHMARK_PREFIX=$(BENCHMARK_PREFIX) && \
	export REV_CONTAINER_NAME=revmffi && \
	export GEV_CONTAINER_NAME=gevm && \
	docker-compose -f ./docker-compose.yml up -d
	docker exec -it revmffi sh -c "cd /app/revmffi && go test -v --count 1000"
	docker exec -it gevm sh -c "cd /app/gevm && go test -v --count 1000"