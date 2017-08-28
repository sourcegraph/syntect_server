docker push sourcegraph/syntect_server

# Deploy to us.gcr.io/sourcegraph-dev as well (we use our own registry).
docker tag sourcegraph/syntect_server us.gcr.io/sourcegraph-dev/syntect_server
gcloud docker -- push us.gcr.io/sourcegraph-dev/syntect_server