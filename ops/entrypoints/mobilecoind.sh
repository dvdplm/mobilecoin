#!/bin/bash
#
# NOTE: Some of the environment variables are set in the Docker container, and
#       some are set in the docker run command

# exit if anything fails.
set -ex

# Make the mobilecoind_db dir (should not yet exist)
mkdir -p "${MOBILECOIND_DB_DIR}"

mobilecoind \
    --peer mc://node1.${BRANCH}.mobilecoin.com/ \
    --peer mc://node2.${BRANCH}.mobilecoin.com/ \
    --peer mc://node3.${BRANCH}.mobilecoin.com/ \
    --peer mc://node4.${BRANCH}.mobilecoin.com/ \
    --peer mc://node5.${BRANCH}.mobilecoin.com/ \
    --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node1.${BRANCH}.mobilecoin.com/ \
    --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node2.${BRANCH}.mobilecoin.com/ \
    --ledger-db "${NODE_LEDGER_DIR}" \
    --poll-interval 1 \
    --mobilecoind-db "${MOBILECOIND_DB_DIR}" \
    --service-port "${MOBILECOIND_SERVICE_PORT}"