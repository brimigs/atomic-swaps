#!/bin/bash

osmosisd tx authz grant $CONTRACT_ADDR send --spend-limit=$makercoin_amount --from=$maker_addr --chain-id=osmo-test-5