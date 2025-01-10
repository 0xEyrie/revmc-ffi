# Versioned by a simple counter that is not bound to a specific CosmWasm version
# See builders/README.md
BUILDERS_PREFIX := 0xeyrie/librevm-builder:0001

.PHONY: docker-image-debian
docker-image-debian:
	docker build --pull . -t $(BUILDERS_PREFIX)-debian -f ./Dockerfile.debian  

.PHONY: docker-image-cross
docker-image-cross:
	docker build --pull . -t $(BUILDERS_PREFIX)-cross -f ./Dockerfile.cross  


.PHONY: docker-images
docker-images: docker-image-cross docker-image-debian

.PHONY: docker-publish
docker-publish: docker-images
	docker push $(BUILDERS_PREFIX)-cross
	docker push $(BUILDERS_PREFIX)-debian
